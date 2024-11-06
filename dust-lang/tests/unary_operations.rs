use dust_lang::*;

#[test]
fn negate() {
    let source = "-(42)";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (*Instruction::negate(0, 0).set_b_is_constant(), Span(0, 1)),
                (Instruction::r#return(true), Span(5, 5)),
            ],
            vec![Value::integer(42)],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::integer(-42))));
}

#[test]
fn not() {
    let source = "!true";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_boolean(0, true, false), Span(1, 5)),
                (Instruction::not(1, 0), Span(0, 1)),
                (Instruction::r#return(true), Span(5, 5)),
            ],
            vec![],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::boolean(false))));
}
