use crate::{
    function_wrapper,
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::Span,
    syntax::{SyntaxId, node::SyntaxKind::*},
};

#[test]
fn simple() {
    let parser = Parser::new_standalone(Lexer::unvalidated(function_wrapper!("foo")));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 21), SyntaxId(5)),
            FunctionItem.with_binary_children(Span::new(0, 21), SyntaxId(1), SyntaxId(4)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 21), SyntaxId(3)),
            PathExpression.with_single_child(Span::new(16, 19), SyntaxId(2)),
            PathSegment.empty(Span::new(16, 19)),
        ]
    );
}

#[test]
fn multi_segment() {
    let parser = Parser::new_standalone(Lexer::unvalidated(function_wrapper!("foo::bar")));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 26), SyntaxId(6)),
            FunctionItem.with_binary_children(Span::new(0, 26), SyntaxId(1), SyntaxId(5)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 26), SyntaxId(4)),
            PathExpression.with_binary_children(Span::new(16, 24), SyntaxId(2), SyntaxId(3)),
            PathSegment.empty(Span::new(16, 19)),
            PathSegment.empty(Span::new(21, 24)),
        ]
    );
}

#[test]
fn with_type_arguments() {
    let parser = Parser::new_standalone(Lexer::unvalidated(function_wrapper!("foo::<Bar>")));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 28), SyntaxId(8)),
            FunctionItem.with_binary_children(Span::new(0, 28), SyntaxId(1), SyntaxId(7)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 28), SyntaxId(6)),
            PathExpression.with_single_child(Span::new(16, 26), SyntaxId(5)),
            PathSegment.with_single_child(Span::new(16, 19), SyntaxId(4)),
            TypeArguments.with_single_child(Span::new(21, 26), SyntaxId(3)),
            TypePath.with_single_child(Span::new(22, 25), SyntaxId(2)),
            PathSegment.empty(Span::new(22, 25)),
        ]
    );
}

#[test]
fn with_multiple_type_arguments() {
    let parser = Parser::new_standalone(Lexer::unvalidated(function_wrapper!("foo::<Bar, Baz>")));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 33), SyntaxId(10)),
            FunctionItem.with_binary_children(Span::new(0, 33), SyntaxId(1), SyntaxId(9)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 33), SyntaxId(8)),
            PathExpression.with_single_child(Span::new(16, 31), SyntaxId(7)),
            PathSegment.with_single_child(Span::new(16, 19), SyntaxId(6)),
            TypeArguments.with_binary_children(Span::new(21, 31), SyntaxId(3), SyntaxId(5)),
            TypePath.with_single_child(Span::new(22, 25), SyntaxId(2)),
            PathSegment.empty(Span::new(22, 25)),
            TypePath.with_single_child(Span::new(27, 30), SyntaxId(4)),
            PathSegment.empty(Span::new(27, 30)),
        ]
    );
}

#[test]
fn multi_segment_with_type_arguments() {
    let parser = Parser::new_standalone(Lexer::unvalidated(function_wrapper!("foo::bar::<Baz>")));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 33), SyntaxId(9)),
            FunctionItem.with_binary_children(Span::new(0, 33), SyntaxId(1), SyntaxId(8)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 33), SyntaxId(7)),
            PathExpression.with_binary_children(Span::new(16, 31), SyntaxId(2), SyntaxId(6)),
            PathSegment.empty(Span::new(16, 19)),
            PathSegment.with_single_child(Span::new(21, 24), SyntaxId(5)),
            TypeArguments.with_single_child(Span::new(26, 31), SyntaxId(4)),
            TypePath.with_single_child(Span::new(27, 30), SyntaxId(3)),
            PathSegment.empty(Span::new(27, 30)),
        ]
    );
}

#[test]
fn type_arguments_on_middle_segment() {
    let parser = Parser::new_standalone(Lexer::unvalidated(function_wrapper!("foo::<Bar>::baz")));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 33), SyntaxId(9)),
            FunctionItem.with_binary_children(Span::new(0, 33), SyntaxId(1), SyntaxId(8)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 33), SyntaxId(7)),
            PathExpression.with_binary_children(Span::new(16, 31), SyntaxId(5), SyntaxId(6)),
            PathSegment.with_single_child(Span::new(16, 19), SyntaxId(4)),
            TypeArguments.with_single_child(Span::new(21, 26), SyntaxId(3)),
            TypePath.with_single_child(Span::new(22, 25), SyntaxId(2)),
            PathSegment.empty(Span::new(22, 25)),
            PathSegment.empty(Span::new(28, 31)),
        ]
    );
}
