use dust_lang::*;

#[test]
fn and() {
    let source = "true && false";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_boolean(0, true, false), Span(0, 4)),
                (Instruction::test(0, false), Span(5, 7)),
                (Instruction::jump(1, true), Span(5, 7)),
                (Instruction::load_boolean(1, false, false), Span(8, 13)),
                (Instruction::r#return(true), Span(13, 13)),
            ],
            vec![],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::boolean(false))));
}

#[test]
fn or() {
    let source = "true || false";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_boolean(0, true, false), Span(0, 4)),
                (Instruction::test(0, true), Span(5, 7)),
                (Instruction::jump(1, true), Span(5, 7)),
                (Instruction::load_boolean(1, false, false), Span(8, 13)),
                (Instruction::r#return(true), Span(13, 13)),
            ],
            vec![],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::boolean(true))));
}
