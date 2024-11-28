use dust_lang::*;

#[test]
fn negate() {
    let source = "-(42)";

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
                    Instruction::negate(Destination::Register(0), Argument::Constant(0)),
                    Type::Integer,
                    Span(0, 1)
                ),
                (Instruction::r#return(true), Type::None, Span(5, 5)),
            ],
            vec![ConcreteValue::Integer(42)],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Integer(-42))));
}

#[test]
fn not() {
    let source = "!true";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::Boolean),
            },
            vec![
                (
                    Instruction::load_boolean(Destination::Register(0), true, false),
                    Type::Boolean,
                    Span(1, 5)
                ),
                (
                    Instruction::not(Destination::Register(1), Argument::Register(0)),
                    Type::Boolean,
                    Span(0, 1)
                ),
                (Instruction::r#return(true), Type::None, Span(5, 5)),
            ],
            vec![],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Boolean(false))));
}
