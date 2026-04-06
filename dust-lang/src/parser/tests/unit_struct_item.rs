use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{SourceFileId, Span},
    syntax::{SyntaxId, node::SyntaxKind::*},
};

#[test]
fn unit_struct() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(b"struct Foo;"));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 11), SyntaxId(3)),
            StructItem.with_binary_children(Span::new(0, 11), SyntaxId(1), SyntaxId(2)),
            SimplePath.empty(Span::new(7, 10)),
            StructItemUnit.empty(Span::new(10, 11)),
        ]
    );
}
