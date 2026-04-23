use crate::error::{Error, Result};
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, TermLogger, TerminalMode, WriteLogger};
use std::path::Path;
use time::macros::format_description;

pub fn log_config(path: &str) -> Result<()> {
    // let loglevel = cfg.log_level.parse()?;
    let mut logconfig = simplelog::ConfigBuilder::new();
    // 第一步：尝试设置本地时区
    // 如果失败了，它什么都不做，继续保持默认的 UTC
    let _ = logconfig.set_time_offset_to_local();
    logconfig.set_time_format_custom(format_description!(
        "[year]-[month]-[day] [hour]:[minute]:[second]"
    ));
    let log_path = Path::new(path);
    CombinedLogger::init(vec![
        // 控制台彩色日志（0.12.x 仅4个参数）
        TermLogger::new(
            LevelFilter::Info,
            logconfig.build(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        // 文件追加日志（0.12.x 仅3个参数）
        WriteLogger::new(
            LevelFilter::Info,
            logconfig.build(),
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_path)
                .map_err(|e| Error::LogError(format!("打开日志文件失败: {}", e)))?,
        ),
    ])
    .map_err(|e| Error::LogError(format!("日志初始化失败: {}", e)))?;
    log::info!("✅ 服务启动，日志已写入文件: {}", log_path.display());
    // log::debug!("🔍 调试日志测试");
    // log::error!("❌ 错误日志测试");
    Ok(())
}
