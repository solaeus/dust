use crate::{
    function_wrapper,
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::Span,
    syntax::{SyntaxId, node::SyntaxKind::*},
};

#[test]
fn field_access() {
    let parser = Parser::new_standalone(Lexer::with_unvalidated_source(function_wrapper!("x.y")));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 21), SyntaxId(7)),
            FunctionItem.with_binary_children(Span::new(0, 21), SyntaxId(1), SyntaxId(6)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 21), SyntaxId(5)),
            FieldAccessExpression.with_binary_children(Span::new(16, 19), SyntaxId(3), SyntaxId(4)),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(2)),
            PathSegment.empty(Span::new(16, 17)),
            SimplePath.empty(Span::new(18, 19)),
        ]
    );
}
