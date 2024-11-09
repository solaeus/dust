use dust_lang::*;

#[test]
fn define_local() {
    let source = "let x = 42;";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(8, 10)),
                (Instruction::define_local(0, 0, false), Span(4, 5)),
                (Instruction::r#return(false), Span(11, 11))
            ],
            vec![Value::integer(42), Value::string("x")],
            vec![Local::new(1, None, false, Scope::default())]
        )),
    );

    assert_eq!(run(source), Ok(None));
}

#[test]
fn let_statement_expects_identifier() {
    let source = "let 1 = 2";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::ExpectedToken {
                expected: TokenKind::Identifier,
                found: Token::Integer("1").to_owned(),
                position: Span(4, 5)
            },
            source
        })
    );
}

#[test]
fn set_local() {
    let source = "let mut x = 41; x = 42; x";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(12, 14)),
                (Instruction::define_local(0, 0, true), Span(8, 9)),
                (Instruction::load_constant(1, 2, false), Span(20, 22)),
                (Instruction::set_local(1, 0), Span(16, 17)),
                (Instruction::get_local(2, 0), Span(24, 25)),
                (Instruction::r#return(true), Span(25, 25)),
            ],
            vec![Value::integer(41), Value::string("x"), Value::integer(42)],
            vec![Local::new(1, None, true, Scope::default())]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::integer(42))));
}
