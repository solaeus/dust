use dust_lang::{error::ValidationError, *};

#[test]
fn simple_type_check() {
    let result = interpret("x <bool> = 1");

    assert_eq!(
        Err(Error::Validation(ValidationError::TypeCheck {
            expected: Type::Boolean,
            actual: Type::Integer,
            position: SourcePosition {
                start_byte: 0,
                end_byte: 0,
                start_row: 0,
                start_column: 0,
                end_row: 0,
                end_column: 0,
            }
        })),
        result
    );
}

#[test]
fn argument_count_check() {
    let source = "
            foo = (x <int>) <int> {
                x
            }
            foo()
            ";
    let result = interpret(source);

    assert_eq!(
        "Expected 1 arguments, but got 0. Occured at (5, 12) to (5, 17). Source: foo()",
        result.unwrap_err().to_string()
    )
}

#[test]
fn callback_type_check() {
    let result = interpret(
        "
            x = (cb <() -> bool>) <bool> {
                cb()
            }
            x(() <int> { 1 })
            ",
    );

    assert_eq!(
        Err(Error::Validation(ValidationError::TypeCheck {
            expected: Type::Function {
                parameter_types: vec![],
                return_type: Box::new(Type::Boolean),
            },
            actual: Type::Function {
                parameter_types: vec![],
                return_type: Box::new(Type::Integer),
            },
            position: SourcePosition {
                start_byte: 0,
                end_byte: 0,
                start_row: 0,
                start_column: 0,
                end_row: 0,
                end_column: 0
            }
        })),
        result
    );
}
