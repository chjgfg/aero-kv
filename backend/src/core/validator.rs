// # 数据校验：key长度、value大小

use crate::constants::{MAX_KEY_LENGTH, MAX_VALUE_LENGTH};

// ------------------------------
// 1. Key 长度校验
// ------------------------------
pub fn check_key(key: &str) -> bool {
    !key.is_empty() && key.len() <= MAX_KEY_LENGTH
}

// ------------------------------
// 2. Value 长度校验
// ------------------------------
pub fn check_value(value: &str) -> bool {
    !value.is_empty() && value.len() <= MAX_VALUE_LENGTH
}
