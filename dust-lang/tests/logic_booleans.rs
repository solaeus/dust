use dust_lang::*;

#[test]
fn and() {
    let source = "true && false";

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
                    Span(0, 4)
                ),
                (Instruction::test(Argument::Register(0), true), Span(5, 7)),
                (Instruction::jump(1, true), Span(5, 7)),
                (
                    Instruction::load_boolean(Destination::Register(1), false, false),
                    Span(8, 13)
                ),
                (Instruction::r#return(true), Span(13, 13)),
            ],
            vec![],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Boolean(false))));
}

#[test]
fn or() {
    let source = "true || false";

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
                    Span(0, 4)
                ),
                (Instruction::test(Argument::Register(0), false), Span(5, 7)),
                (Instruction::jump(1, true), Span(5, 7)),
                (
                    Instruction::load_boolean(Destination::Register(1), false, false),
                    Span(8, 13)
                ),
                (Instruction::r#return(true), Span(13, 13)),
            ],
            vec![],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Boolean(true))));
}
