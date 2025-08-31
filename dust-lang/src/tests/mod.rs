use crate::parser::parse_main;

mod cases {
    pub const BOOLEAN: &str = "true";
    pub const BYTE: &str = "0xFF";
    pub const CHARACTER: &str = "'q'";
    pub const FLOAT: &str = "42.0";
    pub const INTEGER: &str = "42";
    pub const STRING: &str = "\"foobar\"";
}

mod parse {
    use crate::{
        Span,
        resolver::TypeId,
        syntax_tree::{SyntaxKind, SyntaxNode},
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
                    payload: (255, 0),
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
}
