use crate::{
    parser::parse_main,
    source::Span,
    syntax::{SyntaxKind, SyntaxNode, SyntaxPayload},
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::encode_boolean(true),
                span: Span(0, 4),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::encode_byte(42),
                span: Span(0, 4),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 3),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::encode_character('q'),
                span: Span(0, 3),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(42.0),
                span: Span(0, 4),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 2),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::binary_children(42, 0),
                span: Span(0, 2),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 8),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::binary_children(0, 0),
                span: Span(0, 8),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::binary_children(40, 0),
                span: Span(0, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::binary_children(2, 0),
                span: Span(7, 11),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(40.0),
                span: Span(0, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(2.0),
                span: Span(7, 10),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::binary_children(40, 0),
                span: Span(0, 2),
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::binary_children(2, 0),
                span: Span(5, 6),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::binary_children(44, 0),
                span: Span(0, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::SubtractionExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::binary_children(2, 0),
                span: Span(7, 11),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(44.0),
                span: Span(0, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::SubtractionExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(2.0),
                span: Span(7, 10),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::binary_children(44, 0),
                span: Span(0, 2),
            },
            SyntaxNode {
                kind: SyntaxKind::SubtractionExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::binary_children(2, 0),
                span: Span(5, 6),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::binary_children(14, 0),
                span: Span(0, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::MultiplicationExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::binary_children(3, 0),
                span: Span(7, 11),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(14.0),
                span: Span(0, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::MultiplicationExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(3.0),
                span: Span(7, 10),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::binary_children(14, 0),
                span: Span(0, 2),
            },
            SyntaxNode {
                kind: SyntaxKind::MultiplicationExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::binary_children(3, 0),
                span: Span(5, 6),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::binary_children(84, 0),
                span: Span(0, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::DivisionExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::binary_children(2, 0),
                span: Span(7, 11),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(84.0),
                span: Span(0, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::DivisionExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(2.0),
                span: Span(7, 10),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::binary_children(84, 0),
                span: Span(0, 2),
            },
            SyntaxNode {
                kind: SyntaxKind::DivisionExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::binary_children(2, 0),
                span: Span(5, 6),
            }
        ]
    );
}

#[test]
fn constant_byte_modulo() {
    let source = constant_cases::CONSTANT_BYTE_MODULO.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::binary_children(84, 0),
                span: Span(0, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::ModuloExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::binary_children(5, 0),
                span: Span(7, 11),
            }
        ]
    );
}

#[test]
fn constant_float_modulo() {
    let source = constant_cases::CONSTANT_FLOAT_MODULO.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(84.0),
                span: Span(0, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::ModuloExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(5.0),
                span: Span(7, 10),
            }
        ]
    );
}

#[test]
fn constant_integer_modulo() {
    let source = constant_cases::CONSTANT_INTEGER_MODULO.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::binary_children(84, 0),
                span: Span(0, 2),
            },
            SyntaxNode {
                kind: SyntaxKind::ModuloExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::binary_children(5, 0),
                span: Span(5, 6),
            }
        ]
    );
}

#[test]
fn constant_byte_exponent() {
    let source = constant_cases::CONSTANT_BYTE_EXPONENT.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::binary_children(2, 0),
                span: Span(0, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::ExponentExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::binary_children(3, 0),
                span: Span(7, 11),
            }
        ]
    );
}

#[test]
fn constant_float_exponent() {
    let source = constant_cases::CONSTANT_FLOAT_EXPONENT.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 9),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(2.0),
                span: Span(0, 3),
            },
            SyntaxNode {
                kind: SyntaxKind::ExponentExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 9),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(3.0),
                span: Span(6, 9),
            }
        ]
    );
}

#[test]
fn constant_integer_exponent() {
    let source = constant_cases::CONSTANT_INTEGER_EXPONENT.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 5),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::binary_children(2, 0),
                span: Span(0, 1),
            },
            SyntaxNode {
                kind: SyntaxKind::ExponentExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 5),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::binary_children(3, 0),
                span: Span(4, 5),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 5),
            },
            SyntaxNode {
                kind: SyntaxKind::NegationExpression,
                payload: SyntaxPayload::binary_children(2, 0),
                span: Span(0, 5),
            },
            SyntaxNode {
                kind: SyntaxKind::GroupedExpression,
                payload: SyntaxPayload::binary_children(1, 0),
                span: Span(1, 5),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::binary_children(42, 0),
                span: Span(2, 4),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 7),
            },
            SyntaxNode {
                kind: SyntaxKind::NegationExpression,
                payload: SyntaxPayload::binary_children(2, 0),
                span: Span(0, 7),
            },
            SyntaxNode {
                kind: SyntaxKind::GroupedExpression,
                payload: SyntaxPayload::binary_children(1, 0),
                span: Span(1, 7),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(42.0),
                span: Span(2, 6),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::binary_children(0, 0),
                span: Span(0, 5),
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::binary_children(0, 0),
                span: Span(8, 13),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 9),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::binary_children(113, 0),
                span: Span(0, 3),
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 9),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::binary_children(113, 0),
                span: Span(6, 9),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::binary_children(0, 0),
                span: Span(0, 5),
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::binary_children(113, 0),
                span: Span(8, 11),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::binary_children(113, 0),
                span: Span(0, 3),
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::binary_children(0, 0),
                span: Span(6, 11),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::encode_boolean(true),
                span: Span(0, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::AndExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::encode_boolean(false),
                span: Span(8, 13),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::encode_boolean(true),
                span: Span(0, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::OrExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::encode_boolean(false),
                span: Span(8, 13),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 5),
            },
            SyntaxNode {
                kind: SyntaxKind::NotExpression,
                payload: SyntaxPayload::binary_children(1, 0),
                span: Span(0, 5),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::encode_boolean(true),
                span: Span(1, 5),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::binary_children(1, 0),
                span: Span(0, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::binary_children(0, 0),
                span: Span(7, 12),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::binary_children(0, 0),
                span: Span(0, 5),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::binary_children(1, 0),
                span: Span(8, 12),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::binary_children(1, 0),
                span: Span(0, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanOrEqualExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::binary_children(1, 0),
                span: Span(8, 12),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::binary_children(1, 0),
                span: Span(0, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanOrEqualExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::binary_children(1, 0),
                span: Span(8, 12),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::binary_children(1, 0),
                span: Span(0, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::EqualExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::binary_children(1, 0),
                span: Span(8, 12),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::binary_children(1, 0),
                span: Span(0, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::NotEqualExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::binary_children(0, 0),
                span: Span(8, 13),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::binary_children(43, 0),
                span: Span(0, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::binary_children(42, 0),
                span: Span(7, 11),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::binary_children(41, 0),
                span: Span(0, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::binary_children(42, 0),
                span: Span(7, 11),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::binary_children(42, 0),
                span: Span(0, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanOrEqualExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::binary_children(42, 0),
                span: Span(8, 12),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::binary_children(42, 0),
                span: Span(0, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanOrEqualExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::binary_children(42, 0),
                span: Span(8, 12),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::binary_children(42, 0),
                span: Span(0, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::EqualExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::binary_children(42, 0),
                span: Span(8, 12),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::binary_children(42, 0),
                span: Span(0, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::NotEqualExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::binary_children(43, 0),
                span: Span(8, 12),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 9),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::binary_children(123, 0),
                span: Span(0, 3),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 9),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::binary_children(122, 0),
                span: Span(6, 9),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 9),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::binary_children(121, 0),
                span: Span(0, 3),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 9),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::binary_children(122, 0),
                span: Span(6, 9),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::binary_children(122, 0),
                span: Span(0, 3),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanOrEqualExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::binary_children(122, 0),
                span: Span(7, 10),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::binary_children(122, 0),
                span: Span(0, 3),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanOrEqualExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::binary_children(122, 0),
                span: Span(7, 10),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::binary_children(122, 0),
                span: Span(0, 3),
            },
            SyntaxNode {
                kind: SyntaxKind::EqualExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::binary_children(122, 0),
                span: Span(7, 10),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::binary_children(122, 0),
                span: Span(0, 3),
            },
            SyntaxNode {
                kind: SyntaxKind::NotEqualExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::binary_children(123, 0),
                span: Span(7, 10),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(43.0),
                span: Span(0, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(42.0),
                span: Span(7, 11),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(41.0),
                span: Span(0, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(42.0),
                span: Span(7, 11),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(42.0),
                span: Span(0, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanOrEqualExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(42.0),
                span: Span(8, 12),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(42.0),
                span: Span(0, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanOrEqualExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(42.0),
                span: Span(8, 12),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(42.0),
                span: Span(0, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::EqualExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(42.0),
                span: Span(8, 12),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(42.0),
                span: Span(0, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::NotEqualExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(43.0),
                span: Span(8, 12),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 7),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::binary_children(43, 0),
                span: Span(0, 2),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 7),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::binary_children(42, 0),
                span: Span(5, 7),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 7),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::binary_children(41, 0),
                span: Span(0, 2),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 7),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::binary_children(42, 0),
                span: Span(5, 7),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 8),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::binary_children(42, 0),
                span: Span(0, 2),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanOrEqualExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 8),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::binary_children(42, 0),
                span: Span(6, 8),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 8),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::binary_children(42, 0),
                span: Span(0, 2),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanOrEqualExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 8),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::binary_children(42, 0),
                span: Span(6, 8),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 8),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::binary_children(42, 0),
                span: Span(0, 2),
            },
            SyntaxNode {
                kind: SyntaxKind::EqualExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 8),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::binary_children(42, 0),
                span: Span(6, 8),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 8),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::binary_children(42, 0),
                span: Span(0, 2),
            },
            SyntaxNode {
                kind: SyntaxKind::NotEqualExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 8),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::binary_children(43, 0),
                span: Span(6, 8),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::binary_children(0, 0),
                span: Span(0, 5),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::binary_children(0, 0),
                span: Span(8, 13),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::binary_children(0, 0),
                span: Span(0, 5),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::binary_children(0, 0),
                span: Span(8, 13),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 14),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::binary_children(0, 0),
                span: Span(0, 5),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanOrEqualExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 14),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::binary_children(0, 0),
                span: Span(9, 14),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 14),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::binary_children(0, 0),
                span: Span(0, 5),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanOrEqualExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 14),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::binary_children(0, 0),
                span: Span(9, 14),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 14),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::binary_children(0, 0),
                span: Span(0, 5),
            },
            SyntaxNode {
                kind: SyntaxKind::EqualExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 14),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::binary_children(0, 0),
                span: Span(9, 14),
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
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 14),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::binary_children(0, 0),
                span: Span(0, 5),
            },
            SyntaxNode {
                kind: SyntaxKind::NotEqualExpression,
                payload: SyntaxPayload::binary_children(1, 2),
                span: Span(0, 14),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::binary_children(0, 0),
                span: Span(9, 14),
            }
        ]
    );
}
