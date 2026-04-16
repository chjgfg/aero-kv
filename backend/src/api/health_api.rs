use axum::{Json, extract::State, response::IntoResponse};
use serde::Serialize;

use crate::AppState;

// 定义你要返回的结构
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub program_id: String,
    pub message: String,
}

pub async fn health(State(client): State<AppState>) -> impl IntoResponse {
    // 构造你自定义的数据
    let resp = HealthResponse {
        status: "ok".to_string(),
        program_id: client.program_id.to_string(), // 从 Arc<ChainClient> 里拿！
        message: "service is running".to_string(),
    };
    // 返回 JSON
    Json(resp)
}
