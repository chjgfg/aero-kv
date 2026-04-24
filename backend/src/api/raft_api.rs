use std::sync::{Arc, Mutex};

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use log::info;
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use tokio::sync::oneshot;

use crate::{
    auth,
    block_chain::{self, client::ChainClient},
    fee,
    raft::types::{AppContext, Command, Envelope, Message, RaftTask},
    storage::engine::DiskClient,
};

pub async fn kv_set(
    State(ctx): State<Arc<AppContext>>,
    Json(payload): Json<Command>,
) -> impl IntoResponse {
    let (res_tx, res_rx) = oneshot::channel();
    let request_id = rand::random::<u64>();
    println!("payload:{:?}", payload);
    let task = RaftTask {
        id: request_id,
        message: Message::ClientRequest {
            id: request_id,
            request: serde_json::to_vec(&payload).unwrap(),
        },
        response_tx: res_tx,
    };

    if let Err(e) = ctx.task_sender.send(task) {
        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }

    match res_rx.await {
        Ok(Ok(_)) => Json(json!({"status": "success"})).into_response(),
        Ok(Err(e)) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", e)).into_response(),
        Err(_) => (StatusCode::REQUEST_TIMEOUT, "Raft timeout").into_response(),
    }
}

// 1. 定义接收 Raft 消息的 Handler
pub async fn handle_raft_msg(
    State(ctx): State<Arc<AppContext>>,
    Json(envelope): Json<Envelope>,
) -> impl IntoResponse {
    // 💡 加上这行打印，你就能在终端看到节点之间是否在通信了
    // println!("📩 Received Raft message from Node {} to Node {}", envelope.from, envelope.to);

    // 将收到的信封塞进 Server 线程的 peers_rx 通道
    ctx.peers_sender.send(envelope).ok();
    axum::http::StatusCode::OK
}

// -------------------------------------------------------------------------------

pub async fn raft_upsert(
    chain: Arc<ChainClient>,
    storage: Arc<Mutex<DiskClient>>,
    key: Vec<u8>,
    value: Vec<u8>,
) -> impl IntoResponse {
    let _ = block_chain::upsert(chain, storage, key, value).await;
    // 这里写你的 upsert 业务逻辑
    (StatusCode::OK, "upsert success")
}

pub async fn raft_delete(
    chain: Arc<ChainClient>,
    storage: Arc<Mutex<DiskClient>>,
    key: Vec<u8>,
) -> impl IntoResponse {
    info!("delete key: {:?}", key);
    let _ = block_chain::delete(chain, storage, key).await;
    (StatusCode::OK, "delete success")
}

pub async fn raft_admin(chain: Arc<ChainClient>, new_admin: String) -> impl IntoResponse {
    info!("set admin new_admin: {}", new_admin);
    let pubkey = Pubkey::from_str(new_admin.as_str()).unwrap();
    let _ = auth::set_admin(chain, pubkey).await;
    (StatusCode::OK, "set admin success")
}

pub async fn raft_pause(chain: Arc<ChainClient>, paused: bool) -> impl IntoResponse {
    info!("set pause paused: {}", paused);
    let _ = auth::set_pause(chain, paused).await;
    (StatusCode::OK, "set pause success")
}

pub async fn raft_fee(
    chain: Arc<ChainClient>,
    base_fee: u64,
    fee_per_byte: u64,
    scan_fee_per_item: u64,
) -> impl IntoResponse {
    info!(
        "set fee base_fee: {}, fee_per_byte: {}, scan_fee_per_item: {}",
        base_fee, fee_per_byte, scan_fee_per_item
    );
    let _ = fee::set_fee(chain, base_fee, fee_per_byte, scan_fee_per_item).await;
    (StatusCode::OK, "set fee success")
}
