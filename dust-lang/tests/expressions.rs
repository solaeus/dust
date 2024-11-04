use dust_lang::*;

#[test]
fn add() {
    let source = "1 + 2";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::add(0, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(2, 3)
                ),
                (Instruction::r#return(true), Span(5, 5))
            ],
            vec![Value::integer(1), Value::integer(2)],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::integer(3))));
}

#[test]
fn add_assign() {
    let source = "let mut a = 1; a += 2; a";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(12, 13)),
                (Instruction::define_local(0, 0, true), Span(8, 9)),
                (*Instruction::add(0, 0, 1).set_c_is_constant(), Span(17, 19)),
                (Instruction::get_local(1, 0), Span(23, 24)),
                (Instruction::r#return(true), Span(24, 24))
            ],
            vec![Value::integer(1), Value::integer(2)],
            vec![Local::new(0, None, true, 0, 0)]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::integer(3))));
}

#[test]
fn constant() {
    let source = "42";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(0, 2)),
                (Instruction::r#return(true), Span(2, 2))
            ],
            vec![Value::integer(42)],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::integer(42))));
}

#[test]
fn define_local() {
    let source = "let x = 42;";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(8, 10)),
                (Instruction::define_local(0, 0, false), Span(4, 5)),
                (Instruction::r#return(false), Span(11, 11))
            ],
            vec![Value::integer(42)],
            vec![Local::new(0, None, false, 0, 0)]
        )),
    );

    assert_eq!(run(source), Ok(None));
}

#[test]
fn divide() {
    let source = "2 / 2";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::divide(0, 0, 0)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(2, 3)
                ),
                (Instruction::r#return(true), Span(5, 5))
            ],
            vec![Value::integer(2)],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::integer(1))));
}

#[test]
fn divide_assign() {
    let source = "let mut a = 2; a /= 2; a";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(12, 13)),
                (Instruction::define_local(0, 0, true), Span(8, 9)),
                (
                    *Instruction::divide(0, 0, 0).set_c_is_constant(),
                    Span(17, 19)
                ),
                (Instruction::get_local(1, 0), Span(23, 24)),
                (Instruction::r#return(true), Span(24, 24))
            ],
            vec![Value::integer(2)],
            vec![Local::new(0, None, true, 0, 0)]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::integer(1))));
}

#[test]
fn empty() {
    let source = "";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![(Instruction::r#return(false), Span(0, 0))],
            vec![],
            vec![]
        ))
    );
    assert_eq!(run(source), Ok(None));
}

#[test]
fn empty_list() {
    let source = "[]";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_list(0, 0, 0), Span(0, 2)),
                (Instruction::r#return(true), Span(2, 2)),
            ],
            vec![],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::list(0, 0, Type::Any))));
}

#[test]
fn list() {
    let source = "[1, 2, 3]";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(1, 2)),
                (Instruction::load_constant(1, 1, false), Span(4, 5)),
                (Instruction::load_constant(2, 2, false), Span(7, 8)),
                (Instruction::load_list(3, 0, 2), Span(0, 9)),
                (Instruction::r#return(true), Span(9, 9)),
            ],
            vec![Value::integer(1), Value::integer(2), Value::integer(3),],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::list(0, 2, Type::Integer))));
}

#[test]
fn list_with_complex_expression() {
    let source = "[1, 2 + 3 - 4 * 5]";

    assert_eq!(
        parse(source),
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
                (Instruction::load_list(4, 0, 3), Span(0, 18)),
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

    assert_eq!(run(source), Ok(Some(Value::list(0, 3, Type::Integer))));
}

#[test]
fn list_with_simple_expression() {
    let source = "[1, 2 + 3, 4]";

    assert_eq!(
        parse(source),
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
                (Instruction::load_list(3, 0, 2), Span(0, 13)),
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

    assert_eq!(run(source), Ok(Some(Value::list(0, 2, Type::Integer))));
}

#[test]
fn math_operator_precedence() {
    let source = "1 + 2 - 3 * 4 / 5";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::add(0, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(2, 3)
                ),
                (
                    *Instruction::multiply(1, 2, 3)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(10, 11)
                ),
                (
                    *Instruction::divide(2, 1, 4).set_c_is_constant(),
                    Span(14, 15)
                ),
                (Instruction::subtract(3, 0, 2), Span(6, 7)),
                (Instruction::r#return(true), Span(17, 17)),
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

    assert_eq!(run(source), Ok(Some(Value::integer(1))));
}

#[test]
fn multiply() {
    let source = "1 * 2";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::multiply(0, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(2, 3)
                ),
                (Instruction::r#return(true), Span(5, 5)),
            ],
            vec![Value::integer(1), Value::integer(2)],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::integer(2))));
}

#[test]
fn multiply_assign() {
    let source = "let mut a = 2; a *= 3 a";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(12, 13)),
                (Instruction::define_local(0, 0, true), Span(8, 9)),
                (
                    *Instruction::multiply(0, 0, 1).set_c_is_constant(),
                    Span(17, 19)
                ),
                (Instruction::get_local(1, 0), Span(22, 23)),
                (Instruction::r#return(true), Span(23, 23))
            ],
            vec![Value::integer(2), Value::integer(3)],
            vec![Local::new(0, None, true, 0, 0),]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::integer(6))));
}

#[test]
fn negate() {
    let source = "-(42)";

    assert_eq!(
        parse(source),
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
        parse(source),
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

#[test]
fn parentheses_precedence() {
    let source = "(1 + 2) * 3";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::add(0, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(3, 4)
                ),
                (
                    *Instruction::multiply(1, 0, 2).set_c_is_constant(),
                    Span(8, 9)
                ),
                (Instruction::r#return(true), Span(11, 11)),
            ],
            vec![Value::integer(1), Value::integer(2), Value::integer(3)],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::integer(9))));
}

#[test]
fn set_local() {
    let source = "let mut x = 41; x = 42; x";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(12, 14)),
                (Instruction::define_local(0, 0, true), Span(8, 9)),
                (Instruction::load_constant(1, 1, false), Span(20, 22)),
                (Instruction::set_local(1, 0), Span(16, 17)),
                (Instruction::get_local(2, 0), Span(24, 25)),
                (Instruction::r#return(true), Span(25, 25)),
            ],
            vec![Value::integer(41), Value::integer(42)],
            vec![Local::new(0, None, true, 0, 0)]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::integer(42))));
}

#[test]
fn subtract() {
    let source = "1 - 2";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::subtract(0, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(2, 3)
                ),
                (Instruction::r#return(true), Span(5, 5)),
            ],
            vec![Value::integer(1), Value::integer(2)],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::integer(-1))));
}

#[test]
fn subtract_assign() {
    let source = "let mut x = 42; x -= 2; x";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(12, 14)),
                (Instruction::define_local(0, 0, true), Span(8, 9)),
                (
                    *Instruction::subtract(0, 0, 1).set_c_is_constant(),
                    Span(18, 20)
                ),
                (Instruction::get_local(1, 0), Span(24, 25)),
                (Instruction::r#return(true), Span(25, 25)),
            ],
            vec![Value::integer(42), Value::integer(2)],
            vec![Local::new(0, None, true, 0, 0)]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::integer(40))));
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
            vec![],
            vec![
                Local::new(0, None, false, 0, 0),
                Local::new(0, None, false, 0, 1),
            ]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::boolean(false))));
}

#[test]
fn r#while() {
    let source = "let mut x = 0; while x < 5 { x = x + 1 } x";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(12, 13)),
                (Instruction::define_local(0, 0, true), Span(8, 9)),
                (
                    *Instruction::less(true, 0, 1).set_c_is_constant(),
                    Span(23, 24)
                ),
                (Instruction::jump(2, true), Span(41, 42)),
                (*Instruction::add(0, 0, 2).set_c_is_constant(), Span(39, 40)),
                (Instruction::jump(3, false), Span(41, 42)),
                (Instruction::get_local(1, 0), Span(41, 42)),
                (Instruction::r#return(true), Span(42, 42)),
            ],
            vec![Value::integer(0), Value::integer(5), Value::integer(1),],
            vec![Local::new(0, None, true, 0, 0),]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::integer(5))));
}
