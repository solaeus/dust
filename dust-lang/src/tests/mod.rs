pub mod source_examples;

#[macro_export]
macro_rules! function_wrapper {
    ($content:expr) => {
        concat!("fn main() {\n    ", $content, "\n}").as_bytes()
    };
}
