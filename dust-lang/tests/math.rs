use dust_lang::*;

#[test]
fn add() {
    let source = "1 + 2";

    assert_eq!(
        compile(source),
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
        compile(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(12, 13)),
                (Instruction::define_local(0, 0, true), Span(8, 9)),
                (*Instruction::add(0, 0, 2).set_c_is_constant(), Span(17, 19)),
                (Instruction::get_local(1, 0), Span(23, 24)),
                (Instruction::r#return(true), Span(24, 24))
            ],
            vec![Value::integer(1), Value::string("a"), Value::integer(2)],
            vec![Local::new(1, None, true, Scope::default(), 0)]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::integer(3))));
}

#[test]
fn add_assign_expects_mutable_variable() {
    let source = "1 += 2";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::ExpectedMutableVariable {
                found: Token::Integer("1").to_owned(),
                position: Span(0, 1)
            },
            source
        })
    );
}

// #[test]
// fn add_expects_integer_float_or_string() {
//     let source = "true + false";

//     assert_eq!(
//         parse(source),
//         Err(DustError::Parse {
//             error: ParseError::ExpectedIntegerFloatOrString {
//                 found: Token::True,
//                 position: Span(0, 3)
//             },
//             source
//         })
//     );
// }

#[test]
fn divide() {
    let source = "2 / 2";

    assert_eq!(
        compile(source),
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
        compile(source),
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
            vec![Value::integer(2), Value::string("a")],
            vec![Local::new(1, None, true, Scope::default(), 0)]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::integer(1))));
}

#[test]
fn divide_assign_expects_mutable_variable() {
    let source = "1 -= 2";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::ExpectedMutableVariable {
                found: Token::Integer("1").to_owned(),
                position: Span(0, 1)
            },
            source
        })
    );
}

#[test]
fn math_operator_precedence() {
    let source = "1 + 2 - 3 * 4 / 5";

    assert_eq!(
        compile(source),
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
        compile(source),
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
        compile(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(12, 13)),
                (Instruction::define_local(0, 0, true), Span(8, 9)),
                (
                    *Instruction::multiply(0, 0, 2).set_c_is_constant(),
                    Span(17, 19)
                ),
                (Instruction::get_local(1, 0), Span(22, 23)),
                (Instruction::r#return(true), Span(23, 23))
            ],
            vec![Value::integer(2), Value::string("a"), Value::integer(3)],
            vec![Local::new(1, None, true, Scope::default(), 0),]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::integer(6))));
}

#[test]
fn multiply_assign_expects_mutable_variable() {
    let source = "1 *= 2";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::ExpectedMutableVariable {
                found: Token::Integer("1").to_owned(),
                position: Span(0, 1)
            },
            source
        })
    );
}

#[test]
fn subtract() {
    let source = "1 - 2";

    assert_eq!(
        compile(source),
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
        compile(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(12, 14)),
                (Instruction::define_local(0, 0, true), Span(8, 9)),
                (
                    *Instruction::subtract(0, 0, 2).set_c_is_constant(),
                    Span(18, 20)
                ),
                (Instruction::get_local(1, 0), Span(24, 25)),
                (Instruction::r#return(true), Span(25, 25)),
            ],
            vec![Value::integer(42), Value::string("x"), Value::integer(2)],
            vec![Local::new(1, None, true, Scope::default(), 0)]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::integer(40))));
}

#[test]
fn subtract_assign_expects_mutable_variable() {
    let source = "1 -= 2";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::ExpectedMutableVariable {
                found: Token::Integer("1").to_owned(),
                position: Span(0, 1)
            },
            source
        })
    );
}
