use dust_lang::{
    abstract_tree::{Block, Expression, Statement, WithPos},
    identifier::Identifier,
    Type, *,
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
fn function_variable() {
    assert_eq!(
        interpret("test", "foobar = fn (x: int) -> int { x }; foobar"),
        Ok(Some(Value::function(
            None,
            Some(vec![(Identifier::new("x"), Type::Integer)]),
            Some(Type::Integer),
            Block::new(vec![Statement::Expression(Expression::Identifier(
                Identifier::new("x").with_position((30, 31))
            ))]),
        )))
    );
}
