use crate::{
    function_wrapper,
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::Span,
    syntax::{SyntaxId, node::SyntaxKind::*},
};

#[test]
fn negation() {
    let parser = Parser::new_standalone(Lexer::unvalidated(function_wrapper!("-x")));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 20), SyntaxId(6)),
            FunctionItem.with_binary_children(Span::new(0, 20), SyntaxId(1), SyntaxId(5)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 20), SyntaxId(4)),
            NegationExpression.with_single_child(Span::new(16, 18), SyntaxId(3)),
            PathExpression.with_single_child(Span::new(17, 18), SyntaxId(2)),
            PathSegment.empty(Span::new(17, 18)),
        ]
    );
}

#[test]
fn logical_not() {
    let parser = Parser::new_standalone(Lexer::unvalidated(function_wrapper!("!x")));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 20), SyntaxId(6)),
            FunctionItem.with_binary_children(Span::new(0, 20), SyntaxId(1), SyntaxId(5)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 20), SyntaxId(4)),
            NotExpression.with_single_child(Span::new(16, 18), SyntaxId(3)),
            PathExpression.with_single_child(Span::new(17, 18), SyntaxId(2)),
            PathSegment.empty(Span::new(17, 18)),
        ]
    );
}
