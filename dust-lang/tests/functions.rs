use abstract_tree::{Expression, ValueNode, WithPos};
use dust_lang::*;
use error::{DustError, ValidationError};
use identifier::Identifier;

#[test]
fn function_scope() {
    assert_eq!(
        interpret(
            "test",
            "
            x = 2

            foo = fn () -> int {
                x = 42
                x
            }

            x = 1

            foo()
            "
        ),
        Ok(Some(Value::integer(42)))
    );
}

#[test]
fn function_call_with_type_argument() {
    assert_eq!(
        interpret(
            "test",
            "
            foobar = fn <T> (x: T) -> T { x }
            foobar::<int>(42)
            ",
        ),
        Ok(Some(Value::integer(42)))
    );
}

#[test]
fn function_call() {
    assert_eq!(
        interpret(
            "test",
            "
            foobar = fn (message: str) -> str { message }
            foobar('Hiya')
            ",
        ),
        Ok(Some(Value::string("Hiya".to_string())))
    );
}

#[test]
fn callback() {
    assert_eq!(
        interpret(
            "test",
            "
            foobar = fn (cb: fn () -> str) -> str {
                cb()
            }
            foobar(fn () -> str { 'Hiya' })
            ",
        ),
        Ok(Some(Value::string("Hiya".to_string())))
    );
}

#[test]
fn built_in_function_call() {
    assert_eq!(
        interpret("test", "use std.io io.write_line('Hiya')"),
        Ok(None)
    );
}

#[test]
fn function_context_captures_values() {
    assert_eq!(
        interpret(
            "test",
            "
            bar = fn () -> int { 2 }
            foo = fn () -> int { bar() }
            foo()
            "
        ),
        Ok(Some(Value::integer(2)))
    );
}

#[test]
fn recursion() {
    assert_eq!(
        interpret(
            "test",
            "
            fib = fn (i: int) -> int {
            	if i <= 1 {
                    i
                } else {
            		fib(i - 1) + fib(i - 2)
            	}
            }

            fib(7)
            "
        ),
        Ok(Some(Value::integer(13)))
    );
}

#[test]
fn value_argument_error() {
    assert_eq!(
        interpret(
            "test",
            "
            foobar = fn (a: int, b: int) -> int { a + b }
            foobar(1)
            "
        ),
        Err(InterpreterError::new(
            "test".into(),
            vec![DustError::Validation {
                error: ValidationError::WrongValueArguments {
                    parameters: vec![
                        (Identifier::new("a"), Type::Integer),
                        (Identifier::new("b"), Type::Integer),
                    ],
                    arguments: vec![Expression::Value(
                        ValueNode::Integer(1).with_position((78, 79)),
                    )],
                },
                position: (71, 80).into()
            }]
        ))
    );
}

#[test]
fn type_argument_error() {
    assert_eq!(
        interpret(
            "test",
            "
            foobar = fn <T> (a: T) -> T { a }
            foobar(1)
            "
        ),
        Err(InterpreterError::new(
            "test".into(),
            vec![DustError::Validation {
                error: ValidationError::WrongTypeArguments {
                    parameters: vec![Identifier::new("T")],
                    arguments: vec![]
                },
                position: (59, 68).into()
            }]
        ))
    );
}
