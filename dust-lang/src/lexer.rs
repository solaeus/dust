use std::{
    f64::{INFINITY, NAN, NEG_INFINITY},
    fmt::{self, Display, Formatter},
};

use chumsky::prelude::*;

use crate::error::Error;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Token<'src> {
    Boolean(bool),
    Comment(&'src str),
    Integer(i64),
    Float(f64),
    String(&'src str),
    Identifier(&'src str),
    Operator(Operator),
    Control(Control),
    Keyword(Keyword),
}

impl<'src> Display for Token<'src> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Token::Boolean(boolean) => write!(f, "{boolean}"),
            Token::Comment(comment) => write!(f, "# {comment}"),
            Token::Integer(integer) => write!(f, "{integer}"),
            Token::Float(float) => write!(f, "{float}"),
            Token::String(string) => write!(f, "{string}"),
            Token::Identifier(string) => write!(f, "{string}"),
            Token::Operator(operator) => write!(f, "{operator}"),
            Token::Control(control) => write!(f, "{control}"),
            Token::Keyword(keyword) => write!(f, "{keyword}"),
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
    Float,
    Fn,
    Int,
    If,
    List,
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
            Keyword::Float => write!(f, "float"),
            Keyword::Fn => write!(f, "fn"),
            Keyword::Int => write!(f, "int"),
            Keyword::If => write!(f, "if"),
            Keyword::List => write!(f, "list"),
            Keyword::Map => write!(f, "map"),
            Keyword::None => write!(f, "none"),
            Keyword::Range => write!(f, "range"),
            Keyword::Struct => write!(f, "struct"),
            Keyword::Str => write!(f, "str"),
            Keyword::Loop => write!(f, "loop"),
            Keyword::While => write!(f, "while"),
            Keyword::Type => write!(f, "type"),
            Keyword::ReadFile => write!(f, "READ_FILE"),
            Keyword::ReadLine => write!(f, "READ_LINE"),
            Keyword::Sleep => write!(f, "SLEEP"),
            Keyword::WriteLine => write!(f, "WRITE_LINE"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Operator {
    Add,
    AddAssign,
    And,
    Assign,
    Divide,
    Equal,
    Greater,
    GreaterOrEqual,
    Less,
    LessOrEqual,
    Modulo,
    Multiply,
    Not,
    NotEqual,
    Or,
    SubAssign,
    Subtract,
}

impl Display for Operator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Operator::Add => write!(f, "+"),
            Operator::AddAssign => write!(f, "+="),
            Operator::And => write!(f, "&&"),
            Operator::Assign => write!(f, "="),
            Operator::Divide => write!(f, "="),
            Operator::Equal => write!(f, "=="),
            Operator::Greater => write!(f, ">"),
            Operator::GreaterOrEqual => write!(f, ">="),
            Operator::Less => write!(f, "<"),
            Operator::LessOrEqual => write!(f, "<="),
            Operator::Modulo => write!(f, "%"),
            Operator::Multiply => write!(f, "*"),
            Operator::Not => write!(f, "!"),
            Operator::NotEqual => write!(f, "!="),
            Operator::Or => write!(f, "||"),
            Operator::SubAssign => write!(f, "-="),
            Operator::Subtract => write!(f, "-"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Control {
    CurlyOpen,
    CurlyClose,
    SquareOpen,
    SquareClose,
    ParenOpen,
    ParenClose,
    Pipe,
    Comma,
    DoubleColon,
    Colon,
    Dollar,
    Dot,
    DoubleDot,
    Semicolon,
    SkinnyArrow,
    FatArrow,
    DoubleUnderscore,
}

impl Display for Control {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Control::CurlyOpen => write!(f, "{{"),
            Control::CurlyClose => write!(f, "}}"),
            Control::Dollar => write!(f, "$"),
            Control::SquareOpen => write!(f, "["),
            Control::SquareClose => write!(f, "]"),
            Control::ParenOpen => write!(f, "("),
            Control::ParenClose => write!(f, ")"),
            Control::Pipe => write!(f, "|"),
            Control::Comma => write!(f, ","),
            Control::DoubleColon => write!(f, "::"),
            Control::Colon => write!(f, ":"),
            Control::Dot => write!(f, "."),
            Control::Semicolon => write!(f, ";"),
            Control::DoubleDot => write!(f, ".."),
            Control::SkinnyArrow => write!(f, "->"),
            Control::FatArrow => write!(f, "=>"),
            Control::DoubleUnderscore => write!(f, "__"),
        }
    }
}

pub fn lex<'src>(source: &'src str) -> Result<Vec<(Token<'src>, SimpleSpan)>, Vec<Error>> {
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

    let identifier_and_keyword = text::ident().map(|text: &str| match text {
        "any" => Token::Keyword(Keyword::Any),
        "async" => Token::Keyword(Keyword::Async),
        "as" => Token::Keyword(Keyword::As),
        "bool" => Token::Keyword(Keyword::Bool),
        "break" => Token::Keyword(Keyword::Break),
        "else" => Token::Keyword(Keyword::Else),
        "float" => Token::Keyword(Keyword::Float),
        "fn" => Token::Keyword(Keyword::Fn),
        "int" => Token::Keyword(Keyword::Int),
        "if" => Token::Keyword(Keyword::If),
        "list" => Token::Keyword(Keyword::List),
        "map" => Token::Keyword(Keyword::Map),
        "none" => Token::Keyword(Keyword::None),
        "range" => Token::Keyword(Keyword::Range),
        "struct" => Token::Keyword(Keyword::Struct),
        "str" => Token::Keyword(Keyword::Str),
        "type" => Token::Keyword(Keyword::Type),
        "loop" => Token::Keyword(Keyword::Loop),
        "while" => Token::Keyword(Keyword::While),
        "READ_FILE" => Token::Keyword(Keyword::ReadFile),
        "READ_LINE" => Token::Keyword(Keyword::ReadLine),
        "SLEEP" => Token::Keyword(Keyword::Sleep),
        "WRITE_LINE" => Token::Keyword(Keyword::WriteLine),
        _ => Token::Identifier(text),
    });

    let operator = choice((
        // logic
        just("&&").to(Operator::And),
        just("==").to(Operator::Equal),
        just("!=").to(Operator::NotEqual),
        just(">=").to(Operator::GreaterOrEqual),
        just("<=").to(Operator::LessOrEqual),
        just(">").to(Operator::Greater),
        just("<").to(Operator::Less),
        just("!").to(Operator::Not),
        just("!=").to(Operator::NotEqual),
        just("||").to(Operator::Or),
        // assignment
        just("=").to(Operator::Assign),
        just("+=").to(Operator::AddAssign),
        just("-=").to(Operator::SubAssign),
        // math
        just("+").to(Operator::Add),
        just("-").to(Operator::Subtract),
        just("*").to(Operator::Multiply),
        just("/").to(Operator::Divide),
        just("%").to(Operator::Modulo),
    ))
    .map(Token::Operator);

    let control = choice((
        just("->").to(Control::SkinnyArrow),
        just("=>").to(Control::FatArrow),
        just("{").to(Control::CurlyOpen),
        just("}").to(Control::CurlyClose),
        just("[").to(Control::SquareOpen),
        just("]").to(Control::SquareClose),
        just("(").to(Control::ParenOpen),
        just(")").to(Control::ParenClose),
        just("|").to(Control::Pipe),
        just(",").to(Control::Comma),
        just(";").to(Control::Semicolon),
        just("::").to(Control::DoubleColon),
        just(":").to(Control::Colon),
        just("..").to(Control::DoubleDot),
        just(".").to(Control::Dot),
        just("$").to(Control::Dollar),
        just("__").to(Control::DoubleUnderscore),
    ))
    .map(Token::Control);

    choice((
        line_comment,
        multi_line_comment,
        boolean,
        float,
        integer,
        string,
        identifier_and_keyword,
        control,
        operator,
    ))
    .map_with(|token, state| (token, state.span()))
    .padded()
    .repeated()
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

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
                (Token::Control(Control::DoubleDot), (1..3).into()),
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
                (Token::Operator(Operator::Add), (2..3).into()),
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
