use crate::{
    function_wrapper,
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::Span,
    syntax::{SyntaxId, node::SyntaxKind::*},
};

#[test]
fn exclusive() {
    let parser = Parser::new_standalone(Lexer::with_unvalidated_source(function_wrapper!("1..10")));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 23), SyntaxId(6)),
            FunctionItem.with_binary_children(Span::new(0, 23), SyntaxId(1), SyntaxId(5)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 23), SyntaxId(4)),
            RangeExpression.with_binary_children(Span::new(16, 21), SyntaxId(2), SyntaxId(3)),
            IntegerExpression.empty(Span::new(16, 17)),
            IntegerExpression.empty(Span::new(19, 21)),
        ]
    );
}

#[test]
fn inclusive() {
    let parser =
        Parser::new_standalone(Lexer::with_unvalidated_source(function_wrapper!("1..=10")));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 24), SyntaxId(6)),
            FunctionItem.with_binary_children(Span::new(0, 24), SyntaxId(1), SyntaxId(5)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 24), SyntaxId(4)),
            RangeInclusiveExpression.with_binary_children(
                Span::new(16, 22),
                SyntaxId(2),
                SyntaxId(3),
            ),
            IntegerExpression.empty(Span::new(16, 17)),
            IntegerExpression.empty(Span::new(20, 22)),
        ]
    );
}
