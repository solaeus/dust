use crate::{
    Lexer, Span,
    token::{Token, TokenKind},
};

#[test]
fn single_identifier() {
    let source = b"foo";
    let tokens = Lexer::new(source).try_collect::<Vec<Token>>().unwrap();

    assert_eq!(
        tokens,
        vec![
            Token {
                kind: TokenKind::Identifier,
                span: Span(0, 3)
            },
            Token {
                kind: TokenKind::Eof,
                span: Span(3, 3)
            }
        ]
    );
}

#[test]
fn identifier_with_digits_and_underscores() {
    let source = b"a1_b2";
    let tokens = Lexer::new(source).try_collect::<Vec<Token>>().unwrap();

    assert_eq!(
        tokens,
        vec![
            Token {
                kind: TokenKind::Identifier,
                span: Span(0, 5)
            },
            Token {
                kind: TokenKind::Eof,
                span: Span(5, 5)
            }
        ]
    );
}

#[test]
fn multiple_identifiers() {
    let source = b"foo bar_baz qux123";
    let tokens = Lexer::new(source).try_collect::<Vec<Token>>().unwrap();

    assert_eq!(
        tokens,
        vec![
            Token {
                kind: TokenKind::Identifier,
                span: Span(0, 3)
            },
            Token {
                kind: TokenKind::Identifier,
                span: Span(4, 11)
            },
            Token {
                kind: TokenKind::Identifier,
                span: Span(12, 18)
            },
            Token {
                kind: TokenKind::Eof,
                span: Span(18, 18)
            }
        ]
    );
}

#[test]
fn booleans() {
    let source = b"true false";
    let tokens = Lexer::new(source).try_collect::<Vec<Token>>().unwrap();

    assert_eq!(
        tokens,
        vec![
            Token {
                kind: TokenKind::TrueValue,
                span: Span(0, 4)
            },
            Token {
                kind: TokenKind::FalseValue,
                span: Span(5, 10)
            },
            Token {
                kind: TokenKind::Eof,
                span: Span(10, 10)
            }
        ]
    );
}

#[test]
fn bytes() {
    let source = b"0x42 0xFF";
    let tokens = Lexer::new(source).try_collect::<Vec<Token>>().unwrap();

    assert_eq!(
        tokens,
        vec![
            Token {
                kind: TokenKind::ByteValue,
                span: Span(0, 4)
            },
            Token {
                kind: TokenKind::ByteValue,
                span: Span(5, 9)
            },
            Token {
                kind: TokenKind::Eof,
                span: Span(9, 9)
            }
        ]
    );
}

#[test]
fn characters() {
    let source = b"'a' 'b' 'c'";
    let tokens = Lexer::new(source).try_collect::<Vec<Token>>().unwrap();

    assert_eq!(
        tokens,
        vec![
            Token {
                kind: TokenKind::CharacterValue,
                span: Span(0, 3)
            },
            Token {
                kind: TokenKind::CharacterValue,
                span: Span(4, 7)
            },
            Token {
                kind: TokenKind::CharacterValue,
                span: Span(8, 11)
            },
            Token {
                kind: TokenKind::Eof,
                span: Span(11, 11)
            }
        ]
    );
}

#[test]
fn floats() {
    let source = b"3.14 0.001 42.0";
    let tokens = Lexer::new(source).try_collect::<Vec<Token>>().unwrap();

    assert_eq!(
        tokens,
        vec![
            Token {
                kind: TokenKind::FloatValue,
                span: Span(0, 4)
            },
            Token {
                kind: TokenKind::FloatValue,
                span: Span(5, 10)
            },
            Token {
                kind: TokenKind::FloatValue,
                span: Span(11, 15)
            },
            Token {
                kind: TokenKind::Eof,
                span: Span(15, 15)
            }
        ]
    );
}

#[test]
fn integers() {
    let source = b"0 123 456789";
    let tokens = Lexer::new(source).try_collect::<Vec<Token>>().unwrap();

    assert_eq!(
        tokens,
        vec![
            Token {
                kind: TokenKind::IntegerValue,
                span: Span(0, 1)
            },
            Token {
                kind: TokenKind::IntegerValue,
                span: Span(2, 5)
            },
            Token {
                kind: TokenKind::IntegerValue,
                span: Span(6, 12)
            },
            Token {
                kind: TokenKind::Eof,
                span: Span(12, 12)
            }
        ]
    );
}

#[test]
fn strings() {
    let source = b"\"hello\" \"world\"";
    let tokens = Lexer::new(source).try_collect::<Vec<Token>>().unwrap();

    assert_eq!(
        tokens,
        vec![
            Token {
                kind: TokenKind::StringValue,
                span: Span(0, 7)
            },
            Token {
                kind: TokenKind::StringValue,
                span: Span(8, 15)
            },
            Token {
                kind: TokenKind::Eof,
                span: Span(15, 15)
            }
        ]
    );
}

#[test]
fn keywords() {
    let keywords = [
        ("any", TokenKind::Any),
        ("async", TokenKind::Async),
        ("bool", TokenKind::Bool),
        ("break", TokenKind::Break),
        ("byte", TokenKind::Byte),
        ("cell", TokenKind::Cell),
        ("char", TokenKind::Char),
        ("const", TokenKind::Const),
        ("else", TokenKind::Else),
        ("float", TokenKind::Float),
        ("fn", TokenKind::Fn),
        ("if", TokenKind::If),
        ("int", TokenKind::Int),
        ("let", TokenKind::Let),
        ("list", TokenKind::List),
        ("loop", TokenKind::Loop),
        ("map", TokenKind::Map),
        ("mod", TokenKind::Mod),
        ("mut", TokenKind::Mut),
        ("pub", TokenKind::Pub),
        ("return", TokenKind::Return),
        ("str", TokenKind::Str),
        ("struct", TokenKind::Struct),
        ("use", TokenKind::Use),
        ("while", TokenKind::While),
    ];

    let source = keywords
        .iter()
        .map(|(str, _)| *str)
        .collect::<Vec<_>>()
        .join(" ");
    let expected = keywords.iter().map(|(_, kind)| *kind).collect::<Vec<_>>();
    let actual = Lexer::new(source.as_bytes())
        .map(|result| result.map(|token| token.kind))
        .try_collect::<Vec<TokenKind>>()
        .unwrap();

    assert_eq!(actual[..actual.len() - 1], expected);
}

#[test]
fn operators_and_punctuation() {
    let symbols = [
        ("->", TokenKind::ArrowThin),
        ("*", TokenKind::Asterisk),
        ("*=", TokenKind::AsteriskEqual),
        ("!=", TokenKind::BangEqual),
        ("!", TokenKind::Bang),
        (":", TokenKind::Colon),
        (",", TokenKind::Comma),
        (".", TokenKind::Dot),
        ("&&", TokenKind::DoubleAmpersand),
        ("::", TokenKind::DoubleColon),
        ("..", TokenKind::DoubleDot),
        ("==", TokenKind::DoubleEqual),
        ("||", TokenKind::DoublePipe),
        ("=", TokenKind::Equal),
        (">", TokenKind::Greater),
        (">=", TokenKind::GreaterEqual),
        ("{", TokenKind::LeftCurlyBrace),
        ("[", TokenKind::LeftSquareBracket),
        ("(", TokenKind::LeftParenthesis),
        ("<", TokenKind::Less),
        ("<=", TokenKind::LessEqual),
        ("-", TokenKind::Minus),
        ("-=", TokenKind::MinusEqual),
        ("%", TokenKind::Percent),
        ("%=", TokenKind::PercentEqual),
        ("+", TokenKind::Plus),
        ("+=", TokenKind::PlusEqual),
        ("}", TokenKind::RightCurlyBrace),
        ("]", TokenKind::RightSquareBracket),
        (")", TokenKind::RightParenthesis),
        (";", TokenKind::Semicolon),
        ("/", TokenKind::Slash),
        ("/=", TokenKind::SlashEqual),
    ];

    let source = symbols
        .iter()
        .map(|(str, _)| *str)
        .collect::<Vec<_>>()
        .join(" ");
    let expected = symbols.iter().map(|(_, kind)| *kind).collect::<Vec<_>>();
    let actual = Lexer::new(source.as_bytes())
        .map(|result| result.map(|token| token.kind))
        .try_collect::<Vec<TokenKind>>()
        .unwrap();

    assert_eq!(actual[..actual.len() - 1], expected);
}

#[test]
fn adjacent_tokens() {
    let source = b"let x:int=42;";
    let tokens = Lexer::new(source).try_collect::<Vec<Token>>().unwrap();

    assert_eq!(
        tokens,
        vec![
            Token {
                kind: TokenKind::Let,
                span: Span(0, 3)
            },
            Token {
                kind: TokenKind::Identifier,
                span: Span(4, 5)
            },
            Token {
                kind: TokenKind::Colon,
                span: Span(5, 6)
            },
            Token {
                kind: TokenKind::Int,
                span: Span(6, 9)
            },
            Token {
                kind: TokenKind::Equal,
                span: Span(9, 10)
            },
            Token {
                kind: TokenKind::IntegerValue,
                span: Span(10, 12)
            },
            Token {
                kind: TokenKind::Semicolon,
                span: Span(12, 13)
            },
            Token {
                kind: TokenKind::Eof,
                span: Span(13, 13)
            }
        ]
    );
}
