use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

// 2. 通用转发处理器
use axum::{
    Json, Router,
    extract::{Request, State},
    http::{Method as AxumMethod, StatusCode as AxumStatusCode, Uri},
    response::IntoResponse,
};
use tower_http::cors::{Any, CorsLayer};

// 1. 定义状态结构
struct GatewayState {
    nodes: Vec<String>,
    current: Mutex<usize>,
}

async fn proxy_handler(
    State(state): State<Arc<GatewayState>>,
    method: AxumMethod,
    uri: Uri,
    req: Request,
) -> impl IntoResponse {
    // 1. 轮询逻辑 (保持不变)
    let node_base_url = {
        let mut cur = state.current.lock().unwrap();
        let url = state.nodes[*cur].clone();
        *cur = (*cur + 1) % state.nodes.len();
        url
    };

    let full_url = format!(
        "{}{}",
        node_base_url,
        uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("")
    );

    // 2. 将 Axum 的 Method 转换为 reqwest 的 Method
    let reqwest_method = reqwest::Method::from_bytes(method.as_str().as_bytes()).unwrap();

    let client = reqwest::Client::new();
    let body_bytes = axum::body::to_bytes(req.into_body(), usize::MAX)
        .await
        .unwrap_or_default();

    // 3. 发送请求
    let res = client
        .request(reqwest_method, &full_url)
        .body(body_bytes)
        .header("Content-Type", "application/json")
        .send()
        .await;

    // 4. 处理返回结果
    match res {
        Ok(response) => {
            // 关键：将 reqwest 的 StatusCode 转换为 axum 的 StatusCode
            let status_code = AxumStatusCode::from_u16(response.status().as_u16())
                .unwrap_or(AxumStatusCode::INTERNAL_SERVER_ERROR);

            let body = response
                .json::<Value>()
                .await
                .unwrap_or(json!({"error": "response not json"}));

            // 直接返回元组即可，Axum 为 (StatusCode, Json<Value>) 实现了 IntoResponse
            (status_code, Json(body))
        }
        Err(e) => (
            AxumStatusCode::BAD_GATEWAY,
            Json(json!({"error": format!("Node unreachable: {}", e)})),
        ),
    }
}

#[tokio::main]
async fn main() {
    let state = Arc::new(GatewayState {
        nodes: vec![
            "http://127.0.0.1:3001".into(),
            "http://127.0.0.1:3002".into(),
            "http://127.0.0.1:3003".into(),
        ],
        current: Mutex::new(0),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .expose_headers(Any)
        .allow_credentials(false);

    // 使用 any 方法，匹配所有路径，这样不需要一个个写 route
    let app = Router::new()
        .fallback(proxy_handler) //  fallback 会捕获所有未定义的路径并转发
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await.unwrap();
    println!("🚀 AeroKV Universal Gateway running on port 80");
    axum::serve(listener, app).await.unwrap();
}
