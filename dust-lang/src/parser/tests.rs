use crate::Local;

use super::*;

#[test]
fn block_scope() {
    let source = "
        let a = 0;
        {
            let b = 42;
            {
                let c = 1;
            }
            let d = 2;
        }
        let e = 1;
    ";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            vec![
                (Instruction::load_constant(0, 0), Span(17, 18)),
                (Instruction::declare_local(0, 0), Span(13, 14)),
                (Instruction::load_constant(1, 1), Span(50, 52)),
                (Instruction::declare_local(1, 1), Span(46, 47)),
                (Instruction::load_constant(2, 2), Span(92, 93)),
                (Instruction::declare_local(2, 2), Span(88, 89)),
                (Instruction::load_constant(3, 3), Span(129, 130)),
                (Instruction::declare_local(3, 3), Span(125, 126)),
                (Instruction::load_constant(4, 4), Span(158, 159)),
                (Instruction::declare_local(4, 4), Span(154, 155)),
            ],
            vec![
                Value::integer(0),
                Value::integer(42),
                Value::integer(1),
                Value::integer(2),
                Value::integer(1)
            ],
            vec![
                Local::new(Identifier::new("a"), 0, Some(0)),
                Local::new(Identifier::new("b"), 1, Some(1)),
                Local::new(Identifier::new("c"), 2, Some(2)),
                Local::new(Identifier::new("d"), 1, Some(3)),
                Local::new(Identifier::new("e"), 0, Some(4)),
            ]
        )),
    );
}

#[test]
fn empty() {
    assert_eq!(parse(""), Ok(Chunk::with_data(vec![], vec![], vec![])),);
}

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
fn math_operator_precedence() {
    assert_eq!(
        parse("1 + 2 - 3 * 4 / 5"),
        Ok(Chunk::with_data(
            vec![
                (
                    *Instruction::multiply(1, 2, 3)
                        .set_first_argument_to_constant()
                        .set_second_argument_to_constant(),
                    Span(10, 11)
                ),
                (
                    *Instruction::add(0, 0, 1)
                        .set_first_argument_to_constant()
                        .set_second_argument_to_constant(),
                    Span(2, 3)
                ),
                (
                    *Instruction::divide(2, 1, 4).set_second_argument_to_constant(),
                    Span(14, 15)
                ),
                (Instruction::subtract(3, 0, 2), Span(6, 7)),
                (Instruction::r#return(), Span(0, 17)),
            ],
            vec![
                Value::integer(1),
                Value::integer(2),
                Value::integer(3),
                Value::integer(4),
                Value::integer(5),
            ],
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
