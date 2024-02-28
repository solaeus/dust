use chumsky::{input::SpannedInput, pratt::*, prelude::*};

use crate::{abstract_tree::*, error::Error, lexer::Token};

type ParserInput<'tokens, 'src> =
    SpannedInput<Token<'src>, SimpleSpan, &'tokens [(Token<'src>, SimpleSpan)]>;

fn parser<'tokens, 'src: 'tokens>() -> impl Parser<
    'tokens,
    ParserInput<'tokens, 'src>,
    Vec<(Statement<'src>, SimpleSpan)>,
    extra::Err<Rich<'tokens, Token<'src>, SimpleSpan>>,
> {
    let identifier = select! {
        Token::Identifier(text) => Identifier::new(text),
    };

    let expression = recursive(|expression| {
        let basic_value = select! {
            Token::None => ValueNode::Enum("Option", "None"),
            Token::Boolean(boolean) => ValueNode::Boolean(boolean),
            Token::Integer(integer) => ValueNode::Integer(integer),
            Token::Float(float) => ValueNode::Float(float),
            Token::String(string) => ValueNode::String(string),
        };

        let identifier_expression = identifier
            .map(|identifier| Expression::Identifier(identifier))
            .boxed();

        let list = expression
            .clone()
            .separated_by(just(Token::Control(',')))
            .allow_trailing()
            .collect()
            .delimited_by(just(Token::Control('[')), just(Token::Control(']')))
            .map(ValueNode::List);

        let value = choice((
            basic_value.map(|value| Expression::Value(value)),
            list.map(|list| Expression::Value(list)),
        ))
        .boxed();

        let atom = choice((
            identifier_expression.clone(),
            value.clone(),
            expression
                .clone()
                .delimited_by(just(Token::Control('(')), just(Token::Control(')'))),
        ));

        let logic = atom
            .pratt((
                prefix(2, just(Token::Operator("!")), |expression| {
                    Expression::Logic(Box::new(Logic::Not(expression)))
                }),
                infix(left(1), just(Token::Operator("==")), |left, right| {
                    Expression::Logic(Box::new(Logic::Equal(left, right)))
                }),
                infix(left(1), just(Token::Operator("!=")), |left, right| {
                    Expression::Logic(Box::new(Logic::NotEqual(left, right)))
                }),
                infix(left(1), just(Token::Operator(">")), |left, right| {
                    Expression::Logic(Box::new(Logic::Greater(left, right)))
                }),
                infix(left(1), just(Token::Operator("<")), |left, right| {
                    Expression::Logic(Box::new(Logic::Less(left, right)))
                }),
                infix(left(1), just(Token::Operator(">=")), |left, right| {
                    Expression::Logic(Box::new(Logic::GreaterOrEqual(left, right)))
                }),
                infix(left(1), just(Token::Operator("<=")), |left, right| {
                    Expression::Logic(Box::new(Logic::LessOrEqual(left, right)))
                }),
                infix(left(1), just(Token::Operator("&&")), |left, right| {
                    Expression::Logic(Box::new(Logic::And(left, right)))
                }),
                infix(left(1), just(Token::Operator("||")), |left, right| {
                    Expression::Logic(Box::new(Logic::Or(left, right)))
                }),
            ))
            .boxed();

        choice([logic, identifier_expression, value])
    });

    let statement = recursive(|statement| {
        let expression_statement = expression
            .map(|expression| Statement::Expression(expression))
            .boxed();

        let assignment = identifier
            .then_ignore(just(Token::Operator("=")))
            .then(statement.clone())
            .map(|(identifier, statement)| {
                Statement::Assignment(Assignment::new(identifier, statement))
            })
            .boxed();

        choice([assignment, expression_statement])
    });

    statement
        .map_with(|item, state| (item, state.span()))
        .repeated()
        .collect()
}

pub fn parse<'tokens, 'src: 'tokens>(
    tokens: &'tokens [(Token<'src>, SimpleSpan)],
) -> Result<Vec<(Statement<'src>, SimpleSpan)>, Error<'tokens>> {
    parser()
        .parse(tokens.spanned((0..0).into()))
        .into_result()
        .map_err(|error| Error::Parse(error))
}

#[cfg(test)]
mod tests {
    use crate::{abstract_tree::Logic, lexer::lex};

    use super::*;

    #[test]
    fn identifier() {
        assert_eq!(
            parse(&lex("x").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Identifier(Identifier::new("x")))
        );
        assert_eq!(
            parse(&lex("foobar").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Identifier(Identifier::new("foobar")))
        );
        assert_eq!(
            parse(&lex("HELLO").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Identifier(Identifier::new("HELLO")))
        );
    }

    #[test]
    fn assignment() {
        assert_eq!(
            parse(&lex("foobar = 1").unwrap()).unwrap()[0].0,
            Statement::Assignment(Assignment::new(
                Identifier::new("foobar"),
                Statement::Expression(Expression::Value(ValueNode::Integer(1)))
            )),
        );
    }

    #[test]
    fn logic() {
        assert_eq!(
            parse(&lex("x == 1").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Logic(Box::new(Logic::Equal(
                Expression::Identifier(Identifier::new("x")),
                Expression::Value(ValueNode::Integer(1))
            ))))
        );

        assert_eq!(
            parse(&lex("(x == 1) && (y == 2)").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Logic(Box::new(Logic::And(
                Expression::Logic(Box::new(Logic::Equal(
                    Expression::Identifier(Identifier::new("x")),
                    Expression::Value(ValueNode::Integer(1))
                ))),
                Expression::Logic(Box::new(Logic::Equal(
                    Expression::Identifier(Identifier::new("y")),
                    Expression::Value(ValueNode::Integer(2))
                ))),
            ))))
        );

        assert_eq!(
            parse(&lex("(x == 1) && (y == 2) && true").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Logic(Box::new(Logic::And(
                Expression::Logic(Box::new(Logic::And(
                    Expression::Logic(Box::new(Logic::Equal(
                        Expression::Identifier(Identifier::new("x")),
                        Expression::Value(ValueNode::Integer(1))
                    ))),
                    Expression::Logic(Box::new(Logic::Equal(
                        Expression::Identifier(Identifier::new("y")),
                        Expression::Value(ValueNode::Integer(2))
                    ))),
                ))),
                Expression::Value(ValueNode::Boolean(true))
            ))))
        );
    }

    #[test]
    fn list() {
        assert_eq!(
            parse(&lex("[]").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::List(Vec::with_capacity(0))))
        );
        assert_eq!(
            parse(&lex("[42]").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::List(vec![Expression::Value(
                ValueNode::Integer(42)
            )])))
        );
        assert_eq!(
            parse(&lex("[42, 'foo', 'bar', [1, 2, 3,]]").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::List(vec![
                Expression::Value(ValueNode::Integer(42)),
                Expression::Value(ValueNode::String("foo")),
                Expression::Value(ValueNode::String("bar")),
                Expression::Value(ValueNode::List(vec![
                    Expression::Value(ValueNode::Integer(1)),
                    Expression::Value(ValueNode::Integer(2)),
                    Expression::Value(ValueNode::Integer(3)),
                ]))
            ])),)
        );
    }

    #[test]
    fn r#true() {
        assert_eq!(
            parse(&lex("true").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::Boolean(true)))
        );
    }

    #[test]
    fn r#false() {
        assert_eq!(
            parse(&lex("false").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::Boolean(false)))
        );
    }

    #[test]
    fn positive_float() {
        assert_eq!(
            parse(&lex("0.0").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::Float(0.0)))
        );
        assert_eq!(
            parse(&lex("42.0").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::Float(42.0)))
        );

        let max_float = f64::MAX.to_string() + ".0";

        assert_eq!(
            parse(&lex(&max_float).unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::Float(f64::MAX)))
        );

        let min_positive_float = f64::MIN_POSITIVE.to_string();

        assert_eq!(
            parse(&lex(&min_positive_float).unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::Float(f64::MIN_POSITIVE)))
        );
    }

    #[test]
    fn negative_float() {
        assert_eq!(
            parse(&lex("-0.0").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::Float(-0.0)))
        );
        assert_eq!(
            parse(&lex("-42.0").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::Float(-42.0)))
        );

        let min_float = f64::MIN.to_string() + ".0";

        assert_eq!(
            parse(&lex(&min_float).unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::Float(f64::MIN)))
        );

        let max_negative_float = format!("-{}", f64::MIN_POSITIVE);

        assert_eq!(
            parse(&lex(&max_negative_float).unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::Float(-f64::MIN_POSITIVE)))
        );
    }

    #[test]
    fn other_float() {
        assert_eq!(
            parse(&lex("Infinity").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::Float(f64::INFINITY)))
        );
        assert_eq!(
            parse(&lex("-Infinity").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::Float(f64::NEG_INFINITY)))
        );

        if let Statement::Expression(Expression::Value(ValueNode::Float(float))) =
            &parse(&lex("NaN").unwrap()).unwrap()[0].0
        {
            assert!(float.is_nan());
        } else {
            panic!("Expected a float.");
        }
    }

    #[test]
    fn positive_integer() {
        for i in 0..10 {
            let source = i.to_string();
            let statements = parse(&lex(&source).unwrap()).unwrap();

            assert_eq!(
                statements[0].0,
                Statement::Expression(Expression::Value(ValueNode::Integer(i)))
            )
        }

        assert_eq!(
            parse(&lex("42").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::Integer(42)))
        );

        let maximum_integer = i64::MAX.to_string();

        assert_eq!(
            parse(&lex(&maximum_integer).unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::Integer(i64::MAX)))
        );
    }

    #[test]
    fn negative_integer() {
        for i in -9..1 {
            let source = i.to_string();
            let statements = parse(&lex(&source).unwrap()).unwrap();

            assert_eq!(
                statements[0].0,
                Statement::Expression(Expression::Value(ValueNode::Integer(i)))
            )
        }

        assert_eq!(
            parse(&lex("-42").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::Integer(-42)))
        );

        let minimum_integer = i64::MIN.to_string();

        assert_eq!(
            parse(&lex(&minimum_integer).unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::Integer(i64::MIN)))
        );
    }

    #[test]
    fn double_quoted_string() {
        assert_eq!(
            parse(&lex("\"\"").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::String("")))
        );
        assert_eq!(
            parse(&lex("\"42\"").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::String("42")))
        );
        assert_eq!(
            parse(&lex("\"foobar\"").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::String("foobar")))
        );
    }

    #[test]
    fn single_quoted_string() {
        assert_eq!(
            parse(&lex("''").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::String("")))
        );
        assert_eq!(
            parse(&lex("'42'").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::String("42")))
        );
        assert_eq!(
            parse(&lex("'foobar'").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::String("foobar")))
        );
    }

    #[test]
    fn grave_quoted_string() {
        assert_eq!(
            parse(&lex("``").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::String("")))
        );
        assert_eq!(
            parse(&lex("`42`").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::String("42")))
        );
        assert_eq!(
            parse(&lex("`foobar`").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::String("foobar")))
        );
    }
}
