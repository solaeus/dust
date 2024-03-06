use std::{cell::RefCell, collections::HashMap};

use chumsky::{input::SpannedInput, pratt::*, prelude::*};

use crate::{abstract_tree::*, error::Error, lexer::Token};

pub type DustParser<'src> = Boxed<
    'src,
    'src,
    ParserInput<'src>,
    Vec<(Statement<'src>, SimpleSpan)>,
    extra::Err<Rich<'src, Token<'src>, SimpleSpan>>,
>;

pub type ParserInput<'src> =
    SpannedInput<Token<'src>, SimpleSpan, &'src [(Token<'src>, SimpleSpan)]>;

pub fn parse<'src>(
    tokens: &'src [(Token<'src>, SimpleSpan)],
) -> Result<Vec<(Statement<'src>, SimpleSpan)>, Vec<Error>> {
    parser()
        .parse(tokens.spanned((tokens.len()..tokens.len()).into()))
        .into_result()
        .map_err(|errors| errors.into_iter().map(|error| error.into()).collect())
}

pub fn parser<'src>() -> DustParser<'src> {
    let identifiers: RefCell<HashMap<&str, Identifier>> = RefCell::new(HashMap::new());

    let identifier = select! {
        Token::Identifier(text) => {
            let mut identifiers = identifiers.borrow_mut();

            if let Some(identifier) = identifiers.get(&text) {
                identifier.clone()
            } else {
                let new = Identifier::new(text);

                identifiers.insert(text, new.clone());

                new
            }
        }
    };

    let expression = recursive(|expression| {
        let basic_value = select! {
            Token::Boolean(boolean) => ValueNode::Boolean(boolean),
            Token::Integer(integer) => ValueNode::Integer(integer),
            Token::Float(float) => ValueNode::Float(float),
            Token::String(string) => ValueNode::String(string),
        }
        .map(|value| Expression::Value(value))
        .boxed();

        let identifier_expression = identifier
            .clone()
            .map(|identifier| Expression::Identifier(identifier))
            .boxed();

        let list = expression
            .clone()
            .separated_by(just(Token::Control(",")))
            .allow_trailing()
            .collect()
            .delimited_by(just(Token::Control("[")), just(Token::Control("]")))
            .map(|list| Expression::Value(ValueNode::List(list)))
            .boxed();

        let r#enum = identifier
            .clone()
            .then_ignore(just(Token::Control("::")))
            .then(identifier.clone())
            .map(|(name, variant)| Expression::Value(ValueNode::Enum(name, variant)))
            .boxed();

        let atom = choice((
            identifier_expression.clone(),
            basic_value.clone(),
            list.clone(),
            r#enum.clone(),
            expression
                .clone()
                .delimited_by(just(Token::Control("(")), just(Token::Control(")"))),
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

        choice((r#enum, logic, identifier_expression, list, basic_value))
    });

    let statement = recursive(|statement| {
        let expression_statement = expression
            .map(|expression| Statement::Expression(expression))
            .boxed();

        let basic_type = choice((
            just(Token::Keyword("bool")).to(Type::Boolean),
            just(Token::Keyword("float")).to(Type::Float),
            just(Token::Keyword("int")).to(Type::Integer),
            just(Token::Keyword("range")).to(Type::Range),
            just(Token::Keyword("str")).to(Type::String),
            just(Token::Keyword("list")).to(Type::List),
        ));

        let type_arguments = basic_type
            .clone()
            .delimited_by(just(Token::Control("(")), just(Token::Control(")")));

        let type_specification = just(Token::Control(":")).ignore_then(choice((
            basic_type
                .clone()
                .separated_by(just(Token::Control(",")))
                .collect()
                .delimited_by(just(Token::Control("[")), just(Token::Control("]")))
                .map(|types| Type::ListExact(types)),
            just(Token::Keyword("list"))
                .then(type_arguments)
                .map(|(_, item_type)| Type::ListOf(Box::new(item_type))),
            basic_type.clone(),
            identifier
                .clone()
                .map(|identifier| Type::Custom(identifier)),
        )));

        let assignment = identifier
            .then(type_specification.clone().or_not())
            .then_ignore(just(Token::Operator("=")))
            .then(statement.clone())
            .map(|((identifier, r#type), statement)| {
                Statement::Assignment(Assignment::new(identifier, r#type, statement))
            })
            .boxed();

        let block = statement
            .clone()
            .separated_by(just(Token::Control(";")).or_not())
            .collect()
            .delimited_by(just(Token::Control("{")), just(Token::Control("}")))
            .map(|statements| Statement::Block(Block::new(statements)))
            .boxed();

        let r#loop = statement
            .clone()
            .separated_by(just(Token::Control(";")).or_not())
            .collect()
            .delimited_by(
                just(Token::Keyword("loop")).then(just(Token::Control("{"))),
                just(Token::Control("}")),
            )
            .map(|statements| Statement::Loop(Loop::new(statements)))
            .boxed();

        choice((assignment, expression_statement, block, r#loop))
    });

    statement
        .map_with(|item, state| (item, state.span()))
        .repeated()
        .collect()
        .boxed()
}

#[cfg(test)]
mod tests {
    use crate::{abstract_tree::Logic, lexer::lex};

    use super::*;

    #[test]
    fn r#loop() {
        assert_eq!(
            parse(&lex("loop {}").unwrap()).unwrap()[0].0,
            Statement::Loop(Loop::new(vec![]))
        );
    }

    #[test]
    fn block() {
        assert_eq!(
            parse(&lex("{ x }").unwrap()).unwrap()[0].0,
            Statement::Block(Block::new(vec![Statement::Expression(
                Expression::Identifier(Identifier::new("x"))
            ),]))
        );

        assert_eq!(
            parse(
                &lex("
                {
                    x;
                    y;
                    z
                }
                ")
                .unwrap()
            )
            .unwrap()[0]
                .0,
            Statement::Block(Block::new(vec![
                Statement::Expression(Expression::Identifier(Identifier::new("x"))),
                Statement::Expression(Expression::Identifier(Identifier::new("y"))),
                Statement::Expression(Expression::Identifier(Identifier::new("z"))),
            ]))
        );

        assert_eq!(
            parse(
                &lex("
                {
                    1 == 1
                    z
                }
                ")
                .unwrap()
            )
            .unwrap()[0]
                .0,
            Statement::Block(Block::new(vec![
                Statement::Expression(Expression::Logic(Box::new(Logic::Equal(
                    Expression::Value(ValueNode::Integer(1)),
                    Expression::Value(ValueNode::Integer(1))
                )))),
                Statement::Expression(Expression::Identifier(Identifier::new("z"))),
            ]))
        );
    }

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
                None,
                Statement::Expression(Expression::Value(ValueNode::Integer(1)))
            )),
        );
    }

    #[test]
    fn assignment_with_basic_type() {
        assert_eq!(
            parse(&lex("foobar: int = 1").unwrap()).unwrap()[0].0,
            Statement::Assignment(Assignment::new(
                Identifier::new("foobar"),
                Some(Type::Integer),
                Statement::Expression(Expression::Value(ValueNode::Integer(1)))
            )),
        );
    }

    #[test]
    fn assignment_with_custom_type() {
        assert_eq!(
            parse(&lex("foobar: Foo = Foo::Bar").unwrap()).unwrap()[0].0,
            Statement::Assignment(Assignment::new(
                Identifier::new("foobar"),
                Some(Type::Custom(Identifier::new("Foo"))),
                Statement::Expression(Expression::Value(ValueNode::Enum(
                    Identifier::new("Foo"),
                    Identifier::new("Bar")
                )))
            )),
        );
    }

    #[test]
    fn assignment_with_list_types() {
        assert_eq!(
            parse(&lex("foobar: list = []").unwrap()).unwrap()[0].0,
            Statement::Assignment(Assignment::new(
                Identifier::new("foobar"),
                Some(Type::List),
                Statement::Expression(Expression::Value(ValueNode::List(vec![])))
            )),
        );

        assert_eq!(
            parse(&lex("foobar: list(int) = []").unwrap()).unwrap()[0].0,
            Statement::Assignment(Assignment::new(
                Identifier::new("foobar"),
                Some(Type::ListOf(Box::new(Type::Integer))),
                Statement::Expression(Expression::Value(ValueNode::List(vec![])))
            )),
        );

        assert_eq!(
            parse(&lex("foobar: [int, str] = [ 42, 'foo' ]").unwrap()).unwrap()[0].0,
            Statement::Assignment(Assignment::new(
                Identifier::new("foobar"),
                Some(Type::ListExact(vec![Type::Integer, Type::String])),
                Statement::Expression(Expression::Value(ValueNode::List(vec![
                    Expression::Value(ValueNode::Integer(42)),
                    Expression::Value(ValueNode::String("foo"))
                ])))
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
    fn r#enum() {
        assert_eq!(
            parse(&lex("Option::None").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::Enum(
                Identifier::new("Option"),
                Identifier::new("None")
            )))
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
            let tokens = lex(&source).unwrap();
            let statements = parse(&tokens).unwrap();

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
            let tokens = lex(&source).unwrap();
            let statements = parse(&tokens).unwrap();

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
