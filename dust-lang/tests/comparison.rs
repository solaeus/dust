use dust_lang::*;

#[test]
fn equal() {
    let source = "1 == 2";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::equal(true, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(2, 4)
                ),
                (Instruction::jump(1, true), Span(2, 4)),
                (Instruction::load_boolean(0, true, true), Span(2, 4)),
                (Instruction::load_boolean(0, false, false), Span(2, 4)),
                (Instruction::r#return(true), Span(6, 6)),
            ],
            vec![Value::integer(1), Value::integer(2)],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::boolean(false))));
}

#[test]
fn greater() {
    let source = "1 > 2";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::less_equal(false, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(2, 3)
                ),
                (Instruction::jump(1, true), Span(2, 3)),
                (Instruction::load_boolean(0, true, true), Span(2, 3)),
                (Instruction::load_boolean(0, false, false), Span(2, 3)),
                (Instruction::r#return(true), Span(5, 5)),
            ],
            vec![Value::integer(1), Value::integer(2)],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::boolean(false))));
}

#[test]
fn greater_than_or_equal() {
    let source = "1 >= 2";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::less(false, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(2, 4)
                ),
                (Instruction::jump(1, true), Span(2, 4)),
                (Instruction::load_boolean(0, true, true), Span(2, 4)),
                (Instruction::load_boolean(0, false, false), Span(2, 4)),
                (Instruction::r#return(true), Span(6, 6)),
            ],
            vec![Value::integer(1), Value::integer(2)],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::boolean(false))));
}

#[test]
fn less_than() {
    let source = "1 < 2";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::less(true, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(2, 3)
                ),
                (Instruction::jump(1, true), Span(2, 3)),
                (Instruction::load_boolean(0, true, true), Span(2, 3)),
                (Instruction::load_boolean(0, false, false), Span(2, 3)),
                (Instruction::r#return(true), Span(5, 5)),
            ],
            vec![Value::integer(1), Value::integer(2)],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::boolean(true))));
}

#[test]
fn less_than_or_equal() {
    let source = "1 <= 2";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::less_equal(true, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(2, 4)
                ),
                (Instruction::jump(1, true), Span(2, 4)),
                (Instruction::load_boolean(0, true, true), Span(2, 4)),
                (Instruction::load_boolean(0, false, false), Span(2, 4)),
                (Instruction::r#return(true), Span(6, 6)),
            ],
            vec![Value::integer(1), Value::integer(2)],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::boolean(true))));
}

#[test]
fn not_equal() {
    let source = "1 != 2";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::equal(false, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(2, 4)
                ),
                (Instruction::jump(1, true), Span(2, 4)),
                (Instruction::load_boolean(0, true, true), Span(2, 4)),
                (Instruction::load_boolean(0, false, false), Span(2, 4)),
                (Instruction::r#return(true), Span(6, 6)),
            ],
            vec![Value::integer(1), Value::integer(2)],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::boolean(true))));
}