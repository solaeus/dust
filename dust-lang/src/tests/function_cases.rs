use crate::instruction::OperandType;

pub const ANONYMOUS_FUNCTION: &str = r#"
let add_one: fn(int) -> int = fn(x: int) -> int { x + 1 };

add_one(41)
"#;

pub fn create_function_case(source: &str, return_type: OperandType) -> String {
    if return_type == OperandType::NONE {
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

pub fn create_function_with_call_case(source: &str, return_type: OperandType) -> String {
    format!(
        r#"
        {}
        foobar()
        "#,
        create_function_case(source, return_type)
    )
}
