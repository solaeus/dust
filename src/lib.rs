use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    ops::Range,
};

use chumsky::{pratt::*, prelude::*, Parser};

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Assignment(Box<Assignment>),
    Expression(Expression),
    Sequence(Vec<Statement>),
}

impl Statement {
    pub fn value(value: Value) -> Statement {
        Statement::Expression(Expression::Value(value))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Assignment {
    identifier: Identifier,
    statement: Statement,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Logic(Box<Logic>),
    Value(Value),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Logic {
    Equal(Expression, Expression),
    NotEqual(Expression, Expression),
    Greater(Expression, Expression),
    Less(Expression, Expression),
    GreaterOrEqual(Expression, Expression),
    LessOrEqual(Expression, Expression),
    And(Expression, Expression),
    Or(Expression, Expression),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Boolean(bool),
    Float(f64),
    Integer(i64),
    List(Vec<Value>),
    Map(BTreeMap<Identifier, Value>),
    Range(Range<i64>),
    String(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Identifier(String);

impl Identifier {
    pub fn new(text: impl ToString) -> Self {
        Identifier(text.to_string())
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Value::Boolean(boolean) => write!(f, "{boolean}"),
            Value::Float(float) => write!(f, "{float}"),
            Value::Integer(integer) => write!(f, "{integer}"),
            Value::List(_list) => todo!(),
            Value::Map(_map) => todo!(),
            Value::Range(range) => write!(f, "{}..{}", range.start, range.end),
            Value::String(string) => write!(f, "{string}"),
        }
    }
}

pub fn parser<'src>() -> impl Parser<'src, &'src str, Expression> {
    let operator = |text| just(text).padded();

    let value = recursive(|value| {
        let boolean = just("true")
            .or(just("false"))
            .map(|s: &str| Value::Boolean(s.parse().unwrap()));

        let float_numeric = just('-')
            .or_not()
            .then(text::int(10))
            .then(just('.').then(text::digits(10)))
            .to_slice()
            .map(|text: &str| Value::Float(text.parse().unwrap()));

        let float_other = choice((just("Infinity"), just("-Infinity"), just("NaN")))
            .map(|text| Value::Float(text.parse().unwrap()));

        let float = choice((float_numeric, float_other));

        let integer = just('-')
            .or_not()
            .then(text::int(10).padded())
            .to_slice()
            .map(|text: &str| {
                let integer = text.parse::<i64>().unwrap();

                Value::Integer(integer)
            });

        let delimited_string = |delimiter| {
            just(delimiter)
                .ignore_then(none_of(delimiter).repeated())
                .then_ignore(just(delimiter))
                .to_slice()
                .map(|text: &str| Value::String(text[1..text.len() - 1].to_string()))
        };

        let string = choice((
            delimited_string('\''),
            delimited_string('"'),
            delimited_string('`'),
        ));

        let list = value
            .clone()
            .separated_by(just(',').padded())
            .allow_trailing()
            .collect()
            .padded()
            .delimited_by(just('['), just(']'))
            .map(|values| Value::List(values));

        choice((boolean, float, integer, string, list))
    });

    let value_expression = value.map(|value| Expression::Value(value));

    let logic_expression = value_expression.pratt((
        infix(left(1), operator("=="), |left, right| {
            Expression::Logic(Box::new(Logic::Equal(left, right)))
        }),
        infix(left(1), operator("!="), |left, right| {
            Expression::Logic(Box::new(Logic::NotEqual(left, right)))
        }),
        infix(left(1), operator(">"), |left, right| {
            Expression::Logic(Box::new(Logic::Greater(left, right)))
        }),
        infix(left(1), operator("<"), |left, right| {
            Expression::Logic(Box::new(Logic::Less(left, right)))
        }),
        infix(left(1), operator(">="), |left, right| {
            Expression::Logic(Box::new(Logic::GreaterOrEqual(left, right)))
        }),
        infix(left(1), operator("<="), |left, right| {
            Expression::Logic(Box::new(Logic::LessOrEqual(left, right)))
        }),
        infix(left(1), operator("&&"), |left, right| {
            Expression::Logic(Box::new(Logic::And(left, right)))
        }),
        infix(left(1), operator("||"), |left, right| {
            Expression::Logic(Box::new(Logic::Or(left, right)))
        }),
    ));

    logic_expression.then_ignore(end())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_list() {
        assert_eq!(
            parser().parse("[]").unwrap(),
            Expression::Value(Value::List(vec![]))
        );
        assert_eq!(
            parser().parse("[42]").unwrap(),
            Expression::Value(Value::List(vec![Value::Integer(42)]))
        );
        assert_eq!(
            parser().parse("[42, 'foo', \"bar\", [1, 2, 3,]]").unwrap(),
            Expression::Value(Value::List(vec![
                Value::Integer(42),
                Value::String("foo".to_string()),
                Value::String("bar".to_string()),
                Value::List(vec![
                    Value::Integer(1),
                    Value::Integer(2),
                    Value::Integer(3),
                ])
            ]))
        );
    }

    #[test]
    fn parse_true() {
        assert_eq!(
            parser().parse("true").unwrap(),
            Expression::Value(Value::Boolean(true))
        );
    }

    #[test]
    fn parse_false() {
        assert_eq!(
            parser().parse("false").unwrap(),
            Expression::Value(Value::Boolean(false))
        );
    }

    #[test]
    fn parse_positive_float() {
        assert_eq!(
            parser().parse("0.0").unwrap(),
            Expression::Value(Value::Float(0.0))
        );
        assert_eq!(
            parser().parse("42.0").unwrap(),
            Expression::Value(Value::Float(42.0))
        );

        let max_float = f64::MAX.to_string() + ".0";

        assert_eq!(
            parser().parse(&max_float).unwrap(),
            Expression::Value(Value::Float(f64::MAX))
        );

        let min_positive_float = f64::MIN_POSITIVE.to_string();

        assert_eq!(
            parser().parse(&min_positive_float).unwrap(),
            Expression::Value(Value::Float(f64::MIN_POSITIVE))
        );
    }

    #[test]
    fn parse_negative_float() {
        assert_eq!(
            parser().parse("-0.0").unwrap(),
            Expression::Value(Value::Float(-0.0))
        );
        assert_eq!(
            parser().parse("-42.0").unwrap(),
            Expression::Value(Value::Float(-42.0))
        );

        let min_float = f64::MIN.to_string() + ".0";

        assert_eq!(
            parser().parse(&min_float).unwrap(),
            Expression::Value(Value::Float(f64::MIN))
        );

        let max_negative_float = format!("-{}", f64::MIN_POSITIVE);

        assert_eq!(
            parser().parse(&max_negative_float).unwrap(),
            Expression::Value(Value::Float(-f64::MIN_POSITIVE))
        );
    }

    #[test]
    fn parse_other_float() {
        assert_eq!(
            parser().parse("Infinity").unwrap(),
            Expression::Value(Value::Float(f64::INFINITY))
        );
        assert_eq!(
            parser().parse("-Infinity").unwrap(),
            Expression::Value(Value::Float(f64::NEG_INFINITY))
        );

        if let Expression::Value(Value::Float(float)) = parser().parse("NaN").unwrap() {
            assert!(float.is_nan())
        } else {
            panic!("Expected a float.")
        }
    }

    #[test]
    fn parse_positive_integer() {
        assert_eq!(
            parser().parse("0").unwrap(),
            Expression::Value(Value::Integer(0))
        );
        assert_eq!(
            parser().parse("1").unwrap(),
            Expression::Value(Value::Integer(1))
        );
        assert_eq!(
            parser().parse("2").unwrap(),
            Expression::Value(Value::Integer(2))
        );
        assert_eq!(
            parser().parse("3").unwrap(),
            Expression::Value(Value::Integer(3))
        );
        assert_eq!(
            parser().parse("4").unwrap(),
            Expression::Value(Value::Integer(4))
        );
        assert_eq!(
            parser().parse("5").unwrap(),
            Expression::Value(Value::Integer(5))
        );
        assert_eq!(
            parser().parse("6").unwrap(),
            Expression::Value(Value::Integer(6))
        );
        assert_eq!(
            parser().parse("7").unwrap(),
            Expression::Value(Value::Integer(7))
        );
        assert_eq!(
            parser().parse("8").unwrap(),
            Expression::Value(Value::Integer(8))
        );
        assert_eq!(
            parser().parse("9").unwrap(),
            Expression::Value(Value::Integer(9))
        );
        assert_eq!(
            parser().parse("42").unwrap(),
            Expression::Value(Value::Integer(42))
        );

        let maximum_integer = i64::MAX.to_string();

        assert_eq!(
            parser().parse(&maximum_integer).unwrap(),
            Expression::Value(Value::Integer(i64::MAX))
        );
    }

    #[test]
    fn parse_negative_integer() {
        assert_eq!(
            parser().parse("-0").unwrap(),
            Expression::Value(Value::Integer(-0))
        );
        assert_eq!(
            parser().parse("-1").unwrap(),
            Expression::Value(Value::Integer(-1))
        );
        assert_eq!(
            parser().parse("-2").unwrap(),
            Expression::Value(Value::Integer(-2))
        );
        assert_eq!(
            parser().parse("-3").unwrap(),
            Expression::Value(Value::Integer(-3))
        );
        assert_eq!(
            parser().parse("-4").unwrap(),
            Expression::Value(Value::Integer(-4))
        );
        assert_eq!(
            parser().parse("-5").unwrap(),
            Expression::Value(Value::Integer(-5))
        );
        assert_eq!(
            parser().parse("-6").unwrap(),
            Expression::Value(Value::Integer(-6))
        );
        assert_eq!(
            parser().parse("-7").unwrap(),
            Expression::Value(Value::Integer(-7))
        );
        assert_eq!(
            parser().parse("-8").unwrap(),
            Expression::Value(Value::Integer(-8))
        );
        assert_eq!(
            parser().parse("-9").unwrap(),
            Expression::Value(Value::Integer(-9))
        );
        assert_eq!(
            parser().parse("-42").unwrap(),
            Expression::Value(Value::Integer(-42))
        );

        let minimum_integer = i64::MIN.to_string();

        assert_eq!(
            parser().parse(&minimum_integer).unwrap(),
            Expression::Value(Value::Integer(i64::MIN))
        );
    }

    #[test]
    fn double_quoted_string() {
        assert_eq!(
            parser().parse("\"\"").unwrap(),
            Expression::Value(Value::String("".to_string()))
        );
        assert_eq!(
            parser().parse("\"1\"").unwrap(),
            Expression::Value(Value::String("1".to_string()))
        );
        assert_eq!(
            parser().parse("\"42\"").unwrap(),
            Expression::Value(Value::String("42".to_string()))
        );
        assert_eq!(
            parser().parse("\"foobar\"").unwrap(),
            Expression::Value(Value::String("foobar".to_string()))
        );
    }

    #[test]
    fn single_quoted_string() {
        assert_eq!(
            parser().parse("''").unwrap(),
            Expression::Value(Value::String("".to_string()))
        );
        assert_eq!(
            parser().parse("'1'").unwrap(),
            Expression::Value(Value::String("1".to_string()))
        );
        assert_eq!(
            parser().parse("'42'").unwrap(),
            Expression::Value(Value::String("42".to_string()))
        );
        assert_eq!(
            parser().parse("'foobar'").unwrap(),
            Expression::Value(Value::String("foobar".to_string()))
        );
    }

    #[test]
    fn grave_quoted_string() {
        assert_eq!(
            parser().parse("``").unwrap(),
            Expression::Value(Value::String("".to_string()))
        );
        assert_eq!(
            parser().parse("`1`").unwrap(),
            Expression::Value(Value::String("1".to_string()))
        );
        assert_eq!(
            parser().parse("`42`").unwrap(),
            Expression::Value(Value::String("42".to_string()))
        );
        assert_eq!(
            parser().parse("`foobar`").unwrap(),
            Expression::Value(Value::String("foobar".to_string()))
        );
    }
}
