use std::{fs::File, path::Path};
use solana_logger::{LogConfig, Logger};
use crate::error::Result;

pub fn log_config() -> Result<()> {
    // 1. 配置日志：同时输出到控制台 + 文件
    let log_path = Path::new("./logs/app.log"); // 日志文件路径
    std::fs::create_dir_all(log_path.parent().unwrap())?; // 自动创建 logs 目录

    let mut log_config = LogConfig::default();
    // 允许所有日志（替代你乱填的 "log"，直接全开）
    log_config.add_filter_allow_str(""); 
    // 配置文件输出：追加模式，不覆盖历史日志
    log_config.file = Some(log_path.to_path_buf());
    log_config.file_append = true;

    // 2. 初始化日志
    solana_logger::setup_with_config(&log_config);
    Ok(())
}
