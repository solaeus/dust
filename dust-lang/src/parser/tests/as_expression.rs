use crate::function_wrapper;
use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{SourceCodeId, Span},
    syntax::{SyntaxId, node::SyntaxKind::*},
};

#[test]
fn as_expression() {
    let parser = Parser::new(
        SourceCodeId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("x as i32")),
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
            Root.with_single_child(Span::new(0, 26), SyntaxId(7)),
            FnItem.with_binary_children(Span::new(0, 26), SyntaxId(1), SyntaxId(6)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 26), SyntaxId(5)),
            AsExpression.with_binary_children(Span::new(16, 24), SyntaxId(3), SyntaxId(4)),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(2)),
            PathSegment.empty(Span::new(16, 17)),
            I32Type.empty(Span::new(21, 24)),
        ]
    );
}
