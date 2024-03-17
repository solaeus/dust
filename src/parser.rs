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
    Vec<Positioned<Statement>>,
    extra::Err<Rich<'src, Token<'src>, SimpleSpan>>,
>;

pub type ParserInput<'src> =
    SpannedInput<Token<'src>, SimpleSpan, &'src [(Token<'src>, SimpleSpan)]>;

pub fn parse<'src>(
    tokens: &'src [(Token<'src>, SimpleSpan)],
) -> Result<Vec<Positioned<Statement>>, Vec<Error>> {
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
    .map_with(|value, state| Expression::Value(value).positioned(state.span()))
    .boxed();

    let r#type = recursive(|r#type| {
        let function_type = r#type
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
    })
    .map_with(|r#type, state| r#type.positioned(state.span()));

    let type_specification = just(Token::Control(Control::Colon)).ignore_then(r#type.clone());

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

        let positioned_block = block
            .clone()
            .map_with(|block, state| block.positioned(state.span()));

        let expression = recursive(|expression| {
            let identifier_expression = identifier.clone().map_with(|identifier, state| {
                Expression::Identifier(identifier).positioned(state.span())
            });

            let range = {
                let raw_integer = select! {
                    Token::Integer(integer) => integer
                };

                raw_integer
                    .clone()
                    .then_ignore(just(Token::Control(Control::DoubleDot)))
                    .then(raw_integer)
                    .map_with(|(start, end), state| {
                        Expression::Value(ValueNode::Range(start..end)).positioned(state.span())
                    })
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
                .map_with(|list, state| {
                    Expression::Value(ValueNode::List(list)).positioned(state.span())
                });

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
                .map_with(|map_assigment_list, state| {
                    Expression::Value(ValueNode::Map(map_assigment_list)).positioned(state.span())
                });

            let function = identifier
                .clone()
                .then(r#type.clone())
                .separated_by(just(Token::Control(Control::Comma)))
                .collect()
                .delimited_by(
                    just(Token::Control(Control::ParenOpen)),
                    just(Token::Control(Control::ParenClose)),
                )
                .then(r#type.clone())
                .then(block.clone())
                .map_with(|((parameters, return_type), body), state| {
                    Expression::Value(ValueNode::Function {
                        parameters,
                        return_type,
                        body: body.positioned(state.span()),
                    })
                    .positioned(state.span())
                });

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
                .map_with(|(function, arguments), state| {
                    Expression::FunctionCall(FunctionCall::new(function, arguments))
                        .positioned(state.span())
                });

            let atom = choice((
                function_call.clone(),
                identifier_expression.clone(),
                basic_value.clone(),
                list.clone(),
                expression.clone().delimited_by(
                    just(Token::Control(Control::ParenOpen)),
                    just(Token::Control(Control::ParenClose)),
                ),
            ));

            use Operator::*;

            let logic_math_and_index = atom.pratt((
                prefix(2, just(Token::Operator(Not)), |_, expression, span| {
                    Expression::Logic(Box::new(Logic::Not(expression))).positioned(span)
                }),
                infix(
                    left(3),
                    just(Token::Control(Control::Dot)),
                    |left, _, right, span| {
                        Expression::Index(Box::new(Index::new(left, right))).positioned(span)
                    },
                ),
                infix(
                    left(1),
                    just(Token::Operator(Equal)),
                    |left, _, right, span| {
                        Expression::Logic(Box::new(Logic::Equal(left, right))).positioned(span)
                    },
                ),
                infix(
                    left(1),
                    just(Token::Operator(NotEqual)),
                    |left, _, right, span| {
                        Expression::Logic(Box::new(Logic::NotEqual(left, right))).positioned(span)
                    },
                ),
                infix(
                    left(1),
                    just(Token::Operator(Greater)),
                    |left, _, right, span| {
                        Expression::Logic(Box::new(Logic::Greater(left, right))).positioned(span)
                    },
                ),
                infix(
                    left(1),
                    just(Token::Operator(Less)),
                    |left, _, right, span| {
                        Expression::Logic(Box::new(Logic::Less(left, right))).positioned(span)
                    },
                ),
                infix(
                    left(1),
                    just(Token::Operator(GreaterOrEqual)),
                    |left, _, right, span| {
                        Expression::Logic(Box::new(Logic::GreaterOrEqual(left, right)))
                            .positioned(span)
                    },
                ),
                infix(
                    left(1),
                    just(Token::Operator(LessOrEqual)),
                    |left, _, right, span| {
                        Expression::Logic(Box::new(Logic::LessOrEqual(left, right)))
                            .positioned(span)
                    },
                ),
                infix(
                    left(1),
                    just(Token::Operator(And)),
                    |left, _, right, span| {
                        Expression::Logic(Box::new(Logic::And(left, right))).positioned(span)
                    },
                ),
                infix(
                    left(1),
                    just(Token::Operator(Or)),
                    |left, _, right, span| {
                        Expression::Logic(Box::new(Logic::Or(left, right))).positioned(span)
                    },
                ),
                infix(
                    left(1),
                    just(Token::Operator(Add)),
                    |left, _, right, span| {
                        Expression::Math(Box::new(Math::Add(left, right))).positioned(span)
                    },
                ),
                infix(
                    left(1),
                    just(Token::Operator(Subtract)),
                    |left, _, right, span| {
                        Expression::Math(Box::new(Math::Subtract(left, right))).positioned(span)
                    },
                ),
                infix(
                    left(2),
                    just(Token::Operator(Multiply)),
                    |left, _, right, span| {
                        Expression::Math(Box::new(Math::Multiply(left, right))).positioned(span)
                    },
                ),
                infix(
                    left(2),
                    just(Token::Operator(Divide)),
                    |left, _, right, span| {
                        Expression::Math(Box::new(Math::Divide(left, right))).positioned(span)
                    },
                ),
                infix(
                    left(1),
                    just(Token::Operator(Modulo)),
                    |left, _, right, span| {
                        Expression::Math(Box::new(Math::Modulo(left, right))).positioned(span)
                    },
                ),
            ));

            choice((
                function,
                function_call,
                range,
                logic_math_and_index,
                identifier_expression,
                list,
                map,
                basic_value,
            ))
        });

        let expression_statement = expression.clone().map_with(|expression, state| {
            Statement::Expression(expression).positioned(state.span())
        });

        let r#break = just(Token::Keyword("break"))
            .map_with(|_, state| Statement::Break.positioned(state.span()));

        let assignment = identifier
            .clone()
            .then(type_specification.clone().or_not())
            .then(choice((
                just(Token::Operator(Operator::Assign)).to(AssignmentOperator::Assign),
                just(Token::Operator(Operator::AddAssign)).to(AssignmentOperator::AddAssign),
                just(Token::Operator(Operator::SubAssign)).to(AssignmentOperator::SubAssign),
            )))
            .then(statement.clone())
            .map_with(|(((identifier, r#type), operator), statement), state| {
                Statement::Assignment(Assignment::new(identifier, r#type, operator, statement))
                    .positioned(state.span())
            });

        let block_statement = block
            .clone()
            .map_with(|block, state| Statement::Block(block).positioned(state.span()));

        let r#loop = statement
            .clone()
            .repeated()
            .at_least(1)
            .collect()
            .delimited_by(
                just(Token::Keyword("loop")).then(just(Token::Control(Control::CurlyOpen))),
                just(Token::Control(Control::CurlyClose)),
            )
            .map_with(|statements, state| {
                Statement::Loop(Loop::new(statements)).positioned(state.span())
            });

        let r#while = just(Token::Keyword("while"))
            .ignore_then(expression.clone())
            .then(block.clone())
            .map_with(|(expression, block), state| {
                Statement::While(While::new(expression, block)).positioned(state.span())
            });

        let if_else = just(Token::Keyword("if"))
            .ignore_then(expression.clone())
            .then(positioned_block.clone())
            .then(
                just(Token::Keyword("else"))
                    .ignore_then(positioned_block.clone())
                    .or_not(),
            )
            .map_with(|((if_expression, if_block), else_block), state| {
                Statement::IfElse(IfElse::new(if_expression, if_block, else_block))
                    .positioned(state.span())
            });

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
    });

    statement.repeated().collect().boxed()
}

#[cfg(test)]
mod tests {
    use crate::lexer::lex;

    use super::*;

    #[test]
    fn r#while() {
        assert_eq!(
            parse(&lex("while true { output('hi') }").unwrap()).unwrap()[0],
            Statement::While(While::new(
                Expression::Value(ValueNode::Boolean(true)).positioned((6..11).into()),
                Block::new(vec![Statement::Expression(
                    Expression::FunctionCall(FunctionCall::new(
                        Expression::Identifier(Identifier::new("output"))
                            .positioned((13..19).into()),
                        vec![Expression::Value(ValueNode::String("hi".to_string()))
                            .positioned((20..24).into())]
                    ))
                    .positioned((13..26).into())
                )
                .positioned((13..26).into())])
            ))
            .positioned((0..27).into())
        )
    }

    #[test]
    fn boolean_type() {
        assert_eq!(
            parse(&lex("foobar : bool = true").unwrap()).unwrap()[0],
            Statement::Assignment(Assignment::new(
                Identifier::new("foobar"),
                Some(Type::Boolean.positioned((9..14).into())),
                AssignmentOperator::Assign,
                Statement::Expression(
                    Expression::Value(ValueNode::Boolean(true)).positioned((16..20).into())
                )
                .positioned((16..20).into())
            ),)
            .positioned((0..20).into())
        );
    }

    #[test]
    fn list_of_type() {
        assert_eq!(
            parse(&lex("foobar : list(bool) = [true]").unwrap()).unwrap()[0],
            Statement::Assignment(Assignment::new(
                Identifier::new("foobar"),
                Some(Type::ListOf(Box::new(Type::Boolean)).positioned((9..20).into())),
                AssignmentOperator::Assign,
                Statement::Expression(
                    Expression::Value(ValueNode::List(vec![Expression::Value(
                        ValueNode::Boolean(true)
                    )
                    .positioned((23..27).into())]))
                    .positioned((22..28).into())
                )
                .positioned((22..28).into())
            ))
            .positioned((0..28).into())
        );
    }

    #[test]
    fn list_exact_type() {
        assert_eq!(
            parse(&lex("foobar : [bool, str] = [true, '42']").unwrap()).unwrap()[0],
            Statement::Assignment(Assignment::new(
                Identifier::new("foobar"),
                Some(Type::ListExact(vec![Type::Boolean, Type::String]).positioned((9..21).into())),
                AssignmentOperator::Assign,
                Statement::Expression(
                    Expression::Value(ValueNode::List(vec![
                        Expression::Value(ValueNode::Boolean(true)).positioned((24..28).into()),
                        Expression::Value(ValueNode::String("42".to_string()))
                            .positioned((30..34).into())
                    ]))
                    .positioned((23..35).into())
                )
                .positioned((23..35).into())
            ),)
            .positioned((0..35).into())
        );
    }

    #[test]
    fn function_type() {
        assert_eq!(
            parse(&lex("foobar : () -> any = some_function").unwrap()).unwrap()[0],
            Statement::Assignment(Assignment::new(
                Identifier::new("foobar"),
                Some(
                    Type::Function {
                        parameter_types: vec![],
                        return_type: Box::new(Type::Any)
                    }
                    .positioned((9..19).into())
                ),
                AssignmentOperator::Assign,
                Statement::Expression(
                    Expression::Identifier(Identifier::new("some_function"))
                        .positioned((21..34).into())
                )
                .positioned((21..34).into())
            ),)
            .positioned((0..34).into())
        );
    }

    // #[test]
    // fn function_call() {
    //     assert_eq!(
    //         parse(&lex("output()").unwrap()).unwrap()[0],
    //         Statement::expression(
    //             Expression::FunctionCall(FunctionCall::new(
    //                 Expression::Identifier(Identifier::new("output")),
    //                 Vec::with_capacity(0),
    //             )),
    //             (0..0).into()
    //         )
    //     )
    // }

    // #[test]
    // fn range() {
    //     assert_eq!(
    //         parse(&lex("1..10").unwrap()).unwrap()[0],
    //         Statement::expression(Expression::Value(ValueNode::Range(1..10)), (0..0).into())
    //     )
    // }

    // #[test]
    // fn function() {
    //     assert_eq!(
    //         parse(&lex("(x: int): int { x }").unwrap()).unwrap()[0],
    //         Statement::expression(
    //             Expression::Value(ValueNode::Function {
    //                 parameters: vec![(Identifier::new("x"), Type::Integer)],
    //                 return_type: Type::Integer,
    //                 body: Block::new(vec![Statement::expression(
    //                     Expression::Identifier(Identifier::new("x")),
    //                     (0..0).into()
    //                 )])
    //             }),
    //             (0..0).into()
    //         )
    //     )
    // }

    // #[test]
    // fn r#if() {
    //     assert_eq!(
    //         parse(&lex("if true { 'foo' }").unwrap()).unwrap()[0],
    //         Statement::if_else(
    //             IfElse::new(
    //                 Expression::Value(ValueNode::Boolean(true)),
    //                 Block::new(vec![Statement::expression(
    //                     Expression::Value(ValueNode::String("foo".to_string())),
    //                     (0..0).into()
    //                 )]),
    //                 None
    //             ),
    //             (0..0).into()
    //         )
    //     );
    // }

    // #[test]
    // fn if_else() {
    //     assert_eq!(
    //         parse(&lex("if true {'foo' } else { 'bar' }").unwrap()).unwrap()[0],
    //         Statement::if_else(
    //             IfElse::new(
    //                 Expression::Value(ValueNode::Boolean(true)),
    //                 Block::new(vec![Statement::expression(
    //                     Expression::Value(ValueNode::String("foo".to_string())),
    //                     (0..0).into()
    //                 )]),
    //                 Some(Block::new(vec![Statement::expression(
    //                     Expression::Value(ValueNode::String("bar".to_string())),
    //                     (0..0).into()
    //                 )]))
    //             ),
    //             (0..0).into()
    //         )
    //     )
    // }

    // #[test]
    // fn map() {
    //     assert_eq!(
    //         parse(&lex("{ foo = 'bar' }").unwrap()).unwrap()[0],
    //         Statement::expression(
    //             Expression::Value(ValueNode::Map(vec![(
    //                 Identifier::new("foo"),
    //                 None,
    //                 Expression::Value(ValueNode::String("bar".to_string()))
    //             )])),
    //             (0..0).into()
    //         )
    //     );
    //     assert_eq!(
    //         parse(&lex("{ x = 1, y = 2, }").unwrap()).unwrap()[0],
    //         Statement::expression(
    //             Expression::Value(ValueNode::Map(vec![
    //                 (
    //                     Identifier::new("x"),
    //                     None,
    //                     Expression::Value(ValueNode::Integer(1))
    //                 ),
    //                 (
    //                     Identifier::new("y"),
    //                     None,
    //                     Expression::Value(ValueNode::Integer(2))
    //                 ),
    //             ])),
    //             (0..0).into()
    //         )
    //     );
    //     assert_eq!(
    //         parse(&lex("{ x = 1 y = 2 }").unwrap()).unwrap()[0],
    //         Statement::expression(
    //             Expression::Value(ValueNode::Map(vec![
    //                 (
    //                     Identifier::new("x"),
    //                     None,
    //                     Expression::Value(ValueNode::Integer(1))
    //                 ),
    //                 (
    //                     Identifier::new("y"),
    //                     None,
    //                     Expression::Value(ValueNode::Integer(2))
    //                 ),
    //             ])),
    //             (0..0).into()
    //         )
    //     );
    // }

    // #[test]
    // fn math() {
    //     assert_eq!(
    //         parse(&lex("1 + 1").unwrap()).unwrap()[0],
    //         Statement::expression(
    //             Expression::Math(Box::new(Math::Add(
    //                 Expression::Value(ValueNode::Integer(1)),
    //                 Expression::Value(ValueNode::Integer(1))
    //             ))),
    //             (0..0).into()
    //         )
    //     );
    // }

    // #[test]
    // fn r#loop() {
    //     assert_eq!(
    //         parse(&lex("loop { 42 }").unwrap()).unwrap()[0],
    //         Statement::r#loop(
    //             Loop::new(vec![Statement::expression(
    //                 Expression::Value(ValueNode::Integer(42)),
    //                 (0..0).into()
    //             )]),
    //             (0..0).into()
    //         )
    //     );
    // }

    // #[test]
    // fn block() {
    //     assert_eq!(
    //         parse(&lex("{ x }").unwrap()).unwrap()[0],
    //         Statement::block(
    //             Block::new(vec![Statement::expression(
    //                 Expression::Identifier(Identifier::new("x")),
    //                 (0..0).into()
    //             )],),
    //             (0..0).into()
    //         )
    //     );

    //     assert_eq!(
    //         parse(
    //             &lex("
    //             {
    //                 x;
    //                 y;
    //                 z
    //             }
    //             ")
    //             .unwrap()
    //         )
    //         .unwrap()[0],
    //         Statement::block(
    //             Block::new(vec![
    //                 Statement::expression(
    //                     Expression::Identifier(Identifier::new("x")),
    //                     (0..0).into()
    //                 ),
    //                 Statement::expression(
    //                     Expression::Identifier(Identifier::new("y")),
    //                     (0..0).into()
    //                 ),
    //                 Statement::expression(
    //                     Expression::Identifier(Identifier::new("z")),
    //                     (0..0).into()
    //                 ),
    //             ]),
    //             (0..0).into()
    //         )
    //     );

    //     assert_eq!(
    //         parse(
    //             &lex("
    //             {
    //                 1 == 1
    //                 z
    //             }
    //             ")
    //             .unwrap()
    //         )
    //         .unwrap()[0],
    //         Statement::block(
    //             Block::new(vec![
    //                 Statement::expression(
    //                     Expression::Logic(Box::new(Logic::Equal(
    //                         Expression::Value(ValueNode::Integer(1)),
    //                         Expression::Value(ValueNode::Integer(1))
    //                     ))),
    //                     (0..0).into()
    //                 ),
    //                 Statement::expression(
    //                     Expression::Identifier(Identifier::new("z")),
    //                     (0..0).into()
    //                 ),
    //             ]),
    //             (0..0).into()
    //         )
    //     );
    // }

    // #[test]
    // fn identifier() {
    //     assert_eq!(
    //         parse(&lex("x").unwrap()).unwrap()[0],
    //         Statement::expression(Expression::Identifier(Identifier::new("x")), (0..0).into())
    //     );
    //     assert_eq!(
    //         parse(&lex("foobar").unwrap()).unwrap()[0],
    //         Statement::expression(
    //             Expression::Identifier(Identifier::new("foobar")),
    //             (0..0).into()
    //         )
    //     );
    //     assert_eq!(
    //         parse(&lex("HELLO").unwrap()).unwrap()[0],
    //         Statement::expression(
    //             Expression::Identifier(Identifier::new("HELLO")),
    //             (0..0).into()
    //         )
    //     );
    // }

    // #[test]
    // fn assignment() {
    //     assert_eq!(
    //         parse(&lex("foobar = 1").unwrap()).unwrap()[0],
    //         Statement::assignment(
    //             Assignment::new(
    //                 Identifier::new("foobar"),
    //                 None,
    //                 AssignmentOperator::Assign,
    //                 Statement::expression(Expression::Value(ValueNode::Integer(1)), (0..0).into())
    //             ),
    //             (0..0).into()
    //         ),
    //     );
    // }

    // #[test]
    // fn assignment_with_basic_type() {
    //     assert_eq!(
    //         parse(&lex("foobar: int = 1").unwrap()).unwrap()[0],
    //         Statement::assignment(
    //             Assignment::new(
    //                 Identifier::new("foobar"),
    //                 Some(Type::Integer),
    //                 AssignmentOperator::Assign,
    //                 Statement::expression(Expression::Value(ValueNode::Integer(1)), (0..0).into())
    //             ),
    //             (0..0).into()
    //         ),
    //     );
    // }

    // #[test]
    // fn assignment_with_list_types() {
    //     assert_eq!(
    //         parse(&lex("foobar: list = []").unwrap()).unwrap()[0],
    //         Statement::assignment(
    //             Assignment::new(
    //                 Identifier::new("foobar"),
    //                 Some(Type::List),
    //                 AssignmentOperator::Assign,
    //                 Statement::expression(
    //                     Expression::Value(ValueNode::List(vec![])),
    //                     (0..0).into()
    //                 )
    //             ),
    //             (0..0).into()
    //         ),
    //     );

    //     assert_eq!(
    //         parse(&lex("foobar: list(int) = []").unwrap()).unwrap()[0],
    //         Statement::assignment(
    //             Assignment::new(
    //                 Identifier::new("foobar"),
    //                 Some(Type::ListOf(Box::new(Type::Integer))),
    //                 AssignmentOperator::Assign,
    //                 Statement::expression(
    //                     Expression::Value(ValueNode::List(vec![])),
    //                     (0..0).into()
    //                 )
    //             ),
    //             (0..0).into()
    //         ),
    //     );

    //     assert_eq!(
    //         parse(&lex("foobar: [int, str] = [ 42, 'foo' ]").unwrap()).unwrap()[0],
    //         Statement::assignment(
    //             Assignment::new(
    //                 Identifier::new("foobar"),
    //                 Some(Type::ListExact(vec![Type::Integer, Type::String])),
    //                 AssignmentOperator::Assign,
    //                 Statement::expression(
    //                     Expression::Value(ValueNode::List(vec![
    //                         Expression::Value(ValueNode::Integer(42)),
    //                         Expression::Value(ValueNode::String("foo".to_string()))
    //                     ])),
    //                     (0..0).into()
    //                 )
    //             ),
    //             (0..0).into()
    //         ),
    //     );
    // }

    // #[test]
    // fn logic() {
    //     assert_eq!(
    //         parse(&lex("x == 1").unwrap()).unwrap()[0],
    //         Statement::expression(
    //             Expression::Logic(Box::new(Logic::Equal(
    //                 Expression::Identifier(Identifier::new("x")),
    //                 Expression::Value(ValueNode::Integer(1))
    //             ))),
    //             (0..0).into()
    //         )
    //     );

    //     assert_eq!(
    //         parse(&lex("(x == 1) && (y == 2)").unwrap()).unwrap()[0],
    //         Statement::expression(
    //             Expression::Logic(Box::new(Logic::And(
    //                 Expression::Logic(Box::new(Logic::Equal(
    //                     Expression::Identifier(Identifier::new("x")),
    //                     Expression::Value(ValueNode::Integer(1))
    //                 ))),
    //                 Expression::Logic(Box::new(Logic::Equal(
    //                     Expression::Identifier(Identifier::new("y")),
    //                     Expression::Value(ValueNode::Integer(2))
    //                 ))),
    //             ))),
    //             (0..0).into()
    //         )
    //     );

    //     assert_eq!(
    //         parse(&lex("(x == 1) && (y == 2) && true").unwrap()).unwrap()[0],
    //         Statement::expression(
    //             Expression::Logic(Box::new(Logic::And(
    //                 Expression::Logic(Box::new(Logic::And(
    //                     Expression::Logic(Box::new(Logic::Equal(
    //                         Expression::Identifier(Identifier::new("x")),
    //                         Expression::Value(ValueNode::Integer(1))
    //                     ))),
    //                     Expression::Logic(Box::new(Logic::Equal(
    //                         Expression::Identifier(Identifier::new("y")),
    //                         Expression::Value(ValueNode::Integer(2))
    //                     ))),
    //                 ))),
    //                 Expression::Value(ValueNode::Boolean(true))
    //             ))),
    //             (0..0).into()
    //         )
    //     );
    // }

    // #[test]
    // fn list() {
    //     assert_eq!(
    //         parse(&lex("[]").unwrap()).unwrap()[0],
    //         Statement::expression(
    //             Expression::Value(ValueNode::List(Vec::with_capacity(0))),
    //             (0..0).into()
    //         )
    //     );
    //     assert_eq!(
    //         parse(&lex("[42]").unwrap()).unwrap()[0],
    //         Statement::expression(
    //             Expression::Value(ValueNode::List(vec![Expression::Value(
    //                 ValueNode::Integer(42)
    //             )])),
    //             (0..0).into()
    //         )
    //     );
    //     assert_eq!(
    //         parse(&lex("[42, 'foo', 'bar', [1, 2, 3,]]").unwrap()).unwrap()[0],
    //         Statement::expression(
    //             Expression::Value(ValueNode::List(vec![
    //                 Expression::Value(ValueNode::Integer(42)),
    //                 Expression::Value(ValueNode::String("foo".to_string())),
    //                 Expression::Value(ValueNode::String("bar".to_string())),
    //                 Expression::Value(ValueNode::List(vec![
    //                     Expression::Value(ValueNode::Integer(1)),
    //                     Expression::Value(ValueNode::Integer(2)),
    //                     Expression::Value(ValueNode::Integer(3)),
    //                 ]))
    //             ])),
    //             (0..0).into()
    //         )
    //     );
    // }

    // #[test]
    // fn r#true() {
    //     assert_eq!(
    //         parse(&lex("true").unwrap()).unwrap()[0],
    //         Statement::expression(Expression::Value(ValueNode::Boolean(true)), (0..0).into())
    //     );
    // }

    // #[test]
    // fn r#false() {
    //     assert_eq!(
    //         parse(&lex("false").unwrap()).unwrap()[0],
    //         Statement::expression(Expression::Value(ValueNode::Boolean(false)), (0..0).into())
    //     );
    // }

    // #[test]
    // fn positive_float() {
    //     assert_eq!(
    //         parse(&lex("0").unwrap()).unwrap()[0],
    //         Statement::expression(Expression::Value(ValueNode::Float(0.0)), (0..0).into())
    //     );
    //     assert_eq!(
    //         parse(&lex("42").unwrap()).unwrap()[0],
    //         Statement::expression(Expression::Value(ValueNode::Float(42.0)), (0..0).into())
    //     );

    //     let max_float = f64::MAX.to_string() + ".0";

    //     assert_eq!(
    //         parse(&lex(&max_float).unwrap()).unwrap()[0],
    //         Statement::expression(Expression::Value(ValueNode::Float(f64::MAX)), (0..0).into())
    //     );

    //     let min_positive_float = f64::MIN_POSITIVE.to_string();

    //     assert_eq!(
    //         parse(&lex(&min_positive_float).unwrap()).unwrap()[0],
    //         Statement::expression(
    //             Expression::Value(ValueNode::Float(f64::MIN_POSITIVE)),
    //             (0..0).into()
    //         )
    //     );
    // }

    // #[test]
    // fn negative_float() {
    //     assert_eq!(
    //         parse(&lex("-0.0").unwrap()).unwrap()[0],
    //         Statement::expression(Expression::Value(ValueNode::Float(-0.0)), (0..0).into())
    //     );
    //     assert_eq!(
    //         parse(&lex("-42.0").unwrap()).unwrap()[0],
    //         Statement::expression(Expression::Value(ValueNode::Float(-42.0)), (0..0).into())
    //     );

    //     let min_float = f64::MIN.to_string() + ".0";

    //     assert_eq!(
    //         parse(&lex(&min_float).unwrap()).unwrap()[0],
    //         Statement::expression(Expression::Value(ValueNode::Float(f64::MIN)), (0..0).into())
    //     );

    //     let max_negative_float = format!("-{}", f64::MIN_POSITIVE);

    //     assert_eq!(
    //         parse(&lex(&max_negative_float).unwrap()).unwrap()[0],
    //         Statement::expression(
    //             Expression::Value(ValueNode::Float(-f64::MIN_POSITIVE)),
    //             (0..0).into()
    //         )
    //     );
    // }

    // #[test]
    // fn other_float() {
    //     assert_eq!(
    //         parse(&lex("Infinity").unwrap()).unwrap()[0],
    //         Statement::expression(
    //             Expression::Value(ValueNode::Float(f64::INFINITY)),
    //             (0..0).into()
    //         )
    //     );
    //     assert_eq!(
    //         parse(&lex("-Infinity").unwrap()).unwrap()[0],
    //         Statement::expression(
    //             Expression::Value(ValueNode::Float(f64::NEG_INFINITY)),
    //             (0..0).into()
    //         )
    //     );

    //     if let StatementInner::Expression(Expression::Value(ValueNode::Float(float))) =
    //         &parse(&lex("NaN").unwrap()).unwrap()[0].inner
    //     {
    //         assert!(float.is_nan());
    //     } else {
    //         panic!("Expected a float.");
    //     }
    // }

    // #[test]
    // fn positive_integer() {
    //     for i in 0..10 {
    //         let source = i.to_string();
    //         let tokens = lex(&source).unwrap();
    //         let statements = parse(&tokens).unwrap();

    //         assert_eq!(
    //             statements[0],
    //             Statement::expression(Expression::Value(ValueNode::Integer(i)), (0..0).into())
    //         )
    //     }

    //     assert_eq!(
    //         parse(&lex("42").unwrap()).unwrap()[0],
    //         Statement::expression(Expression::Value(ValueNode::Integer(42)), (0..0).into())
    //     );

    //     let maximum_integer = i64::MAX.to_string();

    //     assert_eq!(
    //         parse(&lex(&maximum_integer).unwrap()).unwrap()[0],
    //         Statement::expression(
    //             Expression::Value(ValueNode::Integer(i64::MAX)),
    //             (0..0).into()
    //         )
    //     );
    // }

    // #[test]
    // fn negative_integer() {
    //     for i in -9..1 {
    //         let source = i.to_string();
    //         let tokens = lex(&source).unwrap();
    //         let statements = parse(&tokens).unwrap();

    //         assert_eq!(
    //             statements[0],
    //             Statement::expression(Expression::Value(ValueNode::Integer(i)), (0..0).into())
    //         )
    //     }

    //     assert_eq!(
    //         parse(&lex("-42").unwrap()).unwrap()[0],
    //         Statement::expression(Expression::Value(ValueNode::Integer(-42)), (0..0).into())
    //     );

    //     let minimum_integer = i64::MIN.to_string();

    //     assert_eq!(
    //         parse(&lex(&minimum_integer).unwrap()).unwrap()[0],
    //         Statement::expression(
    //             Expression::Value(ValueNode::Integer(i64::MIN)),
    //             (0..0).into()
    //         )
    //     );
    // }

    // #[test]
    // fn double_quoted_string() {
    //     assert_eq!(
    //         parse(&lex("\"\"").unwrap()).unwrap()[0],
    //         Statement::expression(
    //             Expression::Value(ValueNode::String("".to_string())),
    //             (0..0).into()
    //         )
    //     );
    //     assert_eq!(
    //         parse(&lex("\"42\"").unwrap()).unwrap()[0],
    //         Statement::expression(
    //             Expression::Value(ValueNode::String("42".to_string())),
    //             (0..0).into()
    //         )
    //     );
    //     assert_eq!(
    //         parse(&lex("\"foobar\"").unwrap()).unwrap()[0],
    //         Statement::expression(
    //             Expression::Value(ValueNode::String("foobar".to_string())),
    //             (0..0).into()
    //         )
    //     );
    // }

    // #[test]
    // fn single_quoted_string() {
    //     assert_eq!(
    //         parse(&lex("''").unwrap()).unwrap()[0],
    //         Statement::expression(
    //             Expression::Value(ValueNode::String("".to_string())),
    //             (0..0).into()
    //         )
    //     );
    //     assert_eq!(
    //         parse(&lex("'42'").unwrap()).unwrap()[0],
    //         Statement::expression(
    //             Expression::Value(ValueNode::String("42".to_string())),
    //             (0..0).into()
    //         )
    //     );
    //     assert_eq!(
    //         parse(&lex("'foobar'").unwrap()).unwrap()[0],
    //         Statement::expression(
    //             Expression::Value(ValueNode::String("foobar".to_string())),
    //             (0..0).into()
    //         )
    //     );
    // }

    // #[test]
    // fn grave_quoted_string() {
    //     assert_eq!(
    //         parse(&lex("``").unwrap()).unwrap()[0],
    //         Statement::expression(
    //             Expression::Value(ValueNode::String("".to_string())),
    //             (0..0).into()
    //         )
    //     );
    //     assert_eq!(
    //         parse(&lex("`42`").unwrap()).unwrap()[0],
    //         Statement::expression(
    //             Expression::Value(ValueNode::String("42".to_string())),
    //             (0..0).into()
    //         )
    //     );
    //     assert_eq!(
    //         parse(&lex("`foobar`").unwrap()).unwrap()[0],
    //         Statement::expression(
    //             Expression::Value(ValueNode::String("foobar".to_string())),
    //             (0..0).into()
    //         )
    //     );
    // }
}
