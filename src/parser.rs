use chumsky::{input::SpannedInput, pratt::*, prelude::*};

use crate::{abstract_tree::*, error::Error, lexer::Token};

type ParserInput<'tokens, 'src> =
    SpannedInput<Token<'src>, SimpleSpan, &'tokens [(Token<'src>, SimpleSpan)]>;

fn parser<'tokens, 'src: 'tokens>() -> impl Parser<
    'tokens,
    ParserInput<'tokens, 'src>,
    Vec<(Statement, SimpleSpan)>,
    extra::Err<Rich<'tokens, Token<'src>, SimpleSpan>>,
> {
    recursive(|statement| {
        let identifier = select! {
            Token::Identifier(text) => Identifier::new(text),
        };

        let identifier_statement = identifier.map(|identifier| Statement::Identifier(identifier));

        let basic_value = select! {
            Token::None => Value::none(),
            Token::Boolean(boolean) => Value::boolean(boolean),
            Token::Integer(integer) => Value::integer(integer),
            Token::Float(float) => Value::float(float),
            Token::String(string) => Value::string(string.to_string()),
        };

        let list = statement
            .clone()
            .separated_by(just(Token::Control(',')))
            .allow_trailing()
            .collect()
            .delimited_by(just(Token::Control('[')), just(Token::Control(']')))
            .map(Value::list);

        let value = choice((
            basic_value.map(|value| Statement::Value(value)),
            list.map(|list| Statement::Value(list)),
        ));

        let assignment = identifier
            .then_ignore(just(Token::Operator("=")))
            .then(statement.clone())
            .map(|(identifier, statement)| {
                Statement::Assignment(Assignment::new(identifier, statement))
            });

        let atom = choice((
            identifier_statement,
            value.clone(),
            assignment.clone(),
            statement
                .clone()
                .delimited_by(just(Token::Control('(')), just(Token::Control(')'))),
        ));

        let logic = atom.pratt((
            prefix(2, just(Token::Operator("!")), |statement| {
                Statement::Logic(Box::new(Logic::Not(statement)))
            }),
            infix(left(1), just(Token::Operator("==")), |left, right| {
                Statement::Logic(Box::new(Logic::Equal(left, right)))
            }),
            infix(left(1), just(Token::Operator("!=")), |left, right| {
                Statement::Logic(Box::new(Logic::NotEqual(left, right)))
            }),
            infix(left(1), just(Token::Operator(">")), |left, right| {
                Statement::Logic(Box::new(Logic::Greater(left, right)))
            }),
            infix(left(1), just(Token::Operator("<")), |left, right| {
                Statement::Logic(Box::new(Logic::Less(left, right)))
            }),
            infix(left(1), just(Token::Operator(">=")), |left, right| {
                Statement::Logic(Box::new(Logic::GreaterOrEqual(left, right)))
            }),
            infix(left(1), just(Token::Operator("<=")), |left, right| {
                Statement::Logic(Box::new(Logic::LessOrEqual(left, right)))
            }),
            infix(left(1), just(Token::Operator("&&")), |left, right| {
                Statement::Logic(Box::new(Logic::And(left, right)))
            }),
            infix(left(1), just(Token::Operator("||")), |left, right| {
                Statement::Logic(Box::new(Logic::Or(left, right)))
            }),
        ));

        choice((assignment, logic, value, identifier_statement))
    })
    .map_with(|statement, state| (statement, state.span()))
    .repeated()
    .collect()
}

pub fn parse<'tokens>(
    tokens: &'tokens [(Token, SimpleSpan)],
) -> Result<Vec<(Statement, SimpleSpan)>, Error<'tokens>> {
    parser()
        .parse(tokens.spanned((0..0).into()))
        .into_result()
        .map_err(|error| Error::Parse(error))
}

#[cfg(test)]
mod tests {
    use crate::{
        abstract_tree::{value::ValueInner, Logic},
        lexer::lex,
    };

    use super::*;

    #[test]
    fn identifier() {
        assert_eq!(
            parse(&lex("x").unwrap()).unwrap()[0].0,
            Statement::Identifier(Identifier::new("x")),
        );
        assert_eq!(
            parse(&lex("foobar").unwrap()).unwrap()[0].0,
            Statement::Identifier(Identifier::new("foobar")),
        );
        assert_eq!(
            parse(&lex("HELLO").unwrap()).unwrap()[0].0,
            Statement::Identifier(Identifier::new("HELLO")),
        );
    }

    #[test]
    fn assignment() {
        assert_eq!(
            parse(&lex("foobar = 1").unwrap()).unwrap()[0].0,
            Statement::Assignment(Assignment::new(
                Identifier::new("foobar"),
                Statement::Value(Value::integer(1))
            )),
        );
    }

    #[test]
    fn logic() {
        assert_eq!(
            parse(&lex("x == 1").unwrap()).unwrap()[0].0,
            Statement::Logic(Box::new(Logic::Equal(
                Statement::Identifier(Identifier::new("x")),
                Statement::Value(Value::integer(1))
            ))),
        );
    }

    #[test]
    fn list() {
        assert_eq!(
            parse(&lex("[]").unwrap()).unwrap()[0].0,
            Statement::Value(Value::list(vec![])),
        );
        assert_eq!(
            parse(&lex("[42]").unwrap()).unwrap()[0].0,
            Statement::Value(Value::list(vec![Statement::Value(Value::integer(42))])),
        );
        assert_eq!(
            parse(&lex("[42, 'foo', 'bar', [1, 2, 3,]]").unwrap()).unwrap()[0].0,
            Statement::Value(Value::list(vec![
                Statement::Value(Value::integer(42)),
                Statement::Value(Value::string("foo")),
                Statement::Value(Value::string("bar")),
                Statement::Value(Value::list(vec![
                    Statement::Value(Value::integer(1)),
                    Statement::Value(Value::integer(2)),
                    Statement::Value(Value::integer(3)),
                ]))
            ])),
        );
    }

    #[test]
    fn r#true() {
        assert_eq!(
            parse(&lex("true").unwrap()).unwrap()[0].0,
            Statement::Value(Value::boolean(true))
        );
    }

    #[test]
    fn r#false() {
        assert_eq!(
            parse(&lex("false").unwrap()).unwrap()[0].0,
            Statement::Value(Value::boolean(false))
        );
    }

    #[test]
    fn positive_float() {
        assert_eq!(
            parse(&lex("0.0").unwrap()).unwrap()[0].0,
            Statement::Value(Value::float(0.0))
        );
        assert_eq!(
            parse(&lex("42.0").unwrap()).unwrap()[0].0,
            Statement::Value(Value::float(42.0))
        );

        let max_float = f64::MAX.to_string() + ".0";

        assert_eq!(
            parse(&lex(&max_float).unwrap()).unwrap()[0].0,
            Statement::Value(Value::float(f64::MAX))
        );

        let min_positive_float = f64::MIN_POSITIVE.to_string();

        assert_eq!(
            parse(&lex(&min_positive_float).unwrap()).unwrap()[0].0,
            Statement::Value(Value::float(f64::MIN_POSITIVE))
        );
    }

    #[test]
    fn negative_float() {
        assert_eq!(
            parse(&lex("-0.0").unwrap()).unwrap()[0].0,
            Statement::Value(Value::float(-0.0))
        );
        assert_eq!(
            parse(&lex("-42.0").unwrap()).unwrap()[0].0,
            Statement::Value(Value::float(-42.0))
        );

        let min_float = f64::MIN.to_string() + ".0";

        assert_eq!(
            parse(&lex(&min_float).unwrap()).unwrap()[0].0,
            Statement::Value(Value::float(f64::MIN))
        );

        let max_negative_float = format!("-{}", f64::MIN_POSITIVE);

        assert_eq!(
            parse(&lex(&max_negative_float).unwrap()).unwrap()[0].0,
            Statement::Value(Value::float(-f64::MIN_POSITIVE))
        );
    }

    #[test]
    fn other_float() {
        assert_eq!(
            parse(&lex("Infinity").unwrap()).unwrap()[0].0,
            Statement::Value(Value::float(f64::INFINITY))
        );
        assert_eq!(
            parse(&lex("-Infinity").unwrap()).unwrap()[0].0,
            Statement::Value(Value::float(f64::NEG_INFINITY))
        );

        if let Statement::Value(value) = &parse(&lex("NaN").unwrap()).unwrap()[0].0 {
            if let ValueInner::Float(float) = value.inner().as_ref() {
                return assert!(float.is_nan());
            }
        }

        panic!("Expected a float.")
    }

    #[test]
    fn positive_integer() {
        for i in 0..10 {
            let source = i.to_string();
            let statements = parse(&lex(&source).unwrap()).unwrap();

            assert_eq!(statements[0].0, Statement::Value(Value::integer(i)))
        }

        assert_eq!(
            parse(&lex("42").unwrap()).unwrap()[0].0,
            Statement::Value(Value::integer(42))
        );

        let maximum_integer = i64::MAX.to_string();

        assert_eq!(
            parse(&lex(&maximum_integer).unwrap()).unwrap()[0].0,
            Statement::Value(Value::integer(i64::MAX))
        );
    }

    #[test]
    fn negative_integer() {
        for i in -9..1 {
            let source = i.to_string();
            let statements = parse(&lex(&source).unwrap()).unwrap();

            assert_eq!(statements[0].0, Statement::Value(Value::integer(i)))
        }

        assert_eq!(
            parse(&lex("-42").unwrap()).unwrap()[0].0,
            Statement::Value(Value::integer(-42))
        );

        let minimum_integer = i64::MIN.to_string();

        assert_eq!(
            parse(&lex(&minimum_integer).unwrap()).unwrap()[0].0,
            Statement::Value(Value::integer(i64::MIN))
        );
    }

    #[test]
    fn double_quoted_string() {
        assert_eq!(
            parse(&lex("\"\"").unwrap()).unwrap()[0].0,
            Statement::Value(Value::string("".to_string()))
        );
        assert_eq!(
            parse(&lex("\"42\"").unwrap()).unwrap()[0].0,
            Statement::Value(Value::string("42".to_string()))
        );
        assert_eq!(
            parse(&lex("\"foobar\"").unwrap()).unwrap()[0].0,
            Statement::Value(Value::string("foobar".to_string()))
        );
    }

    #[test]
    fn single_quoted_string() {
        assert_eq!(
            parse(&lex("''").unwrap()).unwrap()[0].0,
            Statement::Value(Value::string("".to_string()))
        );
        assert_eq!(
            parse(&lex("'42'").unwrap()).unwrap()[0].0,
            Statement::Value(Value::string("42".to_string()))
        );
        assert_eq!(
            parse(&lex("'foobar'").unwrap()).unwrap()[0].0,
            Statement::Value(Value::string("foobar".to_string()))
        );
    }

    #[test]
    fn grave_quoted_string() {
        assert_eq!(
            parse(&lex("``").unwrap()).unwrap()[0].0,
            Statement::Value(Value::string("".to_string()))
        );
        assert_eq!(
            parse(&lex("`42`").unwrap()).unwrap()[0].0,
            Statement::Value(Value::string("42".to_string()))
        );
        assert_eq!(
            parse(&lex("`foobar`").unwrap()).unwrap()[0].0,
            Statement::Value(Value::string("foobar".to_string()))
        );
    }
}
