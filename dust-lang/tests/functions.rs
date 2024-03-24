use std::rc::Rc;

use dust_lang::{
    abstract_tree::Identifier,
    error::{Error, ValidationError},
    *,
};

#[test]
fn function_call() {
    assert_eq!(
        interpret(
            Rc::new("test".to_string()),
            "
            foobar = (message : str) str { message }
            foobar('Hiya')
            ",
        ),
        Ok(Some(Value::string("Hiya".to_string())))
    );
}

#[test]
fn call_empty_function() {
    assert_eq!(
        interpret(
            Rc::new("test".to_string()),
            "
            foobar = (message : str) none {}
            foobar('Hiya')
            ",
        ),
        Ok(None)
    );
}

#[test]
fn callback() {
    assert_eq!(
        interpret(
            Rc::new("test".to_string()),
            "
            foobar = (cb: fn() -> str) str {
                cb()
            }
            foobar(() str { 'Hiya' })
            ",
        ),
        Ok(Some(Value::string("Hiya".to_string())))
    );
}

#[test]
fn built_in_function_call() {
    assert_eq!(
        interpret(Rc::new("test".to_string()), "io.write_line('Hiya')"),
        Ok(None)
    );
}

#[test]
fn function_context_does_not_capture_values() {
    assert_eq!(
        interpret(
            Rc::new("test".to_string()),
            "
            x = 1

            foo = () any { x } 
            "
        )
        .unwrap_err()
        .errors(),
        &vec![Error::Validation {
            error: ValidationError::VariableNotFound(Identifier::new("x")),
            position: (32, 50).into()
        }]
    );

    assert_eq!(
        interpret(
            Rc::new("test".to_string()),
            "
            x = 1
            foo = (x: int) int { x }
            foo(2)
            "
        ),
        Ok(Some(Value::integer(2)))
    );
}

#[test]
fn function_context_captures_functions() {
    assert_eq!(
        interpret(
            Rc::new("test".to_string()),
            "
            bar = () int { 2 }
            foo = () int { bar() }
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
            Rc::new("test".to_string()),
            "
            fib = (i: int) int {
            	if i <= 1 {
            		1
            	} else {
            		fib(i - 1) + fib(i - 2)
            	}
            }

            fib(8)
            "
        ),
        Ok(Some(Value::integer(34)))
    );
}
