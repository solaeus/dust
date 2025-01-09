use dust_lang::*;
use smallvec::smallvec;

#[test]
fn function() {
    let source = "fn(a: int, b: int) -> int { a + b }";

    assert_eq!(
        run(source),
        Ok(Some(ConcreteValue::function(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Type::function(FunctionType {
                    type_parameters: None,
                    value_parameters: Some(smallvec![(0, Type::Integer), (1, Type::Integer)]),
                    return_type: Type::Integer,
                })
            },
            vec![
                (
                    Instruction::add(2, Argument::Register(0), Argument::Register(1)),
                    Span(30, 31)
                ),
                (Instruction::r#return(true), Span(34, 35)),
            ],
            vec![ConcreteValue::string("a"), ConcreteValue::string("b"),],
            vec![
                Local::new(0, 0, false, Scope::default()),
                Local::new(1, 1, false, Scope::default())
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
                return_type: Type::Integer
            },
            vec![
                (Instruction::load_constant(0, 0, false), Span(0, 35)),
                (Instruction::load_constant(1, 1, false), Span(36, 37)),
                (Instruction::load_constant(2, 2, false), Span(39, 40)),
                (Instruction::call(3, Argument::Constant(0), 2), Span(35, 41)),
                (Instruction::r#return(true), Span(41, 41)),
            ],
            vec![
                ConcreteValue::function(Chunk::with_data(
                    None,
                    FunctionType {
                        type_parameters: None,
                        value_parameters: Some(smallvec![(0, Type::Integer), (1, Type::Integer)]),
                        return_type: Type::Integer
                    },
                    vec![
                        (
                            Instruction::add(2, Argument::Register(0), Argument::Register(1)),
                            Span(30, 31)
                        ),
                        (Instruction::r#return(true), Span(34, 35)),
                    ],
                    vec![ConcreteValue::string("a"), ConcreteValue::string("b"),],
                    vec![
                        Local::new(0, 0, false, Scope::default()),
                        Local::new(1, 1, false, Scope::default())
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
                return_type: Type::None
            },
            vec![
                (Instruction::load_constant(0, 0, false), Span(0, 40)),
                (Instruction::r#return(false), Span(40, 40))
            ],
            vec![
                ConcreteValue::function(Chunk::with_data(
                    Some("add".into()),
                    FunctionType {
                        type_parameters: None,
                        value_parameters: Some(smallvec![(0, Type::Integer), (1, Type::Integer)]),
                        return_type: Type::Integer
                    },
                    vec![
                        (
                            Instruction::add(2, Argument::Register(0), Argument::Register(1)),
                            Span(35, 36)
                        ),
                        (Instruction::r#return(true), Span(39, 40)),
                    ],
                    vec![ConcreteValue::string("a"), ConcreteValue::string("b")],
                    vec![
                        Local::new(0, 0, false, Scope::default()),
                        Local::new(1, 1, false, Scope::default())
                    ]
                )),
                ConcreteValue::string("add"),
            ],
            vec![Local::new(1, 0, false, Scope::default(),),],
        )),
    );

    assert_eq!(run(source), Ok(None));
}
