use dust_lang::{
    abstract_tree::{AbstractTree, Block, Expression, Identifier, Statement, Type},
    error::{Error, TypeConflict, ValidationError},
    *,
};

#[test]
fn set_and_get_variable() {
    assert_eq!(
        interpret("foobar = true; foobar"),
        Ok(Some(Value::boolean(true)))
    );
}

#[test]
fn set_variable_with_type() {
    assert_eq!(
        interpret("foobar: bool = true; foobar"),
        Ok(Some(Value::boolean(true)))
    );
}

#[test]
fn set_variable_with_type_error() {
    assert_eq!(
        interpret("foobar: str = true"),
        Err(vec![Error::Validation {
            error: ValidationError::TypeCheck {
                conflict: TypeConflict {
                    actual: Type::Boolean,
                    expected: Type::String
                },
                actual_position: (0, 0).into(),
                expected_position: (0, 0).into()
            },
            position: (0, 18).into()
        }])
    );
}

#[test]
fn function_variable() {
    assert_eq!(
        interpret("foobar = (x: int): int { x }; foobar"),
        Ok(Some(Value::function(
            vec![(Identifier::new("x"), Type::Integer.with_position((0, 0)))],
            Type::Integer.with_position((0, 0)),
            Block::new(vec![Statement::Expression(Expression::Identifier(
                Identifier::new("x")
            ))
            .with_position((0, 0))])
            .with_position((0, 0))
        )))
    );
}
