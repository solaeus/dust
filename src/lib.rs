use std::fmt::{self, Display, Formatter};

use chumsky::{prelude::*, Parser};

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Boolean(bool),
    Integer(i64),
    String(String),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Value::Boolean(boolean) => write!(f, "{boolean}"),
            Value::Integer(integer) => write!(f, "{integer}"),
            Value::String(string) => write!(f, "{string}"),
        }
    }
}

pub fn parser() -> impl Parser<char, Value, Error = Simple<char>> {
    let boolean = just("true")
        .or(just("false"))
        .map(|s: &str| Value::Boolean(s.parse().unwrap()));

    let integer = just('-')
        .or_not()
        .then(text::int(10).padded())
        .map(|(c, s)| {
            if let Some(c) = c {
                c.to_string() + &s
            } else {
                s
            }
        })
        .map(|s: String| Value::Integer(s.parse().unwrap()));

    let delimited_string = |delimiter: char| {
        just(delimiter)
            .ignore_then(none_of(delimiter).repeated())
            .then_ignore(just(delimiter))
            .map(|chars| Value::String(chars.into_iter().collect()))
    };

    let string = choice((
        delimited_string('\''),
        delimited_string('"'),
        delimited_string('`'),
    ));

    boolean.or(integer).or(string).then_ignore(end())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_true() {
        assert_eq!(parser().parse("true"), Ok(Value::Boolean(true)))
    }

    #[test]
    fn parse_false() {
        assert_eq!(parser().parse("false"), Ok(Value::Boolean(false)))
    }

    #[test]
    fn parse_positive_integer() {
        let parser = parser();

        assert_eq!(parser.parse("0"), Ok(Value::Integer(0)));
        assert_eq!(parser.parse("1"), Ok(Value::Integer(1)));
        assert_eq!(parser.parse("2"), Ok(Value::Integer(2)));
        assert_eq!(parser.parse("3"), Ok(Value::Integer(3)));
        assert_eq!(parser.parse("4"), Ok(Value::Integer(4)));
        assert_eq!(parser.parse("5"), Ok(Value::Integer(5)));
        assert_eq!(parser.parse("6"), Ok(Value::Integer(6)));
        assert_eq!(parser.parse("7"), Ok(Value::Integer(7)));
        assert_eq!(parser.parse("8"), Ok(Value::Integer(8)));
        assert_eq!(parser.parse("9"), Ok(Value::Integer(9)));
        assert_eq!(parser.parse("42"), Ok(Value::Integer(42)));
        assert_eq!(
            parser.parse(i64::MAX.to_string()),
            Ok(Value::Integer(i64::MAX))
        );
    }

    #[test]
    fn parse_negative_integer() {
        let parser = parser();

        assert_eq!(parser.parse("-0"), Ok(Value::Integer(-0)));
        assert_eq!(parser.parse("-1"), Ok(Value::Integer(-1)));
        assert_eq!(parser.parse("-2"), Ok(Value::Integer(-2)));
        assert_eq!(parser.parse("-3"), Ok(Value::Integer(-3)));
        assert_eq!(parser.parse("-4"), Ok(Value::Integer(-4)));
        assert_eq!(parser.parse("-5"), Ok(Value::Integer(-5)));
        assert_eq!(parser.parse("-6"), Ok(Value::Integer(-6)));
        assert_eq!(parser.parse("-7"), Ok(Value::Integer(-7)));
        assert_eq!(parser.parse("-8"), Ok(Value::Integer(-8)));
        assert_eq!(parser.parse("-9"), Ok(Value::Integer(-9)));
        assert_eq!(parser.parse("-42"), Ok(Value::Integer(-42)));
        assert_eq!(
            parser.parse(i64::MIN.to_string()),
            Ok(Value::Integer(i64::MIN))
        );
    }

    #[test]
    fn double_quoted_string() {
        let parser = parser();

        assert_eq!(parser.parse("\"\""), Ok(Value::String("".to_string())));
        assert_eq!(parser.parse("\"1\""), Ok(Value::String("1".to_string())));
        assert_eq!(parser.parse("\"42\""), Ok(Value::String("42".to_string())));
        assert_eq!(
            parser.parse("\"foobar\""),
            Ok(Value::String("foobar".to_string()))
        );
    }

    #[test]
    fn single_quoted_string() {
        let parser = parser();

        assert_eq!(parser.parse("''"), Ok(Value::String("".to_string())));
        assert_eq!(parser.parse("'1'"), Ok(Value::String("1".to_string())));
        assert_eq!(parser.parse("'42'"), Ok(Value::String("42".to_string())));
        assert_eq!(
            parser.parse("'foobar'"),
            Ok(Value::String("foobar".to_string()))
        );
    }

    #[test]
    fn grave_quoted_string() {
        let parser = parser();

        assert_eq!(parser.parse("``"), Ok(Value::String("".to_string())));
        assert_eq!(parser.parse("`1`"), Ok(Value::String("1".to_string())));
        assert_eq!(parser.parse("`42`"), Ok(Value::String("42".to_string())));
        assert_eq!(
            parser.parse("`foobar`"),
            Ok(Value::String("foobar".to_string()))
        );
    }
}
