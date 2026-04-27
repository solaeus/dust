use crate::function_wrapper;
use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{SourceCodeId, Span},
    syntax::{
        SyntaxId,
        node::{SyntaxChildren, SyntaxFlags, SyntaxKind::*},
    },
};

#[test]
fn empty() {
    let parser = Parser::new(
        SourceCodeId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("Foo;")),
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
            Root.with_single_child(Span::new(0, 22), SyntaxId(6)),
            FnItem.with_binary_children(Span::new(0, 22), SyntaxId(1), SyntaxId(5)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 22), SyntaxId(4)),
            ExpressionStatement.with_single_child(Span::new(16, 20), SyntaxId(3)),
            PathExpression.with_single_child(Span::new(16, 19), SyntaxId(2)),
            PathSegment.empty(Span::new(16, 19)),
        ]
    );
}

#[test]
fn named_fields() {
    let parser = Parser::new(
        SourceCodeId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("Foo { x: 42, y: 666 }")),
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
            Root.with_single_child(Span::new(0, 39), SyntaxId(11)),
            FnItem.with_binary_children(Span::new(0, 39), SyntaxId(1), SyntaxId(10)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 39), SyntaxId(9)),
            StructExpression
                .with_binary_children(Span::new(16, 37), SyntaxId(7), SyntaxId(8))
                .with_flags(SyntaxFlags::NAMED_FIELDS),
            Path.with_single_child(Span::new(16, 19), SyntaxId(2)),
            PathSegment.empty(Span::new(16, 19)),
            StructExpressionNamedFields.with_children(Span::new(20, 37), SyntaxChildren::new(0, 4)),
            SimplePath.empty(Span::new(22, 23)),
            IntegerExpression.empty(Span::new(25, 27)),
            SimplePath.empty(Span::new(29, 30)),
            IntegerExpression.empty(Span::new(32, 35)),
        ]
    );
}
