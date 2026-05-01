use crate::{
    auth::types::{Action, UserSession, action_to_char, init, split_char},
    config::env_config::Config,
    error::{Error, Result},
    storage::engine::DiskClient,
    utils::parse_keypair_array,
};
use dashmap::DashMap;
use solana_sdk::{signature::Keypair, signer::Signer};
use std::sync::Arc;
use tokio::sync::Mutex;

// 这里的 DiskClient 就是你已经写好的那个
#[derive(Clone)]
pub struct SessionManager {
    config: Config,
    pub sessions: Arc<DashMap<String, UserSession>>, // 内存会话缓存
}

impl SessionManager {
    // 初始化会话管理器（复用你的 DiskClient）
    pub fn new(config: Config) -> Self {
        let sessions = Arc::new(DashMap::new());
        Self { sessions, config }
    }

    // ------------------------------
    // 1. 初始化权限系统（必须第一个执行）
    // ------------------------------
    pub async fn init_auth(&mut self, auth_storage: Arc<Mutex<DiskClient>>) -> Result<()> {
        // 超级管理员拥有所有权限
        let mut storage = auth_storage.lock().await;
        for item in storage.scan(..) {
            if let Ok((k, v)) = item {
                let key = String::from_utf8(k).expect("无效公钥");
                let value = String::from_utf8(v).expect("无效权限");
                let Ok(actions) = split_char(value.as_str()) else {
                    continue;
                };
                let user = UserSession {
                    is_logged_in: false,
                    permissions: actions,
                };
                self.sessions.insert(key, user);
            }
        }
        let admin = UserSession {
            is_logged_in: true,
            permissions: init(),
        };
        let Ok(pubkey) = self.admin_pubkey() else {
            return Err(Error::UserDoesNotExistError);
        };
        println!("Payer 地址是: {}", pubkey);
        self.sessions.insert(pubkey, admin);
        Ok(())
    }

    // ------------------------------
    // 2. 用户登录（从 DiskClient 加载权限到内存）
    // ------------------------------
    pub async fn login(
        &self,
        pubkey: &str,
        auth_storage: Arc<Mutex<DiskClient>>,
    ) -> Result<UserSession> {
        // 1. 先看内存有没有
        if let Some(mut user) = self.sessions.get_mut(pubkey) {
            user.is_logged_in = true;
            return Ok(user.clone());
        }

        // 2. 内存没有，去磁盘查
        let mut storage = auth_storage.lock().await;
        if let Some(v) = storage.get(pubkey.as_bytes().to_vec())? {
            let value = String::from_utf8(v).map_err(|_| Error::InvalidKey)?;
            let actions = split_char(&value)?;
            let user = UserSession {
                is_logged_in: true,
                permissions: actions,
            };
            // 同步回内存
            self.sessions.insert(pubkey.to_string(), user.clone());
            return Ok(user);
        }
        Err(Error::UserDoesNotExistError)
    }

    // ------------------------------
    // 3. 用户登出（清除内存会话）
    // ------------------------------
    pub fn logout(&self, pubkey: &str) -> Result<()> {
        let permissions = pubkey.to_string();
        match self.sessions.get_mut(&permissions) {
            Some(mut user) => {
                user.is_logged_in = false;
                Ok(())
            }
            None => Err(Error::UserDoesNotExistError),
        }
    }

    // ------------------------------
    // 4. 核心：接口权限校验（纯内存操作，超快）
    // ------------------------------
    pub fn check_permission(&self, pubkey: &str, action: Action) -> Result<()> {
        // 1. 检查用户是否在线
        let session = self.sessions.get(pubkey).ok_or_else(|| Error::UserLogout)?;

        if !session.is_logged_in {
            return Err(Error::UserLogout);
        }
        // 3. 检查权限
        if session.permissions.contains(&action) {
            Ok(())
        } else {
            Err(Error::PermissionDoesNotExistError)
        }
    }

    // ------------------------------
    // 5. 管理员授权（更新 DiskClient + 同步内存）
    // ------------------------------
    pub async fn grant_permission(
        &mut self,
        admin_pubkey: &str,
        user_pubkey: &str,
        perm_char: Vec<Action>,
        auth_storage: Arc<Mutex<DiskClient>>,
    ) -> Result<()> {
        let Ok(public_key) = self.admin_pubkey() else {
            return Err(Error::UserDoesNotExistError);
        };
        if admin_pubkey.is_empty()
            || (!admin_pubkey.is_empty() && admin_pubkey.to_string() != public_key.to_string())
            || user_pubkey.is_empty()
        {
            return Err(Error::UserDoesNotExistError);
        }
        // if perm_char.is_empty() {
        //     return Err(Error::PermissionDoesNotExistError);
        // }
        let mut storage = auth_storage.lock().await;

        // 2. 从 DiskClient 读取用户现有权限
        let key = user_pubkey.as_bytes().to_vec();
        let mut value_str = String::new();
        for action in perm_char.clone() {
            let chars: &str =
                action_to_char(action).map_err(|_| Error::PermissionDoesNotExistError)?;
            // 2. 使用 push_str 拼接 &str
            value_str.push_str(chars);
        }
        storage.set(key, value_str.as_bytes().to_vec());
        if let Some(mut user_session) = self.sessions.get_mut(&user_pubkey.to_string()) {
            user_session.permissions = perm_char;
        };
        Ok(())
    }

    fn admin_pubkey(&self) -> Result<String> {
        let key_bytes = parse_keypair_array(self.config.payer_keypair.as_str())?;
        let payer = Arc::new(Keypair::from_bytes(&key_bytes).map_err(|_| Error::InvalidKey)?);
        // 提取公钥
        let public_key = payer.pubkey();
        let pubkey = public_key.to_string();
        Ok(pubkey)
    }
}
