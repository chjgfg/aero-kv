use std::sync::{Arc, Mutex};

use axum::{Json, Router, extract::State, routing::post};
use serde_json::{json, Value};

pub async fn proxy_handler(
    State(state): State<Arc<GatewayState>>,
    req_body: Json<Value>,
) -> Json<Value> {
    // 1. 轮询选择一个后端节点
    let node_url = {
        let mut cur = state.current.lock().unwrap();
        let url = state.nodes[*cur].clone();
        *cur = (*cur + 1) % state.nodes.len();
        url
    };
    println!("node_url: {}", node_url);

    // 2. 转发请求 (这里以 /set 为例)
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/set", node_url))
        .json(&req_body.0)
        .send()
        .await;

    println!("res: {:?}", res);

    // 3. 返回结果给前端
    match res {
        Ok(response) => {
            let _status = response.status();
            let body = response
                .json::<Value>()
                .await
                .unwrap_or(json!({"error": "invalid json"}));
            Json(body)
        }
        Err(e) => Json(json!({
            "status": "error",
            "message": format!("Node {} is unreachable: {}", node_url, e)
        })),
    }
}



struct GatewayState {
    // 后端 Raft 节点的 HTTP 地址
    nodes: Vec<String>,
    // 用于轮询 (Round Robin) 的计数器
    current: Mutex<usize>,
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

    let app = Router::new()
        .route("/set", post(proxy_handler))
        .route("/get", post(proxy_handler)) // 假设你统一用 POST 转发
        .with_state(state);

    // 网关监听公共端口，比如 80 或 8080
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("🚀 AeroKV Public Gateway running on port 8080");
    axum::serve(listener, app).await.unwrap();
}

