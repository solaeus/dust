use crate::instruction::ByteType;

pub fn create_function_case(source: &str, return_type: ByteType) -> String {
    if return_type == ByteType::NONE {
        format!(
            r#"
            fn foobar() {{
                {source}
            }}
        "#
        )
    } else {
        format!(
            r#"
            fn foobar() -> {return_type} {{
                {source}
            }}
        "#
        )
    }
}

pub fn create_function_with_call_case(source: &str, return_type: ByteType) -> String {
    format!(
        r#"
        {}
        foobar()
        "#,
        create_function_case(source, return_type)
    )
}
