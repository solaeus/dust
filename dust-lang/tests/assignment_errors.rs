use dust_lang::*;

#[test]
fn add_assign_expects_mutable_variable() {
    let source = "1 += 2";

    assert_eq!(
        parse(source),
        Err(DustError::Parse {
            error: ParseError::ExpectedMutableVariable {
                found: Token::Integer("1").to_owned(),
                position: Span(0, 1)
            },
            source
        })
    );
}

#[test]
fn divide_assign_expects_mutable_variable() {
    let source = "1 -= 2";

    assert_eq!(
        parse(source),
        Err(DustError::Parse {
            error: ParseError::ExpectedMutableVariable {
                found: Token::Integer("1").to_owned(),
                position: Span(0, 1)
            },
            source
        })
    );
}

#[test]
fn multiply_assign_expects_mutable_variable() {
    let source = "1 *= 2";

    assert_eq!(
        parse(source),
        Err(DustError::Parse {
            error: ParseError::ExpectedMutableVariable {
                found: Token::Integer("1").to_owned(),
                position: Span(0, 1)
            },
            source
        })
    );
}

#[test]
fn subtract_assign_expects_mutable_variable() {
    let source = "1 -= 2";

    assert_eq!(
        parse(source),
        Err(DustError::Parse {
            error: ParseError::ExpectedMutableVariable {
                found: Token::Integer("1").to_owned(),
                position: Span(0, 1)
            },
            source
        })
    );
}

#[test]
fn let_statement_expects_identifier() {
    let source = "let 1 = 2";

    assert_eq!(
        parse(source),
        Err(DustError::Parse {
            error: ParseError::ExpectedToken {
                expected: TokenKind::Identifier,
                found: Token::Integer("1").to_owned(),
                position: Span(4, 5)
            },
            source
        })
    );
}
