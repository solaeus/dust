use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    ops::Range,
};

use chumsky::{prelude::*, Parser};

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
pub struct Logic {
    left: Expression,
    operator: LogicOperator,
    right: Expression,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LogicOperator {
    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterOrEqual,
    LessOrEqual,
    And,
    Or,
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

pub fn parser<'src>() -> impl Parser<'src, &'src str, Statement> {
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

        let integer = just('-').or_not().then(text::int(10).padded()).map(
            |(negative, integer_text): (Option<char>, &str)| {
                let integer = integer_text.parse::<i64>().unwrap();

                if negative.is_some() {
                    Value::Integer(-integer)
                } else {
                    Value::Integer(integer)
                }
            },
        );

        let delimited_string = |delimiter| {
            just(delimiter)
                .ignore_then(none_of(delimiter).repeated())
                .then_ignore(just(delimiter))
                .to_slice()
                .map(|text: &str| Value::String(text.to_string()))
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

    let expression = recursive(|expression| {
        let logic = expression
            .clone()
            .then(choice((
                just("==").to(LogicOperator::Equal),
                just("!=").to(LogicOperator::NotEqual),
                just(">").to(LogicOperator::Greater),
                just("<").to(LogicOperator::Less),
                just(">=").to(LogicOperator::GreaterOrEqual),
                just("<=").to(LogicOperator::LessOrEqual),
                just("&&").to(LogicOperator::And),
                just("||").to(LogicOperator::Or),
            )))
            .padded()
            .then(expression)
            .map(|((left, operator), right)| {
                Expression::Logic(Box::new(Logic {
                    left,
                    operator,
                    right,
                }))
            });

        let value = value.map(|value| Expression::Value(value));

        choice((logic, value))
    });

    let statement = recursive(|statement| {
        let assignment = text::ident()
            .map(|text| Identifier::new(text))
            .then(just("=").padded())
            .then(statement)
            .map(|((identifier, _), statement)| {
                Statement::Assignment(Box::new(Assignment {
                    identifier,
                    statement,
                }))
            });

        let expression = expression.map(|expression| Statement::Expression(expression));

        choice((assignment, expression))
    });

    statement.then_ignore(end())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_list() {
        assert_eq!(
            parser().parse("[]").unwrap(),
            Statement::value(Value::List(vec![]))
        );
        assert_eq!(
            parser().parse("[42]").unwrap(),
            Statement::value(Value::List(vec![Value::Integer(42)]))
        );
        assert_eq!(
            parser().parse("[42, 'foo', \"bar\", [1, 2, 3,]]").unwrap(),
            Statement::value(Value::List(vec![
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
            Statement::value(Value::Boolean(true))
        );
    }

    #[test]
    fn parse_false() {
        assert_eq!(
            parser().parse("false").unwrap(),
            Statement::value(Value::Boolean(false))
        );
    }

    #[test]
    fn parse_positive_float() {
        assert_eq!(
            parser().parse("0.0").unwrap(),
            Statement::value(Value::Float(0.0))
        );
        assert_eq!(
            parser().parse("42.0").unwrap(),
            Statement::value(Value::Float(42.0))
        );

        let max_float = f64::MAX.to_string() + ".0";

        assert_eq!(
            parser().parse(&max_float).unwrap(),
            Statement::value(Value::Float(f64::MAX))
        );

        let min_positive_float = f64::MIN_POSITIVE.to_string();

        assert_eq!(
            parser().parse(&min_positive_float).unwrap(),
            Statement::value(Value::Float(f64::MIN_POSITIVE))
        );
    }

    #[test]
    fn parse_negative_float() {
        assert_eq!(
            parser().parse("-0.0").unwrap(),
            Statement::value(Value::Float(-0.0))
        );
        assert_eq!(
            parser().parse("-42.0").unwrap(),
            Statement::value(Value::Float(-42.0))
        );

        let min_float = f64::MIN.to_string() + ".0";

        assert_eq!(
            parser().parse(&min_float).unwrap(),
            Statement::value(Value::Float(f64::MIN))
        );

        let max_negative_float = f64::MIN_POSITIVE.to_string();

        assert_eq!(
            parser().parse(&max_negative_float).unwrap(),
            Statement::value(Value::Float(-f64::MIN_POSITIVE))
        );
    }

    #[test]
    fn parse_other_float() {
        assert_eq!(
            parser().parse("Infinity").unwrap(),
            Statement::value(Value::Float(f64::INFINITY))
        );
        assert_eq!(
            parser().parse("-Infinity").unwrap(),
            Statement::value(Value::Float(f64::NEG_INFINITY))
        );

        if let Statement::Expression(Expression::Value(Value::Float(float))) =
            parser().parse("NaN").unwrap()
        {
            assert!(float.is_nan())
        } else {
            panic!("Expected a float.")
        }
    }

    #[test]
    fn parse_positive_integer() {
        assert_eq!(
            parser().parse("0").unwrap(),
            Statement::value(Value::Integer(0))
        );
        assert_eq!(
            parser().parse("1").unwrap(),
            Statement::value(Value::Integer(1))
        );
        assert_eq!(
            parser().parse("2").unwrap(),
            Statement::value(Value::Integer(2))
        );
        assert_eq!(
            parser().parse("3").unwrap(),
            Statement::value(Value::Integer(3))
        );
        assert_eq!(
            parser().parse("4").unwrap(),
            Statement::value(Value::Integer(4))
        );
        assert_eq!(
            parser().parse("5").unwrap(),
            Statement::value(Value::Integer(5))
        );
        assert_eq!(
            parser().parse("6").unwrap(),
            Statement::value(Value::Integer(6))
        );
        assert_eq!(
            parser().parse("7").unwrap(),
            Statement::value(Value::Integer(7))
        );
        assert_eq!(
            parser().parse("8").unwrap(),
            Statement::value(Value::Integer(8))
        );
        assert_eq!(
            parser().parse("9").unwrap(),
            Statement::value(Value::Integer(9))
        );
        assert_eq!(
            parser().parse("42").unwrap(),
            Statement::value(Value::Integer(42))
        );

        let maximum_integer = i64::MAX.to_string();

        assert_eq!(
            parser().parse(&maximum_integer).unwrap(),
            Statement::value(Value::Integer(i64::MAX))
        );
    }

    #[test]
    fn parse_negative_integer() {
        assert_eq!(
            parser().parse("-0").unwrap(),
            Statement::value(Value::Integer(-0))
        );
        assert_eq!(
            parser().parse("-1").unwrap(),
            Statement::value(Value::Integer(-1))
        );
        assert_eq!(
            parser().parse("-2").unwrap(),
            Statement::value(Value::Integer(-2))
        );
        assert_eq!(
            parser().parse("-3").unwrap(),
            Statement::value(Value::Integer(-3))
        );
        assert_eq!(
            parser().parse("-4").unwrap(),
            Statement::value(Value::Integer(-4))
        );
        assert_eq!(
            parser().parse("-5").unwrap(),
            Statement::value(Value::Integer(-5))
        );
        assert_eq!(
            parser().parse("-6").unwrap(),
            Statement::value(Value::Integer(-6))
        );
        assert_eq!(
            parser().parse("-7").unwrap(),
            Statement::value(Value::Integer(-7))
        );
        assert_eq!(
            parser().parse("-8").unwrap(),
            Statement::value(Value::Integer(-8))
        );
        assert_eq!(
            parser().parse("-9").unwrap(),
            Statement::value(Value::Integer(-9))
        );
        assert_eq!(
            parser().parse("-42").unwrap(),
            Statement::value(Value::Integer(-42))
        );

        let minimum_integer = i64::MIN.to_string();

        assert_eq!(
            parser().parse(&minimum_integer).unwrap(),
            Statement::value(Value::Integer(i64::MIN))
        );
    }

    #[test]
    fn double_quoted_string() {
        assert_eq!(
            parser().parse("\"\"").unwrap(),
            Statement::value(Value::String("".to_string()))
        );
        assert_eq!(
            parser().parse("\"1\"").unwrap(),
            Statement::value(Value::String("1".to_string()))
        );
        assert_eq!(
            parser().parse("\"42\"").unwrap(),
            Statement::value(Value::String("42".to_string()))
        );
        assert_eq!(
            parser().parse("\"foobar\"").unwrap(),
            Statement::value(Value::String("foobar".to_string()))
        );
    }

    #[test]
    fn single_quoted_string() {
        assert_eq!(
            parser().parse("''").unwrap(),
            Statement::value(Value::String("".to_string()))
        );
        assert_eq!(
            parser().parse("'1'").unwrap(),
            Statement::value(Value::String("1".to_string()))
        );
        assert_eq!(
            parser().parse("'42'").unwrap(),
            Statement::value(Value::String("42".to_string()))
        );
        assert_eq!(
            parser().parse("'foobar'").unwrap(),
            Statement::value(Value::String("foobar".to_string()))
        );
    }

    #[test]
    fn grave_quoted_string() {
        assert_eq!(
            parser().parse("``").unwrap(),
            Statement::value(Value::String("".to_string()))
        );
        assert_eq!(
            parser().parse("`1`").unwrap(),
            Statement::value(Value::String("1".to_string()))
        );
        assert_eq!(
            parser().parse("`42`").unwrap(),
            Statement::value(Value::String("42".to_string()))
        );
        assert_eq!(
            parser().parse("`foobar`").unwrap(),
            Statement::value(Value::String("foobar".to_string()))
        );
    }
}
