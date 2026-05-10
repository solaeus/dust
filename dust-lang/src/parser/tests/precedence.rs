use crate::{
    function_wrapper,
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::Span,
    syntax::{SyntaxId, node::SyntaxKind::*},
};

#[test]
fn addition_and_multiplication() {
    let parser = Parser::new_standalone(Lexer::with_unvalidated_source(function_wrapper!(
        "a + b * c"
    )));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 27), SyntaxId(11)),
            FunctionItem.with_binary_children(Span::new(0, 27), SyntaxId(1), SyntaxId(10)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 27), SyntaxId(9)),
            AdditionExpression.with_binary_children(Span::new(16, 25), SyntaxId(3), SyntaxId(8)),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(2)),
            PathSegment.empty(Span::new(16, 17)),
            MultiplicationExpression.with_binary_children(
                Span::new(20, 25),
                SyntaxId(5),
                SyntaxId(7),
            ),
            PathExpression.with_single_child(Span::new(20, 21), SyntaxId(4)),
            PathSegment.empty(Span::new(20, 21)),
            PathExpression.with_single_child(Span::new(24, 25), SyntaxId(6)),
            PathSegment.empty(Span::new(24, 25)),
        ]
    );
}

#[test]
fn right_associative_exponentiation() {
    let parser = Parser::new_standalone(Lexer::with_unvalidated_source(function_wrapper!(
        "a ^ b ^ c"
    )));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 27), SyntaxId(11)),
            FunctionItem.with_binary_children(Span::new(0, 27), SyntaxId(1), SyntaxId(10)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 27), SyntaxId(9)),
            ExponentExpression.with_binary_children(Span::new(16, 25), SyntaxId(3), SyntaxId(8)),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(2)),
            PathSegment.empty(Span::new(16, 17)),
            ExponentExpression.with_binary_children(Span::new(20, 25), SyntaxId(5), SyntaxId(7)),
            PathExpression.with_single_child(Span::new(20, 21), SyntaxId(4)),
            PathSegment.empty(Span::new(20, 21)),
            PathExpression.with_single_child(Span::new(24, 25), SyntaxId(6)),
            PathSegment.empty(Span::new(24, 25)),
        ]
    );
}
