use dust_lang::*;

#[test]
fn subtract_floats() {
    let source = "2.0 - 2.0";

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
                    Instruction::subtract(
                        Destination::Register(0),
                        Argument::Constant(0),
                        Argument::Constant(0)
                    ),
                    Span(4, 5)
                ),
                (Instruction::r#return(true), Span(9, 9)),
            ],
            vec![ConcreteValue::Float(2.0)],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Float(0.0))));
}

#[test]
fn subtract_floats_saturate() {
    let source = "-1.7976931348623157E+308 - 0.0000001";

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
                    Instruction::subtract(
                        Destination::Register(0),
                        Argument::Constant(0),
                        Argument::Constant(1)
                    ),
                    Span(25, 26)
                ),
                (Instruction::r#return(true), Span(36, 36)),
            ],
            vec![
                ConcreteValue::Float(f64::MIN),
                ConcreteValue::Float(0.0000001),
            ],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Float(f64::MIN))));
}

#[test]
fn subtract_integers() {
    let source = "1 - 2";

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
                    Instruction::subtract(
                        Destination::Register(0),
                        Argument::Constant(0),
                        Argument::Constant(1)
                    ),
                    Span(2, 3)
                ),
                (Instruction::r#return(true), Span(5, 5)),
            ],
            vec![ConcreteValue::Integer(1), ConcreteValue::Integer(2)],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Integer(-1))));
}

#[test]
fn subtract_integers_saturate() {
    let source = "-9223372036854775808 - 1";

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
                    Instruction::subtract(
                        Destination::Register(0),
                        Argument::Constant(0),
                        Argument::Constant(1)
                    ),
                    Span(21, 22)
                ),
                (Instruction::r#return(true), Span(24, 24)),
            ],
            vec![ConcreteValue::Integer(i64::MIN), ConcreteValue::Integer(1),],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Integer(i64::MIN))));
}
