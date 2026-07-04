use crate::{
    function_wrapper,
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::Span,
    syntax::{SyntaxId, node::SyntaxKind::*},
};

#[test]
fn call_with_two_arguments() {
    let parser = Parser::new_standalone(Lexer::unvalidated(function_wrapper!("foo(1, 2)")));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 27), SyntaxId(9)),
            FunctionItem.with_binary_children(Span::new(0, 27), SyntaxId(1), SyntaxId(8)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 27), SyntaxId(7)),
            CallExpression.with_binary_children(Span::new(16, 25), SyntaxId(3), SyntaxId(6)),
            PathExpression.with_single_child(Span::new(16, 19), SyntaxId(2)),
            PathSegment.empty(Span::new(16, 19)),
            ValueArguments.with_binary_children(Span::new(19, 25), SyntaxId(4), SyntaxId(5)),
            IntegerExpression.empty(Span::new(20, 21)),
            IntegerExpression.empty(Span::new(23, 24)),
        ]
    );
}
