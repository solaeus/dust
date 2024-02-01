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
