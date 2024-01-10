use dust_lang::*;

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
    assert_eq!(interpret("output('Hiya')"), Ok(Value::Option(None)));
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
        Err(Error::VariableIdentifierNotFound("x".to_string()))
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

    // assert_eq!(
    //     interpret(
    //         "
    //         foo = () <int> { bar() }
    //         foo()
    //         bar = () <int> { 2 }
    //         "
    //     ),
    //     Ok(Value::Integer(2))
    // );
}

#[test]
fn function_context_captures_structure_definitions() {
    let map = Map::new();

    map.set("name".to_string(), Value::string("bob")).unwrap();

    assert_eq!(
        interpret(
            "
            User = struct {
                name <str>
            }
            
            bob = () <User> {
                new User {
                    name = 'bob'
                }
            }

            bob()
            "
        ),
        Ok(Value::Map(map.clone()))
    );

    assert_eq!(
        interpret(
            "
            bob = () <User> {
                new User {
                    name = 'bob'
                }
            }

            User = struct {
                name <str>
            }
            
            bob()
            "
        ),
        Ok(Value::Map(map))
    );
}
