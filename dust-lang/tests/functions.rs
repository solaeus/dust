use dust_lang::*;

#[test]
fn function() {
    let source = "fn(a: int, b: int) -> int { a + b }";

    assert_eq!(
        run_source(source),
        Ok(Some(ConcreteValue::Function(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::Function(FunctionType {
                    type_parameters: None,
                    value_parameters: Some(vec![(0, Type::Integer), (1, Type::Integer)]),
                    return_type: Box::new(Type::Integer),
                }))
            },
            vec![
                (Instruction::add(2, 0, 1), Span(30, 31)),
                (Instruction::r#return(true), Span(35, 35)),
            ],
            vec![ConcreteValue::string("a"), ConcreteValue::string("b"),],
            vec![
                Local::new(0, Type::Integer, false, Scope::default(), 0),
                Local::new(1, Type::Integer, false, Scope::default(), 1)
            ]
        ))))
    );
}

#[test]
fn function_call() {
    let source = "fn(a: int, b: int) -> int { a + b }(1, 2)";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::Integer)
            },
            vec![
                (Instruction::load_constant(0, 0, false), Span(0, 36)),
                (Instruction::load_constant(1, 1, false), Span(36, 37)),
                (Instruction::load_constant(2, 2, false), Span(39, 40)),
                (Instruction::call(3, 0, 2), Span(35, 41)),
                (Instruction::r#return(true), Span(41, 41)),
            ],
            vec![
                ConcreteValue::Function(Chunk::with_data(
                    None,
                    FunctionType {
                        type_parameters: None,
                        value_parameters: Some(vec![(0, Type::Integer), (1, Type::Integer)]),
                        return_type: Box::new(Type::Integer)
                    },
                    vec![
                        (Instruction::add(2, 0, 1), Span(30, 31)),
                        (Instruction::r#return(true), Span(35, 36)),
                    ],
                    vec![ConcreteValue::string("a"), ConcreteValue::string("b"),],
                    vec![
                        Local::new(0, Type::Integer, false, Scope::default(), 0),
                        Local::new(1, Type::Integer, false, Scope::default(), 1)
                    ]
                )),
                ConcreteValue::Integer(1),
                ConcreteValue::Integer(2)
            ],
            vec![]
        )),
    );

    assert_eq!(run_source(source), Ok(Some(ConcreteValue::Integer(3))));
}

#[test]
fn function_declaration() {
    let source = "fn add (a: int, b: int) -> int { a + b }";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::None)
            },
            vec![
                (Instruction::load_constant(0, 0, false), Span(0, 40)),
                (Instruction::r#return(false), Span(40, 40))
            ],
            vec![
                ConcreteValue::Function(Chunk::with_data(
                    Some("add".to_string()),
                    FunctionType {
                        type_parameters: None,
                        value_parameters: Some(vec![(0, Type::Integer), (1, Type::Integer)]),
                        return_type: Box::new(Type::Integer)
                    },
                    vec![
                        (Instruction::add(2, 0, 1), Span(35, 36)),
                        (Instruction::r#return(true), Span(40, 40)),
                    ],
                    vec![ConcreteValue::string("a"), ConcreteValue::string("b")],
                    vec![
                        Local::new(0, Type::Integer, false, Scope::default(), 0),
                        Local::new(1, Type::Integer, false, Scope::default(), 1)
                    ]
                )),
                ConcreteValue::string("add"),
            ],
            vec![Local::new(
                1,
                Type::Function(FunctionType {
                    type_parameters: None,
                    value_parameters: Some(vec![(0, Type::Integer), (1, Type::Integer)]),
                    return_type: Box::new(Type::Integer),
                }),
                false,
                Scope::default(),
                0
            ),],
        )),
    );

    assert_eq!(run_source(source), Ok(None));
}
