use std::{collections::BTreeMap, ops::Range};

use chumsky::{prelude::*, Parser};

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Assignment(Assignment),
    Identifier(Identifier),
    Logic(Logic),
    Value(Value),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Identifier(String);

#[derive(Clone, Debug, PartialEq)]
pub struct Assignment {
    identifier: Identifier,
    value: Value,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Logic {
    left: LogicExpression,
    operator: LogicOperator,
    right: LogicExpression,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LogicOperator {
    Equal,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LogicExpression {
    Identifier(Identifier),
    Logic(Box<Logic>),
    Value(Value),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Boolean(bool),
    Float(f64),
    Integer(i64),
    List(Vec<Value>),
    Map(BTreeMap<String, Value>),
    Range(Range<i64>),
    String(String),
}

pub fn parser<'src>() -> impl Parser<'src, &'src str, Statement, extra::Err<Rich<'src, char>>> {
    let operator = |text: &'src str| just(text).padded();

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

    let identifier = text::ident().map(|text: &str| Identifier(text.to_string()));

    let assignment = identifier
        .then_ignore(operator("="))
        .then(value.clone())
        .map(|(identifier, value)| Assignment { identifier, value });

    let logic = recursive(|logic| {
        choice((
            value.clone().map(|value| LogicExpression::Value(value)),
            identifier.map(|identifier| LogicExpression::Identifier(identifier)),
            logic
                .clone()
                .map(|logic| LogicExpression::Logic(Box::new(logic))),
        ))
        .then(operator("==").map(|_| LogicOperator::Equal))
        .then(choice((
            value.clone().map(|value| LogicExpression::Value(value)),
            identifier.map(|identifier| LogicExpression::Identifier(identifier)),
            logic.map(|logic| LogicExpression::Logic(Box::new(logic))),
        )))
        .map(|((left, operator), right)| Logic {
            left,
            operator,
            right,
        })
    });

    choice((
        logic.map(|logic| Statement::Logic(logic)),
        assignment.map(|assignment| Statement::Assignment(assignment)),
        value.map(|value| Statement::Value(value)),
        identifier.map(|identifier| Statement::Identifier(identifier)),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_identifier() {
        assert_eq!(
            parser().parse("x").unwrap(),
            Statement::Identifier(Identifier("x".to_string()))
        );
        assert_eq!(
            parser().parse("foobar").unwrap(),
            Statement::Identifier(Identifier("foobar".to_string())),
        );
        assert_eq!(
            parser().parse("HELLO").unwrap(),
            Statement::Identifier(Identifier("HELLO".to_string())),
        );
    }

    #[test]
    fn parse_assignment() {
        assert_eq!(
            parser().parse("foobar=1").unwrap(),
            Statement::Assignment(Assignment {
                identifier: Identifier("foobar".to_string()),
                value: Value::Integer(1)
            })
        );
    }

    #[test]
    fn parse_logic() {
        assert_eq!(
            parser().parse("x == 1").unwrap(),
            Statement::Logic(Logic {
                left: LogicExpression::Identifier(Identifier("x".to_string())),
                operator: LogicOperator::Equal,
                right: LogicExpression::Value(Value::Integer(1))
            })
        );
    }

    #[test]
    fn parse_list() {
        assert_eq!(
            parser().parse("[]").unwrap(),
            Statement::Value(Value::List(vec![]))
        );
        assert_eq!(
            parser().parse("[42]").unwrap(),
            Statement::Value(Value::List(vec![Value::Integer(42)]))
        );
        assert_eq!(
            parser().parse("[42, 'foo', \"bar\", [1, 2, 3,]]").unwrap(),
            Statement::Value(Value::List(vec![
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
            Statement::Value(Value::Boolean(true))
        );
    }

    #[test]
    fn parse_false() {
        assert_eq!(
            parser().parse("false").unwrap(),
            Statement::Value(Value::Boolean(false))
        );
    }

    #[test]
    fn parse_positive_float() {
        assert_eq!(
            parser().parse("0.0").unwrap(),
            Statement::Value(Value::Float(0.0))
        );
        assert_eq!(
            parser().parse("42.0").unwrap(),
            Statement::Value(Value::Float(42.0))
        );

        let max_float = f64::MAX.to_string() + ".0";

        assert_eq!(
            parser().parse(&max_float).unwrap(),
            Statement::Value(Value::Float(f64::MAX))
        );

        let min_positive_float = f64::MIN_POSITIVE.to_string();

        assert_eq!(
            parser().parse(&min_positive_float).unwrap(),
            Statement::Value(Value::Float(f64::MIN_POSITIVE))
        );
    }

    #[test]
    fn parse_negative_float() {
        assert_eq!(
            parser().parse("-0.0").unwrap(),
            Statement::Value(Value::Float(-0.0))
        );
        assert_eq!(
            parser().parse("-42.0").unwrap(),
            Statement::Value(Value::Float(-42.0))
        );

        let min_float = f64::MIN.to_string() + ".0";

        assert_eq!(
            parser().parse(&min_float).unwrap(),
            Statement::Value(Value::Float(f64::MIN))
        );

        let max_negative_float = format!("-{}", f64::MIN_POSITIVE);

        assert_eq!(
            parser().parse(&max_negative_float).unwrap(),
            Statement::Value(Value::Float(-f64::MIN_POSITIVE))
        );
    }

    #[test]
    fn parse_other_float() {
        assert_eq!(
            parser().parse("Infinity").unwrap(),
            Statement::Value(Value::Float(f64::INFINITY))
        );
        assert_eq!(
            parser().parse("-Infinity").unwrap(),
            Statement::Value(Value::Float(f64::NEG_INFINITY))
        );

        if let Statement::Value(Value::Float(float)) = parser().parse("NaN").unwrap() {
            assert!(float.is_nan())
        } else {
            panic!("Expected a float.")
        }
    }

    #[test]
    fn parse_positive_integer() {
        assert_eq!(
            parser().parse("0").unwrap(),
            Statement::Value(Value::Integer(0))
        );
        assert_eq!(
            parser().parse("1").unwrap(),
            Statement::Value(Value::Integer(1))
        );
        assert_eq!(
            parser().parse("2").unwrap(),
            Statement::Value(Value::Integer(2))
        );
        assert_eq!(
            parser().parse("3").unwrap(),
            Statement::Value(Value::Integer(3))
        );
        assert_eq!(
            parser().parse("4").unwrap(),
            Statement::Value(Value::Integer(4))
        );
        assert_eq!(
            parser().parse("5").unwrap(),
            Statement::Value(Value::Integer(5))
        );
        assert_eq!(
            parser().parse("6").unwrap(),
            Statement::Value(Value::Integer(6))
        );
        assert_eq!(
            parser().parse("7").unwrap(),
            Statement::Value(Value::Integer(7))
        );
        assert_eq!(
            parser().parse("8").unwrap(),
            Statement::Value(Value::Integer(8))
        );
        assert_eq!(
            parser().parse("9").unwrap(),
            Statement::Value(Value::Integer(9))
        );
        assert_eq!(
            parser().parse("42").unwrap(),
            Statement::Value(Value::Integer(42))
        );

        let maximum_integer = i64::MAX.to_string();

        assert_eq!(
            parser().parse(&maximum_integer).unwrap(),
            Statement::Value(Value::Integer(i64::MAX))
        );
    }

    #[test]
    fn parse_negative_integer() {
        assert_eq!(
            parser().parse("-0").unwrap(),
            Statement::Value(Value::Integer(-0))
        );
        assert_eq!(
            parser().parse("-1").unwrap(),
            Statement::Value(Value::Integer(-1))
        );
        assert_eq!(
            parser().parse("-2").unwrap(),
            Statement::Value(Value::Integer(-2))
        );
        assert_eq!(
            parser().parse("-3").unwrap(),
            Statement::Value(Value::Integer(-3))
        );
        assert_eq!(
            parser().parse("-4").unwrap(),
            Statement::Value(Value::Integer(-4))
        );
        assert_eq!(
            parser().parse("-5").unwrap(),
            Statement::Value(Value::Integer(-5))
        );
        assert_eq!(
            parser().parse("-6").unwrap(),
            Statement::Value(Value::Integer(-6))
        );
        assert_eq!(
            parser().parse("-7").unwrap(),
            Statement::Value(Value::Integer(-7))
        );
        assert_eq!(
            parser().parse("-8").unwrap(),
            Statement::Value(Value::Integer(-8))
        );
        assert_eq!(
            parser().parse("-9").unwrap(),
            Statement::Value(Value::Integer(-9))
        );
        assert_eq!(
            parser().parse("-42").unwrap(),
            Statement::Value(Value::Integer(-42))
        );

        let minimum_integer = i64::MIN.to_string();

        assert_eq!(
            parser().parse(&minimum_integer).unwrap(),
            Statement::Value(Value::Integer(i64::MIN))
        );
    }

    #[test]
    fn double_quoted_string() {
        assert_eq!(
            parser().parse("\"\"").unwrap(),
            Statement::Value(Value::String("".to_string()))
        );
        assert_eq!(
            parser().parse("\"1\"").unwrap(),
            Statement::Value(Value::String("1".to_string()))
        );
        assert_eq!(
            parser().parse("\"42\"").unwrap(),
            Statement::Value(Value::String("42".to_string()))
        );
        assert_eq!(
            parser().parse("\"foobar\"").unwrap(),
            Statement::Value(Value::String("foobar".to_string()))
        );
    }

    #[test]
    fn single_quoted_string() {
        assert_eq!(
            parser().parse("''").unwrap(),
            Statement::Value(Value::String("".to_string()))
        );
        assert_eq!(
            parser().parse("'1'").unwrap(),
            Statement::Value(Value::String("1".to_string()))
        );
        assert_eq!(
            parser().parse("'42'").unwrap(),
            Statement::Value(Value::String("42".to_string()))
        );
        assert_eq!(
            parser().parse("'foobar'").unwrap(),
            Statement::Value(Value::String("foobar".to_string()))
        );
    }

    #[test]
    fn grave_quoted_string() {
        assert_eq!(
            parser().parse("``").unwrap(),
            Statement::Value(Value::String("".to_string()))
        );
        assert_eq!(
            parser().parse("`1`").unwrap(),
            Statement::Value(Value::String("1".to_string()))
        );
        assert_eq!(
            parser().parse("`42`").unwrap(),
            Statement::Value(Value::String("42".to_string()))
        );
        assert_eq!(
            parser().parse("`foobar`").unwrap(),
            Statement::Value(Value::String("foobar".to_string()))
        );
    }
}
