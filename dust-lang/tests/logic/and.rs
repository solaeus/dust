use dust_lang::*;

#[test]
fn true_and_true() {
    let source = "true && true";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Type::Boolean,
            },
            vec![
                (Instruction::load_boolean(0, true, false), Span(0, 4)),
                (Instruction::test(Argument::Register(0), true), Span(5, 7)),
                (Instruction::jump(1, true), Span(5, 7)),
                (Instruction::load_boolean(1, true, false), Span(8, 12)),
                (Instruction::r#return(true), Span(12, 12)),
            ],
            vec![],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Boolean(true))));
}

#[test]
fn false_and_false() {
    let source = "false && false";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Type::Boolean,
            },
            vec![
                (Instruction::load_boolean(0, false, false), Span(0, 5)),
                (Instruction::test(Argument::Register(0), true), Span(6, 8)),
                (Instruction::jump(1, true), Span(6, 8)),
                (Instruction::load_boolean(1, false, false), Span(9, 14)),
                (Instruction::r#return(true), Span(14, 14)),
            ],
            vec![],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Boolean(false))));
}

#[test]
fn false_and_true() {
    let source = "false && true";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Type::Boolean,
            },
            vec![
                (Instruction::load_boolean(0, false, false), Span(0, 5)),
                (Instruction::test(Argument::Register(0), true), Span(6, 8)),
                (Instruction::jump(1, true), Span(6, 8)),
                (Instruction::load_boolean(1, true, false), Span(9, 13)),
                (Instruction::r#return(true), Span(13, 13)),
            ],
            vec![],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Boolean(false))));
}

#[test]
fn true_and_false() {
    let source = "true && false";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Type::Boolean,
            },
            vec![
                (Instruction::load_boolean(0, true, false), Span(0, 4)),
                (Instruction::test(Argument::Register(0), true), Span(5, 7)),
                (Instruction::jump(1, true), Span(5, 7)),
                (Instruction::load_boolean(1, false, false), Span(8, 13)),
                (Instruction::r#return(true), Span(13, 13)),
            ],
            vec![],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Boolean(false))));
}
