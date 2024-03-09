use std::fmt::{self, Display, Formatter};

use chumsky::prelude::*;

use crate::error::Error;

#[derive(Clone, Debug, PartialEq)]
pub enum Token<'src> {
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(&'src str),
    Identifier(&'src str),
    Operator(Operator),
    Control(Control),
    Keyword(&'src str),
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub enum Control {
    CurlyOpen,
    CurlyClose,
    SquareOpen,
    SquareClose,
    ParenOpen,
    ParenClose,
    Comma,
    DoubleColon,
    Colon,
    Dot,
    Semicolon,
}

impl Display for Control {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Control::CurlyOpen => write!(f, "{{"),
            Control::CurlyClose => write!(f, "}}"),
            Control::SquareOpen => write!(f, "["),
            Control::SquareClose => write!(f, "]"),
            Control::ParenOpen => write!(f, "("),
            Control::ParenClose => write!(f, ")"),
            Control::Comma => write!(f, ","),
            Control::DoubleColon => write!(f, "::"),
            Control::Colon => write!(f, ":"),
            Control::Dot => write!(f, "."),
            Control::Semicolon => write!(f, ";"),
        }
    }
}

impl<'src> Display for Token<'src> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Token::Boolean(boolean) => write!(f, "{boolean}"),
            Token::Integer(integer) => write!(f, "{integer}"),
            Token::Float(float) => write!(f, "{float}"),
            Token::String(string) => write!(f, "{string}"),
            Token::Identifier(string) => write!(f, "{string}"),
            Token::Operator(operator) => write!(f, "{operator}"),
            Token::Control(control) => write!(f, "{control}"),
            Token::Keyword(string) => write!(f, "{string}"),
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
        just("true").padded().to(Token::Boolean(true)),
        just("false").padded().to(Token::Boolean(false)),
    ));

    let float_numeric = just('-')
        .or_not()
        .then(text::int(10))
        .then(just('.').then(text::digits(10)))
        .then(just('e').then(text::digits(10)).or_not())
        .to_slice()
        .map(|text: &str| Token::Float(text.parse().unwrap()));

    let float_other = choice((just("Infinity"), just("-Infinity"), just("NaN")))
        .map(|text| Token::Float(text.parse().unwrap()));

    let float = choice((float_numeric, float_other));

    let integer = just('-')
        .or_not()
        .then(text::int(10))
        .to_slice()
        .map(|text: &str| {
            let integer = text.parse::<i64>().unwrap();

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
        just("&&").padded().to(Operator::And),
        just("==").padded().to(Operator::Equal),
        just("!=").padded().to(Operator::NotEqual),
        just(">").padded().to(Operator::Greater),
        just(">=").padded().to(Operator::GreaterOrEqual),
        just("<").padded().to(Operator::Less),
        just("<=").padded().to(Operator::LessOrEqual),
        just("!").padded().to(Operator::Not),
        just("!=").padded().to(Operator::NotEqual),
        just("||").padded().to(Operator::Or),
        // assignment
        just("=").padded().to(Operator::Assign),
        just("+=").padded().to(Operator::AddAssign),
        just("-=").padded().to(Operator::SubAssign),
        // math
        just("+").padded().to(Operator::Add),
        just("-").padded().to(Operator::Subtract),
        just("*").padded().to(Operator::Multiply),
        just("/").padded().to(Operator::Divide),
        just("%").padded().to(Operator::Modulo),
    ))
    .map(Token::Operator);

    let control = choice((
        just("{").padded().to(Control::CurlyOpen),
        just("}").padded().to(Control::CurlyClose),
        just("[").padded().to(Control::SquareOpen),
        just("]").padded().to(Control::SquareClose),
        just("(").padded().to(Control::ParenOpen),
        just(")").padded().to(Control::ParenClose),
        just(",").padded().to(Control::Comma),
        just(";").padded().to(Control::Semicolon),
        just("::").padded().to(Control::DoubleColon),
        just(":").padded().to(Control::Colon),
        just(".").padded().to(Control::Dot),
    ))
    .map(Token::Control);

    let keyword = choice((
        just("bool").padded(),
        just("break").padded(),
        just("else").padded(),
        just("float").padded(),
        just("int").padded(),
        just("if").padded(),
        just("list").padded(),
        just("map").padded(),
        just("range").padded(),
        just("str").padded(),
        just("loop").padded(),
    ))
    .map(Token::Keyword);

    choice((
        boolean, float, integer, string, keyword, identifier, operator, control,
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
    fn math_operators() {
        assert_eq!(
            lex("1 + 1").unwrap(),
            vec![
                (Token::Integer(1), (0..1).into()),
                (Token::Operator(Operator::Add), (2..4).into()),
                (Token::Integer(1), (4..5).into())
            ]
        )
    }

    #[test]
    fn keywords() {
        assert_eq!(lex("int").unwrap()[0].0, Token::Keyword("int"))
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
