use crate::function_wrapper;

pub const ADD_ASSIGN: &[u8] = function_wrapper!("x += 42;");
pub const SUBTRACT_ASSIGN: &[u8] = function_wrapper!("x -= 42;");
pub const MULTIPLY_ASSIGN: &[u8] = function_wrapper!("x *= 42;");
pub const DIVIDE_ASSIGN: &[u8] = function_wrapper!("x /= 42;");
pub const MODULO_ASSIGN: &[u8] = function_wrapper!("x %= 42;");
pub const POWER_ASSIGN: &[u8] = function_wrapper!("x ^= 42;");
