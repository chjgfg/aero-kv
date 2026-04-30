use std::sync::Arc;

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
pub async fn init_fee(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("init fee");
    let chain = state.chain.clone();
    let Ok(res) = fee::init_fee(chain).await else {
        let json_response = serde_json::json!({
            "status": "error",
            "signature": "init fee error",
        });
        return (StatusCode::BAD_REQUEST, Json(json_response)).into_response();
    };
    let json_response = serde_json::json!({
        "status": "success",
        "signature": res.to_string()
    });
    (StatusCode::OK, Json(json_response)).into_response()
}

#[allow(dead_code)]
pub async fn set_fee(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FeeRequest>,
) -> impl IntoResponse {
    info!(
        "set fee base_fee: {}, fee_per_byte: {}, scan_fee_per_item: {}",
        req.base_fee, req.fee_per_byte, req.scan_fee_per_item
    );
    let chain = state.chain.clone();
    let Ok(res) = fee::set_fee(chain, req.base_fee, req.fee_per_byte, req.scan_fee_per_item).await
    else {
        let json_response = serde_json::json!({
            "status": "error",
            "signature": "set fee error",
        });
        return (StatusCode::BAD_REQUEST, Json(json_response)).into_response();
    };
    let json_response = serde_json::json!({
        "status": "success",
        "signature": res.to_string()
    });
    (StatusCode::OK, Json(json_response)).into_response()
}
