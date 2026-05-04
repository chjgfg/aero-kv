// --- Raft 内部 RPC 处理器 (直接转发给 Raft 实例) ---[cite: 1]

use std::sync::Arc;

use axum::{
    Extension, Json, body::Body, extract::{Query, State}, http::{Request, StatusCode, header}, middleware::Next, response::{IntoResponse, Response}
};
use log::info;
use openraft::{BasicNode, error::{CheckIsLeaderError, RaftError}};
use serde_json::json;

use crate::{
    AppState, api::{
        auth_api::{AuthQuery, AuthRequest},
        fee_api::FeeRequest,
        kv_api::{KVQuery, KVRequest},
    }, auth::{self, types::Action}, block_chain, constants::VALUE_SEEDS, core::validator::{check_key, check_value}, fee, raft::types::{KvOp, NodeId, RaftConfig}, utils::{self}
};

pub async fn auth_middleware(
    // 🌟 建议直接去掉泛型 <B>，简单直接
    State(_state): State<Arc<AppState>>,
    mut req: Request<Body>, // 🌟 改为具体的 Body 类型
    next: Next,
) -> Response {
    // 1. 提取公钥
    let pubkey_res = req
        .headers()
        .get("x-user-pubkey")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string()); // 🌟 关键：在这里直接转成 String，断开借用

    let pubkey: String = match pubkey_res {
        // 🌟 现在它是 String 类型
        Some(k) => k,
        None => return (StatusCode::UNAUTHORIZED, "User not logged in").into_response(),
    };

    // 2. 存入扩展（这一步你写对了，非常棒，方便后续 Handler 直接用 Extension 取）
    req.extensions_mut().insert(pubkey.to_string());

    // 3. 继续执行后续逻辑
    next.run(req).await
}

pub async fn leader_forwarding_middleware(
    State(state): State<Arc<AppState>>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Response {
    // 1. 检查自己是不是 Leader
    match state.raft.is_leader().await {
        Ok(_) => {
            // 是 Leader，直接执行本地 Handler
            next.run(req).await
        }
        Err(e) => {
            match e {
                // 🌟 适配 OpenRaft 的错误嵌套结构
                RaftError::APIError(CheckIsLeaderError::ForwardToLeader(f)) => {
                    let leader_addr = match f.leader_node {
                        Some(node) => node.addr,
                        None => return (StatusCode::SERVICE_UNAVAILABLE, "No leader found").into_response(),
                    };

                    // 2. 构造转发逻辑
                    let client = reqwest::Client::new();
                    
                    // 🌟 核心：解构请求以获取 Parts (Headers) 和 Body
                    let (parts, body) = req.into_parts();
                    let method = parts.method.clone();
                    let uri = format!("http://{}{}", leader_addr, parts.uri.path_and_query().map(|x| x.as_str()).unwrap_or(""));
                    
                    // 提取 Body 字节[cite: 1]
                    let body_bytes = match axum::body::to_bytes(body, 100 * 1024 * 1024).await {
                        Ok(b) => b,
                        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
                    };

                    // 🌟 核心修复：通过字符串转换 Method，绕过 http 库版本冲突[cite: 1]
                    let reqwest_method = reqwest::Method::from_bytes(method.as_str().as_bytes()).unwrap();

                    // 3. 构造转发请求并【复制 Headers】[cite: 1]
                    let mut forward_req = client.request(reqwest_method, &uri)
                        .body(reqwest::Body::from(body_bytes));

                    // 🌟 修复 415 错误：必须带上 Content-Type 等原始请求头[cite: 1]
                    // 🌟 修复：将 axum 的 Header 转换为字符串，再传给 reqwest
                    for (key, value) in parts.headers.iter() {
                        if key != header::HOST {
                            // 使用 as_str() 转换为字符串，绕过版本不兼容的类型限制
                            forward_req = forward_req.header(key.as_str(), value.as_bytes());
                        }
                    }
                    // 4. 执行转发并处理响应[cite: 1]
                    match forward_req.send().await {
                        Ok(resp) => {
                            let status = StatusCode::from_u16(resp.status().as_u16()).unwrap();
                            
                            // 🌟 构造响应并回传 Leader 返回的 Headers (如 Content-Type)[cite: 1]
                            let mut response_builder = Response::builder().status(status);
                            
                            // 🌟 修复：将 reqwest 的 Header 转换为字符串，再回传给 axum
                            for (key, value) in resp.headers().iter() {
                                // 同样通过 as_str 和 as_bytes 进行中转
                                response_builder = response_builder.header(key.as_str(), value.as_bytes());
                            }

                            let data = resp.bytes().await.unwrap_or_default();
                            response_builder
                                .body(axum::body::Body::from(data))
                                .unwrap()
                                .into_response()
                        }
                        Err(_) => (StatusCode::BAD_GATEWAY, "Failed to forward to leader").into_response(),
                    }
                }
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "Raft internal error").into_response(),
            }
        }
    }
}

pub async fn raft_append(
    State(state): State<Arc<AppState>>,
    Json(req): Json<openraft::raft::AppendEntriesRequest<RaftConfig>>,
) -> Json<openraft::raft::AppendEntriesResponse<NodeId>> {
    let res = state.raft.append_entries(req).await.unwrap();
    Json(res)
}

pub async fn raft_vote(
    State(state): State<Arc<AppState>>,
    Json(req): Json<openraft::raft::VoteRequest<NodeId>>,
) -> Json<openraft::raft::VoteResponse<NodeId>> {
    let res = state.raft.vote(req).await.unwrap();
    Json(res)
}

pub async fn raft_snapshot(
    State(state): State<Arc<AppState>>,
    Json(req): Json<openraft::raft::InstallSnapshotRequest<RaftConfig>>,
) -> Json<openraft::raft::InstallSnapshotResponse<NodeId>> {
    let res = state.raft.install_snapshot(req).await.unwrap();
    Json(res)
}

pub async fn raft_init(
    State(state): State<Arc<AppState>>,
    Json(nodes): Json<std::collections::BTreeMap<NodeId, BasicNode>>,
) -> String {
    match state.raft.initialize(nodes).await {
        Ok(_) => "Cluster initialized".into(),
        Err(e) => format!("Init error: {:?}", e),
    }
}

pub async fn raft_upsert(
    State(state): State<Arc<AppState>>,
    Extension(pubkey): Extension<String>, // 🌟 直接拿到中间件塞进去的公钥
    Json(req): Json<KVRequest>,
) -> impl IntoResponse {
    // 如果你想在 Handler 里再次手动校验（虽然中间件可能已经做过了）
    info!("pubkey: {}", pubkey);
    let Ok(_) = state.session.check_permission(&pubkey, Action::RaftUpsert) else {
        // 🌟 在这里必须显式返回一个 Response
        return (StatusCode::FORBIDDEN, "Permission denied").into_response();
    };

    if !check_key(req.key.clone().as_str()) {
        return (StatusCode::BAD_REQUEST, "key too long or empty").into_response();
    }
    if !check_value(req.value.clone().as_str()) {
        return (StatusCode::BAD_REQUEST, "value too long or empty").into_response();
    }
    info!("upsert key: {}, value: {}", req.key, req.value);
    let chain = state.chain.clone();
    let k = req.key.clone().into_bytes();
    let (value_pda, _) = chain.find_pda(&[VALUE_SEEDS, k.as_slice()]);
    // 1. 构造 Raft 写操作提案
    let op = KvOp::Upsert {
        key: req.key.clone(),
        value: req.value.clone(),
        pda: value_pda.to_bytes().to_vec(), // 传入 PDA 字节
    };

    // 2. 通过 Raft 提交提案。这一步会把日志同步到大多数节点
    // 只有 Leader 能执行此操作
    // 这个发往 state_machine.rs
    match state.raft.client_write(op).await {
        Ok(_) => {
            // 3. Raft 日志同步成功后，再执行原有的链上逻辑[cite: 1]
            // 注意：此时状态机已经在各节点本地应用了数据，这里只需处理 Solana 交易反馈
            let chain = state.chain.clone();
            let storage = state.storage.clone();
            let v = req.value.into_bytes();

            match block_chain::upsert(chain, storage, k, v).await {
                Ok(res) => {
                    // 重建 Merkle 树（保持你原有的逻辑）
                    let _ = utils::calc_merkle_root(state).await;

                    let json_response = serde_json::json!({
                        "status": "success",
                        "signature": res.to_string()
                    });
                    (StatusCode::OK, Json(json_response)).into_response()
                }
                Err(_) => {
                    let json_response = serde_json::json!({
                        "status": "error",
                        "signature": "upsert chain error"
                    });
                    (StatusCode::BAD_REQUEST, Json(json_response)).into_response()
                }
            }
        }
        Err(e) => {
            // 如果不是 Leader 或同步失败[cite: 1]
            let json_response = serde_json::json!({
                "status": "error",
                "message": format!("Raft write failed: {:?}", e)
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json_response)).into_response()
        }
    }
}

pub async fn raft_delete(
    State(state): State<Arc<AppState>>,
    Extension(pubkey): Extension<String>,
    Query(req): Query<KVQuery>,
) -> impl IntoResponse {
    // 如果你想在 Handler 里再次手动校验（虽然中间件可能已经做过了）
    info!("pubkey: {}", pubkey);
    let Ok(_) = state.session.check_permission(&pubkey, Action::RaftDelete) else {
        // 🌟 在这里必须显式返回一个 Response
        return (StatusCode::FORBIDDEN, "Permission denied").into_response();
    };
    if !check_key(req.key.clone().as_str()) {
        return (StatusCode::BAD_REQUEST, "key too long or empty").into_response();
    }
    info!("delete key: {}", req.key);
    // 1. 构造 Raft 删除提案[cite: 1]
    let op = KvOp::Delete {
        key: req.key.clone(),
    };

    // 2. 提交到 Raft 集群[cite: 1]
    match state.raft.client_write(op).await {
        Ok(_) => {
            // 3. 同步成功后处理链上删除
            let chain = state.chain.clone();
            let storage = state.storage.clone();
            let k = req.key.into_bytes();

            match block_chain::delete(chain, storage, k).await {
                Ok(res) => {
                    let _ = utils::calc_merkle_root(state).await;
                    let json_response = serde_json::json!({
                        "status": "success",
                        "signature": res.to_string()
                    });
                    (StatusCode::OK, Json(json_response)).into_response()
                }
                Err(_) => {
                    let json_response = serde_json::json!({
                        "status": "error",
                        "signature": "delete chain error"
                    });
                    (StatusCode::BAD_REQUEST, Json(json_response)).into_response()
                }
            }
        }
        Err(e) => {
            let json_response = serde_json::json!({
                "status": "error",
                "message": format!("Raft delete failed: {:?}", e)
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json_response)).into_response()
        }
    }
}

pub async fn raft_pause(
    State(state): State<Arc<AppState>>,
    Extension(pubkey): Extension<String>,
    Query(req): Query<AuthQuery>,
) -> impl IntoResponse {
    info!("pubkey: {}", pubkey);
    let Ok(_) = state.session.check_permission(&pubkey, Action::RaftPause) else {
        // 🌟 在这里必须显式返回一个 Response
        return (StatusCode::FORBIDDEN, "Permission denied").into_response();
    };
    let Ok(paused) = req.paused.ok_or_else(|| "missing paused param".to_string()) else {
        return (StatusCode::BAD_REQUEST, "set pause error").into_response();
    };

    info!("raft set pause paused: {}", paused);
    // 1. 提交 Raft 提案
    let op = KvOp::SetPause { paused };
    match state.raft.client_write(op).await {
        Ok(_) => {
            // 2. Raft 达成共识后，由 Leader 执行链上操作
            let chain = state.chain.clone();
            match auth::set_pause(chain, paused).await {
                Ok(res) => {
                    let json_response = serde_json::json!({
                        "status": "success",
                        "signature": res.to_string()
                    });
                    (StatusCode::OK, Json(json_response)).into_response()
                }
                Err(_) => {
                    (StatusCode::INTERNAL_SERVER_ERROR, "chain set_pause error").into_response()
                }
            }
        }
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            format!("Raft Error: {:?}", e),
        )
            .into_response(),
    }
}

pub async fn raft_fee(
    State(state): State<Arc<AppState>>,
    Extension(pubkey): Extension<String>,
    Json(req): Json<FeeRequest>,
) -> impl IntoResponse {
    info!("raft fee pubkey: {}", pubkey);
    let Ok(_) = state.session.check_permission(&pubkey, Action::RaftFee) else {
        // 🌟 在这里必须显式返回一个 Response
        return (StatusCode::FORBIDDEN, "Permission denied").into_response();
    };
    info!("raft set fee base_fee: {}", req.base_fee);
    // 1. 提交 Raft 提案
    let op = KvOp::SetFee {
        base_fee: req.base_fee,
        fee_per_byte: req.fee_per_byte,
        scan_fee_per_item: req.scan_fee_per_item,
    };

    match state.raft.client_write(op).await {
        Ok(_) => {
            // 2. 链上执行
            let chain = state.chain.clone();
            match fee::set_fee(chain, req.base_fee, req.fee_per_byte, req.scan_fee_per_item).await {
                Ok(res) => {
                    let json_response = serde_json::json!({
                        "status": "success",
                        "signature": res.to_string()
                    });
                    (StatusCode::OK, Json(json_response)).into_response()
                }
                Err(_) => {
                    (StatusCode::INTERNAL_SERVER_ERROR, "chain set fee error").into_response()
                }
            }
        }
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            format!("Raft Error: {:?}", e),
        )
            .into_response(),
    }
}

/* #[allow(dead_code)]
pub async fn raft_gets(
    State(state): State<Arc<AppState>>,
    Query(req): Query<KVQuery>,
) -> impl IntoResponse {
    if let Err(err_resp) = ensure_leader_and_fresh(&state).await {
        return err_resp.into_response();
    }

    info!("get key: {}", req.key);
    let chain = state.chain.clone();
    let k = req.key.clone().into_bytes();
    // let _ = block_chain::get(chain, k).await;
    let value = match block_chain::get(chain, k.clone()).await {
        Ok(v) => v,
        // Ok(None) => return (StatusCode::NOT_FOUND, "key not found").into_response(),
        Err(e) => {
            let json_response = serde_json::json!({
                "status": "error",
                "signature": e.to_string()
            });
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json_response)).into_response();
        }
    };

    // 3. 获取全局的叶子哈希列表（用来找索引）
    let leaf_hashes = state.leaf_hashes.lock().await;

    // 4. 计算当前 key 的哈希
    let key_hash = Sha256::hash(&k);

    // 5. 查找 key 在 Merkle 树中的位置（索引）
    // 3. 找不到索引
    let Some(index) = leaf_hashes.iter().position(|h| h == &key_hash) else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "status": "error",
                "msg": "key not in merkle tree"
            })),
        )
            .into_response();
    };

    // 6. 获取全局 Merkle 树
    // 5. 获取树和根
    let tree = state.merkle_tree.lock().await;
    let proof = tree.proof(&[index].to_vec()); // 生成 Merkle 证明路径
    let proof_bytes: Vec<String> = proof
        .proof_hashes()
        .iter()
        .map(|h| hex::encode(h))
        .collect();
    let root = tree.root().unwrap_or_default();

    // 6. 不序列化 proof，直接返回验证所需的信息
    let json_response = serde_json::json!({
        "status": "success",
        "key": req.key,
        "value": value,
        "merkle_root": hex::encode(root),
        "key_hash": hex::encode(key_hash),
        "leaf_index": index,
        "proof": proof_bytes,
    });
    (StatusCode::OK, Json(json_response)).into_response()
}

#[allow(dead_code)]
pub async fn raft_scan(
    State(state): State<Arc<AppState>>,
    Json(req): Json<KVRequest>,
) -> impl IntoResponse {
    if let Err(err_resp) = ensure_leader_and_fresh(&state).await {
        return err_resp.into_response();
    }
    info!("scan key: {}, limit: {}", req.key, req.limit);
    let chain = state.chain.clone();
    let storage = state.storage.clone();
    let key = req.key.into_bytes();
    let l = req.limit;
    // 关键：用 match 处理 scan 的 Result，而不是直接 _
    // 1. 先拿到 scan 的结果（保持你原来的调用方式）
    let scan_result = match block_chain::scan(chain, storage.clone(), key, l).await {
        Ok(keys) => keys, // 假设这里返回 Vec<Vec<u8>>
        // Err(e) => return (StatusCode::BAD_REQUEST, format!("scan failed: {}", e)).into_response(),
        Err(e) => {
            let json_response = serde_json::json!({
                "status": "error",
                "signature": e.to_string()
            });
            return (StatusCode::BAD_REQUEST, Json(json_response)).into_response();
        }
    };
    // ===================== 新增 Merkle 证明部分 =====================
    // 2. 读取全局叶子哈希列表
    let leaf_hashes = state.leaf_hashes.lock().await;

    // 3. 批量找索引
    // 3. 批量找索引和哈希
    let mut indices = Vec::new();
    let mut key_hashes = Vec::new();
    for entry in &scan_result {
        let key = entry.0.clone();
        let key_hash = Sha256::hash(&key.as_bytes().to_vec());
        if let Some(idx) = leaf_hashes.iter().position(|h| h == &key_hash) {
            indices.push(idx);
            key_hashes.push(key_hash);
        }
    }

    // 4. 读取全局 Merkle 树，生成批量证明
    let tree = state.merkle_tree.lock().await;
    let proof = tree.proof(&indices); // 生成 Merkle 证明路径
    let proof_bytes: Vec<String> = proof
        .proof_hashes()
        .iter()
        .map(|h| hex::encode(h))
        .collect();
    let root = tree.root().unwrap_or_default();

    // 5. 返回批量验证所需的信息
    let json_response = serde_json::json!({
        "status": "success",
        "pairs": scan_result.iter().map(|(k, v)| serde_json::json!({
            "key": k,
            "value": v
        })).collect::<Vec<_>>(),
        "merkle_root": hex::encode(root),
        "key_hashes": key_hashes.iter().map(|h| hex::encode(h)).collect::<Vec<_>>(),
        "leaf_indices": indices,
        "proof": proof_bytes,
    });
    (StatusCode::OK, Json(json_response)).into_response()
}

#[allow(dead_code)]
pub async fn raft_page(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PageRequest>,
) -> impl IntoResponse {
    if let Err(err_resp) = ensure_leader_and_fresh(&state).await {
        return err_resp.into_response();
    }
    info!("page offset: {}, limit: {}", req.page, req.limit);
    let chain = state.chain.clone();
    let storage = state.storage.clone();
    let page = req.page;
    let limit: usize = req.limit;
    // let _ = block_chain::page(chain, storage, o, l).await;
    // 1. 拿到分页结果（保持你原来的调用方式）
    let result = match block_chain::page(chain, storage, page, limit).await {
        Ok(keys) => keys, // 假设这里返回 Vec<Vec<u8>>
        // Err(e) => return (StatusCode::BAD_REQUEST, format!("page failed: {}", e)).into_response(),
        Err(e) => {
            let json_response = serde_json::json!({
                "status": "error",
                "signature": e.to_string()
            });
            return (StatusCode::BAD_REQUEST, Json(json_response)).into_response();
        }
    };
    let page_result = result.0;
    let counter = result.1;

    // ===================== 新增 Merkle 证明部分 =====================
    let leaf_hashes = state.leaf_hashes.lock().await;

    // 3. 批量找索引和 key 哈希
    let mut indices = Vec::new();
    let mut key_hashes = Vec::new();
    for entry in &page_result {
        let key = &entry.0;
        let key_hash = rs_merkle::algorithms::Sha256::hash(&key.as_bytes().to_vec());
        if let Some(idx) = leaf_hashes.iter().position(|h| h == &key_hash) {
            indices.push(idx);
            key_hashes.push(key_hash);
        }
    }

    // 4. 获取全局 Merkle 树和根
    let tree = state.merkle_tree.lock().await;
    let proof = tree.proof(&indices);
    let proof_bytes: Vec<String> = proof
        .proof_hashes()
        .iter()
        .map(|h| hex::encode(h))
        .collect();
    let root = tree.root().unwrap_or_default();

    // ===================== 返回带证明的结果 =====================
    let json_response = serde_json::json!({
        "status": "success",
        "page": page,
        "limit": limit,
        "total": counter, // 🔥 这里返回链上真实总数！
        "pairs": page_result.iter().map(|(k, v)| serde_json::json!({
            "key": k,
            "value": v
        })).collect::<Vec<_>>(),
        "merkle_root": hex::encode(root),
        "key_hashes": key_hashes.iter().map(|h| hex::encode(h)).collect::<Vec<_>>(),
        "leaf_indices": indices,
        "proof": proof_bytes,
    });
    (StatusCode::OK, Json(json_response)).into_response()
}
 */

pub async fn raft_login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AuthRequest>,
) -> impl IntoResponse {
    info!("login");
    let auth_storage = state.session.auth_storage.clone();
    let Some(user_pubkey) = req.user_pubkey else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "chain login error").into_response();
    };
    let pubkey = user_pubkey.as_str();
    info!("pubkey: {}", user_pubkey);
    let Ok(res) = state.session.login(pubkey, auth_storage).await else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "raft login error").into_response();
    };

    let op = KvOp::SyncLogin {
        pubkey: user_pubkey.clone(),
        permissions: res.clone().1.permissions,
    };

    match state.raft.client_write(op).await {
        Ok(_) => {
            info!("login write raft ok");
            // 登录成功，返回用户信息
            let json = json!({
                "status": "success",
                "is_admin": res.clone().0, // 返回给前端
                "pubkey": user_pubkey.clone(),
                "permissions": res.clone().1.permissions
            });
            Json(json).into_response()
        }
        Err(e) => {
            // 🌟 错误响应也可以直接返回 Result 的 Err 分支
            // 或者手动构造一个 Response
            let error_response = (
                StatusCode::SERVICE_UNAVAILABLE,
                format!("Raft Consensus Error: {:?}", e),
            );
            error_response.into_response()
        }
    }
}

pub async fn raft_logout(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AuthRequest>,
) -> impl IntoResponse {
    let Some(user_pubkey) = req.user_pubkey else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "chain logout error").into_response();
    };
    let pubkey = user_pubkey.as_str();
    info!("logout pubkey: {}", pubkey);
    let Ok(res) = state.session.logout(pubkey).await else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "chain logout error").into_response();
    };

    let op = KvOp::SyncLogout {
        pubkey: user_pubkey,
    };

    match state.raft.client_write(op).await {
        Ok(_) => {
            // 登录成功，返回用户信息
            Json(res).into_response()
        }
        Err(e) => {
            // 🌟 错误响应也可以直接返回 Result 的 Err 分支
            // 或者手动构造一个 Response
            let error_response = (
                StatusCode::SERVICE_UNAVAILABLE,
                format!("Raft Consensus Error: {:?}", e),
            );
            error_response.into_response()
        }
    }
}

pub async fn raft_grant(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AuthRequest>,
) -> impl IntoResponse {
    let auth_storage = state.session.auth_storage.clone();
    let Some(admin_pubkey) = req.admin_pubkey else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "chain logout error").into_response();
    };
    let Some(user_pubkey) = req.user_pubkey else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "chain logout error").into_response();
    };
    let Some(perm_char) = req.perm_char else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "chain logout error").into_response();
    };
    info!("admin_pubkey: {}, user_pubkey: {}, perm_char: {:?}", admin_pubkey, user_pubkey, perm_char);
    let Ok(res) = state.session.grant_permission(
        admin_pubkey.as_str(),
        &user_pubkey,
        perm_char,
        auth_storage.clone(),
    )
    .await
    else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "chain logout error").into_response();
    };
    
    let op = KvOp::SyncGrant {
        user_pubkey,
        permissions: res.clone(),
    };
    
    match state.raft.client_write(op).await {
        Ok(_) => {
            info!("grant write raft ok");
            // 登录成功，返回用户信息
            Json(res).into_response()
        }
        Err(e) => {
            // 🌟 错误响应也可以直接返回 Result 的 Err 分支
            // 或者手动构造一个 Response
            let error_response = (
                StatusCode::SERVICE_UNAVAILABLE,
                format!("Raft Consensus Error: {:?}", e),
            );
            error_response.into_response()
        }
    }
}


pub async fn raft_revoke(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AuthRequest>,
) -> impl IntoResponse {
    let auth_storage = state.session.auth_storage.clone();
    let Some(admin_pubkey) = req.admin_pubkey else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "chain logout error").into_response();
    };
    let Some(user_pubkey) = req.user_pubkey else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "chain logout error").into_response();
    };
    info!("admin_pubkey: {}, user_pubkey: {}", admin_pubkey, user_pubkey);
    let Ok(res) = state.session.revoke_permission(admin_pubkey.as_str(), &user_pubkey, auth_storage.clone(),).await else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "chain logout error").into_response();
    };
    
    let op = KvOp::SyncRevoke { user_pubkey };
    
    match state.raft.client_write(op).await {
        Ok(_) => {
            info!("revoke write raft ok");
            // 登录成功，返回用户信息
            Json(res).into_response()
        }
        Err(e) => {
            // 🌟 错误响应也可以直接返回 Result 的 Err 分支
            // 或者手动构造一个 Response
            let error_response = (
                StatusCode::SERVICE_UNAVAILABLE,
                format!("Raft Consensus Error: {:?}", e),
            );
            error_response.into_response()
        }
    }
}
