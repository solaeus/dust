use dust_lang::{error::ValidationError, *};

#[test]
fn simple_assignment() {
    let test = interpret("x = 1 x");

    assert_eq!(Ok(Value::Integer(1)), test);
}

#[test]
fn simple_assignment_with_type() {
    let test = interpret("x <int> = 1 x");

    assert_eq!(Ok(Value::Integer(1)), test);
}

#[test]
fn list_add_assign() {
    let test = interpret(
        "
            x <[int]> = []
            x += 1
            x
            ",
    );

    assert_eq!(
        Ok(Value::List(List::with_items(vec![Value::Integer(1)]))),
        test
    );
}

#[test]
fn list_add_wrong_type() {
    let result = interpret(
        "
            x <[str]> = []
            x += 1
            ",
    );

    assert_eq!(
        Err(Error::Validation(ValidationError::TypeCheck {
            expected: Type::String,
            actual: Type::Integer,
            position: SourcePosition {
                start_byte: 40,
                end_byte: 46,
                start_row: 3,
                start_column: 12,
                end_row: 3,
                end_column: 18
            }
        })),
        result
    );
}
