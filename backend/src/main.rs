use crate::{config::Config, error::Error};

mod api;
mod auth;
mod chain;
mod config;
mod constants;
mod core;
mod error;
mod fee;
mod utils;

use crate::error::Result;

fn main() -> Result<()> {
    let config = Config::from_env().map_err(|e| Error::ConfigError("()".to_string()))?;
    println!("{:?}", config);
    Ok(())
}
