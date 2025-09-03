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
    let source = cases::BYTE;
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
    let source = cases::CHARACTER;
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
    let source = cases::FLOAT;
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
    let source = cases::INTEGER;
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
    let source = cases::STRING;
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
    let source = cases::CONSTANT_BYTE_ADDITION;
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
    let source = cases::CONSTANT_FLOAT_ADDITION;
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
    let source = cases::CONSTANT_INTEGER_ADDITION;
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
    let source = cases::CONSTANT_BYTE_SUBTRACTION;
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
    let source = cases::CONSTANT_FLOAT_SUBTRACTION;
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
    let source = cases::CONSTANT_INTEGER_SUBTRACTION;
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
    let source = cases::CONSTANT_BYTE_MULTIPLICATION;
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
    let source = cases::CONSTANT_FLOAT_MULTIPLICATION;
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
    let source = cases::CONSTANT_INTEGER_MULTIPLICATION;
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
    let source = cases::CONSTANT_BYTE_DIVISION;
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
    let source = cases::CONSTANT_FLOAT_DIVISION;
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
    let source = cases::CONSTANT_INTEGER_DIVISION;
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
    let source = cases::CONSTANT_STRING_CONCATENATION;
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
    let source = cases::CONSTANT_CHARACTER_CONCATENATION;
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
    let source = cases::CONSTANT_STRING_CHARACTER_CONCATENATION;
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
    let source = cases::CONSTANT_CHARACTER_STRING_CONCATENATION;
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
fn local_declaration() {
    let source = cases::LOCAL_DECLARATION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 16),
                payload: TypeId::NONE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 2),
                position: Span(0, 16),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                position: Span(7, 10),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                position: Span(13, 15),
                payload: TypeId::INTEGER.0,
            }
        ]
    );
}

#[test]
fn local_mut_declaration() {
    let source = cases::LOCAL_MUT_DECLARATION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 20),
                payload: TypeId::NONE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LetMutStatement,
                children: (1, 2),
                position: Span(0, 20),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                position: Span(11, 14),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                position: Span(17, 19),
                payload: TypeId::INTEGER.0,
            }
        ]
    );
}
