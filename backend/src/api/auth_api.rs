// # set_admin / set_pause 接口
use axum::{
    Extension, Json, extract::{Query, State}, http::StatusCode, response::IntoResponse
};
use log::info;
use serde::Deserialize;
use std::{sync::Arc};

use crate::{AppState, auth::{self, types::Action}, error::Error};

#[derive(Debug, Deserialize)]
// #[allow(dead_code)]
pub struct AuthQuery {
    // pub new_admin: Option<String>,
    pub paused: Option<bool>,
}

#[derive(Debug, serde::Deserialize)]
pub struct AuthRequest {
    pub admin_pubkey: Option<String>,
    pub user_pubkey: Option<String>,
    pub perm_char: Option<Vec<Action>>,
}

// 权限接口示例
pub async fn init_auth(State(state): State<Arc<AppState>>, Extension(pubkey): Extension<String>,) -> impl IntoResponse {
    info!("pubkey: {}", pubkey);
    let Ok(_) = state.session.check_permission(&pubkey, Action::InitAuth) else {
        // 🌟 在这里必须显式返回一个 Response
        return (StatusCode::FORBIDDEN, "Permission denied").into_response();
    };
    info!("init auth");
    let chain = state.chain.clone();
    let Ok(res) = auth::init_auth(chain).await else {
        let json_response = serde_json::json!({
            "status": "error",
            "signature": "init auth error",
        });
        return (StatusCode::UNAUTHORIZED, Json(json_response)).into_response();
    };
    let json_response = serde_json::json!({
        "status": "success",
        "signature": res.to_string()
    });
    (StatusCode::OK, Json(json_response)).into_response()
}

// #[allow(dead_code)]
// pub async fn set_admin(
//     State(state): State<Arc<AppState>>,
//     Query(req): Query<AuthQuery>,
// ) -> impl IntoResponse {
//     let Ok(new_admin) = req.new_admin.ok_or(|e| Error::InvalidParam(e)) else {
//         let json_response = serde_json::json!({
//             "status": "error",
//             "signature": "set admin error",
//         });
//         return (StatusCode::BAD_REQUEST, Json(json_response)).into_response();
//     };
//     info!("set admin new_admin: {}", new_admin);
//     let chain = state.chain.clone();
//     let pubkey = Pubkey::from_str(new_admin.as_str()).unwrap();
//     let Ok(res) = auth::set_admin(chain, pubkey).await else {
//         let json_response = serde_json::json!({
//             "status": "error",
//             "signature": "set admin error",
//         });
//         return (StatusCode::BAD_REQUEST, Json(json_response)).into_response();
//     };
//     (StatusCode::OK, res.to_string()).into_response()
// }

#[allow(dead_code)]
pub async fn set_pause(
    State(state): State<Arc<AppState>>,
    Query(req): Query<AuthQuery>,
) -> impl IntoResponse {
    let Ok(paused) = req.paused.ok_or(|e| Error::InvalidParam(e)) else {
        return (StatusCode::BAD_REQUEST, "set pause error").into_response();
    };
    info!("set pause paused: {}", paused);
    let chain = state.chain.clone();
    let Ok(res) = auth::set_pause(chain, paused).await else {
        let json_response = serde_json::json!({
            "status": "error",
            "signature": "set pause error",
        });
        return (StatusCode::NOT_MODIFIED, Json(json_response)).into_response();
    };
    let json_response = serde_json::json!({
        "status": "success",
        "signature": res.to_string()
    });
    (StatusCode::OK, Json(json_response)).into_response()
}
