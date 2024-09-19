use crate::Local;

use super::*;

#[test]
fn equality_assignment_long() {
    let source = "let a = if 4 == 4 { true } else { false };";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            vec![
                (
                    *Instruction::equal(true, 0, 1)
                        .set_first_argument_to_constant()
                        .set_second_argument_to_constant(),
                    Span(13, 15)
                ),
                (Instruction::jump(1, true), Span(13, 15)),
                (Instruction::load_boolean(0, true, true), Span(20, 24)),
                (Instruction::load_boolean(0, false, false), Span(34, 39)),
                (Instruction::define_local(0, 0, false), Span(4, 5)),
            ],
            vec![Value::integer(4), Value::integer(4),],
            vec![Local::new(Identifier::new("a"), false, 0, Some(0)),]
        )),
    );
}

#[test]
fn equality_assignment_short() {
    let source = "let a = 4 == 4;";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            vec![
                (
                    *Instruction::equal(true, 0, 1)
                        .set_first_argument_to_constant()
                        .set_second_argument_to_constant(),
                    Span(10, 12)
                ),
                (Instruction::jump(1, true), Span(10, 12)),
                (Instruction::load_boolean(0, true, true), Span(14, 15)),
                (Instruction::load_boolean(0, false, false), Span(14, 15)),
                (Instruction::define_local(0, 0, false), Span(4, 5)),
            ],
            vec![Value::integer(4), Value::integer(4),],
            vec![Local::new(Identifier::new("a"), false, 0, Some(0)),]
        )),
    );
}

#[test]
fn if_else_expression() {
    let source = "if 1 == 1 { 2 } else { 3 }";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            vec![
                (
                    *Instruction::equal(true, 0, 1)
                        .set_first_argument_to_constant()
                        .set_second_argument_to_constant(),
                    Span(5, 7)
                ),
                (Instruction::jump(1, true), Span(5, 7)),
                (Instruction::load_constant(0, 2), Span(12, 13)),
                (Instruction::load_constant(1, 3), Span(23, 24)),
                (Instruction::r#return(), Span(0, 26)),
            ],
            vec![
                Value::integer(1),
                Value::integer(1),
                Value::integer(2),
                Value::integer(3)
            ],
            vec![]
        )),
    );
}

#[test]
fn list_with_expression() {
    let source = "[1, 2 + 3, 4]";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            vec![
                (Instruction::load_constant(0, 0), Span(1, 2)),
                (
                    *Instruction::add(1, 1, 2)
                        .set_first_argument_to_constant()
                        .set_second_argument_to_constant(),
                    Span(6, 7)
                ),
                (Instruction::load_constant(2, 3), Span(11, 12)),
                (Instruction::load_list(3, 3), Span(0, 13)),
                (Instruction::r#return(), Span(0, 13)),
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
}

#[test]
fn list() {
    let source = "[1, 2, 3]";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            vec![
                (Instruction::load_constant(0, 0), Span(1, 2)),
                (Instruction::load_constant(1, 1), Span(4, 5)),
                (Instruction::load_constant(2, 2), Span(7, 8)),
                (Instruction::load_list(3, 3), Span(0, 9)),
                (Instruction::r#return(), Span(0, 9)),
            ],
            vec![Value::integer(1), Value::integer(2), Value::integer(3),],
            vec![]
        )),
    );
}

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
                (Instruction::define_local(0, 0, false), Span(13, 14)),
                (Instruction::load_constant(1, 1), Span(50, 52)),
                (Instruction::define_local(1, 1, false), Span(46, 47)),
                (Instruction::load_constant(2, 2), Span(92, 93)),
                (Instruction::define_local(2, 2, false), Span(88, 89)),
                (Instruction::load_constant(3, 3), Span(129, 130)),
                (Instruction::define_local(3, 3, false), Span(125, 126)),
                (Instruction::load_constant(4, 4), Span(158, 159)),
                (Instruction::define_local(4, 4, false), Span(154, 155)),
            ],
            vec![
                Value::integer(0),
                Value::integer(42),
                Value::integer(1),
                Value::integer(2),
                Value::integer(1)
            ],
            vec![
                Local::new(Identifier::new("a"), false, 0, Some(0)),
                Local::new(Identifier::new("b"), false, 1, Some(1)),
                Local::new(Identifier::new("c"), false, 2, Some(2)),
                Local::new(Identifier::new("d"), false, 1, Some(3)),
                Local::new(Identifier::new("e"), false, 0, Some(4)),
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
        parse("let mut x = 41; x = 42;"),
        Ok(Chunk::with_data(
            vec![
                (Instruction::load_constant(0, 0), Span(12, 14)),
                (Instruction::define_local(0, 0, true), Span(8, 9)),
                (Instruction::load_constant(1, 1), Span(20, 22)),
                (Instruction::set_local(1, 0), Span(16, 17)),
            ],
            vec![Value::integer(41), Value::integer(42)],
            vec![Local::new(Identifier::new("x"), true, 0, Some(0)),]
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
                (Instruction::define_local(0, 0, false), Span(4, 5)),
            ],
            vec![Value::integer(42)],
            vec![Local::new(Identifier::new("x"), false, 0, Some(0))]
        )),
    );
}

#[test]
fn and() {
    assert_eq!(
        parse("1 && 2"),
        Ok(Chunk::with_data(
            vec![
                (
                    *Instruction::and(0, 0, 1)
                        .set_first_argument_to_constant()
                        .set_second_argument_to_constant(),
                    Span(2, 4)
                ),
                (Instruction::r#return(), Span(0, 6)),
            ],
            vec![Value::integer(1), Value::integer(2)],
            vec![]
        ))
    );
}

#[test]
fn divide() {
    assert_eq!(
        parse("1 / 2"),
        Ok(Chunk::with_data(
            vec![
                (
                    *Instruction::divide(0, 0, 1)
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
fn multiply() {
    assert_eq!(
        parse("1 * 2"),
        Ok(Chunk::with_data(
            vec![
                (
                    *Instruction::multiply(0, 0, 1)
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
