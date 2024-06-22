#[cfg(test)]
mod tests;

use std::{cell::RefCell, collections::HashMap};

use chumsky::{input::SpannedInput, pratt::*, prelude::*};

use crate::{
    abstract_tree::*,
    error::DustError,
    identifier::Identifier,
    lexer::{Keyword, Symbol, Token},
};

use self::{
    enum_declaration::EnumVariant,
    type_constructor::{RawTypeConstructor, TypeInvokationConstructor},
};

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
            just(Token::Keyword(Keyword::Any)).to(RawTypeConstructor::Any),
            just(Token::Keyword(Keyword::Bool)).to(RawTypeConstructor::Boolean),
            just(Token::Keyword(Keyword::Float)).to(RawTypeConstructor::Float),
            just(Token::Keyword(Keyword::Int)).to(RawTypeConstructor::Integer),
            just(Token::Keyword(Keyword::Map)).to(RawTypeConstructor::Map),
            just(Token::Keyword(Keyword::Range)).to(RawTypeConstructor::Range),
            just(Token::Keyword(Keyword::Str)).to(RawTypeConstructor::String),
        ))
        .map_with(|raw_constructor, state| {
            TypeConstructor::Raw(raw_constructor.with_position(state.span()))
        });

        let function_type = just(Token::Keyword(Keyword::Fn))
            .ignore_then(
                positioned_identifier
                    .clone()
                    .separated_by(just(Token::Symbol(Symbol::Comma)))
                    .at_least(1)
                    .collect()
                    .delimited_by(
                        just(Token::Symbol(Symbol::Pipe)),
                        just(Token::Symbol(Symbol::Pipe)),
                    )
                    .or_not(),
            )
            .then(
                type_constructor
                    .clone()
                    .separated_by(just(Token::Symbol(Symbol::Comma)))
                    .collect()
                    .delimited_by(
                        just(Token::Symbol(Symbol::ParenOpen)),
                        just(Token::Symbol(Symbol::ParenClose)),
                    ),
            )
            .then(
                just(Token::Symbol(Symbol::SkinnyArrow))
                    .ignore_then(type_constructor.clone())
                    .or_not(),
            )
            .map_with(
                |((type_parameters, value_parameters), return_type), state| {
                    TypeConstructor::Function(
                        FunctionTypeConstructor {
                            type_parameters,
                            value_parameters,
                            return_type: return_type.map(|r#type| Box::new(r#type)),
                        }
                        .with_position(state.span()),
                    )
                },
            );

        let list_type = type_constructor
            .clone()
            .then_ignore(just(Token::Symbol(Symbol::Semicolon)))
            .then(raw_integer.clone())
            .delimited_by(
                just(Token::Symbol(Symbol::SquareOpen)),
                just(Token::Symbol(Symbol::SquareClose)),
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
                just(Token::Symbol(Symbol::SquareOpen)),
                just(Token::Symbol(Symbol::SquareClose)),
            )
            .map_with(|item_type, state| {
                TypeConstructor::ListOf(Box::new(item_type).with_position(state.span()))
            });

        let type_invokation = positioned_identifier
            .clone()
            .then(
                type_constructor
                    .clone()
                    .separated_by(just(Token::Symbol(Symbol::Comma)))
                    .at_least(1)
                    .allow_trailing()
                    .collect()
                    .delimited_by(
                        just(Token::Symbol(Symbol::ParenOpen)),
                        just(Token::Symbol(Symbol::ParenClose)),
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
        ))
    });

    let type_specification =
        just(Token::Symbol(Symbol::Colon)).ignore_then(type_constructor.clone());

    let statement = recursive(|statement| {
        let allow_built_ins = allow_built_ins.clone();

        let block = statement
            .clone()
            .repeated()
            .at_least(1)
            .collect()
            .delimited_by(
                just(Token::Symbol(Symbol::CurlyOpen)),
                just(Token::Symbol(Symbol::CurlyClose)),
            )
            .map_with(|statements, state| Block::new(statements).with_position(state.span()));

        let expression = recursive(|expression| {
            let allow_built_ins = allow_built_ins.clone();

            let identifier_expression = identifier.clone().map_with(|identifier, state| {
                Expression::Identifier(identifier.with_position(state.span()))
            });

            let range = raw_integer
                .clone()
                .then_ignore(just(Token::Symbol(Symbol::DoubleDot)))
                .then(raw_integer)
                .map_with(|(start, end), state| {
                    Expression::Value(ValueNode::Range(start..end).with_position(state.span()))
                });

            let list = expression
                .clone()
                .separated_by(just(Token::Symbol(Symbol::Comma)))
                .allow_trailing()
                .collect()
                .delimited_by(
                    just(Token::Symbol(Symbol::SquareOpen)),
                    just(Token::Symbol(Symbol::SquareClose)),
                )
                .map_with(|list, state| {
                    Expression::Value(ValueNode::List(list).with_position(state.span()))
                });

            let map_fields = identifier
                .clone()
                .then(type_specification.clone().or_not())
                .then_ignore(just(Token::Symbol(Symbol::Equal)))
                .then(expression.clone())
                .map(|((identifier, r#type), expression)| (identifier, r#type, expression));

            let map = map_fields
                .separated_by(just(Token::Symbol(Symbol::Comma)).or_not())
                .allow_trailing()
                .collect()
                .delimited_by(
                    just(Token::Symbol(Symbol::CurlyOpen)),
                    just(Token::Symbol(Symbol::CurlyClose)),
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
                        .separated_by(just(Token::Symbol(Symbol::Comma)))
                        .at_least(1)
                        .allow_trailing()
                        .collect()
                        .delimited_by(
                            just(Token::Symbol(Symbol::Pipe)),
                            just(Token::Symbol(Symbol::Pipe)),
                        )
                        .or_not(),
                )
                .then(
                    identifier
                        .clone()
                        .then_ignore(just(Token::Symbol(Symbol::Colon)))
                        .then(type_constructor.clone())
                        .separated_by(just(Token::Symbol(Symbol::Comma)))
                        .allow_trailing()
                        .collect()
                        .delimited_by(
                            just(Token::Symbol(Symbol::ParenOpen)),
                            just(Token::Symbol(Symbol::ParenClose)),
                        ),
                )
                .then(
                    just(Token::Symbol(Symbol::SkinnyArrow))
                        .ignore_then(type_constructor.clone())
                        .or_not(),
                )
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
                .then_ignore(just(Token::Symbol(Symbol::DoubleColon)))
                .then(positioned_identifier.clone())
                .then(
                    expression
                        .clone()
                        .separated_by(just(Token::Symbol(Symbol::Comma)))
                        .collect()
                        .delimited_by(
                            just(Token::Symbol(Symbol::ParenOpen)),
                            just(Token::Symbol(Symbol::ParenClose)),
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
                .separated_by(just(Token::Symbol(Symbol::Comma)))
                .at_least(1)
                .collect()
                .delimited_by(
                    just(Token::Symbol(Symbol::ParenOpen)),
                    just(Token::Symbol(Symbol::ParenClose)),
                )
                .delimited_by(
                    just(Token::Symbol(Symbol::DoubleColon)),
                    just(Token::Symbol(Symbol::DoubleColon)),
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
                    just(Token::Symbol(Symbol::ParenOpen)),
                    just(Token::Symbol(Symbol::ParenClose)),
                ),
            ));

            let logic_math_indexes_as_and_function_calls = atom.pratt((
                // Logic
                prefix(
                    2,
                    just(Token::Symbol(Symbol::Exclamation)),
                    |_, expression, span| {
                        Expression::Logic(Box::new(Logic::Not(expression)).with_position(span))
                    },
                ),
                infix(
                    left(1),
                    just(Token::Symbol(Symbol::DoubleEqual)),
                    |left, _, right, span| {
                        Expression::Logic(Box::new(Logic::Equal(left, right)).with_position(span))
                    },
                ),
                infix(
                    left(1),
                    just(Token::Symbol(Symbol::NotEqual)),
                    |left, _, right, span| {
                        Expression::Logic(
                            Box::new(Logic::NotEqual(left, right)).with_position(span),
                        )
                    },
                ),
                infix(
                    left(1),
                    just(Token::Symbol(Symbol::Greater)),
                    |left, _, right, span| {
                        Expression::Logic(Box::new(Logic::Greater(left, right)).with_position(span))
                    },
                ),
                infix(
                    left(1),
                    just(Token::Symbol(Symbol::Less)),
                    |left, _, right, span| {
                        Expression::Logic(Box::new(Logic::Less(left, right)).with_position(span))
                    },
                ),
                infix(
                    left(1),
                    just(Token::Symbol(Symbol::GreaterOrEqual)),
                    |left, _, right, span| {
                        Expression::Logic(
                            Box::new(Logic::GreaterOrEqual(left, right)).with_position(span),
                        )
                    },
                ),
                infix(
                    left(1),
                    just(Token::Symbol(Symbol::LessOrEqual)),
                    |left, _, right, span| {
                        Expression::Logic(
                            Box::new(Logic::LessOrEqual(left, right)).with_position(span),
                        )
                    },
                ),
                infix(
                    left(1),
                    just(Token::Symbol(Symbol::DoubleAmpersand)),
                    |left, _, right, span| {
                        Expression::Logic(Box::new(Logic::And(left, right)).with_position(span))
                    },
                ),
                infix(
                    left(1),
                    just(Token::Symbol(Symbol::DoublePipe)),
                    |left, _, right, span| {
                        Expression::Logic(Box::new(Logic::Or(left, right)).with_position(span))
                    },
                ),
                // Math
                infix(
                    left(1),
                    just(Token::Symbol(Symbol::Plus)),
                    |left, _, right, span| {
                        Expression::Math(Box::new(Math::Add(left, right)).with_position(span))
                    },
                ),
                infix(
                    left(1),
                    just(Token::Symbol(Symbol::Minus)),
                    |left, _, right, span| {
                        Expression::Math(Box::new(Math::Subtract(left, right)).with_position(span))
                    },
                ),
                infix(
                    left(2),
                    just(Token::Symbol(Symbol::Asterisk)),
                    |left, _, right, span| {
                        Expression::Math(Box::new(Math::Multiply(left, right)).with_position(span))
                    },
                ),
                infix(
                    left(2),
                    just(Token::Symbol(Symbol::Slash)),
                    |left, _, right, span| {
                        Expression::Math(Box::new(Math::Divide(left, right)).with_position(span))
                    },
                ),
                infix(
                    left(1),
                    just(Token::Symbol(Symbol::Slash)),
                    |left, _, right, span| {
                        Expression::Math(Box::new(Math::Modulo(left, right)).with_position(span))
                    },
                ),
                // Indexes
                infix(
                    left(4),
                    just(Token::Symbol(Symbol::Dot)),
                    |left, _, right, span| {
                        Expression::MapIndex(
                            Box::new(MapIndex::new(left, right)).with_position(span),
                        )
                    },
                ),
                postfix(
                    3,
                    expression.clone().delimited_by(
                        just(Token::Symbol(Symbol::SquareOpen)),
                        just(Token::Symbol(Symbol::SquareClose)),
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
                            .separated_by(just(Token::Symbol(Symbol::Comma)))
                            .collect()
                            .delimited_by(
                                just(Token::Symbol(Symbol::ParenOpen)),
                                just(Token::Symbol(Symbol::ParenClose)),
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
                just(Token::Symbol(Symbol::CurlyOpen)),
                just(Token::Symbol(Symbol::CurlyClose)),
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
                just(Token::Symbol(Symbol::Equal)).to(AssignmentOperator::Assign),
                just(Token::Symbol(Symbol::PlusEquals)).to(AssignmentOperator::AddAssign),
                just(Token::Symbol(Symbol::MinusEqual)).to(AssignmentOperator::SubAssign),
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
                just(Token::Keyword(Keyword::Loop)).then(just(Token::Symbol(Symbol::CurlyOpen))),
                just(Token::Symbol(Symbol::CurlyClose)),
            )
            .map_with(|statements, state| {
                Statement::Loop(Loop::new(statements).with_position(state.span()))
            });

        let r#while = just(Token::Keyword(Keyword::While))
            .ignore_then(expression.clone())
            .then(statement.clone().repeated().collect().delimited_by(
                just(Token::Symbol(Symbol::CurlyOpen)),
                just(Token::Symbol(Symbol::CurlyClose)),
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
                    .at_least(1)
                    .collect()
                    .or_not(),
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

        let type_alias = just(Token::Keyword(Keyword::Type))
            .ignore_then(positioned_identifier.clone())
            .then_ignore(just(Token::Symbol(Symbol::Equal)))
            .then(type_constructor.clone())
            .map_with(|(identifier, constructor), state| {
                Statement::TypeAlias(
                    TypeAlias::new(identifier, constructor).with_position(state.span()),
                )
            });

        let enum_variant = positioned_identifier
            .clone()
            .then(
                type_constructor
                    .clone()
                    .separated_by(just(Token::Symbol(Symbol::Comma)))
                    .collect()
                    .delimited_by(
                        just(Token::Symbol(Symbol::ParenOpen)),
                        just(Token::Symbol(Symbol::ParenClose)),
                    )
                    .or_not(),
            )
            .map(|(identifier, constructors)| EnumVariant {
                name: identifier,
                content: constructors,
            });

        let enum_declaration = just(Token::Keyword(Keyword::Enum))
            .ignore_then(positioned_identifier.clone())
            .then(
                positioned_identifier
                    .clone()
                    .separated_by(just(Token::Symbol(Symbol::Comma)))
                    .allow_trailing()
                    .collect()
                    .delimited_by(
                        just(Token::Symbol(Symbol::Less)),
                        just(Token::Symbol(Symbol::Greater)),
                    )
                    .or_not(),
            )
            .then(
                enum_variant
                    .separated_by(just(Token::Symbol(Symbol::Comma)))
                    .allow_trailing()
                    .at_least(1)
                    .collect()
                    .delimited_by(
                        just(Token::Symbol(Symbol::CurlyOpen)),
                        just(Token::Symbol(Symbol::CurlyClose)),
                    ),
            )
            .map_with(|((name, type_parameters), variants), state| {
                Statement::EnumDeclaration(
                    EnumDeclaration::new(name, type_parameters, variants)
                        .with_position(state.span()),
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
                type_alias,
                enum_declaration,
            )))
            .then_ignore(just(Token::Symbol(Symbol::Semicolon)).or_not())
    });

    statement.repeated().collect()
}
