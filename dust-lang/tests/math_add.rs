use dust_lang::*;

#[test]
fn add_bytes() {
    let source = "0xfe + 0x01";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::Byte),
            },
            vec![
                (
                    Instruction::add(
                        Destination::Register(0),
                        Argument::Constant(0),
                        Argument::Constant(1)
                    ),
                    Span(5, 6)
                ),
                (Instruction::r#return(true), Span(11, 11))
            ],
            vec![ConcreteValue::Byte(0xfe), ConcreteValue::Byte(0x01)],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Byte(0xff))));
}

#[test]
fn add_bytes_saturate() {
    let source = "0xff + 0x01";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::Byte),
            },
            vec![
                (
                    Instruction::add(
                        Destination::Register(0),
                        Argument::Constant(0),
                        Argument::Constant(1)
                    ),
                    Span(5, 6)
                ),
                (Instruction::r#return(true), Span(11, 11))
            ],
            vec![ConcreteValue::Byte(0xff), ConcreteValue::Byte(0x01)],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Byte(0xff))));
}

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
fn add_character_and_string() {
    let source = "'a' + \"b\"";

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
            vec![ConcreteValue::Character('a'), ConcreteValue::string("b")],
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
fn add_floats_saturatate() {
    let source = "1.7976931348623157E+308 + 0.00000001";

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
                    Span(24, 25)
                ),
                (Instruction::r#return(true), Span(36, 36))
            ],
            vec![
                ConcreteValue::Float(f64::MAX),
                ConcreteValue::Float(0.00000001)
            ],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Float(f64::MAX))));
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
fn add_integers_saturate() {
    let source = "9223372036854775807 + 1";

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
                    Span(20, 21)
                ),
                (Instruction::r#return(true), Span(23, 23))
            ],
            vec![ConcreteValue::Integer(i64::MAX), ConcreteValue::Integer(1)],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Integer(i64::MAX))));
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
                ConcreteValue::string("Hello, "),
                ConcreteValue::string("world!")
            ],
            vec![]
        ))
    );
}

#[test]
fn add_string_and_character() {
    let source = "\"a\" + 'b'";

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
            vec![ConcreteValue::string("a"), ConcreteValue::Character('b')],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::string("ab"))));
}
