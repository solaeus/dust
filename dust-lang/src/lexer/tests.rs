use crate::{
    lexer::Lexer,
    source::Span,
    token::{Token, TokenKind},
};

#[test]
fn single_identifier() {
    let source = b"foo";
    let tokens = Lexer::from_bytes(source).collect::<Vec<_>>();

    assert_eq!(
        tokens,
        vec![
            Token {
                kind: TokenKind::Identifier,
                span: Span::new(0, 3)
            },
            Token {
                kind: TokenKind::Eof,
                span: Span::new(3, 3)
            }
        ]
    );
}

#[test]
fn identifier_with_digits_and_underscores() {
    let source = b"a1_b2";
    let tokens = Lexer::from_bytes(source).collect::<Vec<_>>();

    assert_eq!(
        tokens,
        vec![
            Token {
                kind: TokenKind::Identifier,
                span: Span::new(0, 5)
            },
            Token {
                kind: TokenKind::Eof,
                span: Span::new(5, 5)
            }
        ]
    );
}

#[test]
fn multiple_identifiers() {
    let source = b"foo bar_baz qux123";
    let tokens = Lexer::from_bytes(source).collect::<Vec<_>>();

    assert_eq!(
        tokens,
        vec![
            Token {
                kind: TokenKind::Identifier,
                span: Span::new(0, 3)
            },
            Token {
                kind: TokenKind::Identifier,
                span: Span::new(4, 11)
            },
            Token {
                kind: TokenKind::Identifier,
                span: Span::new(12, 18)
            },
            Token {
                kind: TokenKind::Eof,
                span: Span::new(18, 18)
            }
        ]
    );
}

#[test]
fn booleans() {
    let source = b"true false";
    let tokens = Lexer::from_bytes(source).collect::<Vec<_>>();

    assert_eq!(
        tokens,
        vec![
            Token {
                kind: TokenKind::TrueValue,
                span: Span::new(0, 4)
            },
            Token {
                kind: TokenKind::FalseValue,
                span: Span::new(5, 10)
            },
            Token {
                kind: TokenKind::Eof,
                span: Span::new(10, 10)
            }
        ]
    );
}

#[test]
fn bytes() {
    let source = b"0x42 0xFF";
    let tokens = Lexer::from_bytes(source).collect::<Vec<_>>();

    assert_eq!(
        tokens,
        vec![
            Token {
                kind: TokenKind::ByteValue,
                span: Span::new(0, 4)
            },
            Token {
                kind: TokenKind::ByteValue,
                span: Span::new(5, 9)
            },
            Token {
                kind: TokenKind::Eof,
                span: Span::new(9, 9)
            }
        ]
    );
}

#[test]
fn characters() {
    let utf_8_range = 0..=0x10FFFF;
    let surrogate_range = 0xD800..=0xDFFF;

    for codepoint in utf_8_range {
        if surrogate_range.contains(&codepoint) || codepoint == '\'' as u32 {
            continue;
        }

        let character = char::from_u32(codepoint).unwrap();
        let source = format!("'{character}'");
        let tokens = Lexer::from_bytes(source.as_bytes()).collect::<Vec<_>>();

        assert_eq!(
            tokens,
            vec![
                Token {
                    kind: TokenKind::CharacterValue,
                    span: Span::new(0, source.len())
                },
                Token {
                    kind: TokenKind::Eof,
                    span: Span::new(source.len(), source.len())
                }
            ],
            "Failed to tokenize character literal {character} (U+{codepoint:04X})"
        );
    }

    let source = br"'\''";
    let tokens = Lexer::from_bytes(source).collect::<Vec<_>>();

    assert_eq!(
        tokens,
        vec![
            Token {
                kind: TokenKind::CharacterValue,
                span: Span::new(0, 4)
            },
            Token {
                kind: TokenKind::Eof,
                span: Span::new(4, 4)
            }
        ]
    );
}

#[test]
fn floats() {
    let source = b"3.14 0.001 42.0";
    let tokens = Lexer::from_bytes(source).collect::<Vec<_>>();

    assert_eq!(
        tokens,
        vec![
            Token {
                kind: TokenKind::FloatValue,
                span: Span::new(0, 4)
            },
            Token {
                kind: TokenKind::FloatValue,
                span: Span::new(5, 10)
            },
            Token {
                kind: TokenKind::FloatValue,
                span: Span::new(11, 15)
            },
            Token {
                kind: TokenKind::Eof,
                span: Span::new(15, 15)
            }
        ]
    );
}

#[test]
fn integers() {
    let source = b"0 123 456789";
    let tokens = Lexer::from_bytes(source).collect::<Vec<_>>();

    assert_eq!(
        tokens,
        vec![
            Token {
                kind: TokenKind::IntegerValue,
                span: Span::new(0, 1)
            },
            Token {
                kind: TokenKind::IntegerValue,
                span: Span::new(2, 5)
            },
            Token {
                kind: TokenKind::IntegerValue,
                span: Span::new(6, 12)
            },
            Token {
                kind: TokenKind::Eof,
                span: Span::new(12, 12)
            }
        ]
    );
}

#[test]
fn strings() {
    let source = b"\"hello\" \"world\"";
    let tokens = Lexer::from_bytes(source).collect::<Vec<_>>();

    assert_eq!(
        tokens,
        vec![
            Token {
                kind: TokenKind::StringValue,
                span: Span::new(0, 7)
            },
            Token {
                kind: TokenKind::StringValue,
                span: Span::new(8, 15)
            },
            Token {
                kind: TokenKind::Eof,
                span: Span::new(15, 15)
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
    let actual = Lexer::from_bytes(source.as_bytes())
        .map(|token| token.kind)
        .collect::<Vec<_>>();

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
        ("^", TokenKind::Caret),
        ("^=", TokenKind::CaretEqual),
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
    let actual = Lexer::from_bytes(source.as_bytes())
        .map(|token| token.kind)
        .collect::<Vec<_>>();

    assert_eq!(actual[..actual.len() - 1], expected);
}

#[test]
fn adjacent_tokens() {
    let source = b"let x:int=42;";
    let tokens = Lexer::from_bytes(source).collect::<Vec<_>>();

    assert_eq!(
        tokens,
        vec![
            Token {
                kind: TokenKind::Let,
                span: Span::new(0, 3)
            },
            Token {
                kind: TokenKind::Identifier,
                span: Span::new(4, 5)
            },
            Token {
                kind: TokenKind::Colon,
                span: Span::new(5, 6)
            },
            Token {
                kind: TokenKind::Int,
                span: Span::new(6, 9)
            },
            Token {
                kind: TokenKind::Equal,
                span: Span::new(9, 10)
            },
            Token {
                kind: TokenKind::IntegerValue,
                span: Span::new(10, 12)
            },
            Token {
                kind: TokenKind::Semicolon,
                span: Span::new(12, 13)
            },
            Token {
                kind: TokenKind::Eof,
                span: Span::new(13, 13)
            }
        ]
    );
}

#[test]
fn example_source_code() {
    let source = br#"
        fn fib (n: int) -> int {
            if n <= 0 {
                0
            } else if n == 1 {
                1
            } else {
                fib(n - 1) + fib(n - 2)
            }
        }

        let mut count = 1;

        while count <= 15 {
            if count % 15 == 0 {
           	    write_line("fizzbuzz");
            } else if count % 3 == 0 {
               	write_line("fizz");
            } else if count % 5 == 0 {
               	write_line("buzz");
            } else {
                write_line(count as str);
            }

            count += 1;
        }

        fn hello_world() {
            write_line("Hello, world!");
            write_line("Enter your name...");

            let name = read_line();

            write_line("Hello " + name + "!");
        }

        hello_world();
    "#;

    let mut lexer = Lexer::from_bytes(source);

    for _ in &mut lexer {}

    assert_eq!(lexer.error_index(), None);
}

#[test]
fn mixed_utf8_input() {
    let mut all_ascii = (0..128).cycle();
    let utf8_range = 0..=0x10FFFF;
    let surrogate_range = 0xD800..=0xDFFF;
    let mut byte_buffer = [0; 4];
    let mut mixed_bytes = Vec::new();

    for codepoint in utf8_range {
        if surrogate_range.contains(&codepoint) {
            continue;
        }

        let ascii = all_ascii.next().or_else(|| all_ascii.next()).unwrap();

        mixed_bytes.push(b' ');
        mixed_bytes.push(ascii);

        let utf8_character = std::char::from_u32(codepoint).unwrap();

        utf8_character.encode_utf8(&mut byte_buffer);
        mixed_bytes.push(b' ');
        mixed_bytes.extend_from_slice(&byte_buffer[..utf8_character.len_utf8()]);
    }

    let mut lexer = Lexer::from_bytes(&mixed_bytes);

    for _ in &mut lexer {}

    assert_eq!(lexer.error_index(), None);
}

#[test]
fn invalid_utf8_in_bytes_errors() {
    let source = b"abc\xFFdef";
    let mut lexer = Lexer::from_bytes(source);

    lexer.next();

    assert_eq!(lexer.error_index(), Some(3));
}

#[test]
fn unicode_identifier() {
    let source = "α".as_bytes();
    let tokens = Lexer::from_bytes(source).collect::<Vec<_>>();

    assert_eq!(
        tokens,
        vec![
            Token {
                kind: TokenKind::Identifier,
                span: Span::new(0, 2)
            },
            Token {
                kind: TokenKind::Eof,
                span: Span::new(2, 2)
            }
        ]
    );
}

#[test]
fn multiple_unicode_identifier() {
    let source = "αβγ".as_bytes();
    let tokens = Lexer::from_bytes(source).collect::<Vec<_>>();

    assert_eq!(
        tokens,
        vec![
            Token {
                kind: TokenKind::Identifier,
                span: Span::new(0, 6)
            },
            Token {
                kind: TokenKind::Eof,
                span: Span::new(6, 6)
            }
        ]
    );
}

#[test]
fn ascii_followed_by_unicode_identifier() {
    let source = "fooα".as_bytes();
    let tokens = Lexer::from_bytes(source).collect::<Vec<_>>();

    assert_eq!(
        tokens,
        vec![
            Token {
                kind: TokenKind::Identifier,
                span: Span::new(0, 5)
            },
            Token {
                kind: TokenKind::Eof,
                span: Span::new(5, 5)
            }
        ]
    );
}

#[test]
fn unicode_followed_by_ascii_identifier() {
    let source = "αfoo".as_bytes();
    let tokens = Lexer::from_bytes(source).collect::<Vec<_>>();

    assert_eq!(
        tokens,
        vec![
            Token {
                kind: TokenKind::Identifier,
                span: Span::new(0, 5)
            },
            Token {
                kind: TokenKind::Eof,
                span: Span::new(5, 5)
            }
        ]
    );
}

#[test]
fn underscore_followed_by_unicode_identifier() {
    let source = "_α".as_bytes();
    let tokens = Lexer::from_bytes(source).collect::<Vec<_>>();

    assert_eq!(
        tokens,
        vec![
            Token {
                kind: TokenKind::Identifier,
                span: Span::new(0, 3)
            },
            Token {
                kind: TokenKind::Eof,
                span: Span::new(3, 3)
            }
        ]
    );
}

#[test]
fn chinese_identifier() {
    let source = "中文".as_bytes();
    let tokens = Lexer::from_bytes(source).collect::<Vec<_>>();

    assert_eq!(
        tokens,
        vec![
            Token {
                kind: TokenKind::Identifier,
                span: Span::new(0, 6)
            },
            Token {
                kind: TokenKind::Eof,
                span: Span::new(6, 6)
            }
        ]
    );
}

#[test]
fn emoji_is_not_identifier() {
    let source = "🍄".as_bytes();
    let tokens = Lexer::from_bytes(source).collect::<Vec<_>>();

    assert_eq!(
        tokens,
        vec![
            Token {
                kind: TokenKind::Unknown,
                span: Span::new(0, 4)
            },
            Token {
                kind: TokenKind::Eof,
                span: Span::new(4, 4)
            }
        ]
    );
}

#[test]
fn emoji_breaks_identifier() {
    let source = "foo🍄bar".as_bytes();
    let tokens = Lexer::from_bytes(source).collect::<Vec<_>>();

    assert_eq!(
        tokens,
        vec![
            Token {
                kind: TokenKind::Identifier,
                span: Span::new(0, 3)
            },
            Token {
                kind: TokenKind::Unknown,
                span: Span::new(3, 7)
            },
            Token {
                kind: TokenKind::Identifier,
                span: Span::new(7, 10)
            },
            Token {
                kind: TokenKind::Eof,
                span: Span::new(10, 10)
            }
        ]
    );
}

#[test]
fn emoji_character() {
    let source = "'🎉'".as_bytes();
    let tokens = Lexer::from_bytes(source).collect::<Vec<_>>();

    assert_eq!(
        tokens,
        vec![
            Token {
                kind: TokenKind::CharacterValue,
                span: Span::new(0, 6)
            },
            Token {
                kind: TokenKind::Eof,
                span: Span::new(6, 6)
            }
        ]
    );
}

#[test]
fn emoji_string() {
    let source = "\"🎉\"".as_bytes();
    let tokens = Lexer::from_bytes(source).collect::<Vec<_>>();

    assert_eq!(
        tokens,
        vec![
            Token {
                kind: TokenKind::StringValue,
                span: Span::new(0, 6)
            },
            Token {
                kind: TokenKind::Eof,
                span: Span::new(6, 6)
            }
        ]
    );
}
