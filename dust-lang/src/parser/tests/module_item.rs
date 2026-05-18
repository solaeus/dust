use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::Span,
    syntax::{SyntaxId, node::SyntaxKind::*},
};

#[test]
fn file() {
    let parser = Parser::new_standalone(Lexer::unvalidated(b"mod foo;"));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 8), SyntaxId(2)),
            ModItem.with_single_child(Span::new(0, 8), SyntaxId(1)),
            SimplePath.empty(Span::new(4, 7)),
        ]
    );
}

#[test]
fn empty() {
    let parser = Parser::new_standalone(Lexer::unvalidated(b"mod foo {}"));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 10), SyntaxId(3)),
            ModItem.with_binary_children(Span::new(0, 10), SyntaxId(1), SyntaxId(2)),
            SimplePath.empty(Span::new(4, 7)),
            ModuleBody.empty(Span::new(8, 10)),
        ]
    );
}

#[test]
fn nested() {
    let parser = Parser::new_standalone(Lexer::unvalidated(b"mod foo { mod bar {} }"));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 22), SyntaxId(6)),
            ModItem.with_binary_children(Span::new(0, 22), SyntaxId(1), SyntaxId(5)),
            SimplePath.empty(Span::new(4, 7)),
            ModuleBody.with_single_child(Span::new(8, 22), SyntaxId(4)),
            ModItem.with_binary_children(Span::new(10, 20), SyntaxId(2), SyntaxId(3)),
            SimplePath.empty(Span::new(14, 17)),
            ModuleBody.empty(Span::new(18, 20)),
        ]
    );
}
