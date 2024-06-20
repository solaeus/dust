use std::{cell::RefCell, collections::HashMap};

use chumsky::{input::SpannedInput, pratt::*, prelude::*};

use crate::{
    abstract_tree::*,
    error::DustError,
    identifier::Identifier,
    lexer::{Control, Keyword, Operator, Token},
};

use self::type_constructor::TypeInvokationConstructor;

pub type ParserInput<'src> =
    SpannedInput<Token<'src>, SimpleSpan, &'src [(Token<'src>, SimpleSpan)]>;

pub type ParserExtra<'src> = extra::Err<Rich<'src, Token<'src>, SimpleSpan>>;

pub fn parse<'src>(
    tokens: &'src [(Token<'src>, SimpleSpan)],
) -> Result<AbstractTree, Vec<DustError>> {
    let statements = parser(false)
        .parse(tokens.spanned((tokens.len()..tokens.len()).into()))
        .into_result()
        .map_err(|errors| {
            errors
                .into_iter()
                .map(|error| DustError::from(error))
                .collect::<Vec<DustError>>()
        })?;

    Ok(AbstractTree::new(statements))
}

pub fn parser<'src>(
    allow_built_ins: bool,
) -> impl Parser<'src, ParserInput<'src>, Vec<Statement>, ParserExtra<'src>> {
    let comment = select_ref! {
        Token::Comment(_) => {}
    };
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

    let positioned_identifier = identifier
        .clone()
        .map_with(|identifier, state| identifier.with_position(state.span()));

    let basic_value = select! {
        Token::Boolean(boolean) => ValueNode::Boolean(boolean),
        Token::Float(float) => ValueNode::Float(float),
        Token::Integer(integer) => ValueNode::Integer(integer),
        Token::String(text) => ValueNode::String(text.to_string()),
    }
    .map_with(|value, state| Expression::Value(value.with_position(state.span())));

    let raw_integer = select! {
        Token::Integer(integer) => integer
    };

    let type_constructor = recursive(|type_constructor| {
        let primitive_type = choice((
            just(Token::Keyword(Keyword::Any)).to(Type::Any),
            just(Token::Keyword(Keyword::Bool)).to(Type::Boolean),
            just(Token::Keyword(Keyword::Float)).to(Type::Float),
            just(Token::Keyword(Keyword::Int)).to(Type::Integer),
            just(Token::Keyword(Keyword::None)).to(Type::None),
            just(Token::Keyword(Keyword::Range)).to(Type::Range),
            just(Token::Keyword(Keyword::Str)).to(Type::String),
        ))
        .map_with(|r#type, state| TypeConstructor::Raw(r#type.with_position(state.span())));

        let function_type = just(Token::Keyword(Keyword::Fn))
            .ignore_then(
                positioned_identifier
                    .clone()
                    .separated_by(just(Token::Control(Control::Comma)))
                    .at_least(1)
                    .collect()
                    .delimited_by(
                        just(Token::Control(Control::Pipe)),
                        just(Token::Control(Control::Pipe)),
                    )
                    .or_not(),
            )
            .then(
                type_constructor
                    .clone()
                    .separated_by(just(Token::Control(Control::Comma)))
                    .collect()
                    .delimited_by(
                        just(Token::Control(Control::ParenOpen)),
                        just(Token::Control(Control::ParenClose)),
                    ),
            )
            .then_ignore(just(Token::Control(Control::SkinnyArrow)))
            .then(type_constructor.clone())
            .map_with(
                |((type_parameters, value_parameters), return_type), state| {
                    TypeConstructor::Function(
                        FunctionTypeConstructor {
                            type_parameters,
                            value_parameters,
                            return_type: Box::new(return_type),
                        }
                        .with_position(state.span()),
                    )
                },
            );

        let list_type = type_constructor
            .clone()
            .then_ignore(just(Token::Control(Control::Semicolon)))
            .then(raw_integer.clone())
            .delimited_by(
                just(Token::Control(Control::SquareOpen)),
                just(Token::Control(Control::SquareClose)),
            )
            .map_with(|(item_type, length), state| {
                TypeConstructor::List(
                    ListTypeConstructor {
                        length: length as usize,
                        item_type: Box::new(item_type),
                    }
                    .with_position(state.span()),
                )
            });

        let list_of_type = type_constructor
            .clone()
            .delimited_by(
                just(Token::Control(Control::SquareOpen)),
                just(Token::Control(Control::SquareClose)),
            )
            .map_with(|item_type, state| {
                TypeConstructor::ListOf(Box::new(item_type).with_position(state.span()))
            });

        let enum_variant = positioned_identifier.clone().then(
            type_constructor
                .clone()
                .separated_by(just(Token::Control(Control::Comma)))
                .collect()
                .delimited_by(
                    just(Token::Control(Control::ParenOpen)),
                    just(Token::Control(Control::ParenClose)),
                )
                .or_not(),
        );

        let enum_type = just(Token::Keyword(Keyword::Enum))
            .ignore_then(
                positioned_identifier
                    .clone()
                    .separated_by(just(Token::Control(Control::Comma)))
                    .collect()
                    .delimited_by(
                        just(Token::Control(Control::Pipe)),
                        just(Token::Control(Control::Pipe)),
                    )
                    .or_not(),
            )
            .then(
                enum_variant
                    .separated_by(just(Token::Control(Control::Comma)))
                    .at_least(1)
                    .allow_trailing()
                    .collect()
                    .delimited_by(
                        just(Token::Control(Control::CurlyOpen)),
                        just(Token::Control(Control::CurlyClose)),
                    ),
            )
            .map_with(|(type_parameters, variants), state| {
                TypeConstructor::Enum(
                    EnumTypeConstructor {
                        type_parameters,
                        variants,
                    }
                    .with_position(state.span()),
                )
            });

        let type_invokation = positioned_identifier
            .clone()
            .then(
                type_constructor
                    .clone()
                    .separated_by(just(Token::Control(Control::Comma)))
                    .at_least(1)
                    .allow_trailing()
                    .collect()
                    .delimited_by(
                        just(Token::Control(Control::ParenOpen)),
                        just(Token::Control(Control::ParenClose)),
                    )
                    .or_not(),
            )
            .map(|(identifier, type_arguments)| {
                TypeConstructor::Invokation(TypeInvokationConstructor {
                    identifier,
                    type_arguments,
                })
            });

        choice((
            type_invokation,
            function_type,
            list_type,
            list_of_type,
            primitive_type,
            enum_type,
        ))
    });

    let type_specification =
        just(Token::Control(Control::Colon)).ignore_then(type_constructor.clone());

    let statement = recursive(|statement| {
        let allow_built_ins = allow_built_ins.clone();

        let block = statement
            .clone()
            .repeated()
            .at_least(1)
            .collect()
            .delimited_by(
                just(Token::Control(Control::CurlyOpen)),
                just(Token::Control(Control::CurlyClose)),
            )
            .map_with(|statements, state| Block::new(statements).with_position(state.span()));

        let expression = recursive(|expression| {
            let allow_built_ins = allow_built_ins.clone();

            let identifier_expression = identifier.clone().map_with(|identifier, state| {
                Expression::Identifier(identifier.with_position(state.span()))
            });

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

            let map_fields = identifier
                .clone()
                .then(type_specification.clone().or_not())
                .then_ignore(just(Token::Operator(Operator::Assign)))
                .then(expression.clone())
                .map(|((identifier, r#type), expression)| (identifier, r#type, expression));

            let map = map_fields
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

            let function = just(Token::Keyword(Keyword::Fn))
                .ignore_then(
                    identifier
                        .clone()
                        .separated_by(just(Token::Control(Control::Comma)))
                        .at_least(1)
                        .allow_trailing()
                        .collect()
                        .delimited_by(
                            just(Token::Control(Control::Pipe)),
                            just(Token::Control(Control::Pipe)),
                        )
                        .or_not(),
                )
                .then(
                    identifier
                        .clone()
                        .then_ignore(just(Token::Control(Control::Colon)))
                        .then(type_constructor.clone())
                        .separated_by(just(Token::Control(Control::Comma)))
                        .allow_trailing()
                        .collect()
                        .delimited_by(
                            just(Token::Control(Control::ParenOpen)),
                            just(Token::Control(Control::ParenClose)),
                        ),
                )
                .then_ignore(just(Token::Control(Control::SkinnyArrow)))
                .then(type_constructor.clone())
                .then(block.clone())
                .map_with(
                    |(((type_parameters, value_parameters), return_type), body), state| {
                        Expression::Value(
                            ValueNode::Function {
                                type_parameters,
                                value_parameters,
                                return_type,
                                body,
                            }
                            .with_position(state.span()),
                        )
                    },
                );

            let enum_instance = positioned_identifier
                .clone()
                .then_ignore(just(Token::Control(Control::DoubleColon)))
                .then(positioned_identifier.clone())
                .then(
                    expression
                        .clone()
                        .separated_by(just(Token::Control(Control::Comma)))
                        .collect()
                        .delimited_by(
                            just(Token::Control(Control::ParenOpen)),
                            just(Token::Control(Control::ParenClose)),
                        )
                        .or_not(),
                )
                .map_with(|((type_name, variant), content), state| {
                    Expression::Value(
                        ValueNode::EnumInstance {
                            type_name,
                            variant,
                            content,
                        }
                        .with_position(state.span()),
                    )
                });

            let built_in_function_call = choice((
                just(Token::Keyword(Keyword::Length))
                    .ignore_then(expression.clone())
                    .map_with(|argument, state| {
                        Expression::BuiltInFunctionCall(
                            Box::new(BuiltInFunctionCall::Length(argument))
                                .with_position(state.span()),
                        )
                    }),
                just(Token::Keyword(Keyword::ReadFile))
                    .ignore_then(expression.clone())
                    .map_with(|argument, state| {
                        Expression::BuiltInFunctionCall(
                            Box::new(BuiltInFunctionCall::ReadFile(argument))
                                .with_position(state.span()),
                        )
                    }),
                just(Token::Keyword(Keyword::ReadLine)).map_with(|_, state| {
                    Expression::BuiltInFunctionCall(
                        Box::new(BuiltInFunctionCall::ReadLine).with_position(state.span()),
                    )
                }),
                just(Token::Keyword(Keyword::Sleep))
                    .ignore_then(expression.clone())
                    .map_with(|argument, state| {
                        Expression::BuiltInFunctionCall(
                            Box::new(BuiltInFunctionCall::Sleep(argument))
                                .with_position(state.span()),
                        )
                    }),
                just(Token::Keyword(Keyword::WriteLine))
                    .ignore_then(expression.clone())
                    .map_with(|argument, state| {
                        Expression::BuiltInFunctionCall(
                            Box::new(BuiltInFunctionCall::WriteLine(argument))
                                .with_position(state.span()),
                        )
                    }),
                just(Token::Keyword(Keyword::JsonParse))
                    .ignore_then(type_constructor.clone())
                    .then(expression.clone())
                    .map_with(|(constructor, argument), state| {
                        Expression::BuiltInFunctionCall(
                            Box::new(BuiltInFunctionCall::JsonParse(constructor, argument))
                                .with_position(state.span()),
                        )
                    }),
            ))
            .try_map_with(move |expression, state| {
                if allow_built_ins {
                    Ok(expression)
                } else {
                    Err(Rich::custom(
                        state.span(),
                        "Built-in function calls can only be used by the standard library.",
                    ))
                }
            });

            let turbofish = type_constructor
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
                enum_instance.clone(),
                range.clone(),
                function.clone(),
                list.clone(),
                map.clone(),
                basic_value.clone(),
                identifier_expression.clone(),
                expression.clone().delimited_by(
                    just(Token::Control(Control::ParenOpen)),
                    just(Token::Control(Control::ParenClose)),
                ),
            ));

            let logic_math_indexes_as_and_function_calls = atom.pratt((
                // Logic
                prefix(
                    2,
                    just(Token::Operator(Operator::Not)),
                    |_, expression, span| {
                        Expression::Logic(Box::new(Logic::Not(expression)).with_position(span))
                    },
                ),
                infix(
                    left(1),
                    just(Token::Operator(Operator::Equal)),
                    |left, _, right, span| {
                        Expression::Logic(Box::new(Logic::Equal(left, right)).with_position(span))
                    },
                ),
                infix(
                    left(1),
                    just(Token::Operator(Operator::NotEqual)),
                    |left, _, right, span| {
                        Expression::Logic(
                            Box::new(Logic::NotEqual(left, right)).with_position(span),
                        )
                    },
                ),
                infix(
                    left(1),
                    just(Token::Operator(Operator::Greater)),
                    |left, _, right, span| {
                        Expression::Logic(Box::new(Logic::Greater(left, right)).with_position(span))
                    },
                ),
                infix(
                    left(1),
                    just(Token::Operator(Operator::Less)),
                    |left, _, right, span| {
                        Expression::Logic(Box::new(Logic::Less(left, right)).with_position(span))
                    },
                ),
                infix(
                    left(1),
                    just(Token::Operator(Operator::GreaterOrEqual)),
                    |left, _, right, span| {
                        Expression::Logic(
                            Box::new(Logic::GreaterOrEqual(left, right)).with_position(span),
                        )
                    },
                ),
                infix(
                    left(1),
                    just(Token::Operator(Operator::LessOrEqual)),
                    |left, _, right, span| {
                        Expression::Logic(
                            Box::new(Logic::LessOrEqual(left, right)).with_position(span),
                        )
                    },
                ),
                infix(
                    left(1),
                    just(Token::Operator(Operator::And)),
                    |left, _, right, span| {
                        Expression::Logic(Box::new(Logic::And(left, right)).with_position(span))
                    },
                ),
                infix(
                    left(1),
                    just(Token::Operator(Operator::Or)),
                    |left, _, right, span| {
                        Expression::Logic(Box::new(Logic::Or(left, right)).with_position(span))
                    },
                ),
                // Math
                infix(
                    left(1),
                    just(Token::Operator(Operator::Add)),
                    |left, _, right, span| {
                        Expression::Math(Box::new(Math::Add(left, right)).with_position(span))
                    },
                ),
                infix(
                    left(1),
                    just(Token::Operator(Operator::Subtract)),
                    |left, _, right, span| {
                        Expression::Math(Box::new(Math::Subtract(left, right)).with_position(span))
                    },
                ),
                infix(
                    left(2),
                    just(Token::Operator(Operator::Multiply)),
                    |left, _, right, span| {
                        Expression::Math(Box::new(Math::Multiply(left, right)).with_position(span))
                    },
                ),
                infix(
                    left(2),
                    just(Token::Operator(Operator::Divide)),
                    |left, _, right, span| {
                        Expression::Math(Box::new(Math::Divide(left, right)).with_position(span))
                    },
                ),
                infix(
                    left(1),
                    just(Token::Operator(Operator::Modulo)),
                    |left, _, right, span| {
                        Expression::Math(Box::new(Math::Modulo(left, right)).with_position(span))
                    },
                ),
                // Indexes
                infix(
                    left(4),
                    just(Token::Control(Control::Dot)),
                    |left, _, right, span| {
                        Expression::MapIndex(
                            Box::new(MapIndex::new(left, right)).with_position(span),
                        )
                    },
                ),
                postfix(
                    3,
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
                // Function call
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
                    |function_expression, (type_parameters, value_parameters), span| {
                        Expression::FunctionCall(
                            FunctionCall::new(
                                function_expression,
                                type_parameters,
                                value_parameters,
                            )
                            .with_position(span),
                        )
                    },
                ),
                // As
                postfix(
                    2,
                    just(Token::Keyword(Keyword::As)).ignore_then(type_constructor.clone()),
                    |expression, constructor, span| {
                        Expression::As(
                            Box::new(As::new(expression, constructor)).with_position(span),
                        )
                    },
                ),
            ));

            choice((
                logic_math_indexes_as_and_function_calls,
                enum_instance,
                built_in_function_call,
                range,
                function,
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

        let type_assignment = just(Token::Keyword(Keyword::Type))
            .ignore_then(positioned_identifier.clone())
            .then_ignore(just(Token::Operator(Operator::Assign)))
            .then(type_constructor.clone())
            .map_with(|(identifier, constructor), state| {
                Statement::TypeAssignment(
                    TypeAssignment::new(identifier, constructor).with_position(state.span()),
                )
            });

        comment
            .repeated()
            .or_not()
            .ignore_then(choice((
                assignment,
                expression_statement,
                async_block,
                if_else,
                r#break,
                block_statement,
                r#loop,
                r#while,
                type_assignment,
            )))
            .then_ignore(just(Token::Control(Control::Semicolon)).or_not())
    });

    statement.repeated().collect()
}

#[cfg(test)]
mod tests {
    use crate::lexer::lex;

    use super::*;

    #[test]
    fn type_invokation() {
        assert_eq!(
            parse(&lex("x: Foo(int) = Foo::Bar(42)").unwrap()).unwrap()[0],
            Statement::Assignment(
                Assignment::new(
                    Identifier::new("x").with_position((0, 1)),
                    Some(TypeConstructor::Invokation(TypeInvokationConstructor {
                        identifier: Identifier::new("Foo").with_position((3, 6)),
                        type_arguments: Some(vec![TypeConstructor::Raw(
                            Type::Integer.with_position((7, 10))
                        )]),
                    })),
                    AssignmentOperator::Assign,
                    Statement::Expression(Expression::Value(
                        ValueNode::EnumInstance {
                            type_name: Identifier::new("Foo").with_position((14, 17)),
                            variant: Identifier::new("Bar").with_position((19, 22)),
                            content: Some(vec![Expression::Value(
                                ValueNode::Integer(42).with_position((23, 25))
                            )])
                        }
                        .with_position((14, 26))
                    ))
                )
                .with_position((0, 26))
            )
        );
    }

    #[test]
    fn enum_instance() {
        assert_eq!(
            parse(&lex("Foo::Bar(42)").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::EnumInstance {
                    type_name: Identifier::new("Foo").with_position((0, 3)),
                    variant: Identifier::new("Bar").with_position((5, 8)),
                    content: Some(vec![Expression::Value(
                        ValueNode::Integer(42).with_position((9, 11))
                    )])
                }
                .with_position((0, 12))
            ))
        );
    }

    #[test]
    fn enum_type_empty() {
        assert_eq!(
            parse(&lex("type MyEnum = enum { X, Y }").unwrap()).unwrap()[0],
            Statement::TypeAssignment(
                TypeAssignment::new(
                    Identifier::new("MyEnum").with_position((5, 11)),
                    TypeConstructor::Enum(
                        EnumTypeConstructor {
                            type_parameters: None,
                            variants: vec![
                                (Identifier::new("X").with_position((21, 22)), None),
                                (Identifier::new("Y").with_position((24, 25)), None)
                            ],
                        }
                        .with_position((14, 27))
                    )
                )
                .with_position((0, 27))
            )
        );
    }

    #[test]
    fn enum_type_with_contents() {
        assert_eq!(
            parse(&lex("type MyEnum = enum { X(str, int), Y(int) }").unwrap()).unwrap()[0],
            Statement::TypeAssignment(
                TypeAssignment::new(
                    Identifier::new("MyEnum").with_position((5, 11)),
                    TypeConstructor::Enum(
                        EnumTypeConstructor {
                            type_parameters: None,
                            variants: vec![
                                (
                                    Identifier::new("X").with_position((21, 22)),
                                    Some(vec![
                                        TypeConstructor::Raw(Type::String.with_position((23, 26))),
                                        TypeConstructor::Raw(Type::Integer.with_position((28, 31)))
                                    ])
                                ),
                                (
                                    Identifier::new("Y").with_position((34, 35)),
                                    Some(vec![TypeConstructor::Raw(
                                        Type::Integer.with_position((36, 39))
                                    )])
                                )
                            ],
                        }
                        .with_position((14, 42))
                    )
                )
                .with_position((0, 42))
            )
        );
    }

    #[test]
    fn enum_type_with_type_parameters() {
        assert_eq!(
            parse(&lex("type MyEnum = enum |T, U| { X(T), Y(U) }").unwrap()).unwrap()[0],
            Statement::TypeAssignment(
                TypeAssignment::new(
                    Identifier::new("MyEnum").with_position((5, 11)),
                    TypeConstructor::Enum(
                        EnumTypeConstructor {
                            type_parameters: Some(vec![
                                Identifier::new("T").with_position((20, 21)),
                                Identifier::new("U").with_position((23, 24)),
                            ]),
                            variants: vec![
                                (
                                    Identifier::new("X").with_position((28, 29)),
                                    Some(vec![TypeConstructor::Invokation(
                                        TypeInvokationConstructor {
                                            identifier: Identifier::new("T")
                                                .with_position((30, 31)),
                                            type_arguments: None,
                                        }
                                    )])
                                ),
                                (
                                    Identifier::new("Y").with_position((34, 35)),
                                    Some(vec![TypeConstructor::Invokation(
                                        TypeInvokationConstructor {
                                            identifier: Identifier::new("U")
                                                .with_position((36, 37)),
                                            type_arguments: None,
                                        }
                                    )])
                                ),
                            ],
                        }
                        .with_position((14, 40))
                    )
                )
                .with_position((0, 40))
            )
        );
    }

    // Reuse these tests when structures are reimplemented
    // #[test]
    // fn structure_instance() {
    //     assert_eq!(
    //         parse(
    //             &lex("
    //                 Foo {
    //                     bar = 42,
    //                     baz = 'hiya',
    //                 }
    //             ")
    //             .unwrap()
    //         )
    //         .unwrap()[0],
    //         Statement::Expression(Expression::Value(
    //             ValueNode::Structure {
    //                 name: Identifier::new("Foo").with_position((21, 24)),
    //                 fields: vec![
    //                     (
    //                         Identifier::new("bar").with_position((0, 0)),
    //                         Expression::Value(ValueNode::Integer(42).with_position((57, 59)))
    //                     ),
    //                     (
    //                         Identifier::new("baz").with_position((0, 0)),
    //                         Expression::Value(
    //                             ValueNode::String("hiya".to_string()).with_position((91, 97))
    //                         )
    //                     ),
    //                 ]
    //             }
    //             .with_position((21, 120))
    //         ))
    //     )
    // }

    // #[test]
    // fn structure_definition() {
    //     assert_eq!(
    //         parse(
    //             &lex("
    //                 struct Foo {
    //                     bar : int,
    //                     baz : str,
    //                 }
    //             ")
    //             .unwrap()
    //         )
    //         .unwrap()[0],
    //         Statement::StructureDefinition(
    //             StructureDefinition::new(
    //                 Identifier::new("Foo"),
    //                 vec![
    //                     (
    //                         Identifier::new("bar"),
    //                         TypeConstructor::Type(Type::Integer.with_position((64, 67)))
    //                     ),
    //                     (
    //                         Identifier::new("baz"),
    //                         TypeConstructor::Type(Type::String.with_position((99, 102)))
    //                     ),
    //                 ]
    //             )
    //             .with_position((21, 125))
    //         )
    //     )
    // }

    #[test]
    fn type_alias() {
        assert_eq!(
            parse(&lex("type MyType = str").unwrap()).unwrap()[0],
            Statement::TypeAssignment(
                TypeAssignment::new(
                    Identifier::new("MyType").with_position((5, 11)),
                    TypeConstructor::Raw(Type::String.with_position((14, 17)))
                )
                .with_position((0, 17))
            )
        )
    }

    #[test]
    fn r#as() {
        assert_eq!(
            parse(&lex("1 as str").unwrap()).unwrap()[0],
            Statement::Expression(Expression::As(
                Box::new(As::new(
                    Expression::Value(ValueNode::Integer(1).with_position((0, 1))),
                    TypeConstructor::Raw(Type::String.with_position((5, 8)))
                ))
                .with_position((0, 8))
            ))
        )
    }

    #[test]
    fn built_in_function() {
        let tokens = lex("READ_LINE").unwrap();
        let statements = parser(true)
            .parse(tokens.spanned((tokens.len()..tokens.len()).into()))
            .into_result()
            .map_err(|errors| {
                errors
                    .into_iter()
                    .map(|error| DustError::from(error))
                    .collect::<Vec<DustError>>()
            })
            .unwrap();

        assert_eq!(
            statements[0],
            Statement::Expression(Expression::BuiltInFunctionCall(
                Box::new(BuiltInFunctionCall::ReadLine).with_position((0, 9))
            ))
        );

        let tokens = lex("WRITE_LINE 'hiya'").unwrap();
        let statements = parser(true)
            .parse(tokens.spanned((tokens.len()..tokens.len()).into()))
            .into_result()
            .map_err(|errors| {
                errors
                    .into_iter()
                    .map(|error| DustError::from(error))
                    .collect::<Vec<DustError>>()
            })
            .unwrap();

        assert_eq!(
            statements[0],
            Statement::Expression(Expression::BuiltInFunctionCall(
                Box::new(BuiltInFunctionCall::WriteLine(Expression::Value(
                    ValueNode::String("hiya".to_string()).with_position((11, 17))
                )))
                .with_position((0, 17))
            ))
        );
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
                            None,
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
                    Some(TypeConstructor::Raw(Type::Boolean.with_position((9, 13)))),
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
            parse(&lex("foobar: [int; 2] = []").unwrap()).unwrap()[0],
            Statement::Assignment(
                Assignment::new(
                    Identifier::new("foobar").with_position((0, 6)),
                    Some(TypeConstructor::List(
                        ListTypeConstructor {
                            length: 2,
                            item_type: Box::new(TypeConstructor::Raw(
                                Type::Integer.with_position((9, 12))
                            ))
                        }
                        .with_position((8, 16))
                    )),
                    AssignmentOperator::Assign,
                    Statement::Expression(Expression::Value(
                        ValueNode::List(vec![]).with_position((19, 21))
                    ))
                )
                .with_position((0, 21))
            )
        );
    }

    #[test]
    fn list_of_type() {
        assert_eq!(
            parse(&lex("foobar : [bool] = [true]").unwrap()).unwrap()[0],
            Statement::Assignment(
                Assignment::new(
                    Identifier::new("foobar").with_position((0, 6)),
                    Some(TypeConstructor::ListOf(
                        Box::new(TypeConstructor::Raw(Type::Boolean.with_position((10, 14))))
                            .with_position((9, 15))
                    )),
                    AssignmentOperator::Assign,
                    Statement::Expression(Expression::Value(
                        ValueNode::List(vec![Expression::Value(
                            ValueNode::Boolean(true).with_position((19, 23))
                        )])
                        .with_position((18, 24))
                    ))
                )
                .with_position((0, 24))
            )
        );
    }

    #[test]
    fn function_type() {
        assert_eq!(
            parse(&lex("type Foo = fn |T| (int) -> T").unwrap()).unwrap()[0],
            Statement::TypeAssignment(
                TypeAssignment::new(
                    Identifier::new("Foo").with_position((5, 8)),
                    TypeConstructor::Function(
                        FunctionTypeConstructor {
                            type_parameters: Some(vec![
                                Identifier::new("T").with_position((15, 16))
                            ]),
                            value_parameters: vec![TypeConstructor::Raw(
                                Type::Integer.with_position((19, 22))
                            )],
                            return_type: Box::new(TypeConstructor::Invokation(
                                TypeInvokationConstructor {
                                    identifier: Identifier::new("T").with_position((27, 28)),
                                    type_arguments: None,
                                }
                            )),
                        }
                        .with_position((11, 28))
                    )
                )
                .with_position((0, 28))
            )
        );
    }

    #[test]
    fn function_call() {
        assert_eq!(
            parse(&lex("foobar()").unwrap()).unwrap()[0],
            Statement::Expression(Expression::FunctionCall(
                FunctionCall::new(
                    Expression::Identifier(Identifier::new("foobar").with_position((0, 6))),
                    None,
                    Vec::with_capacity(0),
                )
                .with_position((0, 8))
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
                    Some(vec![TypeConstructor::Raw(
                        Type::String.with_position((9, 12))
                    )]),
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
            parse(&lex("fn () -> int { 0 }").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::Function {
                    type_parameters: None,
                    value_parameters: vec![],
                    return_type: TypeConstructor::Raw(Type::Integer.with_position((9, 12))),
                    body: Block::new(vec![Statement::Expression(Expression::Value(
                        ValueNode::Integer(0).with_position((15, 16))
                    ))])
                    .with_position((13, 18))
                }
                .with_position((0, 18))
            ),)
        );

        assert_eq!(
            parse(&lex("fn (x: int) -> int { x }").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::Function {
                    type_parameters: None,
                    value_parameters: vec![(
                        Identifier::new("x"),
                        TypeConstructor::Raw(Type::Integer.with_position((7, 10)))
                    )],
                    return_type: TypeConstructor::Raw(Type::Integer.with_position((15, 18))),
                    body: Block::new(vec![Statement::Expression(Expression::Identifier(
                        Identifier::new("x").with_position((21, 22))
                    ))])
                    .with_position((19, 24)),
                }
                .with_position((0, 24))
            ),)
        );
    }

    #[test]
    fn function_with_type_arguments() {
        assert_eq!(
            parse(&lex("fn |T, U| (x: T, y: U) -> T { x }").unwrap()).unwrap()[0],
            Statement::Expression(Expression::Value(
                ValueNode::Function {
                    type_parameters: Some(vec![Identifier::new("T"), Identifier::new("U"),]),
                    value_parameters: vec![
                        (
                            Identifier::new("x"),
                            TypeConstructor::Invokation(TypeInvokationConstructor {
                                identifier: Identifier::new("T").with_position((14, 15)),
                                type_arguments: None,
                            })
                        ),
                        (
                            Identifier::new("y"),
                            TypeConstructor::Invokation(TypeInvokationConstructor {
                                identifier: Identifier::new("U").with_position((20, 21)),
                                type_arguments: None,
                            })
                        )
                    ],
                    return_type: TypeConstructor::Invokation(TypeInvokationConstructor {
                        identifier: Identifier::new("T").with_position((26, 27)),
                        type_arguments: None,
                    }),
                    body: Block::new(vec![Statement::Expression(Expression::Identifier(
                        Identifier::new("x").with_position((30, 31))
                    ))])
                    .with_position((28, 33)),
                }
                .with_position((0, 33))
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
                    Some(TypeConstructor::Raw(Type::Integer.with_position((8, 11)))),
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
