use crate::function_wrapper;

pub const BOOLEAN: &[u8] = function_wrapper!("true");
pub const BYTE: &[u8] = function_wrapper!("0x2A");
pub const CHARACTER: &[u8] = function_wrapper!("'a'");
pub const FLOAT: &[u8] = function_wrapper!("42.0");
pub const INTEGER: &[u8] = function_wrapper!("42");
pub const STRING: &[u8] = function_wrapper!("\"Hello, world!\"");
pub const LIST: &[u8] = function_wrapper!("[1, 2, 3]");
pub const FUNCTION: &[u8] = function_wrapper!("fn() {}");
