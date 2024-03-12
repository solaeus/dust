use std::{cell::RefCell, collections::HashMap};

use chumsky::{input::SpannedInput, pratt::*, prelude::*};

use crate::{
    abstract_tree::*,
    error::Error,
    lexer::{Control, Operator, Token},
};

pub type DustParser<'src> = Boxed<
    'src,
    'src,
    ParserInput<'src>,
    Vec<(Statement, SimpleSpan)>,
    extra::Err<Rich<'src, Token<'src>, SimpleSpan>>,
>;

pub type ParserInput<'src> =
    SpannedInput<Token<'src>, SimpleSpan, &'src [(Token<'src>, SimpleSpan)]>;

pub fn parse<'src>(
    tokens: &'src [(Token<'src>, SimpleSpan)],
) -> Result<Vec<(Statement, SimpleSpan)>, Vec<Error>> {
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

    let basic_value = select! {
        Token::Boolean(boolean) => ValueNode::Boolean(boolean),
        Token::Integer(integer) => ValueNode::Integer(integer),
        Token::Float(float) => ValueNode::Float(float),
        Token::String(string) => ValueNode::String(string.to_string()),
    }
    .map(|value| Expression::Value(value))
    .boxed();

    let type_specification = recursive(|type_specification| {
        let r#type = recursive(|r#type| {
            let function_type = type_specification
                .clone()
                .separated_by(just(Token::Control(Control::Comma)))
                .collect()
                .delimited_by(
                    just(Token::Control(Control::ParenOpen)),
                    just(Token::Control(Control::ParenClose)),
                )
                .then_ignore(just(Token::Control(Control::Arrow)))
                .then(r#type.clone())
                .map(|(parameter_types, return_type)| Type::Function {
                    parameter_types,
                    return_type: Box::new(return_type),
                });

            let list_of = just(Token::Keyword("list"))
                .ignore_then(r#type.clone().delimited_by(
                    just(Token::Control(Control::ParenOpen)),
                    just(Token::Control(Control::ParenClose)),
                ))
                .map(|item_type| Type::ListOf(Box::new(item_type)));

            let list_exact = r#type
                .clone()
                .separated_by(just(Token::Control(Control::Comma)))
                .collect()
                .delimited_by(
                    just(Token::Control(Control::SquareOpen)),
                    just(Token::Control(Control::SquareClose)),
                )
                .map(|types| Type::ListExact(types));

            choice((
                function_type,
                list_of,
                list_exact,
                just(Token::Keyword("any")).to(Type::Any),
                just(Token::Keyword("bool")).to(Type::Boolean),
                just(Token::Keyword("float")).to(Type::Float),
                just(Token::Keyword("int")).to(Type::Integer),
                just(Token::Keyword("none")).to(Type::None),
                just(Token::Keyword("range")).to(Type::Range),
                just(Token::Keyword("str")).to(Type::String),
                just(Token::Keyword("list")).to(Type::List),
                identifier
                    .clone()
                    .map(|identifier| Type::Custom(identifier)),
            ))
        });

        just(Token::Control(Control::Colon)).ignore_then(r#type)
    });

    let statement = recursive(|statement| {
        let block = statement
            .clone()
            .repeated()
            .collect()
            .delimited_by(
                just(Token::Control(Control::CurlyOpen)),
                just(Token::Control(Control::CurlyClose)),
            )
            .map(|statements| Block::new(statements));

        let expression = recursive(|expression| {
            let identifier_expression = identifier
                .clone()
                .map(|identifier| Expression::Identifier(identifier))
                .boxed();

            let range = {
                let raw_integer = select! {
                    Token::Integer(integer) => integer
                };

                raw_integer
                    .clone()
                    .then_ignore(just(Token::Control(Control::DoubleDot)))
                    .then(raw_integer)
                    .map(|(start, end)| Expression::Value(ValueNode::Range(start..end)))
            };

            let list = expression
                .clone()
                .separated_by(just(Token::Control(Control::Comma)))
                .allow_trailing()
                .collect()
                .delimited_by(
                    just(Token::Control(Control::SquareOpen)),
                    just(Token::Control(Control::SquareClose)),
                )
                .map(|list| Expression::Value(ValueNode::List(list)))
                .boxed();

            let map_assignment = identifier
                .clone()
                .then(type_specification.clone().or_not())
                .then_ignore(just(Token::Operator(Operator::Assign)))
                .then(expression.clone())
                .map(|((identifier, r#type), expression)| (identifier, r#type, expression));

            let map = map_assignment
                .separated_by(just(Token::Control(Control::Comma)).or_not())
                .allow_trailing()
                .collect()
                .delimited_by(
                    just(Token::Control(Control::CurlyOpen)),
                    just(Token::Control(Control::CurlyClose)),
                )
                .map(|map_assigment_list| Expression::Value(ValueNode::Map(map_assigment_list)));

            let function = identifier
                .clone()
                .then(type_specification.clone())
                .separated_by(just(Token::Control(Control::Comma)))
                .collect::<Vec<(Identifier, Type)>>()
                .delimited_by(
                    just(Token::Control(Control::ParenOpen)),
                    just(Token::Control(Control::ParenClose)),
                )
                .then(type_specification.clone())
                .then(block.clone())
                .map(|((parameters, return_type), body)| {
                    Expression::Value(ValueNode::Function {
                        parameters,
                        return_type,
                        body,
                    })
                })
                .boxed();

            let function_expression = choice((identifier_expression.clone(), function.clone()));

            let function_call = function_expression
                .then(
                    expression
                        .clone()
                        .separated_by(just(Token::Control(Control::Comma)))
                        .collect()
                        .delimited_by(
                            just(Token::Control(Control::ParenOpen)),
                            just(Token::Control(Control::ParenClose)),
                        ),
                )
                .map(|(function, arguments)| {
                    Expression::FunctionCall(FunctionCall::new(function, arguments))
                })
                .boxed();

            let atom = choice((
                function_call,
                identifier_expression.clone(),
                basic_value.clone(),
                list.clone(),
                expression.clone().delimited_by(
                    just(Token::Control(Control::ParenOpen)),
                    just(Token::Control(Control::ParenClose)),
                ),
            ));

            use Operator::*;

            let logic_math_and_index = atom
                .pratt((
                    prefix(2, just(Token::Operator(Not)), |expression| {
                        Expression::Logic(Box::new(Logic::Not(expression)))
                    }),
                    infix(left(1), just(Token::Operator(Equal)), |left, right| {
                        Expression::Logic(Box::new(Logic::Equal(left, right)))
                    }),
                    infix(left(1), just(Token::Operator(NotEqual)), |left, right| {
                        Expression::Logic(Box::new(Logic::NotEqual(left, right)))
                    }),
                    infix(left(1), just(Token::Operator(Greater)), |left, right| {
                        Expression::Logic(Box::new(Logic::Greater(left, right)))
                    }),
                    infix(left(1), just(Token::Operator(Less)), |left, right| {
                        Expression::Logic(Box::new(Logic::Less(left, right)))
                    }),
                    infix(
                        left(1),
                        just(Token::Operator(GreaterOrEqual)),
                        |left, right| {
                            Expression::Logic(Box::new(Logic::GreaterOrEqual(left, right)))
                        },
                    ),
                    infix(
                        left(1),
                        just(Token::Operator(LessOrEqual)),
                        |left, right| Expression::Logic(Box::new(Logic::LessOrEqual(left, right))),
                    ),
                    infix(left(1), just(Token::Operator(And)), |left, right| {
                        Expression::Logic(Box::new(Logic::And(left, right)))
                    }),
                    infix(left(1), just(Token::Operator(Or)), |left, right| {
                        Expression::Logic(Box::new(Logic::Or(left, right)))
                    }),
                    infix(left(1), just(Token::Operator(Add)), |left, right| {
                        Expression::Math(Box::new(Math::Add(left, right)))
                    }),
                    infix(left(1), just(Token::Operator(Subtract)), |left, right| {
                        Expression::Math(Box::new(Math::Subtract(left, right)))
                    }),
                    infix(left(2), just(Token::Operator(Multiply)), |left, right| {
                        Expression::Math(Box::new(Math::Multiply(left, right)))
                    }),
                    infix(left(2), just(Token::Operator(Divide)), |left, right| {
                        Expression::Math(Box::new(Math::Divide(left, right)))
                    }),
                    infix(left(1), just(Token::Operator(Modulo)), |left, right| {
                        Expression::Math(Box::new(Math::Modulo(left, right)))
                    }),
                    infix(
                        left(3),
                        just(Token::Control(Control::Dot)),
                        |left, right| Expression::Index(Box::new(Index::new(left, right))),
                    ),
                ))
                .boxed();

            choice((
                function,
                range,
                logic_math_and_index,
                identifier_expression,
                list,
                map,
                basic_value,
            ))
            .boxed()
        });

        let expression_statement = expression
            .clone()
            .map(|expression| Statement::Expression(expression))
            .boxed();

        let r#break = just(Token::Keyword("break")).to(Statement::Break);

        let assignment = identifier
            .clone()
            .then(type_specification.clone().or_not())
            .then(choice((
                just(Token::Operator(Operator::Assign)).to(AssignmentOperator::Assign),
                just(Token::Operator(Operator::AddAssign)).to(AssignmentOperator::AddAssign),
                just(Token::Operator(Operator::SubAssign)).to(AssignmentOperator::SubAssign),
            )))
            .then(statement.clone())
            .map(|(((identifier, r#type), operator), statement)| {
                Statement::Assignment(Assignment::new(identifier, r#type, operator, statement))
            })
            .boxed();

        let block_statement = block.clone().map(|block| Statement::Block(block));

        let r#loop = statement
            .clone()
            .repeated()
            .at_least(1)
            .collect()
            .delimited_by(
                just(Token::Keyword("loop")).then(just(Token::Control(Control::CurlyOpen))),
                just(Token::Control(Control::CurlyClose)),
            )
            .map(|statements| Statement::Loop(Loop::new(statements)))
            .boxed();

        let r#while = just(Token::Keyword("while"))
            .ignore_then(expression.clone())
            .then(block.clone())
            .map(|(expression, block)| Statement::While(While::new(expression, block)));

        let if_else = just(Token::Keyword("if"))
            .ignore_then(expression.clone())
            .then(block.clone())
            .then(
                just(Token::Keyword("else"))
                    .ignore_then(block.clone())
                    .or_not(),
            )
            .map(|((if_expression, if_block), else_block)| {
                Statement::IfElse(IfElse::new(if_expression, if_block, else_block))
            })
            .boxed();

        choice((
            if_else,
            assignment,
            expression_statement,
            r#break,
            block_statement,
            r#loop,
            r#while,
        ))
        .then_ignore(just(Token::Control(Control::Semicolon)).or_not())
        .boxed()
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
    fn r#while() {
        assert_eq!(
            parse(&lex("while true { output('hi') }").unwrap()).unwrap()[0].0,
            Statement::While(While::new(
                Expression::Value(ValueNode::Boolean(true)),
                Block::new(vec![Statement::Expression(Expression::FunctionCall(
                    FunctionCall::new(
                        Expression::Identifier(Identifier::new("output")),
                        vec![Expression::Value(ValueNode::String("hi".to_string()))]
                    )
                ))])
            ))
        )
    }

    #[test]
    fn types() {
        assert_eq!(
            parse(&lex("foobar : bool = true").unwrap()).unwrap()[0].0,
            Statement::Assignment(Assignment::new(
                Identifier::new("foobar"),
                Some(Type::Boolean),
                AssignmentOperator::Assign,
                Statement::Expression(Expression::Value(ValueNode::Boolean(true)))
            ))
        );
        assert_eq!(
            parse(&lex("foobar : list(bool) = [true]").unwrap()).unwrap()[0].0,
            Statement::Assignment(Assignment::new(
                Identifier::new("foobar"),
                Some(Type::ListOf(Box::new(Type::Boolean))),
                AssignmentOperator::Assign,
                Statement::Expression(Expression::Value(ValueNode::List(vec![Expression::Value(
                    ValueNode::Boolean(true)
                )])))
            ))
        );
        assert_eq!(
            parse(&lex("foobar : [bool, str] = [true, '42']").unwrap()).unwrap()[0].0,
            Statement::Assignment(Assignment::new(
                Identifier::new("foobar"),
                Some(Type::ListExact(vec![Type::Boolean, Type::String])),
                AssignmentOperator::Assign,
                Statement::Expression(Expression::Value(ValueNode::List(vec![
                    Expression::Value(ValueNode::Boolean(true)),
                    Expression::Value(ValueNode::String("42".to_string()))
                ])))
            ))
        );
        assert_eq!(
            parse(&lex("foobar : () -> any = some_function").unwrap()).unwrap()[0].0,
            Statement::Assignment(Assignment::new(
                Identifier::new("foobar"),
                Some(Type::Function {
                    parameter_types: vec![],
                    return_type: Box::new(Type::Any)
                }),
                AssignmentOperator::Assign,
                Statement::Expression(Expression::Identifier(Identifier::new("some_function")))
            ))
        );
    }

    #[test]
    fn function_call() {
        assert_eq!(
            parse(&lex("output()").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::FunctionCall(FunctionCall::new(
                Expression::Identifier(Identifier::new("output")),
                Vec::with_capacity(0),
            )))
        )
    }

    #[test]
    fn range() {
        assert_eq!(
            parse(&lex("1..10").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::Range(1..10)))
        )
    }

    #[test]
    fn function() {
        assert_eq!(
            parse(&lex("(x: int): int { x }").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::Function {
                parameters: vec![(Identifier::new("x"), Type::Integer)],
                return_type: Type::Integer,
                body: Block::new(vec![Statement::Expression(Expression::Identifier(
                    Identifier::new("x")
                ))])
            }))
        )
    }

    #[test]
    fn r#if() {
        assert_eq!(
            parse(&lex("if true { 'foo' }").unwrap()).unwrap()[0].0,
            Statement::IfElse(IfElse::new(
                Expression::Value(ValueNode::Boolean(true)),
                Block::new(vec![Statement::Expression(Expression::Value(
                    ValueNode::String("foo".to_string())
                ))]),
                None
            ))
        );
    }

    #[test]
    fn if_else() {
        assert_eq!(
            parse(&lex("if true {'foo' } else { 'bar' }").unwrap()).unwrap()[0].0,
            Statement::IfElse(IfElse::new(
                Expression::Value(ValueNode::Boolean(true)),
                Block::new(vec![Statement::Expression(Expression::Value(
                    ValueNode::String("foo".to_string())
                ))]),
                Some(Block::new(vec![Statement::Expression(Expression::Value(
                    ValueNode::String("bar".to_string())
                ))]))
            ))
        )
    }

    #[test]
    fn map() {
        assert_eq!(
            parse(&lex("{ foo = 'bar' }").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::Map(vec![(
                Identifier::new("foo"),
                None,
                Expression::Value(ValueNode::String("bar".to_string()))
            )])))
        );
        assert_eq!(
            parse(&lex("{ x = 1, y = 2, }").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::Map(vec![
                (
                    Identifier::new("x"),
                    None,
                    Expression::Value(ValueNode::Integer(1))
                ),
                (
                    Identifier::new("y"),
                    None,
                    Expression::Value(ValueNode::Integer(2))
                ),
            ])))
        );
        assert_eq!(
            parse(&lex("{ x = 1 y = 2 }").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::Map(vec![
                (
                    Identifier::new("x"),
                    None,
                    Expression::Value(ValueNode::Integer(1))
                ),
                (
                    Identifier::new("y"),
                    None,
                    Expression::Value(ValueNode::Integer(2))
                ),
            ])))
        );
    }

    #[test]
    fn math() {
        assert_eq!(
            parse(&lex("1 + 1").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Math(Box::new(Math::Add(
                Expression::Value(ValueNode::Integer(1)),
                Expression::Value(ValueNode::Integer(1))
            ))))
        );
    }

    #[test]
    fn r#loop() {
        assert_eq!(
            parse(&lex("loop { 42 }").unwrap()).unwrap()[0].0,
            Statement::Loop(Loop::new(vec![Statement::Expression(Expression::Value(
                ValueNode::Integer(42)
            ))]))
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
                AssignmentOperator::Assign,
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
                AssignmentOperator::Assign,
                Statement::Expression(Expression::Value(ValueNode::Integer(1)))
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
                AssignmentOperator::Assign,
                Statement::Expression(Expression::Value(ValueNode::List(vec![])))
            )),
        );

        assert_eq!(
            parse(&lex("foobar: list(int) = []").unwrap()).unwrap()[0].0,
            Statement::Assignment(Assignment::new(
                Identifier::new("foobar"),
                Some(Type::ListOf(Box::new(Type::Integer))),
                AssignmentOperator::Assign,
                Statement::Expression(Expression::Value(ValueNode::List(vec![])))
            )),
        );

        assert_eq!(
            parse(&lex("foobar: [int, str] = [ 42, 'foo' ]").unwrap()).unwrap()[0].0,
            Statement::Assignment(Assignment::new(
                Identifier::new("foobar"),
                Some(Type::ListExact(vec![Type::Integer, Type::String])),
                AssignmentOperator::Assign,
                Statement::Expression(Expression::Value(ValueNode::List(vec![
                    Expression::Value(ValueNode::Integer(42)),
                    Expression::Value(ValueNode::String("foo".to_string()))
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
                Expression::Value(ValueNode::String("foo".to_string())),
                Expression::Value(ValueNode::String("bar".to_string())),
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
            Statement::Expression(Expression::Value(ValueNode::String("".to_string())))
        );
        assert_eq!(
            parse(&lex("\"42\"").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::String("42".to_string())))
        );
        assert_eq!(
            parse(&lex("\"foobar\"").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::String("foobar".to_string())))
        );
    }

    #[test]
    fn single_quoted_string() {
        assert_eq!(
            parse(&lex("''").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::String("".to_string())))
        );
        assert_eq!(
            parse(&lex("'42'").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::String("42".to_string())))
        );
        assert_eq!(
            parse(&lex("'foobar'").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::String("foobar".to_string())))
        );
    }

    #[test]
    fn grave_quoted_string() {
        assert_eq!(
            parse(&lex("``").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::String("".to_string())))
        );
        assert_eq!(
            parse(&lex("`42`").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::String("42".to_string())))
        );
        assert_eq!(
            parse(&lex("`foobar`").unwrap()).unwrap()[0].0,
            Statement::Expression(Expression::Value(ValueNode::String("foobar".to_string())))
        );
    }
}
