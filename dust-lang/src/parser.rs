use std::{cell::RefCell, collections::HashMap, rc::Rc};

use chumsky::{input::SpannedInput, pratt::*, prelude::*};

use crate::{
    abstract_tree::*,
    error::Error,
    identifier::Identifier,
    lexer::{Control, Keyword, Operator, Token},
};

pub type ParserInput<'src> =
    SpannedInput<Token<'src>, SimpleSpan, &'src [(Token<'src>, SimpleSpan)]>;

pub fn parse<'src>(tokens: &'src [(Token<'src>, SimpleSpan)]) -> Result<AbstractTree, Vec<Error>> {
    let statements = parser()
        .parse(tokens.spanned((tokens.len()..tokens.len()).into()))
        .into_result()
        .map_err(|errors| {
            errors
                .into_iter()
                .map(|error| Error::from(error))
                .collect::<Vec<Error>>()
        })?;

    Ok(AbstractTree::new(statements))
}

pub fn parser<'src>(
) -> impl Parser<'src, ParserInput<'src>, Vec<Statement>, extra::Err<Rich<'src, Token<'src>, SimpleSpan>>>
{
    let identifiers: RefCell<HashMap<&str, Identifier>> = RefCell::new(HashMap::new());
    let _custom_types: Rc<RefCell<HashMap<Identifier, Type>>> =
        Rc::new(RefCell::new(HashMap::new()));
    let custom_types = (_custom_types.clone(), _custom_types.clone());

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

    let positioned_identifier = identifier
        .clone()
        .map_with(|identifier, state| identifier.with_position(state.span()));

    let basic_value = select! {
        Token::Boolean(boolean) => ValueNode::Boolean(boolean),
        Token::Float(float) => ValueNode::Float(float),
        Token::Integer(integer) => ValueNode::Integer(integer),
        Token::String(string) => ValueNode::String(string.to_string()),
    }
    .map_with(|value, state| Expression::Value(value.with_position(state.span())));

    let r#type = recursive(|r#type| {
        let function_type = just(Token::Keyword(Keyword::Fn))
            .ignore_then(
                r#type
                    .clone()
                    .separated_by(just(Token::Control(Control::Comma)))
                    .collect()
                    .delimited_by(
                        just(Token::Control(Control::ParenOpen)),
                        just(Token::Control(Control::ParenClose)),
                    ),
            )
            .then_ignore(just(Token::Control(Control::SkinnyArrow)))
            .then(r#type.clone())
            .map(|(parameter_types, return_type)| Type::Function {
                parameter_types,
                return_type: Box::new(return_type),
            });

        let list_of = just(Token::Keyword(Keyword::List))
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
            just(Token::Keyword(Keyword::Any)).to(Type::Any),
            just(Token::Keyword(Keyword::Bool)).to(Type::Boolean),
            just(Token::Keyword(Keyword::Float)).to(Type::Float),
            just(Token::Keyword(Keyword::Int)).to(Type::Integer),
            just(Token::Keyword(Keyword::None)).to(Type::None),
            just(Token::Keyword(Keyword::Range)).to(Type::Range),
            just(Token::Keyword(Keyword::Str)).to(Type::String),
            just(Token::Keyword(Keyword::List)).to(Type::List),
            identifier.clone().map(move |identifier| {
                if let Some(r#type) = custom_types.0.borrow().get(&identifier) {
                    r#type.clone()
                } else {
                    Type::Argument(identifier)
                }
            }),
        ))
        .map_with(|r#type, state| r#type.with_position(state.span()))
    });

    let type_argument = identifier
        .clone()
        .map_with(|identifier, state| Type::Argument(identifier).with_position(state.span()));

    let type_specification = just(Token::Control(Control::Colon)).ignore_then(r#type.clone());

    let structure_field_definition = identifier.clone().then(type_specification.clone());

    let structure_definition = just(Token::Keyword(Keyword::Struct))
        .ignore_then(identifier.clone())
        .then(
            structure_field_definition
                .separated_by(just(Token::Control(Control::Comma)))
                .allow_trailing()
                .collect::<Vec<(Identifier, WithPosition<Type>)>>()
                .delimited_by(
                    just(Token::Control(Control::CurlyOpen)),
                    just(Token::Control(Control::CurlyClose)),
                ),
        )
        .map_with(move |(name, fields), state| {
            let definition = StructureDefinition::new(name.clone(), fields.clone());
            let r#type = Type::Structure {
                name: name.clone(),
                fields,
            };

            custom_types.1.borrow_mut().insert(name, r#type);

            Statement::StructureDefinition(definition.with_position(state.span()))
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
            .map_with(|statements, state| Block::new(statements).with_position(state.span()));

        let expression = recursive(|expression| {
            let identifier_expression = identifier.clone().map_with(|identifier, state| {
                Expression::Identifier(identifier.with_position(state.span()))
            });

            let raw_integer = select! {
                Token::Integer(integer) => integer
            };

            let range = raw_integer
                .clone()
                .then_ignore(just(Token::Control(Control::DoubleDot)))
                .then(raw_integer)
                .map_with(|(start, end), state| {
                    Expression::Value(ValueNode::Range(start..end).with_position(state.span()))
                });

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
                    Expression::Value(ValueNode::List(list).with_position(state.span()))
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
                    Expression::Value(
                        ValueNode::Map(map_assigment_list).with_position(state.span()),
                    )
                });

            let type_arguments = type_argument
                .clone()
                .separated_by(just(Token::Control(Control::Comma)))
                .at_least(1)
                .collect()
                .delimited_by(
                    just(Token::Control(Control::ParenOpen)),
                    just(Token::Control(Control::ParenClose)),
                );

            let parsed_function = type_arguments
                .or_not()
                .then(
                    identifier
                        .clone()
                        .then(type_specification.clone())
                        .separated_by(just(Token::Control(Control::Comma)))
                        .collect()
                        .delimited_by(
                            just(Token::Control(Control::ParenOpen)),
                            just(Token::Control(Control::ParenClose)),
                        )
                        .then(r#type.clone())
                        .then(block.clone()),
                )
                .map_with(
                    |(type_arguments, ((parameters, return_type), body)), state| {
                        Expression::Value(
                            ValueNode::ParsedFunction {
                                type_arguments: type_arguments
                                    .unwrap_or_else(|| Vec::with_capacity(0)),
                                parameters,
                                return_type,
                                body,
                            }
                            .with_position(state.span()),
                        )
                    },
                );

            let built_in_function = {
                select! {
                    Token::BuiltInFunction(built_in_function) => built_in_function,
                }
            }
            .map_with(|built_in_function, state| {
                Expression::Value(
                    ValueNode::BuiltInFunction(built_in_function).with_position(state.span()),
                )
            });

            use Operator::*;

            let structure_field = identifier
                .clone()
                .then_ignore(just(Token::Operator(Operator::Assign)))
                .then(expression.clone());

            let structure_instance = positioned_identifier
                .clone()
                .then(
                    structure_field
                        .separated_by(just(Token::Control(Control::Comma)))
                        .allow_trailing()
                        .collect()
                        .delimited_by(
                            just(Token::Control(Control::CurlyOpen)),
                            just(Token::Control(Control::CurlyClose)),
                        ),
                )
                .map_with(|(name, fields), state| {
                    Expression::Value(
                        ValueNode::Structure { name, fields }.with_position(state.span()),
                    )
                });

            let turbofish = r#type
                .clone()
                .separated_by(just(Token::Control(Control::Comma)))
                .at_least(1)
                .collect()
                .delimited_by(
                    just(Token::Control(Control::ParenOpen)),
                    just(Token::Control(Control::ParenClose)),
                )
                .delimited_by(
                    just(Token::Control(Control::DoubleColon)),
                    just(Token::Control(Control::DoubleColon)),
                );

            let atom = choice((
                range.clone(),
                structure_instance.clone(),
                parsed_function.clone(),
                built_in_function.clone(),
                list.clone(),
                map.clone(),
                basic_value.clone(),
                identifier_expression.clone(),
                expression.clone().delimited_by(
                    just(Token::Control(Control::ParenOpen)),
                    just(Token::Control(Control::ParenClose)),
                ),
            ));

            let logic_math_indexes_and_function_calls = atom.pratt((
                prefix(2, just(Token::Operator(Not)), |_, expression, span| {
                    Expression::Logic(Box::new(Logic::Not(expression)).with_position(span))
                }),
                postfix(
                    2,
                    expression.clone().delimited_by(
                        just(Token::Control(Control::SquareOpen)),
                        just(Token::Control(Control::SquareClose)),
                    ),
                    |left, right, span| {
                        Expression::ListIndex(
                            Box::new(ListIndex::new(left, right)).with_position(span),
                        )
                    },
                ),
                postfix(
                    3,
                    turbofish.clone().or_not().then(
                        expression
                            .clone()
                            .separated_by(just(Token::Control(Control::Comma)))
                            .collect()
                            .delimited_by(
                                just(Token::Control(Control::ParenOpen)),
                                just(Token::Control(Control::ParenClose)),
                            ),
                    ),
                    |function_expression,
                     (type_arguments, arguments): (
                        Option<Vec<WithPosition<Type>>>,
                        Vec<Expression>,
                    ),
                     span| {
                        Expression::FunctionCall(
                            FunctionCall::new(
                                function_expression,
                                type_arguments.unwrap_or_else(|| Vec::with_capacity(0)),
                                arguments,
                            )
                            .with_position(span),
                        )
                    },
                ),
                infix(
                    left(4),
                    just(Token::Control(Control::Dot)),
                    |left, _, right, span| {
                        Expression::MapIndex(
                            Box::new(MapIndex::new(left, right)).with_position(span),
                        )
                    },
                ),
                infix(
                    left(1),
                    just(Token::Operator(Equal)),
                    |left, _, right, span| {
                        Expression::Logic(Box::new(Logic::Equal(left, right)).with_position(span))
                    },
                ),
                infix(
                    left(1),
                    just(Token::Operator(NotEqual)),
                    |left, _, right, span| {
                        Expression::Logic(
                            Box::new(Logic::NotEqual(left, right)).with_position(span),
                        )
                    },
                ),
                infix(
                    left(1),
                    just(Token::Operator(Greater)),
                    |left, _, right, span| {
                        Expression::Logic(Box::new(Logic::Greater(left, right)).with_position(span))
                    },
                ),
                infix(
                    left(1),
                    just(Token::Operator(Less)),
                    |left, _, right, span| {
                        Expression::Logic(Box::new(Logic::Less(left, right)).with_position(span))
                    },
                ),
                infix(
                    left(1),
                    just(Token::Operator(GreaterOrEqual)),
                    |left, _, right, span| {
                        Expression::Logic(
                            Box::new(Logic::GreaterOrEqual(left, right)).with_position(span),
                        )
                    },
                ),
                infix(
                    left(1),
                    just(Token::Operator(LessOrEqual)),
                    |left, _, right, span| {
                        Expression::Logic(
                            Box::new(Logic::LessOrEqual(left, right)).with_position(span),
                        )
                    },
                ),
                infix(
                    left(1),
                    just(Token::Operator(And)),
                    |left, _, right, span| {
                        Expression::Logic(Box::new(Logic::And(left, right)).with_position(span))
                    },
                ),
                infix(
                    left(1),
                    just(Token::Operator(Or)),
                    |left, _, right, span| {
                        Expression::Logic(Box::new(Logic::Or(left, right)).with_position(span))
                    },
                ),
                infix(
                    left(1),
                    just(Token::Operator(Add)),
                    |left, _, right, span| {
                        Expression::Math(Box::new(Math::Add(left, right)).with_position(span))
                    },
                ),
                infix(
                    left(1),
                    just(Token::Operator(Subtract)),
                    |left, _, right, span| {
                        Expression::Math(Box::new(Math::Subtract(left, right)).with_position(span))
                    },
                ),
                infix(
                    left(2),
                    just(Token::Operator(Multiply)),
                    |left, _, right, span| {
                        Expression::Math(Box::new(Math::Multiply(left, right)).with_position(span))
                    },
                ),
                infix(
                    left(2),
                    just(Token::Operator(Divide)),
                    |left, _, right, span| {
                        Expression::Math(Box::new(Math::Divide(left, right)).with_position(span))
                    },
                ),
                infix(
                    left(1),
                    just(Token::Operator(Modulo)),
                    |left, _, right, span| {
                        Expression::Math(Box::new(Math::Modulo(left, right)).with_position(span))
                    },
                ),
            ));

            choice((
                logic_math_indexes_and_function_calls,
                range,
                structure_instance,
                parsed_function,
                built_in_function,
                list,
                map,
                basic_value,
                identifier_expression,
            ))
        });

        let expression_statement = expression
            .clone()
            .map(|expression| Statement::Expression(expression));

        let async_block = just(Token::Keyword(Keyword::Async))
            .ignore_then(statement.clone().repeated().collect().delimited_by(
                just(Token::Control(Control::CurlyOpen)),
                just(Token::Control(Control::CurlyClose)),
            ))
            .map_with(|statements, state| {
                Statement::AsyncBlock(AsyncBlock::new(statements).with_position(state.span()))
            });

        let r#break = just(Token::Keyword(Keyword::Break))
            .map_with(|_, state| Statement::Break(().with_position(state.span())));

        let assignment = positioned_identifier
            .clone()
            .then(type_specification.clone().or_not())
            .then(choice((
                just(Token::Operator(Operator::Assign)).to(AssignmentOperator::Assign),
                just(Token::Operator(Operator::AddAssign)).to(AssignmentOperator::AddAssign),
                just(Token::Operator(Operator::SubAssign)).to(AssignmentOperator::SubAssign),
            )))
            .then(statement.clone())
            .map_with(|(((identifier, r#type), operator), statement), state| {
                Statement::Assignment(
                    Assignment::new(identifier, r#type, operator, statement)
                        .with_position(state.span()),
                )
            });

        let block_statement = block.clone().map(|block| Statement::Block(block));

        let r#loop = statement
            .clone()
            .repeated()
            .at_least(1)
            .collect()
            .delimited_by(
                just(Token::Keyword(Keyword::Loop)).then(just(Token::Control(Control::CurlyOpen))),
                just(Token::Control(Control::CurlyClose)),
            )
            .map_with(|statements, state| {
                Statement::Loop(Loop::new(statements).with_position(state.span()))
            });

        let r#while = just(Token::Keyword(Keyword::While))
            .ignore_then(expression.clone())
            .then(statement.clone().repeated().collect().delimited_by(
                just(Token::Control(Control::CurlyOpen)),
                just(Token::Control(Control::CurlyClose)),
            ))
            .map_with(|(expression, statements), state| {
                Statement::While(While::new(expression, statements).with_position(state.span()))
            });

        let if_else = just(Token::Keyword(Keyword::If))
            .ignore_then(expression.clone())
            .then(block.clone())
            .then(
                just(Token::Keyword(Keyword::Else))
                    .ignore_then(just(Token::Keyword(Keyword::If)))
                    .ignore_then(expression.clone())
                    .then(block.clone())
                    .repeated()
                    .collect(),
            )
            .then(
                just(Token::Keyword(Keyword::Else))
                    .ignore_then(block.clone())
                    .or_not(),
            )
            .map_with(
                |(((if_expression, if_block), else_ifs), else_block), state| {
                    Statement::IfElse(
                        IfElse::new(if_expression, if_block, else_ifs, else_block)
                            .with_position(state.span()),
                    )
                },
            );

        choice((
            async_block,
            structure_definition,
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

    statement.clone().repeated().collect()
}

#[cfg(test)]
mod tests {
    use crate::{built_in_functions::BuiltInFunction, lexer::lex};

    use super::*;

    #[test]
    fn built_in_function() {
        assert_eq!(
            parse(&lex("__READ_LINE__").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::BuiltInFunction(BuiltInFunction::ReadLine).with_position((0, 13))
            ))
        )
    }

    #[test]
    fn async_block() {
        assert_eq!(
            parse(
                &lex("
                    async {
                        1
                        2
                        3
                    }
                ")
                .unwrap()
            )
            .unwrap()[0],
            Statement::AsyncBlock(
                AsyncBlock::new(vec![
                    Statement::Expression(Expression::Value(
                        ValueNode::Integer(1).with_position((53, 54))
                    )),
                    Statement::Expression(Expression::Value(
                        ValueNode::Integer(2).with_position((79, 80))
                    )),
                    Statement::Expression(Expression::Value(
                        ValueNode::Integer(3).with_position((105, 106))
                    )),
                ])
                .with_position((21, 128))
            )
        )
    }

    #[test]
    fn structure_instance() {
        assert_eq!(
            parse(
                &lex("
                    Foo {
                        bar = 42,
                        baz = 'hiya',
                    }
                ")
                .unwrap()
            )
            .unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::Structure {
                    name: Identifier::new("Foo").with_position((21, 24)),
                    fields: vec![
                        (
                            Identifier::new("bar"),
                            Expression::Value(ValueNode::Integer(42).with_position((57, 59)))
                        ),
                        (
                            Identifier::new("baz"),
                            Expression::Value(
                                ValueNode::String("hiya".to_string()).with_position((91, 97))
                            )
                        ),
                    ]
                }
                .with_position((21, 120))
            ))
        )
    }

    #[test]
    fn structure_definition() {
        assert_eq!(
            parse(
                &lex("
                    struct Foo {
                        bar : int,
                        baz : str,
                    }
                ")
                .unwrap()
            )
            .unwrap()[0],
            Statement::StructureDefinition(
                StructureDefinition::new(
                    Identifier::new("Foo"),
                    vec![
                        (
                            Identifier::new("bar"),
                            Type::Integer.with_position((64, 67))
                        ),
                        (
                            Identifier::new("baz"),
                            Type::String.with_position((99, 102))
                        ),
                    ]
                )
                .with_position((21, 125))
            )
        )
    }

    #[test]
    fn map_index() {
        assert_eq!(
            parse(&lex("{ x = 42 }.x").unwrap()).unwrap()[0],
            Statement::Expression(Expression::MapIndex(
                Box::new(MapIndex::new(
                    Expression::Value(
                        ValueNode::Map(vec![(
                            Identifier::new("x"),
                            None,
                            Expression::Value(ValueNode::Integer(42).with_position((6, 8)))
                        )])
                        .with_position((0, 10))
                    ),
                    Expression::Identifier(Identifier::new("x").with_position((11, 12)))
                ))
                .with_position((0, 12))
            ))
        );
        assert_eq!(
            parse(&lex("foo.x").unwrap()).unwrap()[0],
            Statement::Expression(Expression::MapIndex(
                Box::new(MapIndex::new(
                    Expression::Identifier(Identifier::new("foo").with_position((0, 3))),
                    Expression::Identifier(Identifier::new("x").with_position((4, 5)))
                ))
                .with_position((0, 5))
            ))
        );
    }

    #[test]
    fn r#while() {
        assert_eq!(
            parse(&lex("while true { output('hi') }").unwrap()).unwrap()[0],
            Statement::While(
                While::new(
                    Expression::Value(ValueNode::Boolean(true).with_position((6, 10))),
                    vec![Statement::Expression(Expression::FunctionCall(
                        FunctionCall::new(
                            Expression::Identifier(
                                Identifier::new("output").with_position((13, 19))
                            ),
                            Vec::with_capacity(0),
                            vec![Expression::Value(
                                ValueNode::String("hi".to_string()).with_position((20, 24))
                            )]
                        )
                        .with_position((13, 25))
                    ))]
                )
                .with_position((0, 27))
            )
        )
    }

    #[test]
    fn boolean_type() {
        assert_eq!(
            parse(&lex("foobar : bool = true").unwrap()).unwrap()[0],
            Statement::Assignment(
                Assignment::new(
                    Identifier::new("foobar").with_position((0, 6)),
                    Some(Type::Boolean.with_position((9, 13))),
                    AssignmentOperator::Assign,
                    Statement::Expression(Expression::Value(
                        ValueNode::Boolean(true).with_position((16, 20))
                    ))
                )
                .with_position((0, 20))
            )
        );
    }

    #[test]
    fn list_type() {
        assert_eq!(
            parse(&lex("foobar: list = []").unwrap()).unwrap()[0],
            Statement::Assignment(
                Assignment::new(
                    Identifier::new("foobar").with_position((0, 6)),
                    Some(Type::List.with_position((8, 12))),
                    AssignmentOperator::Assign,
                    Statement::Expression(Expression::Value(
                        ValueNode::List(vec![]).with_position((15, 17))
                    ))
                )
                .with_position((0, 17))
            )
        );
    }

    #[test]
    fn list_of_type() {
        assert_eq!(
            parse(&lex("foobar : list(bool) = [true]").unwrap()).unwrap()[0],
            Statement::Assignment(
                Assignment::new(
                    Identifier::new("foobar").with_position((0, 6)),
                    Some(
                        Type::ListOf(Box::new(Type::Boolean.with_position((14, 18))))
                            .with_position((9, 19))
                    ),
                    AssignmentOperator::Assign,
                    Statement::Expression(Expression::Value(
                        ValueNode::List(vec![Expression::Value(
                            ValueNode::Boolean(true).with_position((23, 27))
                        )])
                        .with_position((22, 28))
                    ))
                )
                .with_position((0, 28))
            )
        );
    }

    #[test]
    fn list_exact_type() {
        assert_eq!(
            parse(&lex("foobar : [bool, str] = [true, '42']").unwrap()).unwrap()[0],
            Statement::Assignment(
                Assignment::new(
                    Identifier::new("foobar").with_position((0, 6)),
                    Some(
                        Type::ListExact(vec![
                            Type::Boolean.with_position((10, 14)),
                            Type::String.with_position((16, 19))
                        ])
                        .with_position((9, 20))
                    ),
                    AssignmentOperator::Assign,
                    Statement::Expression(Expression::Value(
                        ValueNode::List(vec![
                            Expression::Value(ValueNode::Boolean(true).with_position((24, 28))),
                            Expression::Value(
                                ValueNode::String("42".to_string()).with_position((30, 34))
                            )
                        ])
                        .with_position((23, 35))
                    ))
                )
                .with_position((0, 35))
            )
        );
    }

    #[test]
    fn function_type() {
        assert_eq!(
            parse(&lex("foobar : fn() -> any = some_function").unwrap()).unwrap()[0],
            Statement::Assignment(
                Assignment::new(
                    Identifier::new("foobar").with_position((0, 6)),
                    Some(
                        Type::Function {
                            parameter_types: vec![],
                            return_type: Box::new(Type::Any.with_position((17, 20)))
                        }
                        .with_position((9, 20))
                    ),
                    AssignmentOperator::Assign,
                    Statement::Expression(Expression::Identifier(
                        Identifier::new("some_function").with_position((23, 36))
                    ))
                )
                .with_position((0, 36))
            )
        );
    }

    #[test]
    fn function_call() {
        assert_eq!(
            parse(&lex("io.read_line()").unwrap()).unwrap()[0],
            Statement::Expression(Expression::FunctionCall(
                FunctionCall::new(
                    Expression::MapIndex(
                        Box::new(MapIndex::new(
                            Expression::Identifier(Identifier::new("io").with_position((0, 2))),
                            Expression::Identifier(
                                Identifier::new("read_line").with_position((3, 12))
                            )
                        ))
                        .with_position((0, 12))
                    ),
                    Vec::with_capacity(0),
                    Vec::with_capacity(0),
                )
                .with_position((0, 14))
            ))
        )
    }

    #[test]
    fn function_call_with_type_arguments() {
        assert_eq!(
            parse(&lex("foobar::(str)::('hi')").unwrap()).unwrap()[0],
            Statement::Expression(Expression::FunctionCall(
                FunctionCall::new(
                    Expression::Identifier(Identifier::new("foobar").with_position((0, 6))),
                    vec![Type::String.with_position((9, 12))],
                    vec![Expression::Value(
                        ValueNode::String("hi".to_string()).with_position((16, 20))
                    )],
                )
                .with_position((0, 21))
            ))
        )
    }

    #[test]
    fn range() {
        assert_eq!(
            parse(&lex("1..10").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::Range(1..10).with_position((0, 5))
            ))
        )
    }

    #[test]
    fn function() {
        assert_eq!(
            parse(&lex("(x: int) int { x }").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::ParsedFunction {
                    type_arguments: Vec::with_capacity(0),
                    parameters: vec![(Identifier::new("x"), Type::Integer.with_position((4, 7)))],
                    return_type: Type::Integer.with_position((9, 12)),
                    body: Block::new(vec![Statement::Expression(Expression::Identifier(
                        Identifier::new("x").with_position((15, 16))
                    ))])
                    .with_position((13, 18)),
                }
                .with_position((0, 18))
            ),)
        )
    }

    #[test]
    fn function_with_type_arguments() {
        assert_eq!(
            parse(&lex("(T, U)(x: T, y: U) T { x }").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::ParsedFunction {
                    type_arguments: vec![
                        Type::Argument(Identifier::new("T")).with_position((1, 2)),
                        Type::Argument(Identifier::new("U")).with_position((4, 5)),
                    ],
                    parameters: vec![
                        (
                            Identifier::new("x"),
                            Type::Argument(Identifier::new("T")).with_position((10, 11))
                        ),
                        (
                            Identifier::new("y"),
                            Type::Argument(Identifier::new("U")).with_position((16, 17))
                        )
                    ],
                    return_type: Type::Argument(Identifier::new("T")).with_position((19, 20)),
                    body: Block::new(vec![Statement::Expression(Expression::Identifier(
                        Identifier::new("x").with_position((23, 24))
                    ))])
                    .with_position((21, 26)),
                }
                .with_position((0, 26))
            ))
        )
    }

    #[test]
    fn r#if() {
        assert_eq!(
            parse(&lex("if true { 'foo' }").unwrap()).unwrap()[0],
            Statement::IfElse(
                IfElse::new(
                    Expression::Value(ValueNode::Boolean(true).with_position((3, 7))),
                    Block::new(vec![Statement::Expression(Expression::Value(
                        ValueNode::String("foo".to_string()).with_position((10, 15))
                    ))])
                    .with_position((8, 17)),
                    Vec::with_capacity(0),
                    None
                )
                .with_position((0, 17))
            )
        );
    }

    #[test]
    fn if_else() {
        assert_eq!(
            parse(&lex("if true {'foo' } else { 'bar' }").unwrap()).unwrap()[0],
            Statement::IfElse(
                IfElse::new(
                    Expression::Value(ValueNode::Boolean(true).with_position((3, 7))),
                    Block::new(vec![Statement::Expression(Expression::Value(
                        ValueNode::String("foo".to_string()).with_position((9, 14))
                    ))])
                    .with_position((8, 16)),
                    Vec::with_capacity(0),
                    Some(
                        Block::new(vec![Statement::Expression(Expression::Value(
                            ValueNode::String("bar".to_string()).with_position((24, 29))
                        ))])
                        .with_position((22, 31))
                    )
                )
                .with_position((0, 31)),
            )
        )
    }

    #[test]
    fn map() {
        assert_eq!(
            parse(&lex("{ foo = 'bar' }").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::Map(vec![(
                    Identifier::new("foo"),
                    None,
                    Expression::Value(ValueNode::String("bar".to_string()).with_position((8, 13)))
                )])
                .with_position((0, 15))
            ),)
        );
        assert_eq!(
            parse(&lex("{ x = 1, y = 2, }").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::Map(vec![
                    (
                        Identifier::new("x"),
                        None,
                        Expression::Value(ValueNode::Integer(1).with_position((6, 7)))
                    ),
                    (
                        Identifier::new("y"),
                        None,
                        Expression::Value(ValueNode::Integer(2).with_position((13, 14)))
                    ),
                ])
                .with_position((0, 17))
            ),)
        );
        assert_eq!(
            parse(&lex("{ x = 1 y = 2 }").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::Map(vec![
                    (
                        Identifier::new("x"),
                        None,
                        Expression::Value(ValueNode::Integer(1).with_position((6, 7)))
                    ),
                    (
                        Identifier::new("y"),
                        None,
                        Expression::Value(ValueNode::Integer(2).with_position((12, 13)))
                    ),
                ])
                .with_position((0, 15))
            ),)
        );
    }

    #[test]
    fn math() {
        assert_eq!(
            parse(&lex("1 + 1").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Math(
                Box::new(Math::Add(
                    Expression::Value(ValueNode::Integer(1).with_position((0, 1))),
                    Expression::Value(ValueNode::Integer(1).with_position((4, 5)))
                ))
                .with_position((0, 5))
            ))
        );
    }

    #[test]
    fn r#loop() {
        assert_eq!(
            parse(&lex("loop { 42 }").unwrap()).unwrap()[0],
            Statement::Loop(
                Loop::new(vec![Statement::Expression(Expression::Value(
                    ValueNode::Integer(42).with_position((7, 9))
                ))])
                .with_position((0, 11))
            )
        );
        assert_eq!(
            parse(&lex("loop { if i > 2 { break } else { i += 1 } }").unwrap()).unwrap()[0],
            Statement::Loop(
                Loop::new(vec![Statement::IfElse(
                    IfElse::new(
                        Expression::Logic(
                            Box::new(Logic::Greater(
                                Expression::Identifier(
                                    Identifier::new("i").with_position((10, 11))
                                ),
                                Expression::Value(ValueNode::Integer(2).with_position((14, 15)))
                            ))
                            .with_position((10, 15))
                        ),
                        Block::new(vec![Statement::Break(().with_position((18, 23)))])
                            .with_position((16, 25)),
                        Vec::with_capacity(0),
                        Some(
                            Block::new(vec![Statement::Assignment(
                                Assignment::new(
                                    Identifier::new("i").with_position((33, 34)),
                                    None,
                                    AssignmentOperator::AddAssign,
                                    Statement::Expression(Expression::Value(
                                        ValueNode::Integer(1).with_position((38, 39))
                                    ))
                                )
                                .with_position((33, 39))
                            )])
                            .with_position((31, 41))
                        )
                    )
                    .with_position((7, 41))
                )])
                .with_position((0, 43))
            )
        );
    }

    #[test]
    fn block() {
        assert_eq!(
            parse(&lex("{ x }").unwrap()).unwrap()[0],
            Statement::Block(
                Block::new(vec![Statement::Expression(Expression::Identifier(
                    Identifier::new("x").with_position((2, 3))
                ),)])
                .with_position((0, 5))
            )
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
            .unwrap()[0],
            Statement::Block(
                Block::new(vec![
                    Statement::Expression(Expression::Identifier(
                        Identifier::new("x").with_position((39, 40))
                    )),
                    Statement::Expression(Expression::Identifier(
                        Identifier::new("y").with_position((62, 63))
                    )),
                    Statement::Expression(Expression::Identifier(
                        Identifier::new("z").with_position((85, 86))
                    )),
                ])
                .with_position((17, 104)),
            )
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
            .unwrap()[0],
            Statement::Block(
                Block::new(vec![
                    Statement::Expression(Expression::Logic(
                        Box::new(Logic::Equal(
                            Expression::Value(ValueNode::Integer(1).with_position((39, 40))),
                            Expression::Value(ValueNode::Integer(1).with_position((44, 45)))
                        ))
                        .with_position((39, 45))
                    )),
                    Statement::Expression(Expression::Identifier(
                        Identifier::new("z").with_position((66, 67))
                    )),
                ])
                .with_position((17, 85)),
            )
        );
    }

    #[test]
    fn identifier() {
        assert_eq!(
            parse(&lex("x").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Identifier(
                Identifier::new("x").with_position((0, 1))
            ))
        );
        assert_eq!(
            parse(&lex("foobar").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Identifier(
                Identifier::new("foobar").with_position((0, 6))
            ))
        );
        assert_eq!(
            parse(&lex("HELLO").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Identifier(
                Identifier::new("HELLO").with_position((0, 5))
            ))
        );
    }

    #[test]
    fn assignment() {
        assert_eq!(
            parse(&lex("foobar = 1").unwrap()).unwrap()[0],
            Statement::Assignment(
                Assignment::new(
                    Identifier::new("foobar").with_position((0, 6)),
                    None,
                    AssignmentOperator::Assign,
                    Statement::Expression(Expression::Value(
                        ValueNode::Integer(1).with_position((9, 10))
                    ))
                )
                .with_position((0, 10)),
            )
        );
    }

    #[test]
    fn assignment_with_type() {
        assert_eq!(
            parse(&lex("foobar: int = 1").unwrap()).unwrap()[0],
            Statement::Assignment(
                Assignment::new(
                    Identifier::new("foobar").with_position((0, 6)),
                    Some(Type::Integer.with_position((8, 11))),
                    AssignmentOperator::Assign,
                    Statement::Expression(Expression::Value(
                        ValueNode::Integer(1).with_position((14, 15))
                    ))
                )
                .with_position((0, 15)),
            )
        );
    }

    #[test]
    fn logic() {
        assert_eq!(
            parse(&lex("x == 1").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Logic(
                Box::new(Logic::Equal(
                    Expression::Identifier(Identifier::new("x").with_position((0, 1))),
                    Expression::Value(ValueNode::Integer(1).with_position((5, 6))),
                ))
                .with_position((0, 6))
            ))
        );

        assert_eq!(
            parse(&lex("(x == 1) && (y == 2)").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Logic(
                Box::new(Logic::And(
                    Expression::Logic(
                        Box::new(Logic::Equal(
                            Expression::Identifier(Identifier::new("x").with_position((1, 2))),
                            Expression::Value(ValueNode::Integer(1).with_position((6, 7))),
                        ))
                        .with_position((1, 7))
                    ),
                    Expression::Logic(
                        Box::new(Logic::Equal(
                            Expression::Identifier(Identifier::new("y").with_position((13, 14))),
                            Expression::Value(ValueNode::Integer(2).with_position((18, 19))),
                        ))
                        .with_position((13, 19))
                    )
                ))
                .with_position((0, 20))
            ))
        );

        assert_eq!(
            parse(&lex("(x == 1) && (y == 2) && true").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Logic(
                Box::new(Logic::And(
                    Expression::Logic(
                        Box::new(Logic::And(
                            Expression::Logic(
                                Box::new(Logic::Equal(
                                    Expression::Identifier(
                                        Identifier::new("x").with_position((1, 2))
                                    ),
                                    Expression::Value(ValueNode::Integer(1).with_position((6, 7)))
                                ))
                                .with_position((1, 7))
                            ),
                            Expression::Logic(
                                Box::new(Logic::Equal(
                                    Expression::Identifier(
                                        Identifier::new("y").with_position((13, 14))
                                    ),
                                    Expression::Value(
                                        ValueNode::Integer(2).with_position((18, 19))
                                    )
                                ))
                                .with_position((13, 19))
                            ),
                        ))
                        .with_position((0, 20))
                    ),
                    Expression::Value(ValueNode::Boolean(true).with_position((24, 28)))
                ))
                .with_position((0, 28))
            ))
        );
    }

    #[test]
    fn list() {
        assert_eq!(
            parse(&lex("[]").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::List(Vec::with_capacity(0)).with_position((0, 2))
            ),)
        );
        assert_eq!(
            parse(&lex("[42]").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::List(vec![Expression::Value(
                    ValueNode::Integer(42).with_position((1, 3))
                )])
                .with_position((0, 4))
            ))
        );
        assert_eq!(
            parse(&lex("[42, 'foo', 'bar', [1, 2, 3,]]").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::List(vec![
                    Expression::Value(ValueNode::Integer(42).with_position((1, 3))),
                    Expression::Value(ValueNode::String("foo".to_string()).with_position((5, 10))),
                    Expression::Value(ValueNode::String("bar".to_string()).with_position((12, 17))),
                    Expression::Value(
                        ValueNode::List(vec![
                            Expression::Value(ValueNode::Integer(1).with_position((20, 21))),
                            Expression::Value(ValueNode::Integer(2).with_position((23, 24))),
                            Expression::Value(ValueNode::Integer(3).with_position((26, 27))),
                        ])
                        .with_position((19, 29))
                    )
                ])
                .with_position((0, 30))
            ),)
        );
    }

    #[test]
    fn r#true() {
        assert_eq!(
            parse(&lex("true").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::Boolean(true).with_position((0, 4))
            ))
        );
    }

    #[test]
    fn r#false() {
        assert_eq!(
            parse(&lex("false").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::Boolean(false).with_position((0, 5))
            ))
        );
    }

    #[test]
    fn positive_float() {
        assert_eq!(
            parse(&lex("0.0").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::Float(0.0).with_position((0, 3))
            ))
        );
        assert_eq!(
            parse(&lex("42.0").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::Float(42.0).with_position((0, 4))
            ))
        );

        let max_float = f64::MAX.to_string() + ".0";

        assert_eq!(
            parse(&lex(&max_float).unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::Float(f64::MAX).with_position((0, 311))
            ))
        );

        let min_positive_float = f64::MIN_POSITIVE.to_string();

        assert_eq!(
            parse(&lex(&min_positive_float).unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::Float(f64::MIN_POSITIVE).with_position((0, 326))
            ),)
        );
    }

    #[test]
    fn negative_float() {
        assert_eq!(
            parse(&lex("-0.0").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::Float(-0.0).with_position((0, 4))
            ))
        );
        assert_eq!(
            parse(&lex("-42.0").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::Float(-42.0).with_position((0, 5))
            ))
        );

        let min_float = f64::MIN.to_string() + ".0";

        assert_eq!(
            parse(&lex(&min_float).unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::Float(f64::MIN).with_position((0, 312))
            ))
        );

        let max_negative_float = format!("-{}", f64::MIN_POSITIVE);

        assert_eq!(
            parse(&lex(&max_negative_float).unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::Float(-f64::MIN_POSITIVE).with_position((0, 327))
            ),)
        );
    }

    #[test]
    fn other_float() {
        assert_eq!(
            parse(&lex("Infinity").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::Float(f64::INFINITY).with_position((0, 8))
            ))
        );
        assert_eq!(
            parse(&lex("-Infinity").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::Float(f64::NEG_INFINITY).with_position((0, 9))
            ))
        );

        if let Statement::Expression(Expression::Value(WithPosition {
            node: ValueNode::Float(float),
            ..
        })) = &parse(&lex("NaN").unwrap()).unwrap()[0]
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
                statements[0],
                Statement::Expression(Expression::Value(
                    ValueNode::Integer(i).with_position((0, 1))
                ))
            )
        }

        assert_eq!(
            parse(&lex("42").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::Integer(42).with_position((0, 2))
            ))
        );

        let maximum_integer = i64::MAX.to_string();

        assert_eq!(
            parse(&lex(&maximum_integer).unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::Integer(i64::MAX).with_position((0, 19))
            ))
        );
    }

    #[test]
    fn negative_integer() {
        for i in -9..0 {
            let source = i.to_string();
            let tokens = lex(&source).unwrap();
            let statements = parse(&tokens).unwrap();

            assert_eq!(
                statements[0],
                Statement::Expression(Expression::Value(
                    ValueNode::Integer(i).with_position((0, 2))
                ))
            )
        }

        assert_eq!(
            parse(&lex("-42").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::Integer(-42).with_position((0, 3))
            ))
        );

        let minimum_integer = i64::MIN.to_string();

        assert_eq!(
            parse(&lex(&minimum_integer).unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::Integer(i64::MIN).with_position((0, 20))
            ))
        );
    }

    #[test]
    fn double_quoted_string() {
        assert_eq!(
            parse(&lex("\"\"").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::String("".to_string()).with_position((0, 2))
            ))
        );
        assert_eq!(
            parse(&lex("\"42\"").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::String("42".to_string()).with_position((0, 4))
            ),)
        );
        assert_eq!(
            parse(&lex("\"foobar\"").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::String("foobar".to_string()).with_position((0, 8))
            ),)
        );
    }

    #[test]
    fn single_quoted_string() {
        assert_eq!(
            parse(&lex("''").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::String("".to_string()).with_position((0, 2))
            ))
        );
        assert_eq!(
            parse(&lex("'42'").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::String("42".to_string()).with_position((0, 4))
            ),)
        );
        assert_eq!(
            parse(&lex("'foobar'").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::String("foobar".to_string()).with_position((0, 8))
            ),)
        );
    }

    #[test]
    fn grave_quoted_string() {
        assert_eq!(
            parse(&lex("``").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::String("".to_string()).with_position((0, 2))
            ))
        );
        assert_eq!(
            parse(&lex("`42`").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::String("42".to_string()).with_position((0, 4))
            ),)
        );
        assert_eq!(
            parse(&lex("`foobar`").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::String("foobar".to_string()).with_position((0, 8))
            ),)
        );
    }
}
