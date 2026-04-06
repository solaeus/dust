use crate::function_wrapper;
use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{SourceFileId, Span},
    syntax::{
        SyntaxId,
        node::{SyntaxKind::*, SyntaxPayload},
    },
};

#[test]
fn call_with_two_arguments() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("foo(1, 2)")),
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
            Root.with_child(Span::new(0, 27), SyntaxId(11)),
            FunctionItem
                .with_multiple_children(Span::new(0, 27), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 27), SyntaxId(9)),
            CallExpression.with_binary_children(Span::new(16, 25), SyntaxId(5), SyntaxId(8)),
            PathExpression.with_child(Span::new(16, 19), SyntaxId(4)),
            PathSegment.empty(Span::new(16, 19)),
            ValueArguments.with_binary_children(Span::new(16, 25), SyntaxId(6), SyntaxId(7)),
            IntegerExpression.empty(Span::new(20, 21)),
            IntegerExpression.empty(Span::new(23, 24)),
        ]
    );
}
