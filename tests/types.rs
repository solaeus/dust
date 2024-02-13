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
                end_byte: 12,
                start_row: 1,
                start_column: 0,
                end_row: 1,
                end_column: 12,
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
        Err(Error::Validation(
            ValidationError::ExpectedFunctionArgumentAmount {
                expected: 1,
                actual: 0,
                position: SourcePosition {
                    start_byte: 81,
                    end_byte: 86,
                    start_row: 5,
                    start_column: 12,
                    end_row: 5,
                    end_column: 17
                }
            }
        )),
        result
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
                start_byte: 91,
                end_byte: 108,
                start_row: 5,
                start_column: 12,
                end_row: 5,
                end_column: 29,
            }
        })),
        result
    );
}
