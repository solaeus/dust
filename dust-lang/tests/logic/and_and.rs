use dust_lang::*;

#[test]
fn true_and_true_and_true() {
    let source = "true && true && true";

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
                (Instruction::test(Argument::Register(1), true), Span(13, 15)),
                (Instruction::jump(1, true), Span(13, 15)),
                (Instruction::load_boolean(2, true, false), Span(16, 20)),
                (Instruction::r#return(true), Span(20, 20)),
            ],
            vec![],
            vec![]
        ))
    );
}
