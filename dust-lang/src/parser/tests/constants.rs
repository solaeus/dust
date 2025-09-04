use crate::{
    Span,
    parser::parse_main,
    resolver::TypeId,
    syntax_tree::{SyntaxKind, SyntaxNode},
    tests::constant_cases,
};

#[test]
fn boolean() {
    let source = constant_cases::BOOLEAN;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 4),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (true as u32, 0),
                position: Span(0, 4),
                payload: TypeId::BOOLEAN.0,
            }
        ]
    );
}

#[test]
fn byte() {
    let source = constant_cases::BYTE;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 4),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (42, 0),
                position: Span(0, 4),
                payload: TypeId::BYTE.0,
            }
        ]
    );
}

#[test]
fn character() {
    let source = constant_cases::CHARACTER;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 3),
                payload: TypeId::CHARACTER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (113, 0),
                position: Span(0, 3),
                payload: TypeId::CHARACTER.0,
            }
        ]
    );
}

#[test]
fn float() {
    let source = constant_cases::FLOAT;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 4),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(42.0),
                position: Span(0, 4),
                payload: TypeId::FLOAT.0,
            }
        ]
    );
}

#[test]
fn integer() {
    let source = constant_cases::INTEGER;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 2),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                position: Span(0, 2),
                payload: TypeId::INTEGER.0,
            }
        ]
    );
}

#[test]
fn string() {
    let source = constant_cases::STRING;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 8),
                payload: TypeId::STRING.0,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 6),
                position: Span(0, 8),
                payload: TypeId::STRING.0,
            }
        ]
    );
}

#[test]
fn constant_byte_addition() {
    let source = constant_cases::CONSTANT_BYTE_ADDITION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 11),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (40, 0),
                position: Span(0, 4),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                children: (1, 2),
                position: Span(0, 11),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (2, 0),
                position: Span(7, 11),
                payload: TypeId::BYTE.0,
            }
        ]
    );
}

#[test]
fn constant_float_addition() {
    let source = constant_cases::CONSTANT_FLOAT_ADDITION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 10),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(40.0),
                position: Span(0, 4),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                children: (1, 2),
                position: Span(0, 10),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(2.0),
                position: Span(7, 10),
                payload: TypeId::FLOAT.0,
            }
        ]
    );
}

#[test]
fn constant_integer_addition() {
    let source = constant_cases::CONSTANT_INTEGER_ADDITION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 6),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (40, 0),
                position: Span(0, 2),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                children: (1, 2),
                position: Span(0, 6),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (2, 0),
                position: Span(5, 6),
                payload: TypeId::INTEGER.0,
            }
        ]
    );
}

#[test]
fn constant_byte_subtraction() {
    let source = constant_cases::CONSTANT_BYTE_SUBTRACTION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 11),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (44, 0),
                position: Span(0, 4),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::SubtractionExpression,
                children: (1, 2),
                position: Span(0, 11),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (2, 0),
                position: Span(7, 11),
                payload: TypeId::BYTE.0,
            }
        ]
    );
}

#[test]
fn constant_float_subtraction() {
    let source = constant_cases::CONSTANT_FLOAT_SUBTRACTION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 10),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(44.0),
                position: Span(0, 4),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::SubtractionExpression,
                children: (1, 2),
                position: Span(0, 10),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(2.0),
                position: Span(7, 10),
                payload: TypeId::FLOAT.0,
            }
        ]
    );
}

#[test]
fn constant_integer_subtraction() {
    let source = constant_cases::CONSTANT_INTEGER_SUBTRACTION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 6),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (44, 0),
                position: Span(0, 2),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::SubtractionExpression,
                children: (1, 2),
                position: Span(0, 6),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (2, 0),
                position: Span(5, 6),
                payload: TypeId::INTEGER.0,
            }
        ]
    );
}

#[test]
fn constant_byte_multiplication() {
    let source = constant_cases::CONSTANT_BYTE_MULTIPLICATION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 11),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (14, 0),
                position: Span(0, 4),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::MultiplicationExpression,
                children: (1, 2),
                position: Span(0, 11),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (3, 0),
                position: Span(7, 11),
                payload: TypeId::BYTE.0,
            }
        ]
    );
}

#[test]
fn constant_float_multiplication() {
    let source = constant_cases::CONSTANT_FLOAT_MULTIPLICATION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 10),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(14.0),
                position: Span(0, 4),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::MultiplicationExpression,
                children: (1, 2),
                position: Span(0, 10),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(3.0),
                position: Span(7, 10),
                payload: TypeId::FLOAT.0,
            }
        ]
    );
}

#[test]
fn constant_integer_multiplication() {
    let source = constant_cases::CONSTANT_INTEGER_MULTIPLICATION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 6),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (14, 0),
                position: Span(0, 2),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::MultiplicationExpression,
                children: (1, 2),
                position: Span(0, 6),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (3, 0),
                position: Span(5, 6),
                payload: TypeId::INTEGER.0,
            }
        ]
    );
}

#[test]
fn constant_byte_division() {
    let source = constant_cases::CONSTANT_BYTE_DIVISION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 11),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (84, 0),
                position: Span(0, 4),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::DivisionExpression,
                children: (1, 2),
                position: Span(0, 11),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (2, 0),
                position: Span(7, 11),
                payload: TypeId::BYTE.0,
            }
        ]
    );
}

#[test]
fn constant_float_division() {
    let source = constant_cases::CONSTANT_FLOAT_DIVISION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 10),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(84.0),
                position: Span(0, 4),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::DivisionExpression,
                children: (1, 2),
                position: Span(0, 10),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(2.0),
                position: Span(7, 10),
                payload: TypeId::FLOAT.0,
            }
        ]
    );
}

#[test]
fn constant_integer_division() {
    let source = constant_cases::CONSTANT_INTEGER_DIVISION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 6),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (84, 0),
                position: Span(0, 2),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::DivisionExpression,
                children: (1, 2),
                position: Span(0, 6),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (2, 0),
                position: Span(5, 6),
                payload: TypeId::INTEGER.0,
            }
        ]
    );
}

#[test]
fn constant_string_concatenation() {
    let source = constant_cases::CONSTANT_STRING_CONCATENATION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 13),
                payload: TypeId::STRING.0,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 3),
                position: Span(0, 5),
                payload: TypeId::STRING.0,
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                children: (1, 2),
                position: Span(0, 13),
                payload: TypeId::STRING.0,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (3, 6),
                position: Span(8, 13),
                payload: TypeId::STRING.0,
            }
        ]
    );
}

#[test]
fn constant_character_concatenation() {
    let source = constant_cases::CONSTANT_CHARACTER_CONCATENATION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 9),
                payload: TypeId::STRING.0,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (113, 0),
                position: Span(0, 3),
                payload: TypeId::CHARACTER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                children: (1, 2),
                position: Span(0, 9),
                payload: TypeId::STRING.0,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (113, 0),
                position: Span(6, 9),
                payload: TypeId::CHARACTER.0,
            }
        ]
    );
}

#[test]
fn constant_string_character_concatenation() {
    let source = constant_cases::CONSTANT_STRING_CHARACTER_CONCATENATION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 11),
                payload: TypeId::STRING.0,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 3),
                position: Span(0, 5),
                payload: TypeId::STRING.0,
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                children: (1, 2),
                position: Span(0, 11),
                payload: TypeId::STRING.0,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (113, 0),
                position: Span(8, 11),
                payload: TypeId::CHARACTER.0,
            }
        ]
    );
}

#[test]
fn constant_character_string_concatenation() {
    let source = constant_cases::CONSTANT_CHARACTER_STRING_CONCATENATION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 11),
                payload: TypeId::STRING.0,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (113, 0),
                position: Span(0, 3),
                payload: TypeId::CHARACTER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                children: (1, 2),
                position: Span(0, 11),
                payload: TypeId::STRING.0,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 3),
                position: Span(6, 11),
                payload: TypeId::STRING.0,
            }
        ]
    );
}

#[test]
fn constant_boolean_and() {
    let source = constant_cases::CONSTANT_BOOLEAN_AND;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 13),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (true as u32, 0),
                position: Span(0, 4),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::AndExpression,
                children: (1, 2),
                position: Span(0, 13),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (false as u32, 0),
                position: Span(8, 13),
                payload: TypeId::BOOLEAN.0,
            }
        ]
    );
}

#[test]
fn constant_boolean_or() {
    let source = constant_cases::CONSTANT_BOOLEAN_OR;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 13),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (true as u32, 0),
                position: Span(0, 4),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::OrExpression,
                children: (1, 2),
                position: Span(0, 13),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (false as u32, 0),
                position: Span(8, 13),
                payload: TypeId::BOOLEAN.0,
            }
        ]
    );
}

#[test]
fn constant_boolean_not() {
    let source = constant_cases::CONSTANT_BOOLEAN_NOT;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 5),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::NotExpression,
                children: (1, 0),
                position: Span(0, 5),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (true as u32, 0),
                position: Span(1, 5),
                payload: TypeId::BOOLEAN.0,
            }
        ]
    );
}

#[test]
fn constant_boolean_greater_than() {
    let source = constant_cases::CONSTANT_BOOLEAN_GREATER_THAN;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 12),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (1, 0),
                position: Span(0, 4),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                children: (1, 2),
                position: Span(0, 12),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (0, 0),
                position: Span(7, 12),
                payload: TypeId::BOOLEAN.0,
            }
        ]
    );
}

#[test]
fn constant_boolean_less_than() {
    let source = constant_cases::CONSTANT_BOOLEAN_LESS_THAN;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 12),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (0, 0),
                position: Span(0, 5),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                children: (1, 2),
                position: Span(0, 12),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (1, 0),
                position: Span(8, 12),
                payload: TypeId::BOOLEAN.0,
            }
        ]
    );
}

#[test]
fn constant_boolean_greater_than_or_equal() {
    let source = constant_cases::CONSTANT_BOOLEAN_GREATER_THAN_OR_EQUAL;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 12),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (1, 0),
                position: Span(0, 4),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanOrEqualExpression,
                children: (1, 2),
                position: Span(0, 12),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (1, 0),
                position: Span(8, 12),
                payload: TypeId::BOOLEAN.0,
            }
        ]
    );
}

#[test]
fn constant_boolean_less_than_or_equal() {
    let source = constant_cases::CONSTANT_BOOLEAN_LESS_THAN_OR_EQUAL;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 12),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (1, 0),
                position: Span(0, 4),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanOrEqualExpression,
                children: (1, 2),
                position: Span(0, 12),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (1, 0),
                position: Span(8, 12),
                payload: TypeId::BOOLEAN.0,
            }
        ]
    );
}

#[test]
fn constant_boolean_equal() {
    let source = constant_cases::CONSTANT_BOOLEAN_EQUAL;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 12),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (1, 0),
                position: Span(0, 4),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::EqualExpression,
                children: (1, 2),
                position: Span(0, 12),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (1, 0),
                position: Span(8, 12),
                payload: TypeId::BOOLEAN.0,
            }
        ]
    );
}

#[test]
fn constant_boolean_not_equal() {
    let source = constant_cases::CONSTANT_BOOLEAN_NOT_EQUAL;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 13),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (1, 0),
                position: Span(0, 4),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::NotEqualExpression,
                children: (1, 2),
                position: Span(0, 13),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (0, 0),
                position: Span(8, 13),
                payload: TypeId::BOOLEAN.0,
            }
        ]
    );
}

#[test]
fn constant_byte_greater_than() {
    let source = constant_cases::CONSTANT_BYTE_GREATER_THAN;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 11),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (43, 0),
                position: Span(0, 4),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                children: (1, 2),
                position: Span(0, 11),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (42, 0),
                position: Span(7, 11),
                payload: TypeId::BYTE.0,
            }
        ]
    );
}

#[test]
fn constant_byte_less_than() {
    let source = constant_cases::CONSTANT_BYTE_LESS_THAN;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 11),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (41, 0),
                position: Span(0, 4),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                children: (1, 2),
                position: Span(0, 11),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (42, 0),
                position: Span(7, 11),
                payload: TypeId::BYTE.0,
            }
        ]
    );
}

#[test]
fn constant_byte_greater_than_or_equal() {
    let source = constant_cases::CONSTANT_BYTE_GREATER_THAN_OR_EQUAL;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 12),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (42, 0),
                position: Span(0, 4),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanOrEqualExpression,
                children: (1, 2),
                position: Span(0, 12),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (42, 0),
                position: Span(8, 12),
                payload: TypeId::BYTE.0,
            }
        ]
    );
}

#[test]
fn constant_byte_less_than_or_equal() {
    let source = constant_cases::CONSTANT_BYTE_LESS_THAN_OR_EQUAL;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 12),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (42, 0),
                position: Span(0, 4),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanOrEqualExpression,
                children: (1, 2),
                position: Span(0, 12),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (42, 0),
                position: Span(8, 12),
                payload: TypeId::BYTE.0,
            }
        ]
    );
}

#[test]
fn constant_byte_equal() {
    let source = constant_cases::CONSTANT_BYTE_EQUAL;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 12),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (42, 0),
                position: Span(0, 4),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::EqualExpression,
                children: (1, 2),
                position: Span(0, 12),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (42, 0),
                position: Span(8, 12),
                payload: TypeId::BYTE.0,
            }
        ]
    );
}

#[test]
fn constant_byte_not_equal() {
    let source = constant_cases::CONSTANT_BYTE_NOT_EQUAL;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 12),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (42, 0),
                position: Span(0, 4),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::NotEqualExpression,
                children: (1, 2),
                position: Span(0, 12),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (43, 0),
                position: Span(8, 12),
                payload: TypeId::BYTE.0,
            }
        ]
    );
}

#[test]
fn constant_character_greater_than() {
    let source = constant_cases::CONSTANT_CHARACTER_GREATER_THAN;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 9),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (123, 0),
                position: Span(0, 3),
                payload: TypeId::CHARACTER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                children: (1, 2),
                position: Span(0, 9),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (122, 0),
                position: Span(6, 9),
                payload: TypeId::CHARACTER.0,
            }
        ]
    );
}

#[test]
fn constant_character_less_than() {
    let source = constant_cases::CONSTANT_CHARACTER_LESS_THAN;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 9),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (121, 0),
                position: Span(0, 3),
                payload: TypeId::CHARACTER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                children: (1, 2),
                position: Span(0, 9),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (122, 0),
                position: Span(6, 9),
                payload: TypeId::CHARACTER.0,
            }
        ]
    );
}

#[test]
fn constant_character_greater_than_or_equal() {
    let source = constant_cases::CONSTANT_CHARACTER_GREATER_THAN_OR_EQUAL;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 10),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (122, 0),
                position: Span(0, 3),
                payload: TypeId::CHARACTER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanOrEqualExpression,
                children: (1, 2),
                position: Span(0, 10),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (122, 0),
                position: Span(7, 10),
                payload: TypeId::CHARACTER.0,
            }
        ]
    );
}

#[test]
fn constant_character_less_than_or_equal() {
    let source = constant_cases::CONSTANT_CHARACTER_LESS_THAN_OR_EQUAL;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 10),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (122, 0),
                position: Span(0, 3),
                payload: TypeId::CHARACTER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanOrEqualExpression,
                children: (1, 2),
                position: Span(0, 10),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (122, 0),
                position: Span(7, 10),
                payload: TypeId::CHARACTER.0,
            }
        ]
    );
}

#[test]
fn constant_character_equal() {
    let source = constant_cases::CONSTANT_CHARACTER_EQUAL;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 10),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (122, 0),
                position: Span(0, 3),
                payload: TypeId::CHARACTER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::EqualExpression,
                children: (1, 2),
                position: Span(0, 10),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (122, 0),
                position: Span(7, 10),
                payload: TypeId::CHARACTER.0,
            }
        ]
    );
}

#[test]
fn constant_character_not_equal() {
    let source = constant_cases::CONSTANT_CHARACTER_NOT_EQUAL;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 10),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (122, 0),
                position: Span(0, 3),
                payload: TypeId::CHARACTER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::NotEqualExpression,
                children: (1, 2),
                position: Span(0, 10),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (123, 0),
                position: Span(7, 10),
                payload: TypeId::CHARACTER.0,
            }
        ]
    );
}

#[test]
fn constant_float_greater_than() {
    let source = constant_cases::CONSTANT_FLOAT_GREATER_THAN;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 11),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(43.0),
                position: Span(0, 4),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                children: (1, 2),
                position: Span(0, 11),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(42.0),
                position: Span(7, 11),
                payload: TypeId::FLOAT.0,
            }
        ]
    );
}

#[test]
fn constant_float_less_than() {
    let source = constant_cases::CONSTANT_FLOAT_LESS_THAN;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 11),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(41.0),
                position: Span(0, 4),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                children: (1, 2),
                position: Span(0, 11),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(42.0),
                position: Span(7, 11),
                payload: TypeId::FLOAT.0,
            }
        ]
    );
}

#[test]
fn constant_float_greater_than_or_equal() {
    let source = constant_cases::CONSTANT_FLOAT_GREATER_THAN_OR_EQUAL;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 12),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(42.0),
                position: Span(0, 4),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanOrEqualExpression,
                children: (1, 2),
                position: Span(0, 12),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(42.0),
                position: Span(8, 12),
                payload: TypeId::FLOAT.0,
            }
        ]
    );
}

#[test]
fn constant_float_less_than_or_equal() {
    let source = constant_cases::CONSTANT_FLOAT_LESS_THAN_OR_EQUAL;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 12),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(42.0),
                position: Span(0, 4),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanOrEqualExpression,
                children: (1, 2),
                position: Span(0, 12),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(42.0),
                position: Span(8, 12),
                payload: TypeId::FLOAT.0,
            }
        ]
    );
}

#[test]
fn constant_float_equal() {
    let source = constant_cases::CONSTANT_FLOAT_EQUAL;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 12),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(42.0),
                position: Span(0, 4),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::EqualExpression,
                children: (1, 2),
                position: Span(0, 12),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(42.0),
                position: Span(8, 12),
                payload: TypeId::FLOAT.0,
            }
        ]
    );
}

#[test]
fn constant_float_not_equal() {
    let source = constant_cases::CONSTANT_FLOAT_NOT_EQUAL;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 12),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(42.0),
                position: Span(0, 4),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::NotEqualExpression,
                children: (1, 2),
                position: Span(0, 12),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(43.0),
                position: Span(8, 12),
                payload: TypeId::FLOAT.0,
            }
        ]
    );
}

#[test]
fn constant_integer_greater_than() {
    let source = constant_cases::CONSTANT_INTEGER_GREATER_THAN;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 7),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (43, 0),
                position: Span(0, 2),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                children: (1, 2),
                position: Span(0, 7),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                position: Span(5, 7),
                payload: TypeId::INTEGER.0,
            }
        ]
    );
}

#[test]
fn constant_integer_less_than() {
    let source = constant_cases::CONSTANT_INTEGER_LESS_THAN;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 7),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (41, 0),
                position: Span(0, 2),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                children: (1, 2),
                position: Span(0, 7),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                position: Span(5, 7),
                payload: TypeId::INTEGER.0,
            }
        ]
    );
}

#[test]
fn constant_integer_greater_than_or_equal() {
    let source = constant_cases::CONSTANT_INTEGER_GREATER_THAN_OR_EQUAL;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 8),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                position: Span(0, 2),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanOrEqualExpression,
                children: (1, 2),
                position: Span(0, 8),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                position: Span(6, 8),
                payload: TypeId::INTEGER.0,
            }
        ]
    );
}

#[test]
fn constant_integer_less_than_or_equal() {
    let source = constant_cases::CONSTANT_INTEGER_LESS_THAN_OR_EQUAL;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 8),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                position: Span(0, 2),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanOrEqualExpression,
                children: (1, 2),
                position: Span(0, 8),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                position: Span(6, 8),
                payload: TypeId::INTEGER.0,
            }
        ]
    );
}

#[test]
fn constant_integer_equal() {
    let source = constant_cases::CONSTANT_INTEGER_EQUAL;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 8),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                position: Span(0, 2),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::EqualExpression,
                children: (1, 2),
                position: Span(0, 8),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                position: Span(6, 8),
                payload: TypeId::INTEGER.0,
            }
        ]
    );
}

#[test]
fn constant_integer_not_equal() {
    let source = constant_cases::CONSTANT_INTEGER_NOT_EQUAL;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 8),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                position: Span(0, 2),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::NotEqualExpression,
                children: (1, 2),
                position: Span(0, 8),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (43, 0),
                position: Span(6, 8),
                payload: TypeId::INTEGER.0,
            }
        ]
    );
}

#[test]
fn constant_string_greater_than() {
    let source = constant_cases::CONSTANT_STRING_GREATER_THAN;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 13),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 3),
                position: Span(0, 5),
                payload: TypeId::STRING.0,
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                children: (1, 2),
                position: Span(0, 13),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (3, 6),
                position: Span(8, 13),
                payload: TypeId::STRING.0,
            }
        ]
    );
}

#[test]
fn constant_string_less_than() {
    let source = constant_cases::CONSTANT_STRING_LESS_THAN;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 13),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 3),
                position: Span(0, 5),
                payload: TypeId::STRING.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                children: (1, 2),
                position: Span(0, 13),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (3, 6),
                position: Span(8, 13),
                payload: TypeId::STRING.0,
            }
        ]
    );
}

#[test]
fn constant_string_greater_than_or_equal() {
    let source = constant_cases::CONSTANT_STRING_GREATER_THAN_OR_EQUAL;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 14),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 3),
                position: Span(0, 5),
                payload: TypeId::STRING.0,
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanOrEqualExpression,
                children: (1, 2),
                position: Span(0, 14),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 3),
                position: Span(9, 14),
                payload: TypeId::STRING.0,
            }
        ]
    );
}

#[test]
fn constant_string_less_than_or_equal() {
    let source = constant_cases::CONSTANT_STRING_LESS_THAN_OR_EQUAL;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 14),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 3),
                position: Span(0, 5),
                payload: TypeId::STRING.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanOrEqualExpression,
                children: (1, 2),
                position: Span(0, 14),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 3),
                position: Span(9, 14),
                payload: TypeId::STRING.0,
            }
        ]
    );
}

#[test]
fn constant_string_equal() {
    let source = constant_cases::CONSTANT_STRING_EQUAL;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 14),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 3),
                position: Span(0, 5),
                payload: TypeId::STRING.0,
            },
            SyntaxNode {
                kind: SyntaxKind::EqualExpression,
                children: (1, 2),
                position: Span(0, 14),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 3),
                position: Span(9, 14),
                payload: TypeId::STRING.0,
            }
        ]
    );
}

#[test]
fn constant_string_not_equal() {
    let source = constant_cases::CONSTANT_STRING_NOT_EQUAL;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 14),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 3),
                position: Span(0, 5),
                payload: TypeId::STRING.0,
            },
            SyntaxNode {
                kind: SyntaxKind::NotEqualExpression,
                children: (1, 2),
                position: Span(0, 14),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (3, 6),
                position: Span(9, 14),
                payload: TypeId::STRING.0,
            }
        ]
    );
}
