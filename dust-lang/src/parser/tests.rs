use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{SourceFileId, Span},
    syntax::{SyntaxId, SyntaxKind::*, SyntaxPayload},
    tests::{FUNCTION_ITEM, LET_STATEMENT, REASSIGNMENT_STATEMENT},
};

#[test]
fn function_item() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(FUNCTION_ITEM));
    let ParseResult {
        syntax_tree,
        errors,
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(syntax_tree.sorted_nodes(), []);
}

#[test]
fn let_statement() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(LET_STATEMENT));
    let ParseResult {
        syntax_tree,
        errors,
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(syntax_tree.sorted_nodes(), []);
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
    assert_eq!(syntax_tree.sorted_nodes(), []);
}

mod binary_assignment_statement {
    use super::*;
    use crate::tests::binary_assignment_statement::ADD_ASSIGN;

    #[test]
    fn add_assign() {
        let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(ADD_ASSIGN));
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(syntax_tree.sorted_nodes(), []);
    }
}
