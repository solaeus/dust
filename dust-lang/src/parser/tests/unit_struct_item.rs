use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{SourceCodeId, Span},
    syntax::{SyntaxId, node::SyntaxKind::*},
};

#[test]
fn unit_struct() {
    let parser = Parser::new(
        SourceCodeId::MAIN,
        Lexer::with_unvalidated_source(b"struct Foo;"),
    );
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_single_child(Span::new(0, 11), SyntaxId(2)),
            StructItem.with_single_child(Span::new(0, 11), SyntaxId(1)),
            SimplePath.empty(Span::new(7, 10)),
        ]
    );
}
