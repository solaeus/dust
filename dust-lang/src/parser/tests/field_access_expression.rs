use crate::{
    function_wrapper,
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{SourceFileId, Span},
    syntax::{
        SyntaxId,
        node::{SyntaxKind::*, SyntaxPayload},
    },
};

#[test]
fn field_access() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("x.y")),
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
            Root.with_child(Span::new(0, 21), SyntaxId(9)),
            FunctionItem
                .with_multiple_children(Span::new(0, 21), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 21), SyntaxId(7)),
            FieldAccessExpression.with_binary_children(Span::new(16, 19), SyntaxId(5), SyntaxId(6)),
            PathExpression.with_child(Span::new(16, 17), SyntaxId(4)),
            PathSegment.empty(Span::new(16, 17)),
            SimplePath.empty(Span::new(18, 19)),
        ]
    );
}

#[test]
fn method_call() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("x.foo(1)")),
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
            Root.with_child(Span::new(0, 26), SyntaxId(12)),
            FunctionItem
                .with_multiple_children(Span::new(0, 26), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 26), SyntaxId(10)),
            CallExpression.with_binary_children(Span::new(16, 24), SyntaxId(7), SyntaxId(9)),
            FieldAccessExpression.with_binary_children(Span::new(16, 21), SyntaxId(5), SyntaxId(6)),
            PathExpression.with_child(Span::new(16, 17), SyntaxId(4)),
            PathSegment.empty(Span::new(16, 17)),
            SimplePath.empty(Span::new(18, 21)),
            ValueArguments.with_child(Span::new(16, 24), SyntaxId(8)),
            IntegerExpression.empty(Span::new(22, 23)),
        ]
    );
}
