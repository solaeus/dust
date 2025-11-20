use crate::{
    parser::parse_main,
    source::Span,
    syntax_tree::{SyntaxKind, SyntaxNode},
    tests::list_cases,
};

#[test]
fn list_boolean() {
    let source = list_cases::LIST_BOOLEAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (3, 1),
                span: Span(0, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                children: (0, 3),
                span: Span(0, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (1, 0),
                span: Span(1, 5),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (0, 0),
                span: Span(7, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (1, 0),
                span: Span(14, 18),
            }
        ]
    );
}

#[test]
fn list_byte() {
    let source = list_cases::LIST_BYTE.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (3, 1),
                span: Span(0, 18),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                children: (0, 3),
                span: Span(0, 18),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (42, 0),
                span: Span(1, 5),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (43, 0),
                span: Span(7, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (44, 0),
                span: Span(13, 17),
            }
        ]
    );
}

#[test]
fn list_character() {
    let source = list_cases::LIST_CHARACTER.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (3, 1),
                span: Span(0, 15),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                children: (0, 3),
                span: Span(0, 15),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (97, 0),
                span: Span(1, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (98, 0),
                span: Span(6, 9),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (99, 0),
                span: Span(11, 14),
            }
        ]
    );
}

#[test]
fn list_float() {
    let source = list_cases::LIST_FLOAT.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (3, 1),
                span: Span(0, 15),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                children: (0, 3),
                span: Span(0, 15),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(1.0),
                span: Span(1, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(2.0),
                span: Span(6, 9),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(3.0),
                span: Span(11, 14),
            }
        ]
    );
}

#[test]
fn list_integer() {
    let source = list_cases::LIST_INTEGER.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (3, 1),
                span: Span(0, 9),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                children: (0, 3),
                span: Span(0, 9),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (1, 0),
                span: Span(1, 2),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (2, 0),
                span: Span(4, 5),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (3, 0),
                span: Span(7, 8),
            }
        ]
    );
}

#[test]
fn list_string() {
    let source = list_cases::LIST_STRING.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (3, 1),
                span: Span(0, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                children: (0, 3),
                span: Span(0, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(1, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(8, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(15, 20),
            }
        ]
    );
}
