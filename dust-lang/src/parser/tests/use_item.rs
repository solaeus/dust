use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{SourceFileId, Span},
    syntax::{SyntaxId, node::SyntaxKind::*},
};

#[test]
fn use_item() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(b"use foo;"));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 8), SyntaxId(3)),
            UseItem.with_child(Span::new(0, 8), SyntaxId(2)),
            Path.with_child(Span::new(4, 7), SyntaxId(1)),
            PathSegment.empty(Span::new(4, 7)),
        ]
    );
}
