// # set_admin / set_pause 接口
use axum::{
    Extension, Json, extract::{Query, State}, http::StatusCode, response::IntoResponse
};
use log::info;
use serde::Deserialize;
use serde_json::json;
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

#[derive(Debug, serde::Deserialize)]
pub struct AuthPageRequest {
    pub pubkey: Option<String>,
    pub page: Option<usize>,
    pub limit: usize,
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

pub async fn admin_page(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AuthPageRequest>,
) -> impl IntoResponse {
    let Ok(page) = req.page.ok_or(|e| Error::InvalidParam(e)) else {
        return (StatusCode::BAD_REQUEST, "admin page error").into_response();
    };
    let auth_storage = state.session.auth_storage.clone();
    let limit = req.limit;
    let Ok(res) = state.session.admin_page(page, limit, auth_storage).await else {
        let json_response = serde_json::json!({
            "status": "error",
            "signature": "admin page error",
        });
        return (StatusCode::NOT_MODIFIED, Json(json_response)).into_response();
    };
    // 🌟 将 HashMap 转换为 Vec<{pubkey, permissions}> 以保证 JSON 格式友好
    let auth_list: Vec<_> = res.1.into_iter()
        .map(|(pubkey, permissions)| {
            json!({
                "pubkey": pubkey,
                "permissions": permissions
            })
        })
        .collect();

    let json_response = json!({
        "status": "success",
        "total": res.0,      // 🌟 返回总条数供前端分页器使用
        "auth_list": auth_list // 🌟 返回列表供前端表格使用
    });

    (StatusCode::OK, Json(json_response)).into_response()
}


pub async fn admin_get(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AuthPageRequest>,
) -> impl IntoResponse {
    let auth_storage = state.session.auth_storage.clone();
    let Some(pubkey) = req.pubkey else {
        return (StatusCode::BAD_REQUEST, "admin get error").into_response();
    };
    let limit = req.limit;
    let Ok(res) = state.session.admin_get(pubkey, limit, auth_storage).await else {
        let json_response = serde_json::json!({
            "status": "error",
            "signature": "admin get error",
        });
        return (StatusCode::NOT_MODIFIED, Json(json_response)).into_response();
    };
    // 🌟 将 HashMap 转换为 Vec<{pubkey, permissions}> 以保证 JSON 格式友好
    let auth_list: Vec<_> = res.into_iter()
        .map(|(pubkey, permissions)| {
            json!({
                "pubkey": pubkey,
                "permissions": permissions
            })
        })
        .collect();

    let json_response = json!({
        "status": "success",
        "auth_list": auth_list // 🌟 返回列表供前端表格使用
    });

    (StatusCode::OK, Json(json_response)).into_response()
}
