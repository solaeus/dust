use dust_lang::{
    abstract_tree::{Block, Expression, Statement, Type, WithPos},
    context::Context,
    error::{DustError, TypeConflict, ValidationError},
    identifier::Identifier,
    *,
};

#[test]
fn set_and_get_variable() {
    assert_eq!(
        interpret("test", "foobar = true; foobar"),
        Ok(Some(Value::boolean(true)))
    );
}

#[test]
fn set_variable_with_type() {
    assert_eq!(
        interpret("test", "foobar: bool = true; foobar"),
        Ok(Some(Value::boolean(true)))
    );
}

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
fn function_variable() {
    assert_eq!(
        interpret("test", "foobar = fn (x: int) -> int { x }; foobar"),
        Ok(Some(Value::function(
            None,
            vec![(Identifier::new("x"), Type::Integer)],
            Some(Type::Integer),
            Block::new(vec![Statement::Expression(Expression::Identifier(
                Identifier::new("x").with_position((30, 31))
            ))]),
            Context::new(None)
        )))
    );
}
