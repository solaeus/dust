use crate::{
    Span,
    resolver::TypeId,
    syntax_tree::{SyntaxKind, SyntaxNode},
    tests::cases,
};

use super::*;

#[test]
fn boolean() {
    let source = cases::BOOLEAN;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (0, 1),
                position: Span(0, 4),
                r#type: TypeId::BOOLEAN,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: (true as u32, 0),
                position: Span(0, 4),
                r#type: TypeId::BOOLEAN,
            }
        ]
    );
}

#[test]
fn byte() {
    let source = cases::BYTE;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (0, 1),
                position: Span(0, 4),
                r#type: TypeId::BYTE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: (42, 0),
                position: Span(0, 4),
                r#type: TypeId::BYTE,
            }
        ]
    );
}

#[test]
fn character() {
    let source = cases::CHARACTER;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (0, 1),
                position: Span(0, 3),
                r#type: TypeId::CHARACTER,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: (113, 0),
                position: Span(0, 3),
                r#type: TypeId::CHARACTER,
            }
        ]
    );
}

#[test]
fn float() {
    let source = cases::FLOAT;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (0, 1),
                position: Span(0, 4),
                r#type: TypeId::FLOAT,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxNode::encode_float(42.0),
                position: Span(0, 4),
                r#type: TypeId::FLOAT,
            }
        ]
    );
}

#[test]
fn integer() {
    let source = cases::INTEGER;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (0, 1),
                position: Span(0, 2),
                r#type: TypeId::INTEGER,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: (42, 0),
                position: Span(0, 2),
                r#type: TypeId::INTEGER,
            }
        ]
    );
}

#[test]
fn string() {
    let source = cases::STRING;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (0, 1),
                position: Span(0, 8),
                r#type: TypeId::STRING,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: (0, 0),
                position: Span(0, 8),
                r#type: TypeId::STRING,
            }
        ]
    );
}

#[test]
fn constant_byte_addition() {
    let source = cases::CONSTANT_BYTE_ADDITION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (0, 1),
                position: Span(0, 11),
                r#type: TypeId::BYTE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: (40, 0),
                position: Span(0, 4),
                r#type: TypeId::BYTE,
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                payload: (1, 2),
                position: Span(0, 11),
                r#type: TypeId::BYTE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: (2, 0),
                position: Span(7, 11),
                r#type: TypeId::BYTE,
            }
        ]
    );
}

#[test]
fn constant_float_addition() {
    let source = cases::CONSTANT_FLOAT_ADDITION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (0, 1),
                position: Span(0, 10),
                r#type: TypeId::FLOAT,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxNode::encode_float(40.0),
                position: Span(0, 4),
                r#type: TypeId::FLOAT,
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                payload: (1, 2),
                position: Span(0, 10),
                r#type: TypeId::FLOAT,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxNode::encode_float(2.0),
                position: Span(7, 10),
                r#type: TypeId::FLOAT,
            }
        ]
    );
}

#[test]
fn constant_integer_addition() {
    let source = cases::CONSTANT_INTEGER_ADDITION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (0, 1),
                position: Span(0, 6),
                r#type: TypeId::INTEGER,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: (40, 0),
                position: Span(0, 2),
                r#type: TypeId::INTEGER,
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                payload: (1, 2),
                position: Span(0, 6),
                r#type: TypeId::INTEGER,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: (2, 0),
                position: Span(5, 6),
                r#type: TypeId::INTEGER,
            }
        ]
    );
}

#[test]
fn constant_byte_subtraction() {
    let source = cases::CONSTANT_BYTE_SUBTRACTION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (0, 1),
                position: Span(0, 11),
                r#type: TypeId::BYTE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: (44, 0),
                position: Span(0, 4),
                r#type: TypeId::BYTE,
            },
            SyntaxNode {
                kind: SyntaxKind::SubtractionExpression,
                payload: (1, 2),
                position: Span(0, 11),
                r#type: TypeId::BYTE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: (2, 0),
                position: Span(7, 11),
                r#type: TypeId::BYTE,
            }
        ]
    );
}

#[test]
fn constant_float_subtraction() {
    let source = cases::CONSTANT_FLOAT_SUBTRACTION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (0, 1),
                position: Span(0, 10),
                r#type: TypeId::FLOAT,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxNode::encode_float(44.0),
                position: Span(0, 4),
                r#type: TypeId::FLOAT,
            },
            SyntaxNode {
                kind: SyntaxKind::SubtractionExpression,
                payload: (1, 2),
                position: Span(0, 10),
                r#type: TypeId::FLOAT,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxNode::encode_float(2.0),
                position: Span(7, 10),
                r#type: TypeId::FLOAT,
            }
        ]
    );
}

#[test]
fn constant_integer_subtraction() {
    let source = cases::CONSTANT_INTEGER_SUBTRACTION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (0, 1),
                position: Span(0, 6),
                r#type: TypeId::INTEGER,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: (44, 0),
                position: Span(0, 2),
                r#type: TypeId::INTEGER,
            },
            SyntaxNode {
                kind: SyntaxKind::SubtractionExpression,
                payload: (1, 2),
                position: Span(0, 6),
                r#type: TypeId::INTEGER,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: (2, 0),
                position: Span(5, 6),
                r#type: TypeId::INTEGER,
            }
        ]
    );
}

#[test]
fn constant_byte_multiplication() {
    let source = cases::CONSTANT_BYTE_MULTIPLICATION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (0, 1),
                position: Span(0, 11),
                r#type: TypeId::BYTE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: (14, 0),
                position: Span(0, 4),
                r#type: TypeId::BYTE,
            },
            SyntaxNode {
                kind: SyntaxKind::MultiplicationExpression,
                payload: (1, 2),
                position: Span(0, 11),
                r#type: TypeId::BYTE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: (3, 0),
                position: Span(7, 11),
                r#type: TypeId::BYTE,
            }
        ]
    );
}

#[test]
fn constant_float_multiplication() {
    let source = cases::CONSTANT_FLOAT_MULTIPLICATION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (0, 1),
                position: Span(0, 10),
                r#type: TypeId::FLOAT,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxNode::encode_float(14.0),
                position: Span(0, 4),
                r#type: TypeId::FLOAT,
            },
            SyntaxNode {
                kind: SyntaxKind::MultiplicationExpression,
                payload: (1, 2),
                position: Span(0, 10),
                r#type: TypeId::FLOAT,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxNode::encode_float(3.0),
                position: Span(7, 10),
                r#type: TypeId::FLOAT,
            }
        ]
    );
}

#[test]
fn constant_integer_multiplication() {
    let source = cases::CONSTANT_INTEGER_MULTIPLICATION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (0, 1),
                position: Span(0, 6),
                r#type: TypeId::INTEGER,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: (14, 0),
                position: Span(0, 2),
                r#type: TypeId::INTEGER,
            },
            SyntaxNode {
                kind: SyntaxKind::MultiplicationExpression,
                payload: (1, 2),
                position: Span(0, 6),
                r#type: TypeId::INTEGER,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: (3, 0),
                position: Span(5, 6),
                r#type: TypeId::INTEGER,
            }
        ]
    );
}

#[test]
fn constant_byte_division() {
    let source = cases::CONSTANT_BYTE_DIVISION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (0, 1),
                position: Span(0, 11),
                r#type: TypeId::BYTE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: (84, 0),
                position: Span(0, 4),
                r#type: TypeId::BYTE,
            },
            SyntaxNode {
                kind: SyntaxKind::DivisionExpression,
                payload: (1, 2),
                position: Span(0, 11),
                r#type: TypeId::BYTE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: (2, 0),
                position: Span(7, 11),
                r#type: TypeId::BYTE,
            }
        ]
    );
}

#[test]
fn constant_float_division() {
    let source = cases::CONSTANT_FLOAT_DIVISION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (0, 1),
                position: Span(0, 10),
                r#type: TypeId::FLOAT,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxNode::encode_float(84.0),
                position: Span(0, 4),
                r#type: TypeId::FLOAT,
            },
            SyntaxNode {
                kind: SyntaxKind::DivisionExpression,
                payload: (1, 2),
                position: Span(0, 10),
                r#type: TypeId::FLOAT,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxNode::encode_float(2.0),
                position: Span(7, 10),
                r#type: TypeId::FLOAT,
            }
        ]
    );
}

#[test]
fn constant_integer_division() {
    let source = cases::CONSTANT_INTEGER_DIVISION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (0, 1),
                position: Span(0, 6),
                r#type: TypeId::INTEGER,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: (84, 0),
                position: Span(0, 2),
                r#type: TypeId::INTEGER,
            },
            SyntaxNode {
                kind: SyntaxKind::DivisionExpression,
                payload: (1, 2),
                position: Span(0, 6),
                r#type: TypeId::INTEGER,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: (2, 0),
                position: Span(5, 6),
                r#type: TypeId::INTEGER,
            }
        ]
    );
}
