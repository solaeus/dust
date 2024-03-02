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
    Operator(&'src str),
    Control(&'src str),
    Keyword(&'src str),
}

impl<'src> Display for Token<'src> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Token::Boolean(boolean) => write!(f, "{boolean}"),
            Token::Integer(integer) => write!(f, "{integer}"),
            Token::Float(float) => write!(f, "{float}"),
            Token::String(string) => write!(f, "{string}"),
            Token::Identifier(string) => write!(f, "{string}"),
            Token::Operator(string) => write!(f, "{string}"),
            Token::Control(string) => write!(f, "{string}"),
            Token::Keyword(string) => write!(f, "{string}"),
        }
    }
}

pub fn lex<'src>(source: &'src str) -> Result<Vec<(Token, SimpleSpan)>, Error<'src>> {
    lexer()
        .parse(source)
        .into_result()
        .map_err(|error| Error::Lex(error))
}

pub fn lexer<'src>() -> impl Parser<
    'src,
    &'src str,
    Vec<(Token<'src>, SimpleSpan<usize>)>,
    extra::Err<Rich<'src, char, SimpleSpan<usize>>>,
> {
    let boolean = just("true")
        .or(just("false"))
        .map(|s: &str| Token::Boolean(s.parse().unwrap()));

    let float_numeric = just('-')
        .or_not()
        .then(text::int(10))
        .then(just('.').then(text::digits(10)))
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
        just("==").padded(),
        just("!=").padded(),
        just(">").padded(),
        just("<").padded(),
        just(">=").padded(),
        just("<=").padded(),
        just("&&").padded(),
        just("||").padded(),
        just("=").padded(),
        just("+=").padded(),
        just("-=").padded(),
    ))
    .map(Token::Operator);

    let control = choice((
        just("[").padded(),
        just("]").padded(),
        just("(").padded(),
        just(")").padded(),
        just("{").padded(),
        just("}").padded(),
        just(",").padded(),
        just(";").padded(),
        just("::").padded(),
        just(":").padded(),
    ))
    .map(Token::Control);

    let keyword = choice((
        just("bool").padded(),
        just("float").padded(),
        just("int").padded(),
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
