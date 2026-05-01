use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

// 你的操作枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum Action {
    InitAuth,
    InitStorage,
    InitCounter,
    InitFee,
    RaftUpsert,
    RaftDelete,
    Get,
    Scan,
    Page,
    RaftPause,
    RaftFee,
}

pub fn init() -> Vec<Action> {
    let mut init_action: Vec<Action> = Vec::new();
    init_action.push(Action::InitAuth);
    init_action.push(Action::InitStorage);
    init_action.push(Action::InitCounter);
    init_action.push(Action::InitFee);
    init_action.push(Action::RaftUpsert);
    init_action.push(Action::RaftDelete);
    init_action.push(Action::Get);
    init_action.push(Action::Scan);
    init_action.push(Action::Page);
    init_action.push(Action::RaftPause);
    init_action.push(Action::RaftFee);
    init_action
}

pub fn action_to_char(action: Action) -> Result<&'static str> {
    let perm_char = match action {
        Action::InitAuth => "A",
        Action::InitStorage => "S",
        Action::InitCounter => "C",
        Action::InitFee => "F",
        Action::RaftUpsert => "W",
        Action::RaftDelete => "D",
        Action::Get => "R",
        Action::Scan => "N",
        Action::Page => "P",
        Action::RaftPause => "M",
        Action::RaftFee => "E",
    };
    Ok(perm_char)
}

pub fn char_to_action(ch: &str) -> Result<Action> {
    let perm_action = match ch {
        "A" => Action::InitAuth,
        "S" => Action::InitStorage,
        "C" => Action::InitCounter,
        "F" => Action::InitFee,
        "W" => Action::RaftUpsert,
        "D" => Action::RaftDelete,
        "R" => Action::Get,
        "N" => Action::Scan,
        "P" => Action::Page,
        "M" => Action::RaftPause,
        "E" => Action::RaftFee,
        _ => {
            return Err(Error::UnsupportedCharacter(
                "Unsupported action character".to_string(),
            ));
        }
    };
    Ok(perm_action)
}

pub fn split_char(str: &str) -> Result<Vec<Action>> {
    let str_arr = str.split("");
    let mut actions: Vec<Action> = Vec::new();
    for item in str_arr {
        let Ok(action) = char_to_action(item) else {
            return Err(Error::UnsupportedCharacter(
                "Unsupported action character".to_string(),
            ));
        };
        actions.push(action);
    }
    Ok(actions)
}

// 在线用户会话（内存）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub is_logged_in: bool,
    pub permissions: Vec<Action>, // 登录时从 DiskClient 加载
}
