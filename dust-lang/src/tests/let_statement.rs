use crate::function_wrapper;

pub const WITHOUT_TYPE: &[u8] = function_wrapper!("let x = 42;");
pub const WITH_TYPE: &[u8] = function_wrapper!("let x: int = 42;");
pub const WITHOUT_TYPE_MUT: &[u8] = function_wrapper!("let mut x = 42;");
pub const WITH_TYPE_MUT: &[u8] = function_wrapper!("let mut x: int = 42;");
