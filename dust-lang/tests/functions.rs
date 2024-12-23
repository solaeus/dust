use dust_lang::*;

#[test]
fn function() {
    let source = "fn(a: int, b: int) -> int { a + b }";

    assert_eq!(
        run(source),
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
                (
                    Instruction::add(
                        Destination::Register(2),
                        Argument::Local(0),
                        Argument::Local(1)
                    ),
                    Type::Integer,
                    Span(30, 31)
                ),
                (Instruction::r#return(true), Type::None, Span(35, 35)),
            ],
            vec![ConcreteValue::string("a"), ConcreteValue::string("b"),],
            vec![
                Local::new(0, Type::Integer, false, Scope::default()),
                Local::new(1, Type::Integer, false, Scope::default())
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
                (
                    Instruction::load_constant(Destination::Register(0), 0, false),
                    Type::Function(FunctionType {
                        type_parameters: None,
                        value_parameters: Some(vec![(0, Type::Integer), (1, Type::Integer)]),
                        return_type: Box::new(Type::Integer),
                    }),
                    Span(0, 36)
                ),
                (
                    Instruction::load_constant(Destination::Register(1), 1, false),
                    Type::Integer,
                    Span(36, 37)
                ),
                (
                    Instruction::load_constant(Destination::Register(2), 2, false),
                    Type::Integer,
                    Span(39, 40)
                ),
                (
                    Instruction::call(Destination::Register(3), Argument::Constant(0), 2),
                    Type::Integer,
                    Span(35, 41)
                ),
                (Instruction::r#return(true), Type::None, Span(41, 41)),
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
                        (
                            Instruction::add(
                                Destination::Register(2),
                                Argument::Local(0),
                                Argument::Local(1)
                            ),
                            Type::Integer,
                            Span(30, 31)
                        ),
                        (Instruction::r#return(true), Type::None, Span(35, 36)),
                    ],
                    vec![ConcreteValue::string("a"), ConcreteValue::string("b"),],
                    vec![
                        Local::new(0, Type::Integer, false, Scope::default()),
                        Local::new(1, Type::Integer, false, Scope::default())
                    ]
                )),
                ConcreteValue::Integer(1),
                ConcreteValue::Integer(2)
            ],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Integer(3))));
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
                (
                    Instruction::load_constant(Destination::Register(0), 0, false),
                    Type::Function(FunctionType {
                        type_parameters: None,
                        value_parameters: Some(vec![(0, Type::Integer), (1, Type::Integer)]),
                        return_type: Box::new(Type::Integer),
                    }),
                    Span(0, 40)
                ),
                (
                    Instruction::define_local(0, 0, false),
                    Type::None,
                    Span(3, 6)
                ),
                (Instruction::r#return(false), Type::None, Span(40, 40))
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
                        (
                            Instruction::add(
                                Destination::Register(2),
                                Argument::Local(0),
                                Argument::Local(1)
                            ),
                            Type::Integer,
                            Span(35, 36)
                        ),
                        (Instruction::r#return(true), Type::None, Span(40, 40)),
                    ],
                    vec![ConcreteValue::string("a"), ConcreteValue::string("b")],
                    vec![
                        Local::new(0, Type::Integer, false, Scope::default()),
                        Local::new(1, Type::Integer, false, Scope::default())
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
            ),],
        )),
    );

    assert_eq!(run(source), Ok(None));
}
