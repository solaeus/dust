use crate::function_wrapper;

pub const EMPTY: &[u8] = function_wrapper!("{}");
pub const ITEM: &[u8] = function_wrapper!("{ fn foo() {} }");
pub const STATEMENT: &[u8] = function_wrapper!("{ let x = 42; }");
pub const EXPRESSION: &[u8] = function_wrapper!("{ x + y }");
pub const MIXED: &[u8] = function_wrapper!("{ fn foo() {} let x = 42; x + y }");
