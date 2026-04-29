use crate::{
    function_wrapper,
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{SourceCodeId, Span},
    syntax::{SyntaxId, node::SyntaxKind::*},
};

#[test]
fn field_access() {
    let parser = Parser::new(
        SourceCodeId::MAIN,
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
            Root.with_single_child(Span::new(0, 21), SyntaxId(7)),
            FnItem.with_binary_children(Span::new(0, 21), SyntaxId(1), SyntaxId(6)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 21), SyntaxId(5)),
            FieldAccessExpression.with_binary_children(Span::new(16, 19), SyntaxId(3), SyntaxId(4)),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(2)),
            PathSegment.empty(Span::new(16, 17)),
            SimplePath.empty(Span::new(18, 19)),
        ]
    );
}

#[test]
fn method_call() {
    let parser = Parser::new(
        SourceCodeId::MAIN,
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
            Root.with_single_child(Span::new(0, 26), SyntaxId(10)),
            FnItem.with_binary_children(Span::new(0, 26), SyntaxId(1), SyntaxId(9)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 26), SyntaxId(8)),
            CallExpression.with_binary_children(Span::new(16, 24), SyntaxId(5), SyntaxId(7)),
            FieldAccessExpression.with_binary_children(Span::new(16, 21), SyntaxId(3), SyntaxId(4)),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(2)),
            PathSegment.empty(Span::new(16, 17)),
            SimplePath.empty(Span::new(18, 21)),
            ValueArguments.with_single_child(Span::new(16, 24), SyntaxId(6)),
            IntegerExpression.empty(Span::new(22, 23)),
        ]
    );
}
