mod assignment {
    use dust_lang::*;

    #[test]
    fn simple_assignment() {
        let test = interpret("x = 1 x");

        assert_eq!(Ok(Value::Integer(1)), test);
    }

    #[test]
    fn simple_assignment_with_type() {
        let test = interpret("x <int> = 1 x");

        assert_eq!(Ok(Value::Integer(1)), test);
    }

    #[test]
    fn list_add_assign() {
        let test = interpret(
            "
            x <[int]> = []
            x += 1
            x
            ",
        );

        assert_eq!(
            Ok(Value::List(List::with_items(vec![Value::Integer(1)]))),
            test
        );
    }

    #[test]
    fn list_add_wrong_type() {
        let result = interpret(
            "
            x <[str]> = []
            x += 1
            ",
        );

        assert!(result.unwrap_err().is_type_check_error(&Error::TypeCheck {
            expected: Type::String,
            actual: Type::Integer
        }))
    }
}

mod for_loop {
    use dust_lang::*;

    #[test]
    fn simple_for_loop() {
        let result = interpret("for i in [1 2 3] { output(i) }");

        assert_eq!(Ok(Value::none()), result);
    }

    #[test]
    fn modify_value() {
        let result = interpret(
            "
            list = []
            for i in [1 2 3] { list += i }
            list
            ",
        );

        assert_eq!(
            Ok(Value::List(List::with_items(vec![
                Value::Integer(1),
                Value::Integer(2),
                Value::Integer(3),
            ]))),
            result
        );
    }
}

mod logic {
    use dust_lang::*;

    #[test]
    fn complex_logic_sequence() {
        let result =
            interpret("(length([0]) == 1) && (length([0 0]) == 2) && (length([0 0 0]) == 3)");

        assert_eq!(Ok(Value::Boolean(true)), result);
    }
}

mod value {
    use dust_lang::*;

    #[test]
    fn empty() {
        assert_eq!(interpret("x = 9"), Ok(Value::Option(None)));
        assert_eq!(interpret("x = 1 + 1"), Ok(Value::Option(None)));
    }

    #[test]
    fn integer() {
        assert_eq!(interpret("1"), Ok(Value::Integer(1)));
        assert_eq!(interpret("123"), Ok(Value::Integer(123)));
        assert_eq!(interpret("-666"), Ok(Value::Integer(-666)));
    }

    #[test]
    fn float() {
        assert_eq!(interpret("0.1"), Ok(Value::Float(0.1)));
        assert_eq!(interpret("12.3"), Ok(Value::Float(12.3)));
        assert_eq!(interpret("-6.66"), Ok(Value::Float(-6.66)));
    }

    #[test]
    fn string() {
        assert_eq!(interpret("\"one\""), Ok(Value::String("one".to_string())));
        assert_eq!(interpret("'one'"), Ok(Value::String("one".to_string())));
        assert_eq!(interpret("`one`"), Ok(Value::String("one".to_string())));
        assert_eq!(interpret("`'one'`"), Ok(Value::String("'one'".to_string())));
        assert_eq!(interpret("'`one`'"), Ok(Value::String("`one`".to_string())));
        assert_eq!(
            interpret("\"'one'\""),
            Ok(Value::String("'one'".to_string()))
        );
    }

    #[test]
    fn list() {
        assert_eq!(
            interpret("[1, 2, 'foobar']"),
            Ok(Value::List(List::with_items(vec![
                Value::Integer(1),
                Value::Integer(2),
                Value::String("foobar".to_string()),
            ])))
        );
    }

    #[test]
    fn map() {
        let map = Map::new();

        map.set("x".to_string(), Value::Integer(1), None).unwrap();
        map.set("foo".to_string(), Value::String("bar".to_string()), None)
            .unwrap();

        assert_eq!(interpret("{ x = 1, foo = 'bar' }"), Ok(Value::Map(map)));
    }

    #[test]
    fn map_types() {
        let map = Map::new();

        map.set("x".to_string(), Value::Integer(1), Some(Type::Integer))
            .unwrap();
        map.set(
            "foo".to_string(),
            Value::String("bar".to_string()),
            Some(Type::String),
        )
        .unwrap();

        assert_eq!(
            interpret("{ x <int> = 1, foo <str> = 'bar' }"),
            Ok(Value::Map(map))
        );
    }

    #[test]
    fn map_type_errors() {
        assert!(interpret("{ foo <bool> = 'bar' }")
            .unwrap_err()
            .is_type_check_error(&Error::TypeCheck {
                expected: Type::Boolean,
                actual: Type::String
            }))
    }

    #[test]
    fn function() {
        let result = interpret("() <int> { 1 }");
        let value = result.unwrap();
        let function = value.as_function().unwrap();

        assert_eq!(&Vec::<Identifier>::with_capacity(0), function.parameters());
        assert_eq!(&Type::Integer, function.return_type());

        let result = interpret("(x <bool>) <bool> { true }");
        let value = result.unwrap();
        let function = value.as_function().unwrap();

        assert_eq!(
            &vec![Identifier::new("x".to_string())],
            function.parameters()
        );
        assert_eq!(&Type::Boolean, function.return_type());
    }

    #[test]
    fn option() {
        let result = interpret("x <option(int)> = some(1); x").unwrap();

        assert_eq!(Value::Option(Some(Box::new(Value::Integer(1)))), result);
    }
}

mod function_call {
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
            Ok(Value::String("Hiya".to_string()))
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
            Ok(Value::String("Hiya".to_string()))
        );
    }

    #[test]
    fn built_in_function_call() {
        assert_eq!(interpret("output('Hiya')"), Ok(Value::Option(None)));
    }
}

mod if_else {
    use dust_lang::*;

    #[test]
    fn r#if() {
        assert_eq!(
            interpret("if true { 'true' }"),
            Ok(Value::String("true".to_string()))
        );
    }

    #[test]
    fn if_else() {
        assert_eq!(
            interpret("if false { 1 } else { 2 }"),
            Ok(Value::Integer(2))
        );
        assert_eq!(
            interpret("if true { 1.0 } else { 42.0 }"),
            Ok(Value::Float(1.0))
        );
    }

    #[test]
    fn if_else_else_if_else() {
        assert_eq!(
            interpret(
                "
                    if false {
                        'no'
                    } else if 1 + 1 == 3 {
                        'nope'
                    } else {
                        'ok'
                    }
                "
            ),
            Ok(Value::String("ok".to_string()))
        );
    }

    #[test]
    fn if_else_if_else_if_else_if_else() {
        assert_eq!(
            interpret(
                "
                    if false {
                        'no'
                    } else if 1 + 1 == 1 {
                        'nope'
                    } else if 9 / 2 == 4 {
                        'nope'
                    } else if 'foo' == 'bar' {
                        'nope'
                    } else {
                        'ok'
                    }
                "
            ),
            Ok(Value::String("ok".to_string()))
        );
    }
}

mod index {
    use dust_lang::*;

    #[test]
    fn list_index() {
        let test = interpret("x = [1 [2] 3] x:1:0").unwrap();

        assert_eq!(Value::Integer(2), test);
    }

    #[test]
    fn map_index() {
        let test = interpret("x = {y = {z = 2}} x:y:z").unwrap();

        assert_eq!(Value::Integer(2), test);
    }

    #[test]
    fn complex_index() {
        let test = interpret(
            "
            x = [1 2 3]
            y = () <int> { 2 }
            x:y()
            ",
        )
        .unwrap();

        assert_eq!(Value::Integer(3), test);
    }
}

mod r#match {
    use dust_lang::*;

    #[test]
    fn r#match() {
        let test = interpret(
            "
                match 1 {
                    3 => false
                    2 => { false }
                    1 => true
                }
            ",
        )
        .unwrap();

        assert_eq!(Value::Boolean(true), test);
    }

    #[test]
    fn match_assignment() {
        let test = interpret(
            "
                x = match 1 {
                    3 => false
                    2 => { false }
                    1 => true
                }
                x
            ",
        )
        .unwrap();

        assert_eq!(Value::Boolean(true), test);
    }
}

mod r#while {
    use dust_lang::*;

    #[test]
    fn while_loop() {
        assert_eq!(interpret("while false { 'foo' }"), Ok(Value::Option(None)))
    }

    #[test]
    fn while_loop_iteration_count() {
        assert_eq!(
            interpret("i = 0; while i < 3 { i += 1 }; i"),
            Ok(Value::Integer(3))
        )
    }
}

mod type_definition {
    use dust_lang::*;

    #[test]
    fn simple_type_check() {
        let result = interpret("x <bool> = 1");

        assert!(result.unwrap_err().is_type_check_error(&Error::TypeCheck {
            expected: Type::Boolean,
            actual: Type::Integer
        }));
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

        assert!(result.unwrap_err().is_type_check_error(&Error::TypeCheck {
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
}

mod blocks {
    use dust_lang::*;

    #[test]
    fn simple() {
        assert_eq!(interpret("{ 1 }"), Ok(Value::Integer(1)));
    }

    #[test]
    fn nested() {
        assert_eq!(interpret("{ 1 { 1 + 1 } }"), Ok(Value::Integer(2)));
    }

    #[test]
    fn with_return() {
        assert_eq!(interpret("{ return 1; 1 + 1; }"), Ok(Value::Integer(1)));
    }

    #[test]
    fn async_with_return() {
        assert_eq!(
            interpret(
                "
                async {
                    return 1
                    1 + 1
                    3
                }
                "
            ),
            Ok(Value::Integer(1))
        );
    }
}
