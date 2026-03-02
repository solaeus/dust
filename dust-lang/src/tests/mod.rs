pub mod source_examples;

#[macro_export]
macro_rules! function_wrapper {
    ($content:expr) => {
        concat!("fn main() {\n    ", $content, "\n}").as_bytes()
    };
}

pub const REASSIGNMENT_STATEMENT: &[u8] = function_wrapper!("x = 42;");

pub const GROUPED_EXPRESSION: &[u8] = function_wrapper!("(x + y)");
