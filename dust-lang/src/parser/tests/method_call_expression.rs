use crate::{
    function_wrapper,
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::Span,
    syntax::{
        SyntaxId,
        node::{SyntaxChildren, SyntaxFlags, SyntaxKind::*},
    },
};

#[test]
fn field_access() {
    let parser = Parser::new_standalone(Lexer::unvalidated(function_wrapper!("x.y()")));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 23), SyntaxId(8)),
            FunctionItem.with_binary_children(Span::new(0, 23), SyntaxId(1), SyntaxId(7)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 23), SyntaxId(6)),
            MethodCallExpression
                .with_children(Span::new(16, 21), SyntaxChildren::new(0, 3))
                .with_flags(SyntaxFlags::VALUE_ARGUMENTS),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(2)),
            PathSegment.empty(Span::new(16, 17)),
            SimplePath.empty(Span::new(18, 19)),
            ValueArguments.empty(Span::new(19, 21)),
        ]
    );
}
