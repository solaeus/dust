pub mod binary_assignment_statements;
pub mod binary_expressions;
pub mod block_expression;
pub mod enum_item;
pub mod function_item;
pub mod if_expression;
pub mod let_statement;
pub mod mod_item;
pub mod struct_expression;
pub mod struct_item;
pub mod unary_expressions;
pub mod value_expressions;

#[macro_export]
macro_rules! function_wrapper {
    ($content:expr) => {
        concat!("fn main() {\n    ", $content, "\n}").as_bytes()
    };
}

pub const REASSIGNMENT_STATEMENT: &[u8] = function_wrapper!("x = 42;");

pub const GROUPED_EXPRESSION: &[u8] = function_wrapper!("(x + y)");
