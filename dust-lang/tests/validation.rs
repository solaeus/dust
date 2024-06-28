use dust_lang::{
    error::{DustError, TypeConflict, ValidationError},
    identifier::Identifier,
    *,
};

#[test]
fn set_variable_with_type_error() {
    assert_eq!(
        interpret("test", "foobar: str = true")
            .unwrap_err()
            .errors(),
        &vec![DustError::Validation {
            error: ValidationError::TypeCheck {
                conflict: TypeConflict {
                    actual: Type::Boolean,
                    expected: Type::String
                },
                actual_position: (14, 18).into(),
                expected_position: Some((8, 11).into())
            },
            position: (0, 18).into()
        }]
    );
}

#[test]
fn function_return_type_error() {
    assert_eq!(
        interpret(
            "test",
            "
            foo = fn () -> str { 'foo' }

            bar: int = foo()
            "
        )
        .unwrap_err()
        .errors(),
        &vec![DustError::Validation {
            error: ValidationError::TypeCheck {
                conflict: TypeConflict {
                    actual: Type::String,
                    expected: Type::Integer
                },
                actual_position: (66, 71).into(),
                expected_position: Some((60, 63).into())
            },
            position: (55, 71).into()
        }]
    );
}

#[test]
fn scope() {
    assert_eq!(
        interpret(
            "test",
            "
            x = 1

            foo = fn () -> int {
                x
                1
            }

            foo()
            "
        )
        .unwrap_err()
        .errors(),
        &vec![DustError::Validation {
            error: ValidationError::VariableNotFound {
                identifier: Identifier::new("x"),
                position: (69, 70).into()
            },
            position: (32, 102).into()
        }]
    );
}
