use axum::{
    body::Bytes,
    extract::{Request, State},
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{any, get},
    Json, Router,
};
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use tokio::sync::RwLock;
use serde_json::json;
use tower_http::cors::{Any, CorsLayer};

struct ProxyState {
    nodes: Vec<String>,
    current_leader: RwLock<String>,
    rr_index: AtomicUsize,
}

type AppStateProxy = ProxyState;

#[tokio::main]
async fn main() {
    let state = Arc::new(ProxyState {
        nodes: vec![
            "http://127.0.0.1:8001".to_string(),
            "http://127.0.0.1:8002".to_string(),
            "http://127.0.0.1:8003".to_string(),
        ],
        current_leader: RwLock::new("http://127.0.0.1:8001".to_string()),
        rr_index: AtomicUsize::new(0),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .expose_headers(Any)
        .allow_credentials(false);

    let app = Router::new()
        .route("/proxy/status", get(get_status))
        .fallback(any(proxy_handler))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await.unwrap();
    println!("Gateway running on port 80...");
    axum::serve(listener, app).await.unwrap();
}

async fn proxy_handler(
    State(state): State<Arc<AppStateProxy>>,
    req: Request,
) -> Response {
    // 1. 预先提取所有需要的信息，因为 into_parts 会消耗 req
    let path_query = req.uri()
        .path_and_query()
        .map(|v| v.as_str().to_string())
        .unwrap_or_default();
    let path = req.uri().path().to_string();
    let method = req.method().clone();

    // 2. 消耗请求，获取 Header 和 Body
    let (parts, body) = req.into_parts();
    
    // 将流式的 Body 转换为内存中的 Bytes，这样就可以多次 clone 了
    let body_bytes = match axum::body::to_bytes(body, 100 * 1024 * 1024).await {
        Ok(b) => b,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    let client = reqwest::Client::new();

    // 3. 路由逻辑判定
    let is_read_only_path = path.contains("/kv/scan") || path.contains("/kv/page") || path.contains("/kv/get");
    let is_write = method != Method::GET && !is_read_only_path;

    let mut target_node = if is_write {
        state.current_leader.read().await.clone()
    } else {
        let idx = state.rr_index.fetch_add(1, Ordering::SeqCst) % state.nodes.len();
        state.nodes[idx].clone()
    };

    // 4. 重试逻辑
    for i in 0..3 {
        let full_url = format!("{}{}", target_node, path_query);
        let reqwest_method = reqwest::Method::from_bytes(method.as_str().as_bytes()).unwrap();

        // 构造转发请求
        let mut forward_req = client
            .request(reqwest_method, &full_url)
            .body(body_bytes.clone());

        // 复制原始 Headers
        let mut headers = reqwest::header::HeaderMap::new();
        for (key, value) in parts.headers.iter() {
            if key != "host" {
                headers.insert(key.clone(), value.clone());
            }
        }
        forward_req = forward_req.headers(headers);

        match forward_req.send().await {
            Ok(resp) => {
                let status_raw = resp.status().as_u16();
                let bytes = resp.bytes().await.unwrap_or_default();
                
                // 尝试解析是否为 Leader 转发错误
                let json_res: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or_default();

                if let Some(leader_info) = json_res.get("leader_node") {
                    if let Some(addr) = leader_info.get("addr") {
                        let new_leader = format!("http://{}", addr.as_str().unwrap());
                        println!("Auto-updating leader to: {}", new_leader);
                        
                        let mut cache = state.current_leader.write().await;
                        *cache = new_leader.clone();
                        target_node = new_leader;
                        continue; // 修正后立即重试
                    }
                }
                
                let axum_status = StatusCode::from_u16(status_raw).unwrap();
                return (axum_status, bytes).into_response();
            }
            Err(e) => {
                eprintln!("Attempt {} failed: {}", i, e);
                if i < 2 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    continue;
                }
            }
        }
    }

    StatusCode::SERVICE_UNAVAILABLE.into_response()
}

async fn get_status(State(state): State<Arc<ProxyState>>) -> Json<serde_json::Value> {
    let leader = state.current_leader.read().await;
    Json(json!({ "current_leader": *leader }))
}