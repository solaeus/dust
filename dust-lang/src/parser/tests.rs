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
                        RawTypeConstructor::Integer.with_position((7, 10))
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
fn enum_declaration() {
    assert_eq!(
        parse(&lex("enum MyEnum { X, Y }").unwrap()).unwrap()[0],
        Statement::EnumDeclaration(
            EnumDeclaration::new(
                Identifier::new("MyEnum").with_position((5, 11)),
                None,
                vec![
                    EnumVariant {
                        name: Identifier::new("X").with_position((14, 15)),
                        content: None
                    },
                    EnumVariant {
                        name: Identifier::new("Y").with_position((17, 18)),
                        content: None
                    }
                ],
            )
            .with_position((0, 20))
        )
    );
}

#[test]
fn enum_with_contents() {
    assert_eq!(
        parse(&lex("enum MyEnum { X(str, int), Y(int) }").unwrap()).unwrap()[0],
        Statement::EnumDeclaration(
            EnumDeclaration::new(
                Identifier::new("MyEnum").with_position((5, 11)),
                None,
                vec![
                    EnumVariant {
                        name: Identifier::new("X").with_position((14, 15)),
                        content: Some(vec![
                            TypeConstructor::Raw(
                                RawTypeConstructor::String.with_position((16, 19))
                            ),
                            TypeConstructor::Raw(
                                RawTypeConstructor::Integer.with_position((21, 24))
                            ),
                        ])
                    },
                    EnumVariant {
                        name: Identifier::new("Y").with_position((27, 28)),
                        content: Some(vec![TypeConstructor::Raw(
                            RawTypeConstructor::Integer.with_position((29, 32))
                        ),])
                    }
                ]
            )
            .with_position((0, 35))
        )
    );
}

#[test]
fn enum_with_type_parameters() {
    assert_eq!(
        parse(&lex("enum MyEnum <T, U> { X(T), Y(U) }").unwrap()).unwrap()[0],
        Statement::EnumDeclaration(
            EnumDeclaration::new(
                Identifier::new("MyEnum").with_position((5, 11)),
                Some(vec![
                    Identifier::new("T").with_position((13, 14)),
                    Identifier::new("U").with_position((16, 17))
                ]),
                vec![
                    EnumVariant {
                        name: Identifier::new("X").with_position((21, 22)),
                        content: Some(vec![TypeConstructor::Invokation(
                            TypeInvokationConstructor {
                                identifier: Identifier::new("T").with_position((23, 24)),
                                type_arguments: None
                            }
                        )])
                    },
                    EnumVariant {
                        name: Identifier::new("Y").with_position((27, 28)),
                        content: Some(vec![TypeConstructor::Invokation(
                            TypeInvokationConstructor {
                                identifier: Identifier::new("U").with_position((29, 30)),
                                type_arguments: None
                            }
                        )])
                    },
                ]
            )
            .with_position((0, 33))
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
fn r#as() {
    assert_eq!(
        parse(&lex("1 as str").unwrap()).unwrap()[0],
        Statement::Expression(Expression::As(
            Box::new(As::new(
                Expression::Value(ValueNode::Integer(1).with_position((0, 1))),
                TypeConstructor::Raw(RawTypeConstructor::String.with_position((5, 8)))
            ))
            .with_position((0, 8))
        ))
    )
}

#[test]
fn built_in_function() {
    let tokens = lex("__READ_LINE__").unwrap();
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
        Statement::Expression(Expression::BuiltIn(
            BuiltInFunctionCall::read_line().with_position((0, 13))
        ))
    );
}

#[test]
fn built_in_function_with_arg() {
    let tokens = lex("__WRITE_LINE__ 'hiya'").unwrap();
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
        Statement::Expression(Expression::BuiltIn(
            BuiltInFunctionCall::write_line(Expression::Value(
                ValueNode::String("hiya".to_string()).with_position((15, 21))
            ))
            .with_position((0, 21))
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
                        Expression::Identifier(Identifier::new("output").with_position((13, 19))),
                        None,
                        Some(vec![Expression::Value(
                            ValueNode::String("hi".to_string()).with_position((20, 24))
                        )])
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
                Some(TypeConstructor::Raw(
                    RawTypeConstructor::Boolean.with_position((9, 13))
                )),
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
                            RawTypeConstructor::Integer.with_position((9, 12))
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
                    Box::new(TypeConstructor::Raw(
                        RawTypeConstructor::Boolean.with_position((10, 14))
                    ))
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
        parse(&lex("type Foo = fn |T| (x: int)").unwrap()).unwrap()[0],
        Statement::TypeAlias(
            TypeAlias::new(
                Identifier::new("Foo").with_position((5, 8)),
                TypeConstructor::Function(
                    FunctionTypeConstructor {
                        type_parameters: Some(vec![Identifier::new("T").with_position((15, 16))]),
                        value_parameters: Some(vec![(
                            Identifier::new("x").with_position((19, 20)),
                            TypeConstructor::Raw(
                                RawTypeConstructor::Integer.with_position((22, 25))
                            )
                        )]),
                        return_type: None
                    }
                    .with_position((11, 26))
                )
            )
            .with_position((0, 26))
        )
    );
}

#[test]
fn function_type_with_return() {
    assert_eq!(
        parse(&lex("type Foo = fn |T| (x: int) -> T").unwrap()).unwrap()[0],
        Statement::TypeAlias(
            TypeAlias::new(
                Identifier::new("Foo").with_position((5, 8)),
                TypeConstructor::Function(
                    FunctionTypeConstructor {
                        type_parameters: Some(vec![Identifier::new("T").with_position((15, 16))]),
                        value_parameters: Some(vec![(
                            Identifier::new("x").with_position((19, 20)),
                            TypeConstructor::Raw(
                                RawTypeConstructor::Integer.with_position((22, 25))
                            )
                        )]),
                        return_type: Some(Box::new(TypeConstructor::Invokation(
                            TypeInvokationConstructor {
                                identifier: Identifier::new("T").with_position((30, 31)),
                                type_arguments: None
                            }
                        )))
                    }
                    .with_position((11, 31))
                )
            )
            .with_position((0, 31))
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
                None,
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
                    RawTypeConstructor::String.with_position((9, 12))
                )]),
                Some(vec![Expression::Value(
                    ValueNode::String("hi".to_string()).with_position((16, 20))
                )]),
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
            ValueNode::function(
                None,
                None,
                Some(TypeConstructor::Raw(
                    RawTypeConstructor::Integer.with_position((9, 12))
                )),
                Block::new(vec![Statement::Expression(Expression::Value(
                    ValueNode::Integer(0).with_position((15, 16))
                ))])
                .with_position((13, 18)),
            )
            .with_position((0, 18))
        ),)
    );

    assert_eq!(
        parse(&lex("fn (x: int) -> int { x }").unwrap()).unwrap()[0],
        Statement::Expression(Expression::Value(
            ValueNode::function(
                None,
                Some(vec![(
                    Identifier::new("x"),
                    TypeConstructor::Raw(RawTypeConstructor::Integer.with_position((7, 10)))
                )]),
                Some(TypeConstructor::Raw(
                    RawTypeConstructor::Integer.with_position((15, 18))
                )),
                Block::new(vec![Statement::Expression(Expression::Identifier(
                    Identifier::new("x").with_position((21, 22))
                ))])
                .with_position((19, 24)),
            )
            .with_position((0, 24))
        ),)
    );
}

#[test]
fn function_with_type_arguments() {
    assert_eq!(
        parse(&lex("fn |T, U| (x: T, y: U) -> T { x }").unwrap()).unwrap()[0],
        Statement::Expression(Expression::Value(
            ValueNode::function(
                Some(vec![Identifier::new("T"), Identifier::new("U"),]),
                Some(vec![
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
                ]),
                Some(TypeConstructor::Invokation(TypeInvokationConstructor {
                    identifier: Identifier::new("T").with_position((26, 27)),
                    type_arguments: None,
                })),
                Block::new(vec![Statement::Expression(Expression::Identifier(
                    Identifier::new("x").with_position((30, 31))
                ))])
                .with_position((28, 33)),
            )
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
                None,
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
                None,
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
        ))
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
}

#[test]
fn complex_loop() {
    assert_eq!(
        parse(&lex("loop { if i > 2 { break } else { i += 1 } }").unwrap()).unwrap()[0],
        Statement::Loop(
            Loop::new(vec![Statement::IfElse(
                IfElse::new(
                    Expression::Logic(
                        Box::new(Logic::Greater(
                            Expression::Identifier(Identifier::new("i").with_position((10, 11))),
                            Expression::Value(ValueNode::Integer(2).with_position((14, 15)))
                        ))
                        .with_position((10, 15))
                    ),
                    Block::new(vec![Statement::Break(().with_position((18, 23)))])
                        .with_position((16, 25)),
                    None,
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
                Some(TypeConstructor::Raw(
                    RawTypeConstructor::Integer.with_position((8, 11))
                )),
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
                                Expression::Identifier(Identifier::new("x").with_position((1, 2))),
                                Expression::Value(ValueNode::Integer(1).with_position((6, 7)))
                            ))
                            .with_position((1, 7))
                        ),
                        Expression::Logic(
                            Box::new(Logic::Equal(
                                Expression::Identifier(
                                    Identifier::new("y").with_position((13, 14))
                                ),
                                Expression::Value(ValueNode::Integer(2).with_position((18, 19)))
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
