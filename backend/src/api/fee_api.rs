// # set_fee 接口
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use log::info;

use crate::{AppState, fee};

#[derive(Debug, serde::Deserialize)]
pub struct FeeRequest {
    pub base_fee: u64,
    pub fee_per_byte: u64,
    pub scan_fee_per_item: u64,
}

// 手续费接口示例
pub async fn init_fee(State(client): State<AppState>) -> impl IntoResponse {
    info!("init fee");
    let _ = fee::init_fee(client).await;
    (StatusCode::OK, "init fee success")
}

pub async fn set_fee(
    State(client): State<AppState>,
    Json(req): Json<FeeRequest>,
) -> impl IntoResponse {
    info!(
        "set fee base_fee: {}, fee_per_byte: {}, scan_fee_per_item: {}",
        req.base_fee, req.fee_per_byte, req.scan_fee_per_item
    );
    let _ = fee::set_fee(
        client,
        req.base_fee,
        req.fee_per_byte,
        req.scan_fee_per_item,
    )
    .await;
    (StatusCode::OK, "set fee success")
}
