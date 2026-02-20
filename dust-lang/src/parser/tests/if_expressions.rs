use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{SourceFileId, Span},
    syntax::{SyntaxId, SyntaxKind::*},
    tests::if_expressions::{IF, IF_ELSE, IF_ELSE_IF},
};

#[test]
fn if_only() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(IF));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 40), SyntaxId(18)),
            FunctionItem.with_binary_children(Span::new(0, 40), SyntaxId(1), SyntaxId(17)),
            SimplePath.empty(Span::new(3, 7)),
            ValueParameters.empty(Span::new(7, 9)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(2)),
            FunctionExpression.with_binary_children(Span::new(7, 40), SyntaxId(3), SyntaxId(16)),
            BlockExpression.with_child(Span::new(10, 40), SyntaxId(15)),
            IfExpression.with_binary_children(Span::new(16, 38), SyntaxId(6), SyntaxId(14)),
            PathSegment.empty(Span::new(19, 28)),
            Path.with_child(Span::new(19, 28), SyntaxId(4)),
            PathExpression.with_child(Span::new(19, 28), SyntaxId(5)),
            BlockExpression.with_child(Span::new(29, 38), SyntaxId(13)),
            PathSegment.empty(Span::new(31, 32)),
            Path.with_child(Span::new(31, 32), SyntaxId(7)),
            PathExpression.with_child(Span::new(31, 32), SyntaxId(8)),
            AdditionExpression.with_binary_children(Span::new(31, 36), SyntaxId(9), SyntaxId(12)),
            PathSegment.empty(Span::new(35, 36)),
            Path.with_child(Span::new(35, 36), SyntaxId(10)),
            PathExpression.with_child(Span::new(35, 36), SyntaxId(11)),
        ]
    );
}

#[test]
fn if_else() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(IF_ELSE));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 55), SyntaxId(27)),
            FunctionItem.with_binary_children(Span::new(0, 55), SyntaxId(1), SyntaxId(26)),
            SimplePath.empty(Span::new(3, 7)),
            ValueParameters.empty(Span::new(7, 9)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(2)),
            FunctionExpression.with_binary_children(Span::new(7, 55), SyntaxId(3), SyntaxId(25)),
            BlockExpression.with_child(Span::new(10, 55), SyntaxId(24)),
            IfExpression.with_multiple_children(Span::new(16, 53), 0, 3),
            PathSegment.empty(Span::new(19, 28)),
            Path.with_child(Span::new(19, 28), SyntaxId(4)),
            PathExpression.with_child(Span::new(19, 28), SyntaxId(5)),
            BlockExpression.with_child(Span::new(29, 38), SyntaxId(13)),
            PathSegment.empty(Span::new(31, 32)),
            Path.with_child(Span::new(31, 32), SyntaxId(7)),
            PathExpression.with_child(Span::new(31, 32), SyntaxId(8)),
            AdditionExpression.with_binary_children(Span::new(31, 36), SyntaxId(9), SyntaxId(12)),
            PathSegment.empty(Span::new(35, 36)),
            Path.with_child(Span::new(35, 36), SyntaxId(10)),
            PathExpression.with_child(Span::new(35, 36), SyntaxId(11)),
            ElseExpression.with_child(Span::new(39, 53), SyntaxId(22)),
            BlockExpression.with_child(Span::new(44, 53), SyntaxId(21)),
            PathSegment.empty(Span::new(46, 47)),
            Path.with_child(Span::new(46, 47), SyntaxId(15)),
            PathExpression.with_child(Span::new(46, 47), SyntaxId(16)),
            SubtractionExpression.with_binary_children(
                Span::new(46, 51),
                SyntaxId(17),
                SyntaxId(20)
            ),
            PathSegment.empty(Span::new(50, 51)),
            Path.with_child(Span::new(50, 51), SyntaxId(18)),
            PathExpression.with_child(Span::new(50, 51), SyntaxId(19)),
        ]
    );
}

#[test]
fn if_else_if() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(IF_ELSE_IF));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 74), SyntaxId(40)),
            FunctionItem.with_binary_children(Span::new(0, 74), SyntaxId(1), SyntaxId(39)),
            SimplePath.empty(Span::new(3, 7)),
            ValueParameters.empty(Span::new(7, 9)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(2)),
            FunctionExpression.with_binary_children(Span::new(7, 74), SyntaxId(3), SyntaxId(38)),
            BlockExpression.with_child(Span::new(10, 74), SyntaxId(37)),
            IfExpression.with_multiple_children(Span::new(16, 72), 3, 3),
            PathSegment.empty(Span::new(19, 23)),
            Path.with_child(Span::new(19, 23), SyntaxId(4)),
            PathExpression.with_child(Span::new(19, 23), SyntaxId(5)),
            BlockExpression.with_child(Span::new(24, 33), SyntaxId(13)),
            PathSegment.empty(Span::new(26, 27)),
            Path.with_child(Span::new(26, 27), SyntaxId(7)),
            PathExpression.with_child(Span::new(26, 27), SyntaxId(8)),
            AdditionExpression.with_binary_children(Span::new(26, 31), SyntaxId(9), SyntaxId(12)),
            PathSegment.empty(Span::new(30, 31)),
            Path.with_child(Span::new(30, 31), SyntaxId(10)),
            PathExpression.with_child(Span::new(30, 31), SyntaxId(11)),
            ElseExpression.with_child(Span::new(34, 72), SyntaxId(35)),
            IfExpression.with_multiple_children(Span::new(39, 72), 0, 3),
            PathSegment.empty(Span::new(42, 47)),
            Path.with_child(Span::new(42, 47), SyntaxId(15)),
            PathExpression.with_child(Span::new(42, 47), SyntaxId(16)),
            BlockExpression.with_child(Span::new(48, 57), SyntaxId(24)),
            PathSegment.empty(Span::new(50, 51)),
            Path.with_child(Span::new(50, 51), SyntaxId(18)),
            PathExpression.with_child(Span::new(50, 51), SyntaxId(19)),
            SubtractionExpression.with_binary_children(
                Span::new(50, 55),
                SyntaxId(20),
                SyntaxId(23)
            ),
            PathSegment.empty(Span::new(54, 55)),
            Path.with_child(Span::new(54, 55), SyntaxId(21)),
            PathExpression.with_child(Span::new(54, 55), SyntaxId(22)),
            ElseExpression.with_child(Span::new(58, 72), SyntaxId(33)),
            BlockExpression.with_child(Span::new(63, 72), SyntaxId(32)),
            PathSegment.empty(Span::new(65, 66)),
            Path.with_child(Span::new(65, 66), SyntaxId(26)),
            PathExpression.with_child(Span::new(65, 66), SyntaxId(27)),
            MultiplicationExpression.with_binary_children(
                Span::new(65, 70),
                SyntaxId(28),
                SyntaxId(31)
            ),
            PathSegment.empty(Span::new(69, 70)),
            Path.with_child(Span::new(69, 70), SyntaxId(29)),
            PathExpression.with_child(Span::new(69, 70), SyntaxId(30)),
        ]
    );
}
