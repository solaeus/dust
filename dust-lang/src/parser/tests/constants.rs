use crate::{
    instruction::OperandType,
    parser::parse_main,
    source::Span,
    syntax_tree::{SyntaxKind, SyntaxNode},
    tests::constant_cases,
};

#[test]
fn boolean() {
    let source = constant_cases::BOOLEAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 4),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (true as u32, 0),
                span: Span(0, 4),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn byte() {
    let source = constant_cases::BYTE.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 4),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (42, 0),
                span: Span(0, 4),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn character() {
    let source = constant_cases::CHARACTER.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 3),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (113, 0),
                span: Span(0, 3),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn float() {
    let source = constant_cases::FLOAT.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 4),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(42.0),
                span: Span(0, 4),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn integer() {
    let source = constant_cases::INTEGER.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 2),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                span: Span(0, 2),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn string() {
    let source = constant_cases::STRING.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 8),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(0, 8),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_byte_addition() {
    let source = constant_cases::CONSTANT_BYTE_ADDITION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 11),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (40, 0),
                span: Span(0, 4),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                children: (1, 2),
                span: Span(0, 11),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (2, 0),
                span: Span(7, 11),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_float_addition() {
    let source = constant_cases::CONSTANT_FLOAT_ADDITION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 10),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(40.0),
                span: Span(0, 4),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                children: (1, 2),
                span: Span(0, 10),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(2.0),
                span: Span(7, 10),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_integer_addition() {
    let source = constant_cases::CONSTANT_INTEGER_ADDITION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 6),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (40, 0),
                span: Span(0, 2),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                children: (1, 2),
                span: Span(0, 6),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (2, 0),
                span: Span(5, 6),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_byte_subtraction() {
    let source = constant_cases::CONSTANT_BYTE_SUBTRACTION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 11),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (44, 0),
                span: Span(0, 4),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::SubtractionExpression,
                children: (1, 2),
                span: Span(0, 11),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (2, 0),
                span: Span(7, 11),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_float_subtraction() {
    let source = constant_cases::CONSTANT_FLOAT_SUBTRACTION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 10),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(44.0),
                span: Span(0, 4),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::SubtractionExpression,
                children: (1, 2),
                span: Span(0, 10),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(2.0),
                span: Span(7, 10),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_integer_subtraction() {
    let source = constant_cases::CONSTANT_INTEGER_SUBTRACTION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 6),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (44, 0),
                span: Span(0, 2),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::SubtractionExpression,
                children: (1, 2),
                span: Span(0, 6),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (2, 0),
                span: Span(5, 6),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_byte_multiplication() {
    let source = constant_cases::CONSTANT_BYTE_MULTIPLICATION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 11),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (14, 0),
                span: Span(0, 4),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::MultiplicationExpression,
                children: (1, 2),
                span: Span(0, 11),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (3, 0),
                span: Span(7, 11),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_float_multiplication() {
    let source = constant_cases::CONSTANT_FLOAT_MULTIPLICATION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 10),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(14.0),
                span: Span(0, 4),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::MultiplicationExpression,
                children: (1, 2),
                span: Span(0, 10),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(3.0),
                span: Span(7, 10),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_integer_multiplication() {
    let source = constant_cases::CONSTANT_INTEGER_MULTIPLICATION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 6),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (14, 0),
                span: Span(0, 2),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::MultiplicationExpression,
                children: (1, 2),
                span: Span(0, 6),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (3, 0),
                span: Span(5, 6),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_byte_division() {
    let source = constant_cases::CONSTANT_BYTE_DIVISION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 11),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (84, 0),
                span: Span(0, 4),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::DivisionExpression,
                children: (1, 2),
                span: Span(0, 11),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (2, 0),
                span: Span(7, 11),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_float_division() {
    let source = constant_cases::CONSTANT_FLOAT_DIVISION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 10),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(84.0),
                span: Span(0, 4),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::DivisionExpression,
                children: (1, 2),
                span: Span(0, 10),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(2.0),
                span: Span(7, 10),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_integer_division() {
    let source = constant_cases::CONSTANT_INTEGER_DIVISION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 6),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (84, 0),
                span: Span(0, 2),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::DivisionExpression,
                children: (1, 2),
                span: Span(0, 6),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (2, 0),
                span: Span(5, 6),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_integer_negation() {
    let source = constant_cases::CONSTANT_INTEGER_NEGATION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 5),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::NegationExpression,
                children: (2, 0),
                span: Span(0, 5),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::GroupedExpression,
                children: (1, 0),
                span: Span(1, 5),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                span: Span(2, 4),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_float_negation() {
    let source = constant_cases::CONSTANT_FLOAT_NEGATION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 7),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::NegationExpression,
                children: (2, 0),
                span: Span(0, 7),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::GroupedExpression,
                children: (1, 0),
                span: Span(1, 7),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(42.0),
                span: Span(2, 6),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_string_concatenation() {
    let source = constant_cases::CONSTANT_STRING_CONCATENATION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 13),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(0, 5),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                children: (1, 2),
                span: Span(0, 13),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(8, 13),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_character_concatenation() {
    let source = constant_cases::CONSTANT_CHARACTER_CONCATENATION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 9),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (113, 0),
                span: Span(0, 3),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                children: (1, 2),
                span: Span(0, 9),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (113, 0),
                span: Span(6, 9),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_string_character_concatenation() {
    let source = constant_cases::CONSTANT_STRING_CHARACTER_CONCATENATION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 11),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(0, 5),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                children: (1, 2),
                span: Span(0, 11),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (113, 0),
                span: Span(8, 11),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_character_string_concatenation() {
    let source = constant_cases::CONSTANT_CHARACTER_STRING_CONCATENATION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 11),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (113, 0),
                span: Span(0, 3),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                children: (1, 2),
                span: Span(0, 11),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(6, 11),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_boolean_and() {
    let source = constant_cases::CONSTANT_BOOLEAN_AND.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 13),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (true as u32, 0),
                span: Span(0, 4),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::AndExpression,
                children: (1, 2),
                span: Span(0, 13),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (false as u32, 0),
                span: Span(8, 13),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_boolean_or() {
    let source = constant_cases::CONSTANT_BOOLEAN_OR.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 13),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (true as u32, 0),
                span: Span(0, 4),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::OrExpression,
                children: (1, 2),
                span: Span(0, 13),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (false as u32, 0),
                span: Span(8, 13),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_boolean_not() {
    let source = constant_cases::CONSTANT_BOOLEAN_NOT.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 5),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::NotExpression,
                children: (1, 0),
                span: Span(0, 5),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (true as u32, 0),
                span: Span(1, 5),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_boolean_greater_than() {
    let source = constant_cases::CONSTANT_BOOLEAN_GREATER_THAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 12),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (1, 0),
                span: Span(0, 4),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                children: (1, 2),
                span: Span(0, 12),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (0, 0),
                span: Span(7, 12),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_boolean_less_than() {
    let source = constant_cases::CONSTANT_BOOLEAN_LESS_THAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 12),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (0, 0),
                span: Span(0, 5),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                children: (1, 2),
                span: Span(0, 12),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (1, 0),
                span: Span(8, 12),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_boolean_greater_than_or_equal() {
    let source = constant_cases::CONSTANT_BOOLEAN_GREATER_THAN_OR_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 12),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (1, 0),
                span: Span(0, 4),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanOrEqualExpression,
                children: (1, 2),
                span: Span(0, 12),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (1, 0),
                span: Span(8, 12),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_boolean_less_than_or_equal() {
    let source = constant_cases::CONSTANT_BOOLEAN_LESS_THAN_OR_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 12),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (1, 0),
                span: Span(0, 4),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanOrEqualExpression,
                children: (1, 2),
                span: Span(0, 12),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (1, 0),
                span: Span(8, 12),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_boolean_equal() {
    let source = constant_cases::CONSTANT_BOOLEAN_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 12),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (1, 0),
                span: Span(0, 4),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::EqualExpression,
                children: (1, 2),
                span: Span(0, 12),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (1, 0),
                span: Span(8, 12),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_boolean_not_equal() {
    let source = constant_cases::CONSTANT_BOOLEAN_NOT_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 13),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (1, 0),
                span: Span(0, 4),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::NotEqualExpression,
                children: (1, 2),
                span: Span(0, 13),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (0, 0),
                span: Span(8, 13),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_byte_greater_than() {
    let source = constant_cases::CONSTANT_BYTE_GREATER_THAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 11),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (43, 0),
                span: Span(0, 4),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                children: (1, 2),
                span: Span(0, 11),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (42, 0),
                span: Span(7, 11),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_byte_less_than() {
    let source = constant_cases::CONSTANT_BYTE_LESS_THAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 11),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (41, 0),
                span: Span(0, 4),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                children: (1, 2),
                span: Span(0, 11),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (42, 0),
                span: Span(7, 11),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_byte_greater_than_or_equal() {
    let source = constant_cases::CONSTANT_BYTE_GREATER_THAN_OR_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 12),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (42, 0),
                span: Span(0, 4),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanOrEqualExpression,
                children: (1, 2),
                span: Span(0, 12),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (42, 0),
                span: Span(8, 12),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_byte_less_than_or_equal() {
    let source = constant_cases::CONSTANT_BYTE_LESS_THAN_OR_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 12),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (42, 0),
                span: Span(0, 4),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanOrEqualExpression,
                children: (1, 2),
                span: Span(0, 12),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (42, 0),
                span: Span(8, 12),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_byte_equal() {
    let source = constant_cases::CONSTANT_BYTE_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 12),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (42, 0),
                span: Span(0, 4),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::EqualExpression,
                children: (1, 2),
                span: Span(0, 12),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (42, 0),
                span: Span(8, 12),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_byte_not_equal() {
    let source = constant_cases::CONSTANT_BYTE_NOT_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 12),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (42, 0),
                span: Span(0, 4),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::NotEqualExpression,
                children: (1, 2),
                span: Span(0, 12),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (43, 0),
                span: Span(8, 12),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_character_greater_than() {
    let source = constant_cases::CONSTANT_CHARACTER_GREATER_THAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 9),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (123, 0),
                span: Span(0, 3),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                children: (1, 2),
                span: Span(0, 9),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (122, 0),
                span: Span(6, 9),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_character_less_than() {
    let source = constant_cases::CONSTANT_CHARACTER_LESS_THAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 9),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (121, 0),
                span: Span(0, 3),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                children: (1, 2),
                span: Span(0, 9),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (122, 0),
                span: Span(6, 9),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_character_greater_than_or_equal() {
    let source = constant_cases::CONSTANT_CHARACTER_GREATER_THAN_OR_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 10),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (122, 0),
                span: Span(0, 3),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanOrEqualExpression,
                children: (1, 2),
                span: Span(0, 10),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (122, 0),
                span: Span(7, 10),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_character_less_than_or_equal() {
    let source = constant_cases::CONSTANT_CHARACTER_LESS_THAN_OR_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 10),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (122, 0),
                span: Span(0, 3),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanOrEqualExpression,
                children: (1, 2),
                span: Span(0, 10),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (122, 0),
                span: Span(7, 10),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_character_equal() {
    let source = constant_cases::CONSTANT_CHARACTER_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 10),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (122, 0),
                span: Span(0, 3),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::EqualExpression,
                children: (1, 2),
                span: Span(0, 10),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (122, 0),
                span: Span(7, 10),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_character_not_equal() {
    let source = constant_cases::CONSTANT_CHARACTER_NOT_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 10),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (122, 0),
                span: Span(0, 3),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::NotEqualExpression,
                children: (1, 2),
                span: Span(0, 10),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (123, 0),
                span: Span(7, 10),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_float_greater_than() {
    let source = constant_cases::CONSTANT_FLOAT_GREATER_THAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 11),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(43.0),
                span: Span(0, 4),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                children: (1, 2),
                span: Span(0, 11),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(42.0),
                span: Span(7, 11),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_float_less_than() {
    let source = constant_cases::CONSTANT_FLOAT_LESS_THAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 11),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(41.0),
                span: Span(0, 4),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                children: (1, 2),
                span: Span(0, 11),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(42.0),
                span: Span(7, 11),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_float_greater_than_or_equal() {
    let source = constant_cases::CONSTANT_FLOAT_GREATER_THAN_OR_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 12),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(42.0),
                span: Span(0, 4),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanOrEqualExpression,
                children: (1, 2),
                span: Span(0, 12),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(42.0),
                span: Span(8, 12),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_float_less_than_or_equal() {
    let source = constant_cases::CONSTANT_FLOAT_LESS_THAN_OR_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 12),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(42.0),
                span: Span(0, 4),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanOrEqualExpression,
                children: (1, 2),
                span: Span(0, 12),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(42.0),
                span: Span(8, 12),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_float_equal() {
    let source = constant_cases::CONSTANT_FLOAT_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 12),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(42.0),
                span: Span(0, 4),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::EqualExpression,
                children: (1, 2),
                span: Span(0, 12),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(42.0),
                span: Span(8, 12),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_float_not_equal() {
    let source = constant_cases::CONSTANT_FLOAT_NOT_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 12),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(42.0),
                span: Span(0, 4),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::NotEqualExpression,
                children: (1, 2),
                span: Span(0, 12),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(43.0),
                span: Span(8, 12),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_integer_greater_than() {
    let source = constant_cases::CONSTANT_INTEGER_GREATER_THAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 7),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (43, 0),
                span: Span(0, 2),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                children: (1, 2),
                span: Span(0, 7),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                span: Span(5, 7),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_integer_less_than() {
    let source = constant_cases::CONSTANT_INTEGER_LESS_THAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 7),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (41, 0),
                span: Span(0, 2),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                children: (1, 2),
                span: Span(0, 7),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                span: Span(5, 7),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_integer_greater_than_or_equal() {
    let source = constant_cases::CONSTANT_INTEGER_GREATER_THAN_OR_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 8),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                span: Span(0, 2),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanOrEqualExpression,
                children: (1, 2),
                span: Span(0, 8),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                span: Span(6, 8),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_integer_less_than_or_equal() {
    let source = constant_cases::CONSTANT_INTEGER_LESS_THAN_OR_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 8),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                span: Span(0, 2),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanOrEqualExpression,
                children: (1, 2),
                span: Span(0, 8),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                span: Span(6, 8),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_integer_equal() {
    let source = constant_cases::CONSTANT_INTEGER_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 8),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                span: Span(0, 2),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::EqualExpression,
                children: (1, 2),
                span: Span(0, 8),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                span: Span(6, 8),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_integer_not_equal() {
    let source = constant_cases::CONSTANT_INTEGER_NOT_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 8),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                span: Span(0, 2),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::NotEqualExpression,
                children: (1, 2),
                span: Span(0, 8),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (43, 0),
                span: Span(6, 8),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_string_greater_than() {
    let source = constant_cases::CONSTANT_STRING_GREATER_THAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 13),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(0, 5),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                children: (1, 2),
                span: Span(0, 13),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(8, 13),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_string_less_than() {
    let source = constant_cases::CONSTANT_STRING_LESS_THAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 13),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(0, 5),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                children: (1, 2),
                span: Span(0, 13),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(8, 13),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_string_greater_than_or_equal() {
    let source = constant_cases::CONSTANT_STRING_GREATER_THAN_OR_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 14),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(0, 5),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanOrEqualExpression,
                children: (1, 2),
                span: Span(0, 14),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(9, 14),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_string_less_than_or_equal() {
    let source = constant_cases::CONSTANT_STRING_LESS_THAN_OR_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 14),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(0, 5),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanOrEqualExpression,
                children: (1, 2),
                span: Span(0, 14),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(9, 14),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_string_equal() {
    let source = constant_cases::CONSTANT_STRING_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 14),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(0, 5),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::EqualExpression,
                children: (1, 2),
                span: Span(0, 14),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(9, 14),
                r#type: OperandType::NONE,
            }
        ]
    );
}

#[test]
fn constant_string_not_equal() {
    let source = constant_cases::CONSTANT_STRING_NOT_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                span: Span(0, 14),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(0, 5),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::NotEqualExpression,
                children: (1, 2),
                span: Span(0, 14),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(9, 14),
                r#type: OperandType::NONE,
            }
        ]
    );
}
