use dust_lang::*;

#[test]
fn empty_list() {
    let source = "[]";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_list(0, 0), Span(0, 2)),
                (Instruction::r#return(true), Span(2, 2)),
            ],
            vec![],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::abstract_list(0, 0, Type::Any))));
}

#[test]
fn list() {
    let source = "[1, 2, 3]";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(1, 2)),
                (Instruction::load_constant(1, 1, false), Span(4, 5)),
                (Instruction::load_constant(2, 2, false), Span(7, 8)),
                (Instruction::load_list(3, 0), Span(0, 9)),
                (Instruction::r#return(true), Span(9, 9)),
            ],
            vec![Value::integer(1), Value::integer(2), Value::integer(3)],
            vec![]
        )),
    );

    assert_eq!(
        run(source),
        Ok(Some(Value::abstract_list(0, 3, Type::Integer)))
    );
}

#[test]
fn list_with_complex_expression() {
    let source = "[1, 2 + 3 - 4 * 5]";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(1, 2)),
                (
                    *Instruction::add(1, 1, 2)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(6, 7)
                ),
                (
                    *Instruction::multiply(2, 3, 4)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(14, 15)
                ),
                (Instruction::subtract(3, 1, 2), Span(10, 11)),
                (Instruction::close(1, 3), Span(17, 18)),
                (Instruction::load_list(4, 0), Span(0, 18)),
                (Instruction::r#return(true), Span(18, 18)),
            ],
            vec![
                Value::integer(1),
                Value::integer(2),
                Value::integer(3),
                Value::integer(4),
                Value::integer(5)
            ],
            vec![]
        )),
    );

    assert_eq!(
        run(source),
        Ok(Some(Value::abstract_list(0, 4, Type::Integer)))
    );
}

#[test]
fn list_with_simple_expression() {
    let source = "[1, 2 + 3, 4]";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(1, 2)),
                (
                    *Instruction::add(1, 1, 2)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(6, 7)
                ),
                (Instruction::load_constant(2, 3, false), Span(11, 12)),
                (Instruction::load_list(3, 0), Span(0, 13)),
                (Instruction::r#return(true), Span(13, 13)),
            ],
            vec![
                Value::integer(1),
                Value::integer(2),
                Value::integer(3),
                Value::integer(4),
            ],
            vec![]
        )),
    );

    assert_eq!(
        run(source),
        Ok(Some(Value::abstract_list(0, 3, Type::Integer)))
    );
}
