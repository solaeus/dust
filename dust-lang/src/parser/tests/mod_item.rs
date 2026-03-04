use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{SourceFileId, Span},
    syntax::{SyntaxId, SyntaxKind::*},
    tests::source_examples::mod_item::{EMPTY, FILE, NESTED},
};

#[test]
fn file() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(FILE));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 8), SyntaxId(2)),
            ModuleItem.with_child(Span::new(0, 8), SyntaxId(1)),
            SimplePath.empty(Span::new(4, 7)),
        ]
    );
}

#[test]
fn empty() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(EMPTY));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 10), SyntaxId(3)),
            ModuleItem.with_binary_children(Span::new(0, 10), SyntaxId(1), SyntaxId(2)),
            SimplePath.empty(Span::new(4, 7)),
            ModuleBody.empty(Span::new(8, 10)),
        ]
    );
}

#[test]
fn nested() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(NESTED));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 22), SyntaxId(6)),
            ModuleItem.with_binary_children(Span::new(0, 22), SyntaxId(1), SyntaxId(5)),
            SimplePath.empty(Span::new(4, 7)),
            ModuleBody.with_child(Span::new(8, 22), SyntaxId(4)),
            ModuleItem.with_binary_children(Span::new(10, 20), SyntaxId(2), SyntaxId(3)),
            SimplePath.empty(Span::new(14, 17)),
            ModuleBody.empty(Span::new(18, 20)),
        ]
    );
}
