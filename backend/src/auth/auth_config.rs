use crate::{
    auth::types::{Action, UserSession, action_to_char, init, split_char},
    config::env_config::Config,
    error::{Error, Result},
    storage::engine::DiskClient,
    utils::parse_keypair_array,
};
use dashmap::DashMap;
use log::info;
use solana_sdk::{signature::Keypair, signer::Signer};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

// 这里的 DiskClient 就是你已经写好的那个
#[derive(Clone)]
pub struct SessionManager {
    config: Config,
    pub sessions: Arc<DashMap<String, UserSession>>, // 内存会话缓存
    pub auth_storage: Arc<Mutex<DiskClient>>,
}

impl SessionManager {
    // 初始化会话管理器（复用你的 DiskClient）
    pub fn new(config: Config, auth_storage: Arc<Mutex<DiskClient>>) -> Self {
        let sessions = Arc::new(DashMap::new());
        Self {
            sessions,
            config,
            auth_storage,
        }
    }

    // ------------------------------
    // 1. 初始化权限系统（必须第一个执行）
    // ------------------------------
    pub async fn init_auth(&mut self) -> Result<()> {
        // 超级管理员拥有所有权限
        let mut storage = self.auth_storage.lock().await;
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
        self.sessions.insert(pubkey.clone(), admin);
        info!("admin 地址是: {}", pubkey);
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
        info!("auth config: {}", pubkey.to_string());
        let mut storage = auth_storage.lock().await;
        let memory_keys: Vec<String> = self
            .sessions
            .iter()
            .map(|entry| entry.key().clone())
            .collect();
        info!(
            "login current active session memory keys: {:?}",
            memory_keys
        );
        let disk_keys: Vec<String> = storage
            .get_keys()
            .map_err(|_| Error::InvalidKey)?
            .into_iter()
            .map(|item| String::from_utf8(item).map_err(|_| Error::InvalidKey))
            .collect::<Result<Vec<String>>>()?; // 🌟 3. 处理转换可能失败的情况        
        info!("login current active session disk keys: {:?}", disk_keys);
        if let Some(mut user) = self.sessions.get_mut(&pubkey.to_string()) {
            info!("memory pubkey: {}, user: {:?}", pubkey.to_string(), user);
            user.is_logged_in = true;
            return Ok(user.clone());
        }

        // 2. 内存没有，去磁盘查
        if let Some(v) = storage.get(pubkey.as_bytes().to_vec())? {
            let value = String::from_utf8(v).map_err(|_| Error::InvalidKey)?;
            info!("disk pubkey: {}, value: {}", pubkey.to_string(), value);
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
    pub async fn logout(&self, pubkey: &str) -> Result<String> {
        let permissions = pubkey.to_string();
        match self.sessions.get_mut(&permissions) {
            Some(mut user) => {
                user.is_logged_in = false;
                Ok(permissions)
            }
            None => Err(Error::UserDoesNotExistError),
        }
    }

    // ------------------------------
    // 4. 核心：接口权限校验（纯内存操作，超快）
    // ------------------------------
    pub fn check_permission(&self, pubkey: &str, action: Action) -> Result<()> {
        // 判断是管理员
        if pubkey.to_string() == self.admin_pubkey()?.to_string() {
            return Ok(());
        }
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
        &self,
        admin_pubkey: &str,
        user_pubkey: &str,
        perm_char: Vec<Action>,
        auth_storage: Arc<Mutex<DiskClient>>,
    ) -> Result<Vec<Action>> {
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
        info!(
            "grant admin_pubkey: {}, user_pubkey: {}, perm_char: {:?}",
            admin_pubkey.to_string(),
            user_pubkey.to_string(),
            value_str
        );
        let _ = storage.set(key, value_str.as_bytes().to_vec());
        let user = UserSession {
            is_logged_in: false,
            permissions: perm_char.clone(),
        };
        // 尝试获取现有会话并只更新权限，保留登录状态
        if let Some(mut session) = self.sessions.get_mut(&user_pubkey.to_string()) {
            session.permissions = perm_char.clone();
        } else {
            // 如果用户不在内存里，再执行你现在的 insert 逻辑
            self.sessions.insert(user_pubkey.to_string(), user);
        }
        let memory_keys: Vec<String> = self
            .sessions
            .iter()
            .map(|entry| entry.key().clone())
            .collect();
        info!(
            "grant current active session memory keys: {:?}",
            memory_keys
        );
        let disk_keys: Vec<String> = storage
            .get_keys()
            .map_err(|_| Error::InvalidKey)?
            .into_iter()
            .map(|item| String::from_utf8(item).map_err(|_| Error::InvalidKey))
            .collect::<Result<Vec<String>>>()?; // 🌟 3. 处理转换可能失败的情况        
        info!("grant current active session disk keys: {:?}", disk_keys);
        Ok(perm_char)
    }

    // ------------------------------
    // 6. 管理员取消授权（更新 DiskClient + 同步内存）
    // ------------------------------
    pub async fn revoke_permission(
        &self,
        admin_pubkey: &str,
        user_pubkey: &str,
        auth_storage: Arc<Mutex<DiskClient>>,
    ) -> Result<String> {
        let Ok(public_key) = self.admin_pubkey() else {
            return Err(Error::UserDoesNotExistError);
        };
        if admin_pubkey.is_empty()
            || (!admin_pubkey.is_empty() && admin_pubkey.to_string() != public_key.to_string())
            || user_pubkey.is_empty()
        {
            return Err(Error::UserDoesNotExistError);
        }
        let mut storage = auth_storage.lock().await;

        let key = user_pubkey.as_bytes().to_vec();

        info!(
            "revoke admin_pubkey: {}, user_pubkey: {}",
            admin_pubkey.to_string(),
            user_pubkey.to_string(),
        );
        let _ = storage.delete(key.clone());
        self.sessions.remove(user_pubkey);

        let memory_keys: Vec<String> = self
            .sessions
            .iter()
            .map(|entry| entry.key().clone())
            .collect();
        info!(
            "revoke current active session memory keys: {:?}",
            memory_keys
        );
        let disk_keys: Vec<String> = storage
            .get_keys()
            .map_err(|_| Error::InvalidKey)?
            .into_iter()
            .map(|item| String::from_utf8(item).map_err(|_| Error::InvalidKey))
            .collect::<Result<Vec<String>>>()?; // 🌟 3. 处理转换可能失败的情况        
        info!("revoke current active session disk keys: {:?}", disk_keys);
        Ok(user_pubkey.to_string())
    }

    pub async fn admin_page(
        &self,
        mut page: usize,
        limit: usize,
        auth_storage: Arc<Mutex<DiskClient>>,
    ) -> Result<(usize, HashMap<String, Vec<Action>>)> {
        if page == 0 { page = 1; }
        let admin_pub = self.admin_pubkey().map_err(|_| Error::UserDoesNotExistError)?;

        // --- 1. 构造去重后的全量逻辑列表 ---
        // 使用 BTreeSet 可以保证公钥排序稳定，分页时不会乱序
        let mut all_keys_set = std::collections::BTreeSet::new();

        // 注入磁盘所有的 Key
        let mut storage = auth_storage.lock().await;
        let disk_keys = storage.get_keys()?; // 确保你底层有这个获取所有 key 的方法
        for k in disk_keys {
            let key_str = String::from_utf8(k).map_err(|_| Error::InvalidKey)?;
            if key_str != admin_pub {
                all_keys_set.insert(key_str);
            }
        }

        // 注入内存所有的 Key (自动去重)
        for entry in self.sessions.iter() {
            let k = entry.key();
            if k != &admin_pub {
                all_keys_set.insert(k.clone());
            }
        }

        let total_count = all_keys_set.len(); // 🌟 准确的去重总数
        let all_keys: Vec<String> = all_keys_set.into_iter().collect();

        // --- 2. 计算切片范围 ---
        let start = (page - 1) * limit;
        let mut result_map = HashMap::new();

        if start < total_count {
            let end = std::cmp::min(start + limit, total_count);
            let target_keys = &all_keys[start..end];

            // --- 3. 填充数据：优先从内存取，内存没有再去磁盘取 ---
            for key in target_keys {
                if let Some(session) = self.sessions.get(key) {
                    // 内存命中
                    result_map.insert(key.clone(), session.permissions.clone());
                } else {
                    // 内存未命中，从磁盘读取具体 Value
                    if let Ok(Some(v)) = storage.get(key.as_bytes().to_vec()) {
                        let val_str = String::from_utf8(v).map_err(|_| Error::InvalidKey)?;
                        result_map.insert(key.clone(), split_char(&val_str)?);
                    }
                }
            }
        }

        info!("Admin Page: page={}, limit={}, total={}", page, limit, total_count);
        Ok((total_count, result_map))
    }

    pub async fn admin_get(
        &self,
        pubkey: String,
        limit: usize,
        auth_storage: Arc<Mutex<DiskClient>>,
    ) -> Result<HashMap<String, Vec<Action>>> {
        let mut result_map = HashMap::new();
        let admin_pub = self.admin_pubkey().map_err(|_| Error::UserDoesNotExistError)?;

        // --- 1. 优先从内存筛选匹配项 ---
        for entry in self.sessions.iter() {
            if result_map.len() >= limit { break; } // 🌟 严格遵守 limit
            
            let k = entry.key();
            if k.starts_with(&pubkey) && k != &admin_pub {
                result_map.insert(k.clone(), entry.value().permissions.clone());
            }
        }

        // --- 2. 如果内存没凑够，从磁盘扫描补齐 ---
        if result_map.len() < limit {
            let remaining = limit - result_map.len();
            let mut storage = auth_storage.lock().await;
            let scan_data = storage.scan_prefix(pubkey.as_bytes().to_vec());

            // 使用 try_for_each 在凑够数量后可以立即停止扫描
            scan_data.take(remaining).try_for_each(|item| -> Result<()> {
                let (k, v) = item.map_err(|_| Error::InvalidKey)?;
                let key_str = String::from_utf8(k).map_err(|_| Error::InvalidKey)?;
                
                // 排除管理员，且只添加内存中不存在的 key
                if key_str != admin_pub && !result_map.contains_key(&key_str) {
                    let val_str = String::from_utf8(v).map_err(|_| Error::InvalidKey)?;
                    result_map.insert(key_str, split_char(&val_str)?);
                }
                Ok(())
            })?;
        }

        info!("Admin Get: prefix='{}', found={}", pubkey, result_map.len());
        Ok(result_map)
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
