use dust_lang::*;

#[test]
fn add_characters() {
    let source = "'a' + 'b'";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::String),
            },
            vec![
                (
                    Instruction::add(
                        Destination::Register(0),
                        Argument::Constant(0),
                        Argument::Constant(1)
                    ),
                    Span(4, 5)
                ),
                (Instruction::r#return(true), Span(9, 9))
            ],
            vec![ConcreteValue::Character('a'), ConcreteValue::Character('b')],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::string("ab"))));
}

#[test]
fn add_floats() {
    let source = "1.0 + 2.0";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::Float),
            },
            vec![
                (
                    Instruction::add(
                        Destination::Register(0),
                        Argument::Constant(0),
                        Argument::Constant(1)
                    ),
                    Span(4, 5)
                ),
                (Instruction::r#return(true), Span(9, 9))
            ],
            vec![ConcreteValue::Float(1.0), ConcreteValue::Float(2.0)],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Float(3.0))));
}

#[test]
fn add_integers() {
    let source = "1 + 2";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::Integer),
            },
            vec![
                (
                    Instruction::add(
                        Destination::Register(0),
                        Argument::Constant(0),
                        Argument::Constant(1)
                    ),
                    Span(2, 3)
                ),
                (Instruction::r#return(true), Span(5, 5))
            ],
            vec![ConcreteValue::Integer(1), ConcreteValue::Integer(2)],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Integer(3))));
}

#[test]
fn add_strings() {
    let source = "\"Hello, \" + \"world!\"";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::String),
            },
            vec![
                (
                    Instruction::add(
                        Destination::Register(0),
                        Argument::Constant(0),
                        Argument::Constant(1)
                    ),
                    Span(10, 11)
                ),
                (Instruction::r#return(true), Span(20, 20))
            ],
            vec![
                ConcreteValue::String("Hello, ".to_string()),
                ConcreteValue::String("world!".to_string())
            ],
            vec![]
        ))
    );
}
