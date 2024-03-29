use dust_lang::{error::ValidationError, *};

#[test]
fn function_call() {
    assert_eq!(
        interpret(
            "
            foobar = (message <str>) <str> { message }
            foobar('Hiya')
            ",
        ),
        Ok(Value::string("Hiya".to_string()))
    );
}

#[test]
fn call_empty_function() {
    assert_eq!(
        interpret(
            "
            foobar = (message <str>) <none> {}
            foobar('Hiya')
            ",
        ),
        Ok(Value::none())
    );
}

#[test]
fn callback() {
    assert_eq!(
        interpret(
            "
            foobar = (cb <() -> str>) <str> {
                cb()
            }
            foobar(() <str> { 'Hiya' })
            ",
        ),
        Ok(Value::string("Hiya".to_string()))
    );
}

#[test]
fn built_in_function_call() {
    assert_eq!(interpret("output('Hiya')"), Ok(Value::none()));
}

#[test]
fn function_context_does_not_capture_normal_values() {
    assert_eq!(
        interpret(
            "
            x = 1

            foo = () <any> { x }
            "
        ),
        Err(Error::Validation(
            ValidationError::VariableIdentifierNotFound(Identifier::new("x"))
        ))
    );

    assert_eq!(
        interpret(
            "
            x = 1
            foo = (x <int>) <int> { x }
            foo(2)
            "
        ),
        Ok(Value::Integer(2))
    );
}

#[test]
fn function_context_captures_functions() {
    assert_eq!(
        interpret(
            "
            bar = () <int> { 2 }
            foo = () <int> { bar() }
            foo()
            "
        ),
        Ok(Value::Integer(2))
    );
}

#[test]
fn function_context_captures_structure_definitions() {
    let mut map = Map::new();

    map.set(Identifier::new("name"), Value::string("bob"));

    assert_eq!(
        interpret(
            "
            struct User {
                name <str>
            }
            
            bob = () <User> {
                User::{
                    name = 'bob'
                }
            }

            bob()
            "
        ),
        Ok(Value::Struct(StructInstance::new("User".into(), map)))
    );
}

#[test]
fn recursion() {
    env_logger::builder().is_test(true).try_init().unwrap();

    assert_eq!(
        interpret(
            "
            fib = (i <int>) <int> {
            	if i <= 1 {
            		1
            	} else {
            		fib(i - 1) + fib(i - 2)
            	}
            }

            fib(8)
            "
        ),
        Ok(Value::Integer(34))
    );
}
