use crate::function_wrapper;

pub const LET_STATEMENT: &[u8] = function_wrapper!("let x = 42;");
pub const LET_STATEMENT_WITH_TYPE: &[u8] = function_wrapper!("let x: int = 42;");
pub const LET_MUT_STATEMENT: &[u8] = function_wrapper!("let mut x = 42;");
pub const LET_MUT_STATEMENT_WITH_TYPE: &[u8] = function_wrapper!("let mut x: int = 42;");
