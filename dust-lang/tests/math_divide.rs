use dust_lang::*;

#[test]
fn divide_floats() {
    let source = "2.0 / 2.0";

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
                    Instruction::divide(
                        Destination::Register(0),
                        Argument::Constant(0),
                        Argument::Constant(0)
                    ),
                    Span(4, 5)
                ),
                (Instruction::r#return(true), Span(9, 9))
            ],
            vec![ConcreteValue::Float(2.0)],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Float(1.0))));
}

#[test]
fn divide_integers() {
    let source = "2 / 2";

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
                    Instruction::divide(
                        Destination::Register(0),
                        Argument::Constant(0),
                        Argument::Constant(0)
                    ),
                    Span(2, 3)
                ),
                (Instruction::r#return(true), Span(5, 5))
            ],
            vec![ConcreteValue::Integer(2)],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Integer(1))));
}
