use std::rc::Rc;

use dust_lang::{
    abstract_tree::{Block, Expression, Identifier, Statement, Type, WithPos},
    error::{Error, TypeConflict, ValidationError},
    *,
};

#[test]
fn set_and_get_variable() {
    assert_eq!(
        interpret(Rc::new("test".to_string()), "foobar = true; foobar"),
        Ok(Some(Value::boolean(true)))
    );
}

#[test]
fn set_variable_with_type() {
    assert_eq!(
        interpret(Rc::new("test".to_string()), "foobar: bool = true; foobar"),
        Ok(Some(Value::boolean(true)))
    );
}

#[test]
fn set_variable_with_type_error() {
    assert_eq!(
        interpret(Rc::new("test".to_string()), "foobar: str = true")
            .unwrap_err()
            .errors(),
        &vec![Error::Validation {
            error: ValidationError::TypeCheck {
                conflict: TypeConflict {
                    actual: Type::Boolean,
                    expected: Type::String
                },
                actual_position: (14, 18).into(),
                expected_position: (8, 11).into()
            },
            position: (0, 18).into()
        }]
    );
}

#[test]
fn function_variable() {
    assert_eq!(
        interpret(
            Rc::new("test".to_string()),
            "foobar = (x: int) int { x }; foobar"
        ),
        Ok(Some(Value::function(
            Vec::with_capacity(0),
            vec![(Identifier::new("x"), Type::Integer.with_position((13, 16)))],
            Type::Integer.with_position((18, 21)),
            Block::new(vec![Statement::Expression(Expression::Identifier(
                Identifier::new("x")
            ))
            .with_position((24, 25))])
            .with_position((9, 27))
        )))
    );
}
