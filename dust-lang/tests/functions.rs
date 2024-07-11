use dust_lang::*;

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
