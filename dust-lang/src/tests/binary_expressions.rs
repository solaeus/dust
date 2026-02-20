use crate::function_wrapper;

pub const ADDITION: &[u8] = function_wrapper!("x + y");
pub const SUBTRACTION: &[u8] = function_wrapper!("x - y");
pub const MULTIPLICATION: &[u8] = function_wrapper!("x * y");
pub const DIVISION: &[u8] = function_wrapper!("x / y");
pub const MODULO: &[u8] = function_wrapper!("x % y");
pub const POWER: &[u8] = function_wrapper!("x ^ y");

pub const EQUAL: &[u8] = function_wrapper!("x == y");
pub const NOT_EQUAL: &[u8] = function_wrapper!("x != y");
pub const LESS_THAN: &[u8] = function_wrapper!("x < y");
pub const LESS_THAN_OR_EQUAL: &[u8] = function_wrapper!("x <= y");
pub const GREATER_THAN: &[u8] = function_wrapper!("x > y");
pub const GREATER_THAN_OR_EQUAL: &[u8] = function_wrapper!("x >= y");

pub const LOGICAL_AND: &[u8] = function_wrapper!("x && y");
pub const LOGICAL_OR: &[u8] = function_wrapper!("x || y");
