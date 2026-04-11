use crate::function_wrapper;
use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{FileId, Span},
    syntax::{
        SyntaxId,
        node::{SyntaxKind::*, SyntaxChildren},
    },
};

#[test]
fn call_with_two_arguments() {
    let parser = Parser::new(
        FileId::MAIN,
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
            Root.with_single_child(Span::new(0, 27), SyntaxId(12)),
            FunctionItem.with_children(Span::new(0, 27), SyntaxChildren::new(1, 4)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionSignature.with_children(Span::new(0, 9), SyntaxChildren::new(0, 1)),
            FunctionParameters.with_single_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_single_child(Span::new(10, 27), SyntaxId(10)),
            CallExpression.with_binary_children(Span::new(16, 25), SyntaxId(6), SyntaxId(9)),
            PathExpression.with_single_child(Span::new(16, 19), SyntaxId(5)),
            PathSegment.empty(Span::new(16, 19)),
            ValueArguments.with_binary_children(Span::new(16, 25), SyntaxId(7), SyntaxId(8)),
            IntegerExpression.empty(Span::new(20, 21)),
            IntegerExpression.empty(Span::new(23, 24)),
        ]
    );
}
