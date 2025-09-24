use crate::{Lexer, Span, TokenKind, token::Token};

#[test]
fn single_identifier() {
    let source = b"foo";
    let tokens = Lexer::new(source).unwrap().collect::<Vec<Token>>();

    assert_eq!(
        tokens,
        vec![Token {
            kind: TokenKind::Identifier,
            span: Span(0, 3)
        }]
    );
}

#[test]
fn identifier_with_digits_and_underscores() {
    let source = b"a1_b2";
    let tokens = Lexer::new(source).unwrap().collect::<Vec<Token>>();

    assert_eq!(
        tokens,
        vec![Token {
            kind: TokenKind::Identifier,
            span: Span(0, 5)
        }]
    );
}

#[test]
fn multiple_identifiers() {
    let source = b"foo bar_baz qux123";
    let tokens = Lexer::new(source).unwrap().collect::<Vec<Token>>();

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
        ]
    );
}

#[test]
fn booleans() {
    let source = b"true false";
    let tokens = Lexer::new(source).unwrap().collect::<Vec<Token>>();

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
        ]
    );
}

#[test]
fn bytes() {
    let source = b"0x42 0xFF";
    let tokens = Lexer::new(source).unwrap().collect::<Vec<Token>>();

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
        ]
    );
}

#[test]
fn characters() {
    let source = b"'a' 'b' 'c'";
    let tokens = Lexer::new(source).unwrap().collect::<Vec<Token>>();

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
        ]
    );
}

#[test]
fn floats() {
    let source = b"3.14 0.001 42.0";
    let tokens = Lexer::new(source).unwrap().collect::<Vec<Token>>();

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
        ]
    );
}

#[test]
fn integers() {
    let source = b"0 123 456789";
    let tokens = Lexer::new(source).unwrap().collect::<Vec<Token>>();

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
        ]
    );
}

#[test]
fn strings() {
    let source = b"\"hello\" \"world\"";
    let tokens = Lexer::new(source).unwrap().collect::<Vec<Token>>();

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
        .map(|(s, _)| *s)
        .collect::<Vec<_>>()
        .join(" ");
    let expected = keywords.iter().map(|(_, kind)| *kind).collect::<Vec<_>>();
    let actual = Lexer::new(source.as_bytes())
        .unwrap()
        .map(|token| token.kind)
        .collect::<Vec<_>>();

    assert_eq!(actual, expected);
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
        .map(|(s, _)| *s)
        .collect::<Vec<_>>()
        .join(" ");
    let expected = symbols.iter().map(|(_, kind)| *kind).collect::<Vec<_>>();
    let actual = Lexer::new(source.as_bytes())
        .unwrap()
        .map(|token| token.kind)
        .collect::<Vec<_>>();

    assert_eq!(actual, expected);
}
