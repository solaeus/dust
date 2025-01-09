use dust_lang::*;

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

#[test]
fn modulo_assign_expects_mutable_variable() {
    let source = "1 %= 2";

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
