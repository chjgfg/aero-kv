// # set_fee 接口
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::AppState;

#[derive(Debug, serde::Deserialize)]
pub struct FeeRequest {
    pub base_fee: u64,
    pub fee_per_byte: u64,
    pub scan_fee_per_item: u64,
}

// 手续费接口示例
pub async fn init_fee(State(client): State<AppState>) -> impl IntoResponse {
    client.init_fee().await;
    (StatusCode::OK, "init fee success")
}

pub async fn set_fee(
    State(client): State<AppState>,
    Json(req): Json<FeeRequest>,
) -> impl IntoResponse {
    client.set_fee(req.base_fee, req.fee_per_byte, req.scan_fee_per_item).await;
    (StatusCode::OK, "set fee success")
}
