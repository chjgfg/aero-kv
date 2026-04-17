// # 链交互模块（对应合约 storage）

use std::str::FromStr;

use solana_sdk::pubkey::Pubkey;

use crate::error::{Error, Result};

pub fn parse_keypair_array(s: &str) -> Result<Vec<u8>> {
    // 去掉前后的 []，按逗号分割，解析成 u8
    let s = s
        .trim()
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .ok_or(Error::InvalidKey)?;
    s.split(',')
        .map(|part| part.trim().parse::<u8>().map_err(|_| Error::InvalidKey))
        .collect()
}
