use dust_lang::*;

#[test]
fn simple_type_check() {
    let result = interpret("x <bool> = 1");

    assert!(result.unwrap_err().is_error(&Error::TypeCheck {
        expected: Type::Boolean,
        actual: Type::Integer
    }));
}

#[test]
fn argument_count_check() {
    env_logger::builder().is_test(true).try_init().unwrap();

    let source = "
            foo = (x <int>) <int> {
                x
            }
            foo()
            ";
    let result = interpret(&source);

    assert_eq!(
        "Expected 1 arguments, but got 0. Occured at (5, 12) to (5, 17). Source: foo()",
        result.unwrap_err().to_string()
    )
}

#[test]
fn callback_type_check() {
    let result = interpret(
        "
            x = (cb <() -> bool>) <bool> {
                cb()
            }
            x(() <int> { 1 })
            ",
    );

    assert!(result.unwrap_err().is_error(&Error::TypeCheck {
        expected: Type::Function {
            parameter_types: vec![],
            return_type: Box::new(Type::Boolean),
        },
        actual: Type::Function {
            parameter_types: vec![],
            return_type: Box::new(Type::Integer),
        },
    }));
}
