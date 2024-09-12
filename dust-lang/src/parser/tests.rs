use super::*;

#[test]
fn integer() {
    assert_eq!(
        parse("42"),
        Ok(Chunk::with_data(
            vec![
                (Instruction::load_constant(0, 0), Span(0, 2)),
                (Instruction::r#return(), Span(0, 2)),
            ],
            vec![Value::integer(42),],
            vec![]
        ))
    );
}

#[test]
fn add() {
    assert_eq!(
        parse("1 + 2"),
        Ok(Chunk::with_data(
            vec![
                (Instruction::load_constant(0, 0), Span(0, 1)),
                (Instruction::load_constant(1, 1), Span(4, 5)),
                (Instruction::add(2, 1, 0), Span(2, 3)),
                (Instruction::r#return(), Span(0, 5)),
            ],
            vec![Value::integer(1), Value::integer(2),],
            vec![]
        ))
    );
}
