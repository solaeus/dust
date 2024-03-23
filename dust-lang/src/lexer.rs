use std::{
    f64::{INFINITY, NAN, NEG_INFINITY},
    fmt::{self, Display, Formatter},
};

use chumsky::prelude::*;

use crate::error::Error;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Token<'src> {
    Boolean(bool),
    BuiltInIdentifier(BuiltInIdentifier),
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
            Token::BuiltInIdentifier(built_in_identifier) => write!(f, "{built_in_identifier}"),
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
pub enum BuiltInIdentifier {
    ReadLine,
    WriteLine,
}

impl Display for BuiltInIdentifier {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            BuiltInIdentifier::ReadLine => write!(f, "__READ_LINE__"),
            BuiltInIdentifier::WriteLine => write!(f, "__WRITE_LINE__"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Keyword {
    Any,
    Async,
    Bool,
    Break,
    Else,
    Float,
    Int,
    If,
    List,
    Map,
    None,
    Range,
    Struct,
    Str,
    Loop,
    While,
}

impl Display for Keyword {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Keyword::Any => write!(f, "any"),
            Keyword::Async => write!(f, "async"),
            Keyword::Bool => write!(f, "bool"),
            Keyword::Break => write!(f, "break"),
            Keyword::Else => write!(f, "else"),
            Keyword::Float => write!(f, "float"),
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
    Arrow,
    CurlyOpen,
    CurlyClose,
    SquareOpen,
    SquareClose,
    ParenOpen,
    ParenClose,
    Comma,
    DoubleColon,
    Colon,
    Dollar,
    Dot,
    DoubleDot,
    Semicolon,
}

impl Display for Control {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Control::Arrow => write!(f, "->"),
            Control::CurlyOpen => write!(f, "{{"),
            Control::CurlyClose => write!(f, "}}"),
            Control::Dollar => write!(f, "$"),
            Control::SquareOpen => write!(f, "["),
            Control::SquareClose => write!(f, "]"),
            Control::ParenOpen => write!(f, "("),
            Control::ParenClose => write!(f, ")"),
            Control::Comma => write!(f, ","),
            Control::DoubleColon => write!(f, "::"),
            Control::Colon => write!(f, ":"),
            Control::Dot => write!(f, "."),
            Control::Semicolon => write!(f, ";"),
            Control::DoubleDot => write!(f, ".."),
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

    let identifier = text::ident().map(|text: &str| Token::Identifier(text));

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
        just("->").to(Control::Arrow),
        just("{").to(Control::CurlyOpen),
        just("}").to(Control::CurlyClose),
        just("[").to(Control::SquareOpen),
        just("]").to(Control::SquareClose),
        just("(").to(Control::ParenOpen),
        just(")").to(Control::ParenClose),
        just(",").to(Control::Comma),
        just(";").to(Control::Semicolon),
        just("::").to(Control::DoubleColon),
        just(":").to(Control::Colon),
        just("..").to(Control::DoubleDot),
        just(".").to(Control::Dot),
        just("$").to(Control::Dollar),
    ))
    .map(Token::Control);

    let keyword = choice((
        just("any").to(Keyword::Any),
        just("async").to(Keyword::Async),
        just("bool").to(Keyword::Bool),
        just("break").to(Keyword::Break),
        just("else").to(Keyword::Else),
        just("float").to(Keyword::Float),
        just("int").to(Keyword::Int),
        just("if").to(Keyword::If),
        just("list").to(Keyword::List),
        just("map").to(Keyword::Map),
        just("none").to(Keyword::None),
        just("range").to(Keyword::Range),
        just("struct").to(Keyword::Struct),
        just("str").to(Keyword::Str),
        just("loop").to(Keyword::Loop),
        just("while").to(Keyword::While),
    ))
    .map(Token::Keyword);

    let built_in_identifier = choice((
        just("__READ_LINE__").to(BuiltInIdentifier::ReadLine),
        just("__WRITE_LINE__").to(BuiltInIdentifier::WriteLine),
    ))
    .map(Token::BuiltInIdentifier);

    choice((
        boolean,
        float,
        integer,
        string,
        keyword,
        identifier,
        control,
        operator,
        built_in_identifier,
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
