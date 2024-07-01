use std::{
    f64::{INFINITY, NAN, NEG_INFINITY},
    fmt::{self, Display, Formatter},
};

use chumsky::prelude::*;

use crate::error::DustError;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Token<'src> {
    Boolean(bool),
    Comment(&'src str),
    Integer(i64),
    Float(f64),
    String(&'src str),
    Identifier(&'src str),
    Symbol(Symbol),
    Keyword(Keyword),
    Use(&'src str),
}

impl<'src> Display for Token<'src> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Token::Boolean(boolean) => write!(f, "{boolean}"),
            Token::Comment(comment) => write!(f, "// {comment}"),
            Token::Integer(integer) => write!(f, "{integer}"),
            Token::Float(float) => write!(f, "{float}"),
            Token::String(string) => write!(f, "{string}"),
            Token::Identifier(string) => write!(f, "{string}"),
            Token::Symbol(control) => write!(f, "{control}"),
            Token::Keyword(keyword) => write!(f, "{keyword}"),
            Token::Use(path) => write!(f, "use {path}"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Keyword {
    Any,
    As,
    Async,
    Bool,
    Break,
    Else,
    Enum,
    Float,
    Fn,
    Int,
    If,
    JsonParse,
    Length,
    Map,
    None,
    Range,
    ReadFile,
    ReadLine,
    Sleep,
    Struct,
    Str,
    Type,
    Loop,
    While,
    WriteLine,
}

impl Display for Keyword {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Keyword::Any => write!(f, "any"),
            Keyword::As => write!(f, "as"),
            Keyword::Async => write!(f, "async"),
            Keyword::Bool => write!(f, "bool"),
            Keyword::Break => write!(f, "break"),
            Keyword::Else => write!(f, "else"),
            Keyword::Enum => write!(f, "enum"),
            Keyword::Float => write!(f, "float"),
            Keyword::Fn => write!(f, "fn"),
            Keyword::Int => write!(f, "int"),
            Keyword::If => write!(f, "if"),
            Keyword::Map => write!(f, "map"),
            Keyword::None => write!(f, "none"),
            Keyword::Range => write!(f, "range"),
            Keyword::Struct => write!(f, "struct"),
            Keyword::Str => write!(f, "str"),
            Keyword::Loop => write!(f, "loop"),
            Keyword::While => write!(f, "while"),
            Keyword::Type => write!(f, "type"),
            Keyword::JsonParse => write!(f, "JSON_PARSE"),
            Keyword::Length => write!(f, "LENGTH"),
            Keyword::ReadFile => write!(f, "READ_FILE"),
            Keyword::ReadLine => write!(f, "READ_LINE"),
            Keyword::Sleep => write!(f, "SLEEP"),
            Keyword::WriteLine => write!(f, "WRITE_LINE"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Symbol {
    Plus,
    PlusEqual,
    DoubleAmpersand,
    Colon,
    Comma,
    CurlyClose,
    CurlyOpen,
    Slash,
    Dollar,
    Dot,
    DoubleColon,
    DoubleDot,
    DoubleEqual,
    DoubleUnderscore,
    Equal,
    FatArrow,
    Greater,
    GreaterOrEqual,
    Less,
    LessOrEqual,
    Percent,
    Asterisk,
    Exclamation,
    NotEqual,
    DoublePipe,
    ParenClose,
    ParenOpen,
    Pipe,
    Semicolon,
    SkinnyArrow,
    SquareClose,
    SquareOpen,
    MinusEqual,
    Minus,
}

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Symbol::Asterisk => write!(f, "*"),
            Symbol::Colon => write!(f, ":"),
            Symbol::Comma => write!(f, ","),
            Symbol::CurlyClose => write!(f, "}}"),
            Symbol::CurlyOpen => write!(f, "{{"),
            Symbol::Dollar => write!(f, "$"),
            Symbol::Dot => write!(f, "."),
            Symbol::DoubleAmpersand => write!(f, "&&"),
            Symbol::DoubleColon => write!(f, "::"),
            Symbol::DoubleDot => write!(f, ".."),
            Symbol::DoubleEqual => write!(f, "=="),
            Symbol::DoublePipe => write!(f, "||"),
            Symbol::DoubleUnderscore => write!(f, "__"),
            Symbol::Equal => write!(f, "="),
            Symbol::Exclamation => write!(f, "!"),
            Symbol::FatArrow => write!(f, "=>"),
            Symbol::Greater => write!(f, ">"),
            Symbol::GreaterOrEqual => write!(f, ">="),
            Symbol::Less => write!(f, "<"),
            Symbol::LessOrEqual => write!(f, "<="),
            Symbol::Minus => write!(f, "-"),
            Symbol::MinusEqual => write!(f, "-="),
            Symbol::NotEqual => write!(f, "!="),
            Symbol::ParenClose => write!(f, ")"),
            Symbol::ParenOpen => write!(f, "("),
            Symbol::Percent => write!(f, "%"),
            Symbol::Pipe => write!(f, "|"),
            Symbol::Plus => write!(f, "+"),
            Symbol::PlusEqual => write!(f, "+="),
            Symbol::Semicolon => write!(f, ";"),
            Symbol::SkinnyArrow => write!(f, "->"),
            Symbol::Slash => write!(f, "/"),
            Symbol::SquareClose => write!(f, "]"),
            Symbol::SquareOpen => write!(f, "["),
        }
    }
}

pub fn lex<'src>(source: &'src str) -> Result<Vec<(Token<'src>, SimpleSpan)>, Vec<DustError>> {
    lexer()
        .parse(source)
        .into_result()
        .map_err(|errors| errors.into_iter().map(|error| error.into()).collect())
}

pub fn lexer<'src>() -> impl Parser<
    'src,
    &'src str,
    Vec<(Token<'src>, SimpleSpan<usize>)>,
    extra::Err<Rich<'src, char, SimpleSpan<usize>>>,
> {
    let line_comment = just("//")
        .ignore_then(
            none_of('\n')
                .repeated()
                .to_slice()
                .map(|text: &str| Token::Comment(text.trim())),
        )
        .then_ignore(just('\n').or_not());

    let multi_line_comment = just("/*")
        .ignore_then(
            none_of('*')
                .repeated()
                .to_slice()
                .map(|text: &str| Token::Comment(text.trim())),
        )
        .then_ignore(just("*/"));

    let boolean = choice((
        just("true").to(Token::Boolean(true)),
        just("false").to(Token::Boolean(false)),
    ));

    let float_numeric = just('-')
        .or_not()
        .then(text::int(10))
        .then(just('.').then(text::digits(10)))
        .then(just('e').then(text::digits(10)).or_not())
        .to_slice()
        .map(|text: &str| Token::Float(text.parse().unwrap()));

    let float = choice((
        float_numeric,
        just("Infinity").to(Token::Float(INFINITY)),
        just("-Infinity").to(Token::Float(NEG_INFINITY)),
        just("NaN").to(Token::Float(NAN)),
    ));

    let integer = just('-')
        .or_not()
        .then(text::int(10))
        .to_slice()
        .map(|text: &str| {
            let integer = text.parse().unwrap();

            Token::Integer(integer)
        });

    let delimited_string = |delimiter| {
        just(delimiter)
            .then(none_of(delimiter).repeated())
            .then(just(delimiter))
            .to_slice()
            .map(|text: &str| Token::String(&text[1..text.len() - 1]))
    };

    let string = choice((
        delimited_string('\''),
        delimited_string('"'),
        delimited_string('`'),
    ));

    let keyword = choice((
        just("any").to(Token::Keyword(Keyword::Any)),
        just("async").to(Token::Keyword(Keyword::Async)),
        just("as").to(Token::Keyword(Keyword::As)),
        just("bool").to(Token::Keyword(Keyword::Bool)),
        just("break").to(Token::Keyword(Keyword::Break)),
        just("enum").to(Token::Keyword(Keyword::Enum)),
        just("else").to(Token::Keyword(Keyword::Else)),
        just("float").to(Token::Keyword(Keyword::Float)),
        just("fn").to(Token::Keyword(Keyword::Fn)),
        just("int").to(Token::Keyword(Keyword::Int)),
        just("if").to(Token::Keyword(Keyword::If)),
        just("map").to(Token::Keyword(Keyword::Map)),
        just("none").to(Token::Keyword(Keyword::None)),
        just("range").to(Token::Keyword(Keyword::Range)),
        just("struct").to(Token::Keyword(Keyword::Struct)),
        just("str").to(Token::Keyword(Keyword::Str)),
        just("type").to(Token::Keyword(Keyword::Type)),
        just("loop").to(Token::Keyword(Keyword::Loop)),
        just("while").to(Token::Keyword(Keyword::While)),
        just("JSON_PARSE").to(Token::Keyword(Keyword::JsonParse)),
        just("LENGTH").to(Token::Keyword(Keyword::Length)),
        just("READ_FILE").to(Token::Keyword(Keyword::ReadFile)),
        just("READ_LINE").to(Token::Keyword(Keyword::ReadLine)),
        just("SLEEP").to(Token::Keyword(Keyword::Sleep)),
        just("WRITE_LINE").to(Token::Keyword(Keyword::WriteLine)),
    ));

    let symbol = choice([
        just("!=").to(Token::Symbol(Symbol::NotEqual)),
        just("!").to(Token::Symbol(Symbol::Exclamation)),
        just("$").to(Token::Symbol(Symbol::Dollar)),
        just("%").to(Token::Symbol(Symbol::Percent)),
        just("&&").to(Token::Symbol(Symbol::DoubleAmpersand)),
        just("(").to(Token::Symbol(Symbol::ParenOpen)),
        just(")").to(Token::Symbol(Symbol::ParenClose)),
        just("*").to(Token::Symbol(Symbol::Asterisk)),
        just("+=").to(Token::Symbol(Symbol::PlusEqual)),
        just("+").to(Token::Symbol(Symbol::Plus)),
        just(",").to(Token::Symbol(Symbol::Comma)),
        just("->").to(Token::Symbol(Symbol::SkinnyArrow)),
        just("-=").to(Token::Symbol(Symbol::MinusEqual)),
        just("-").to(Token::Symbol(Symbol::Minus)),
        just("..").to(Token::Symbol(Symbol::DoubleDot)),
        just(".").to(Token::Symbol(Symbol::Dot)),
        just("/").to(Token::Symbol(Symbol::Slash)),
        just("::").to(Token::Symbol(Symbol::DoubleColon)),
        just(":").to(Token::Symbol(Symbol::Colon)),
        just(";").to(Token::Symbol(Symbol::Semicolon)),
        just("<=").to(Token::Symbol(Symbol::LessOrEqual)),
        just("<").to(Token::Symbol(Symbol::Less)),
        just("=>").to(Token::Symbol(Symbol::FatArrow)),
        just("==").to(Token::Symbol(Symbol::DoubleEqual)),
        just("=").to(Token::Symbol(Symbol::Equal)),
        just(">=").to(Token::Symbol(Symbol::GreaterOrEqual)),
        just(">").to(Token::Symbol(Symbol::Greater)),
        just("[").to(Token::Symbol(Symbol::SquareOpen)),
        just("]").to(Token::Symbol(Symbol::SquareClose)),
        just("__").to(Token::Symbol(Symbol::DoubleUnderscore)),
        just("{").to(Token::Symbol(Symbol::CurlyOpen)),
        just("}").to(Token::Symbol(Symbol::CurlyClose)),
        just("||").to(Token::Symbol(Symbol::DoublePipe)),
        just("|").to(Token::Symbol(Symbol::Pipe)),
    ]);

    let identifier = text::ident().map(|text: &str| Token::Identifier(text));

    let r#use = just("use").padded().ignore_then(
        none_of(" \n\r;")
            .repeated()
            .to_slice()
            .map(|text: &str| Token::Use(text.trim())),
    );

    choice((
        line_comment,
        multi_line_comment,
        boolean,
        float,
        integer,
        string,
        keyword,
        symbol,
        r#use,
        identifier,
    ))
    .map_with(|token: Token, state| (token, state.span()))
    .padded()
    .repeated()
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r#use() {
        assert_eq!(
            lex("use std.io").unwrap(),
            vec![(Token::Use("std.io"), (0..10).into())]
        );

        assert_eq!(
            lex("use https://example.com/std.ds").unwrap(),
            vec![(Token::Use("https://example.com/std.ds"), (0..30).into())]
        );
    }

    #[test]
    fn line_comment() {
        assert_eq!(
            lex("// 42").unwrap(),
            vec![(Token::Comment("42"), (0..5).into())]
        );

        assert_eq!(
            lex("1// 42//2").unwrap(),
            vec![
                (Token::Integer(1), (0..1).into()),
                (Token::Comment("42//2"), (1..9).into()),
            ]
        );
        assert_eq!(
            lex("
                1
                // 42
                2
                ")
            .unwrap(),
            vec![
                (Token::Integer(1), (17..18).into()),
                (Token::Comment("42"), (35..41).into()),
                (Token::Integer(2), (57..58).into()),
            ]
        );
    }

    #[test]
    fn multi_line_comment() {
        assert_eq!(
            lex("/* 42 */").unwrap(),
            vec![(Token::Comment("42"), (0..8).into())]
        );

        assert_eq!(
            lex("1/* 42//2 */").unwrap(),
            vec![
                (Token::Integer(1), (0..1).into()),
                (Token::Comment("42//2"), (1..12).into()),
            ]
        );
        assert_eq!(
            lex("
                1
                /*
                    42
                */
                2
                ")
            .unwrap(),
            vec![
                (Token::Integer(1), (17..18).into()),
                (Token::Comment("42"), (35..79).into()),
                (Token::Integer(2), (96..97).into()),
            ]
        );
    }

    #[test]
    fn range() {
        assert_eq!(
            lex("1..10").unwrap(),
            vec![
                (Token::Integer(1), (0..1).into()),
                (Token::Symbol(Symbol::DoubleDot), (1..3).into()),
                (Token::Integer(10), (3..5).into())
            ]
        )
    }

    #[test]
    fn math_operators() {
        assert_eq!(
            lex("1 + 1").unwrap(),
            vec![
                (Token::Integer(1), (0..1).into()),
                (Token::Symbol(Symbol::Plus), (2..3).into()),
                (Token::Integer(1), (4..5).into())
            ]
        )
    }

    #[test]
    fn keywords() {
        assert_eq!(lex("int").unwrap()[0].0, Token::Keyword(Keyword::Int))
    }

    #[test]
    fn identifier() {
        assert_eq!(lex("x").unwrap()[0].0, Token::Identifier("x"));
        assert_eq!(lex("foobar").unwrap()[0].0, Token::Identifier("foobar"));
        assert_eq!(lex("HELLO").unwrap()[0].0, Token::Identifier("HELLO"));
    }

    #[test]
    fn r#true() {
        assert_eq!(lex("true").unwrap()[0].0, Token::Boolean(true));
    }

    #[test]
    fn r#false() {
        assert_eq!(lex("false").unwrap()[0].0, Token::Boolean(false));
    }

    #[test]
    fn positive_float() {
        assert_eq!(lex("0.0").unwrap()[0].0, Token::Float(0.0));
        assert_eq!(lex("42.0").unwrap()[0].0, Token::Float(42.0));

        let max_float = f64::MAX.to_string() + ".0";

        assert_eq!(lex(&max_float).unwrap()[0].0, Token::Float(f64::MAX));

        let min_positive_float = f64::MIN_POSITIVE.to_string();

        assert_eq!(
            lex(&min_positive_float).unwrap()[0].0,
            Token::Float(f64::MIN_POSITIVE)
        );
    }

    #[test]
    fn negative_float() {
        assert_eq!(lex("-0.0").unwrap()[0].0, Token::Float(-0.0));
        assert_eq!(lex("-42.0").unwrap()[0].0, Token::Float(-42.0));

        let min_float = f64::MIN.to_string() + ".0";

        assert_eq!(lex(&min_float).unwrap()[0].0, Token::Float(f64::MIN));

        let max_negative_float = format!("-{}", f64::MIN_POSITIVE);

        assert_eq!(
            lex(&max_negative_float).unwrap()[0].0,
            Token::Float(-f64::MIN_POSITIVE)
        );
    }

    #[test]
    fn other_float() {
        assert_eq!(lex("Infinity").unwrap()[0].0, Token::Float(f64::INFINITY));
        assert_eq!(
            lex("-Infinity").unwrap()[0].0,
            Token::Float(f64::NEG_INFINITY)
        );

        if let Token::Float(float) = &lex("NaN").unwrap()[0].0 {
            assert!(float.is_nan());
        } else {
            panic!("Expected a float.")
        }
    }

    #[test]
    fn positive_integer() {
        for i in 0..10 {
            let source = i.to_string();
            let tokens = lex(&source).unwrap();

            assert_eq!(tokens[0].0, Token::Integer(i))
        }

        assert_eq!(lex("42").unwrap()[0].0, Token::Integer(42));

        let maximum_integer = i64::MAX.to_string();

        assert_eq!(
            lex(&maximum_integer).unwrap()[0].0,
            Token::Integer(i64::MAX)
        );
    }

    #[test]
    fn negative_integer() {
        for i in -9..1 {
            let source = i.to_string();
            let tokens = lex(&source).unwrap();

            assert_eq!(tokens[0].0, Token::Integer(i))
        }

        assert_eq!(lex("-42").unwrap()[0].0, Token::Integer(-42));

        let minimum_integer = i64::MIN.to_string();

        assert_eq!(
            lex(&minimum_integer).unwrap()[0].0,
            Token::Integer(i64::MIN)
        );
    }

    #[test]
    fn double_quoted_string() {
        assert_eq!(lex("\"\"").unwrap()[0].0, Token::String(""));
        assert_eq!(lex("\"42\"").unwrap()[0].0, Token::String("42"));
        assert_eq!(lex("\"foobar\"").unwrap()[0].0, Token::String("foobar"));
    }

    #[test]
    fn single_quoted_string() {
        assert_eq!(lex("''").unwrap()[0].0, Token::String(""));
        assert_eq!(lex("'42'").unwrap()[0].0, Token::String("42"));
        assert_eq!(lex("'foobar'").unwrap()[0].0, Token::String("foobar"));
    }

    #[test]
    fn grave_quoted_string() {
        assert_eq!(lex("``").unwrap()[0].0, Token::String(""));
        assert_eq!(lex("`42`").unwrap()[0].0, Token::String("42"));
        assert_eq!(lex("`foobar`").unwrap()[0].0, Token::String("foobar"));
    }
}
