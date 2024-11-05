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

#[test]
fn variable_and() {
    let source = "let a = true; let b = false; a && b";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_boolean(0, true, false), Span(8, 12)),
                (Instruction::define_local(0, 0, false), Span(4, 5)),
                (Instruction::load_boolean(1, false, false), Span(22, 27)),
                (Instruction::define_local(1, 1, false), Span(18, 19)),
                (Instruction::get_local(2, 0), Span(29, 30)),
                (Instruction::test(2, false), Span(31, 33)),
                (Instruction::jump(1, true), Span(31, 33)),
                (Instruction::get_local(3, 1), Span(34, 35)),
                (Instruction::r#return(true), Span(35, 35)),
            ],
            vec![Value::string("a"), Value::string("b"),],
            vec![
                Local::new(0, None, false, Scope::default(), 0),
                Local::new(1, None, false, Scope::default(), 1),
            ]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::boolean(false))));
}
