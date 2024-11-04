use dust_lang::*;

#[test]
fn function() {
    let source = "fn(a: int, b: int) -> int { a + b }";

    assert_eq!(
        run(source),
        Ok(Some(Value::function(
            Chunk::with_data(
                None,
                vec![
                    (Instruction::add(2, 0, 1), Span(30, 31)),
                    (Instruction::r#return(true), Span(35, 35)),
                ],
                vec![],
                vec![
                    Local::new(0, Some(Type::Integer), false, 0, 0),
                    Local::new(0, Some(Type::Integer), false, 0, 1)
                ]
            ),
            FunctionType {
                type_parameters: None,
                value_parameters: Some(vec![(0, Type::Integer), (0, Type::Integer)]),
                return_type: Some(Box::new(Type::Integer)),
            }
        )))
    );
}

#[test]
fn function_declaration() {
    let source = "fn add (a: int, b: int) -> int { a + b }";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(0, 40)),
                (Instruction::define_local(0, 0, false), Span(3, 6)),
                (Instruction::r#return(false), Span(40, 40))
            ],
            vec![Value::function(
                Chunk::with_data(
                    None,
                    vec![
                        (Instruction::add(2, 0, 1), Span(35, 36)),
                        (Instruction::r#return(true), Span(40, 40)),
                    ],
                    vec![],
                    vec![
                        Local::new(0, Some(Type::Integer), false, 0, 0),
                        Local::new(0, Some(Type::Integer), false, 0, 1)
                    ]
                ),
                FunctionType {
                    type_parameters: None,
                    value_parameters: Some(vec![(0, Type::Integer), (0, Type::Integer)]),
                    return_type: Some(Box::new(Type::Integer)),
                },
            )],
            vec![Local::new(
                0,
                Some(Type::Function(FunctionType {
                    type_parameters: None,
                    value_parameters: Some(vec![(0, Type::Integer), (0, Type::Integer)]),
                    return_type: Some(Box::new(Type::Integer)),
                })),
                false,
                0,
                0
            ),],
        )),
    );

    assert_eq!(run(source), Ok(None));
}

#[test]
fn function_call() {
    let source = "fn(a: int, b: int) -> int { a + b }(1, 2)";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(0, 36)),
                (Instruction::load_constant(1, 1, false), Span(36, 37)),
                (Instruction::load_constant(2, 2, false), Span(39, 40)),
                (Instruction::call(3, 0, 2), Span(35, 41)),
                (Instruction::r#return(true), Span(41, 41)),
            ],
            vec![
                Value::function(
                    Chunk::with_data(
                        None,
                        vec![
                            (Instruction::add(2, 0, 1), Span(30, 31)),
                            (Instruction::r#return(true), Span(35, 36)),
                        ],
                        vec![],
                        vec![
                            Local::new(0, Some(Type::Integer), false, 0, 0),
                            Local::new(0, Some(Type::Integer), false, 0, 1)
                        ]
                    ),
                    FunctionType {
                        type_parameters: None,
                        value_parameters: Some(vec![(0, Type::Integer), (0, Type::Integer)]),
                        return_type: Some(Box::new(Type::Integer)),
                    }
                ),
                Value::integer(1),
                Value::integer(2)
            ],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::integer(3))));
}
