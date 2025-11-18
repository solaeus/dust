pub mod block_cases;
pub mod constant_cases;
pub mod if_else_cases;
pub mod local_cases;
pub mod loop_cases;

pub fn create_function_case(source: &str) -> String {
    format!(
        r#"
            fn foobar() {{
                {source}
            }}
        "#
    )
}

pub fn create_function_with_call_case(source: &str, return_type: &str) -> String {
    if return_type.is_empty() {
        format!(
            r#"
            fn foobar() {{
                {source}
            }}

            foobar()
        "#
        )
    } else {
        format!(
            r#"
            fn foobar() -> {return_type} {{
                {source}
            }}

            foobar()
        "#
        )
    }
}
