use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{SourceFileId, Span},
    syntax::{SyntaxId, SyntaxKind::*, SyntaxPayload},
    tests::{BINARY_ASSIGNMENT_STATEMENT, FUNCTION_ITEM, LET_STATEMENT, REASSIGNMENT_STATEMENT},
};

#[test]
fn function_item() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(FUNCTION_ITEM));
    let ParseResult {
        syntax_tree,
        errors,
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            Root.with_child(Span::new(0, 17), SyntaxId(7)),
            FunctionItem.with_binary_children(Span::new(5, 18), SyntaxId(2), SyntaxId(6)),
            PathSegment.empty(Span::new(8, 12)),
            Path.with_child(Span::new(8, 12), SyntaxId(1)),
            ValueParameters.empty(Span::new(12, 14)),
            FunctionSignature.with_child(Span::new(12, 14), SyntaxId(3)),
            FunctionExpression.with_binary_children(Span::new(12, 17), SyntaxId(4), SyntaxId(5)),
            BlockExpression.empty(Span::new(15, 17)),
        ]
    );
}

#[test]
fn let_statement() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(LET_STATEMENT));
    let ParseResult {
        syntax_tree,
        errors,
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            Root.empty(Span::new(0, 42)),
            PathSegment.empty(Span::new(8, 12)),
            Path.with_child(Span::new(8, 12), SyntaxId(1)),
            ValueParameters.empty(Span::new(12, 14)),
            FunctionSignature.with_child(Span::new(12, 14), SyntaxId(3)),
            PathSegment.empty(Span::new(29, 30)),
            Path.with_child(Span::new(29, 30), SyntaxId(5)),
        ]
    );
}

#[test]
fn reassignment_statement() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(REASSIGNMENT_STATEMENT),
    );
    let ParseResult {
        syntax_tree,
        errors,
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            Root.empty(Span::new(0, 38)),
            PathSegment.empty(Span::new(8, 12)),
            Path.with_child(Span::new(8, 12), SyntaxId(1)),
            ValueParameters.empty(Span::new(12, 14)),
            FunctionSignature.with_child(Span::new(12, 14), SyntaxId(3)),
            PathSegment.empty(Span::new(25, 26)),
        ]
    );
}

#[test]
fn binary_assignment_statement() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(BINARY_ASSIGNMENT_STATEMENT),
    );
    let ParseResult {
        syntax_tree,
        errors,
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            Root.with_child(Span::new(0, 39), SyntaxId(11)),
            FunctionItem.with_binary_children(Span::new(5, 40), SyntaxId(2), SyntaxId(10)),
            PathSegment.empty(Span::new(8, 12)),
            Path.with_child(Span::new(8, 12), SyntaxId(1)),
            ValueParameters.empty(Span::new(12, 14)),
            FunctionSignature.with_child(Span::new(12, 14), SyntaxId(3)),
            FunctionExpression.with_binary_children(Span::new(12, 39), SyntaxId(4), SyntaxId(9)),
            BlockExpression.with_child(Span::new(15, 39), SyntaxId(8)),
            PathSegment.empty(Span::new(25, 26)),
            Path.with_child(Span::new(25, 26), SyntaxId(5)),
            AdditionAssignmentStatement.with_binary_children(
                Span::new(25, 33),
                SyntaxId(6),
                SyntaxId(7)
            ),
            IntegerExpression.with_value(Span::new(30, 32), SyntaxPayload::encode_integer(42)),
        ]
    );
}
