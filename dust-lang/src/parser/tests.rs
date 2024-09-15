use crate::Local;

use super::*;

#[test]
fn set_local() {
    assert_eq!(
        parse("let x = 41; x = 42;"),
        Ok(Chunk::with_data(
            vec![
                (Instruction::load_constant(0, 0), Span(8, 10)),
                (Instruction::declare_local(0, 0), Span(4, 5)),
                (Instruction::load_constant(1, 1), Span(16, 18)),
                (Instruction::set_local(1, 0), Span(12, 13)),
            ],
            vec![Value::integer(41), Value::integer(42)],
            vec![Local::new(Identifier::new("x"), 0, Some(0)),]
        )),
    );
}

#[test]
fn parentheses_precedence() {
    assert_eq!(
        parse("(1 + 2) * 3"),
        Ok(Chunk::with_data(
            vec![
                (
                    *Instruction::add(0, 0, 1)
                        .set_first_argument_to_constant()
                        .set_second_argument_to_constant(),
                    Span(3, 4)
                ),
                (
                    *Instruction::multiply(1, 0, 2).set_second_argument_to_constant(),
                    Span(8, 9)
                ),
                (Instruction::r#return(), Span(0, 11)),
            ],
            vec![Value::integer(1), Value::integer(2), Value::integer(3)],
            vec![]
        ))
    );
}

#[test]
fn add_multiply_precedence() {
    assert_eq!(
        parse("1 + 2 * 3"),
        Ok(Chunk::with_data(
            vec![
                (
                    *Instruction::multiply(0, 1, 2)
                        .set_first_argument_to_constant()
                        .set_second_argument_to_constant(),
                    Span(6, 7)
                ),
                (
                    *Instruction::add(1, 0, 0).set_first_argument_to_constant(),
                    Span(2, 3)
                ),
                (Instruction::r#return(), Span(0, 9)),
            ],
            vec![Value::integer(1), Value::integer(2), Value::integer(3)],
            vec![]
        ))
    );
}

#[test]
fn declare_local() {
    assert_eq!(
        parse("let x = 42;"),
        Ok(Chunk::with_data(
            vec![
                (Instruction::load_constant(0, 0), Span(8, 10)),
                (Instruction::declare_local(0, 0), Span(4, 5)),
            ],
            vec![Value::integer(42)],
            vec![Local::new(Identifier::new("x"), 0, Some(0))]
        )),
    );
}

#[test]
fn constant() {
    assert_eq!(
        parse("42"),
        Ok(Chunk::with_data(
            vec![
                (Instruction::load_constant(0, 0), Span(0, 2)),
                (Instruction::r#return(), Span(0, 2)),
            ],
            vec![Value::integer(42)],
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
                (
                    *Instruction::add(0, 0, 1)
                        .set_first_argument_to_constant()
                        .set_second_argument_to_constant(),
                    Span(2, 3)
                ),
                (Instruction::r#return(), Span(0, 5)),
            ],
            vec![Value::integer(1), Value::integer(2)],
            vec![]
        ))
    );
}

#[test]
fn subtract() {
    assert_eq!(
        parse("1 - 2"),
        Ok(Chunk::with_data(
            vec![
                (
                    *Instruction::subtract(0, 0, 1)
                        .set_first_argument_to_constant()
                        .set_second_argument_to_constant(),
                    Span(2, 3)
                ),
                (Instruction::r#return(), Span(0, 5)),
            ],
            vec![Value::integer(1), Value::integer(2)],
            vec![]
        ))
    );
}
