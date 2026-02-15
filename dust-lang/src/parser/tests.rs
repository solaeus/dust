use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{SourceFileId, Span},
    syntax::{SyntaxId, SyntaxKind::*, SyntaxPayload},
    tests::*,
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
        [
            Root.with_child(Span::new(0, 18), SyntaxId(7)),
            FunctionItem.with_binary_children(Span::new(0, 18), SyntaxId(2), SyntaxId(6)),
            PathSegment.empty(Span::new(3, 7)),
            Path.with_child(Span::new(3, 7), SyntaxId(1)),
            ValueParameters.empty(Span::new(7, 9)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
            FunctionExpression.with_binary_children(Span::new(7, 18), SyntaxId(4), SyntaxId(5)),
            BlockExpression.empty(Span::new(10, 18)),
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
        [
            Root.with_child(Span::new(0, 29), SyntaxId(11)),
            FunctionItem.with_binary_children(Span::new(0, 29), SyntaxId(2), SyntaxId(10)),
            PathSegment.empty(Span::new(3, 7)),
            Path.with_child(Span::new(3, 7), SyntaxId(1)),
            ValueParameters.empty(Span::new(7, 9)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
            FunctionExpression.with_binary_children(Span::new(7, 29), SyntaxId(4), SyntaxId(9)),
            BlockExpression.with_child(Span::new(10, 29), SyntaxId(8)),
            LetStatement.with_binary_children(Span::new(16, 27), SyntaxId(6), SyntaxId(7)),
            PathSegment.empty(Span::new(20, 21)),
            Path.with_child(Span::new(20, 21), SyntaxId(5)),
            IntegerExpression.with_value(Span::new(24, 26), SyntaxPayload::encode_integer(42)),
        ]
    );
}

#[test]
fn let_statement_with_type() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(LET_STATEMENT_WITH_TYPE),
    );
    let ParseResult {
        syntax_tree,
        errors,
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 34), SyntaxId(12)),
            FunctionItem.with_binary_children(Span::new(0, 34), SyntaxId(2), SyntaxId(11)),
            PathSegment.empty(Span::new(3, 7)),
            Path.with_child(Span::new(3, 7), SyntaxId(1)),
            ValueParameters.empty(Span::new(7, 9)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
            FunctionExpression.with_binary_children(Span::new(7, 34), SyntaxId(4), SyntaxId(10)),
            BlockExpression.with_child(Span::new(10, 34), SyntaxId(9)),
            LetStatement.with_multiple_children(Span::new(16, 32), 0, 3),
            PathSegment.empty(Span::new(20, 21)),
            Path.with_child(Span::new(20, 21), SyntaxId(5)),
            IntegerType.empty(Span::new(23, 26)),
            IntegerExpression.with_value(Span::new(29, 31), SyntaxPayload::encode_integer(42)),
        ]
    );
}

#[test]
fn let_mut_statement() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(LET_MUT_STATEMENT));
    let ParseResult {
        syntax_tree,
        errors,
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 33), SyntaxId(11)),
            FunctionItem.with_binary_children(Span::new(0, 33), SyntaxId(2), SyntaxId(10)),
            PathSegment.empty(Span::new(3, 7)),
            Path.with_child(Span::new(3, 7), SyntaxId(1)),
            ValueParameters.empty(Span::new(7, 9)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
            FunctionExpression.with_binary_children(Span::new(7, 33), SyntaxId(4), SyntaxId(9)),
            BlockExpression.with_child(Span::new(10, 33), SyntaxId(8)),
            LetMutStatement.with_binary_children(Span::new(16, 31), SyntaxId(6), SyntaxId(7)),
            PathSegment.empty(Span::new(24, 25)),
            Path.with_child(Span::new(24, 25), SyntaxId(5)),
            IntegerExpression.with_value(Span::new(28, 30), SyntaxPayload::encode_integer(42)),
        ]
    );
}

#[test]
fn let_mut_statement_with_type() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(LET_MUT_STATEMENT_WITH_TYPE),
    );
    let ParseResult {
        syntax_tree,
        errors,
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 38), SyntaxId(12)),
            FunctionItem.with_binary_children(Span::new(0, 38), SyntaxId(2), SyntaxId(11)),
            PathSegment.empty(Span::new(3, 7)),
            Path.with_child(Span::new(3, 7), SyntaxId(1)),
            ValueParameters.empty(Span::new(7, 9)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
            FunctionExpression.with_binary_children(Span::new(7, 38), SyntaxId(4), SyntaxId(10)),
            BlockExpression.with_child(Span::new(10, 38), SyntaxId(9)),
            LetMutStatement.with_multiple_children(Span::new(16, 36), 0, 3),
            PathSegment.empty(Span::new(24, 25)),
            Path.with_child(Span::new(24, 25), SyntaxId(5)),
            IntegerType.empty(Span::new(27, 30)),
            IntegerExpression.with_value(Span::new(33, 35), SyntaxPayload::encode_integer(42)),
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
        [
            Root.with_child(Span::new(0, 25), SyntaxId(10)),
            FunctionItem.with_binary_children(Span::new(0, 25), SyntaxId(2), SyntaxId(9)),
            PathSegment.empty(Span::new(3, 7)),
            Path.with_child(Span::new(3, 7), SyntaxId(1)),
            ValueParameters.empty(Span::new(7, 9)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
            FunctionExpression.with_binary_children(Span::new(7, 25), SyntaxId(4), SyntaxId(8)),
            BlockExpression.with_child(Span::new(10, 25), SyntaxId(7)),
            PathSegment.empty(Span::new(16, 17)),
            ReassignmentStatement.with_binary_children(Span::new(16, 23), SyntaxId(5), SyntaxId(6)),
            IntegerExpression.with_value(Span::new(20, 22), SyntaxPayload::encode_integer(42)),
        ]
    );
}

mod binary_assignment_statement {
    use super::*;
    use crate::tests::binary_assignment_statement as test_cases;

    #[test]
    fn add_assign() {
        let parser = Parser::new(
            SourceFileId::MAIN,
            Lexer::from_bytes(test_cases::ADD_ASSIGN),
        );
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 26), SyntaxId(11)),
                FunctionItem.with_binary_children(Span::new(0, 26), SyntaxId(2), SyntaxId(10)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 26), SyntaxId(4), SyntaxId(9)),
                BlockExpression.with_child(Span::new(10, 26), SyntaxId(8)),
                PathSegment.empty(Span::new(16, 17)),
                Path.with_child(Span::new(16, 17), SyntaxId(5)),
                AdditionAssignmentStatement.with_binary_children(Span::new(16, 24), SyntaxId(6), SyntaxId(7)),
                IntegerExpression.with_value(Span::new(21, 23), SyntaxPayload::encode_integer(42)),
            ]
        );
    }

    #[test]
    fn subtract_assign() {
        let parser = Parser::new(
            SourceFileId::MAIN,
            Lexer::from_bytes(test_cases::SUBTRACT_ASSIGN),
        );
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 26), SyntaxId(11)),
                FunctionItem.with_binary_children(Span::new(0, 26), SyntaxId(2), SyntaxId(10)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 26), SyntaxId(4), SyntaxId(9)),
                BlockExpression.with_child(Span::new(10, 26), SyntaxId(8)),
                PathSegment.empty(Span::new(16, 17)),
                Path.with_child(Span::new(16, 17), SyntaxId(5)),
                SubtractionAssignmentStatement.with_binary_children(
                    Span::new(16, 24),
                    SyntaxId(6),
                    SyntaxId(7)
                ),
                IntegerExpression.with_value(Span::new(21, 23), SyntaxPayload::encode_integer(42)),
            ]
        );
    }

    #[test]
    fn multiply_assign() {
        let parser = Parser::new(
            SourceFileId::MAIN,
            Lexer::from_bytes(test_cases::MULTIPLY_ASSIGN),
        );
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 26), SyntaxId(11)),
                FunctionItem.with_binary_children(Span::new(0, 26), SyntaxId(2), SyntaxId(10)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 26), SyntaxId(4), SyntaxId(9)),
                BlockExpression.with_child(Span::new(10, 26), SyntaxId(8)),
                PathSegment.empty(Span::new(16, 17)),
                Path.with_child(Span::new(16, 17), SyntaxId(5)),
                MultiplicationAssignmentStatement.with_binary_children(
                    Span::new(16, 24),
                    SyntaxId(6),
                    SyntaxId(7)
                ),
                IntegerExpression.with_value(Span::new(21, 23), SyntaxPayload::encode_integer(42)),
            ]
        );
    }

    #[test]
    fn divide_assign() {
        let parser = Parser::new(
            SourceFileId::MAIN,
            Lexer::from_bytes(test_cases::DIVIDE_ASSIGN),
        );
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 26), SyntaxId(11)),
                FunctionItem.with_binary_children(Span::new(0, 26), SyntaxId(2), SyntaxId(10)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 26), SyntaxId(4), SyntaxId(9)),
                BlockExpression.with_child(Span::new(10, 26), SyntaxId(8)),
                PathSegment.empty(Span::new(16, 17)),
                Path.with_child(Span::new(16, 17), SyntaxId(5)),
                DivisionAssignmentStatement.with_binary_children(Span::new(16, 24), SyntaxId(6), SyntaxId(7)),
                IntegerExpression.with_value(Span::new(21, 23), SyntaxPayload::encode_integer(42)),
            ]
        );
    }

    #[test]
    fn modulo_assign() {
        let parser = Parser::new(
            SourceFileId::MAIN,
            Lexer::from_bytes(test_cases::MODULO_ASSIGN),
        );
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 26), SyntaxId(11)),
                FunctionItem.with_binary_children(Span::new(0, 26), SyntaxId(2), SyntaxId(10)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 26), SyntaxId(4), SyntaxId(9)),
                BlockExpression.with_child(Span::new(10, 26), SyntaxId(8)),
                PathSegment.empty(Span::new(16, 17)),
                Path.with_child(Span::new(16, 17), SyntaxId(5)),
                ModuloAssignmentStatement.with_binary_children(Span::new(16, 24), SyntaxId(6), SyntaxId(7)),
                IntegerExpression.with_value(Span::new(21, 23), SyntaxPayload::encode_integer(42)),
            ]
        );
    }

    #[test]
    fn power_assign() {
        let parser = Parser::new(
            SourceFileId::MAIN,
            Lexer::from_bytes(test_cases::POWER_ASSIGN),
        );
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 26), SyntaxId(11)),
                FunctionItem.with_binary_children(Span::new(0, 26), SyntaxId(2), SyntaxId(10)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 26), SyntaxId(4), SyntaxId(9)),
                BlockExpression.with_child(Span::new(10, 26), SyntaxId(8)),
                PathSegment.empty(Span::new(16, 17)),
                Path.with_child(Span::new(16, 17), SyntaxId(5)),
                ExponentAssignmentStatement.with_binary_children(Span::new(16, 24), SyntaxId(6), SyntaxId(7)),
                IntegerExpression.with_value(Span::new(21, 23), SyntaxPayload::encode_integer(42)),
            ]
        );
    }
}

mod value_expression {
    use super::*;
    use crate::tests::value_expression as test_cases;

    #[test]
    fn boolean() {
        let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(test_cases::BOOLEAN));
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 22), SyntaxId(8)),
                FunctionItem.with_binary_children(Span::new(0, 22), SyntaxId(2), SyntaxId(7)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 22), SyntaxId(4), SyntaxId(6)),
                BlockExpression.with_child(Span::new(10, 22), SyntaxId(5)),
                BooleanExpression
                    .with_value(Span::new(16, 20), SyntaxPayload::encode_boolean(true)),
            ]
        );
    }

    #[test]
    fn byte() {
        let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(test_cases::BYTE));
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 22), SyntaxId(8)),
                FunctionItem.with_binary_children(Span::new(0, 22), SyntaxId(2), SyntaxId(7)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 22), SyntaxId(4), SyntaxId(6)),
                BlockExpression.with_child(Span::new(10, 22), SyntaxId(5)),
                ByteExpression.with_value(Span::new(16, 20), SyntaxPayload::encode_byte(42)),
            ]
        );
    }

    #[test]
    fn character() {
        let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(test_cases::CHARACTER));
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 21), SyntaxId(8)),
                FunctionItem.with_binary_children(Span::new(0, 21), SyntaxId(2), SyntaxId(7)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 21), SyntaxId(4), SyntaxId(6)),
                BlockExpression.with_child(Span::new(10, 21), SyntaxId(5)),
                CharacterExpression
                    .with_value(Span::new(16, 19), SyntaxPayload::encode_character('a')),
            ]
        );
    }

    #[test]
    fn float() {
        let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(test_cases::FLOAT));
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 22), SyntaxId(8)),
                FunctionItem.with_binary_children(Span::new(0, 22), SyntaxId(2), SyntaxId(7)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 22), SyntaxId(4), SyntaxId(6)),
                BlockExpression.with_child(Span::new(10, 22), SyntaxId(5)),
                FloatExpression.with_value(Span::new(16, 20), SyntaxPayload::encode_float(42.0)),
            ]
        );
    }

    #[test]
    fn integer() {
        let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(test_cases::INTEGER));
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 20), SyntaxId(8)),
                FunctionItem.with_binary_children(Span::new(0, 20), SyntaxId(2), SyntaxId(7)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 20), SyntaxId(4), SyntaxId(6)),
                BlockExpression.with_child(Span::new(10, 20), SyntaxId(5)),
                IntegerExpression.with_value(Span::new(16, 18), SyntaxPayload::encode_integer(42)),
            ]
        );
    }

    #[test]
    fn string() {
        let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(test_cases::STRING));
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 33), SyntaxId(8)),
                FunctionItem.with_binary_children(Span::new(0, 33), SyntaxId(2), SyntaxId(7)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 33), SyntaxId(4), SyntaxId(6)),
                BlockExpression.with_child(Span::new(10, 33), SyntaxId(5)),
                StringExpression
                    .with_value(Span::new(16, 31), SyntaxPayload::encode_string(b"Hello, ")),
            ]
        );
    }

    #[test]
    fn list() {
        let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(test_cases::LIST));
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 27), SyntaxId(11)),
                FunctionItem.with_binary_children(Span::new(0, 27), SyntaxId(2), SyntaxId(10)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 27), SyntaxId(4), SyntaxId(9)),
                BlockExpression.with_child(Span::new(10, 27), SyntaxId(8)),
                ListExpression.with_multiple_children(Span::new(16, 25), 0, 3),
                IntegerExpression.with_value(Span::new(17, 18), SyntaxPayload::encode_integer(1)),
                IntegerExpression.with_value(Span::new(20, 21), SyntaxPayload::encode_integer(2)),
                IntegerExpression.with_value(Span::new(23, 24), SyntaxPayload::encode_integer(3)),
            ]
        );
    }

    #[test]
    fn function() {
        let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(test_cases::FUNCTION));
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 25), SyntaxId(11)),
                FunctionItem.with_binary_children(Span::new(0, 25), SyntaxId(2), SyntaxId(10)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 25), SyntaxId(4), SyntaxId(9)),
                BlockExpression.with_child(Span::new(10, 25), SyntaxId(8)),
                ValueParameters.empty(Span::new(18, 20)),
                FunctionSignature.with_child(Span::new(18, 20), SyntaxId(5)),
                FunctionExpression.with_binary_children(Span::new(18, 23), SyntaxId(6), SyntaxId(7)),
                BlockExpression.empty(Span::new(21, 23)),
            ]
        );
    }
}

mod unary_expression {
    use super::*;
    use crate::tests::unary_expression as test_cases;

    #[test]
    fn negation() {
        let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(test_cases::NEGATION));
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 20), SyntaxId(11)),
                FunctionItem.with_binary_children(Span::new(0, 20), SyntaxId(2), SyntaxId(10)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 20), SyntaxId(4), SyntaxId(9)),
                BlockExpression.with_child(Span::new(10, 20), SyntaxId(8)),
                NegationExpression.with_child(Span::new(16, 18), SyntaxId(7)),
                PathSegment.empty(Span::new(17, 18)),
                Path.with_child(Span::new(17, 18), SyntaxId(5)),
                PathExpression.with_child(Span::new(17, 18), SyntaxId(6)),
            ]
        );
    }

    #[test]
    fn logical_not() {
        let parser = Parser::new(
            SourceFileId::MAIN,
            Lexer::from_bytes(test_cases::LOGICAL_NOT),
        );
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 20), SyntaxId(11)),
                FunctionItem.with_binary_children(Span::new(0, 20), SyntaxId(2), SyntaxId(10)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 20), SyntaxId(4), SyntaxId(9)),
                BlockExpression.with_child(Span::new(10, 20), SyntaxId(8)),
                NotExpression.with_child(Span::new(16, 18), SyntaxId(7)),
                PathSegment.empty(Span::new(17, 18)),
                Path.with_child(Span::new(17, 18), SyntaxId(5)),
                PathExpression.with_child(Span::new(17, 18), SyntaxId(6)),
            ]
        );
    }
}

mod binary_expression {
    use super::*;
    use crate::tests::binary_expression as test_cases;

    #[test]
    fn addition() {
        let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(test_cases::ADDITION));
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 23), SyntaxId(13)),
                FunctionItem.with_binary_children(Span::new(0, 23), SyntaxId(2), SyntaxId(12)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 23), SyntaxId(4), SyntaxId(11)),
                BlockExpression.with_child(Span::new(10, 23), SyntaxId(10)),
                PathSegment.empty(Span::new(16, 17)),
                Path.with_child(Span::new(16, 17), SyntaxId(5)),
                AdditionExpression.with_binary_children(Span::new(16, 21), SyntaxId(6), SyntaxId(9)),
                PathSegment.empty(Span::new(20, 21)),
                Path.with_child(Span::new(20, 21), SyntaxId(7)),
                PathExpression.with_child(Span::new(20, 21), SyntaxId(8)),
            ]
        );
    }

    #[test]
    fn subtraction() {
        let parser = Parser::new(
            SourceFileId::MAIN,
            Lexer::from_bytes(test_cases::SUBTRACTION),
        );
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 23), SyntaxId(13)),
                FunctionItem.with_binary_children(Span::new(0, 23), SyntaxId(2), SyntaxId(12)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 23), SyntaxId(4), SyntaxId(11)),
                BlockExpression.with_child(Span::new(10, 23), SyntaxId(10)),
                PathSegment.empty(Span::new(16, 17)),
                Path.with_child(Span::new(16, 17), SyntaxId(5)),
                SubtractionExpression.with_binary_children(Span::new(16, 21), SyntaxId(6), SyntaxId(9)),
                PathSegment.empty(Span::new(20, 21)),
                Path.with_child(Span::new(20, 21), SyntaxId(7)),
                PathExpression.with_child(Span::new(20, 21), SyntaxId(8)),
            ]
        );
    }

    #[test]
    fn multiplication() {
        let parser = Parser::new(
            SourceFileId::MAIN,
            Lexer::from_bytes(test_cases::MULTIPLICATION),
        );
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 23), SyntaxId(13)),
                FunctionItem.with_binary_children(Span::new(0, 23), SyntaxId(2), SyntaxId(12)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 23), SyntaxId(4), SyntaxId(11)),
                BlockExpression.with_child(Span::new(10, 23), SyntaxId(10)),
                PathSegment.empty(Span::new(16, 17)),
                Path.with_child(Span::new(16, 17), SyntaxId(5)),
                MultiplicationExpression.with_binary_children(Span::new(16, 21), SyntaxId(6), SyntaxId(9)),
                PathSegment.empty(Span::new(20, 21)),
                Path.with_child(Span::new(20, 21), SyntaxId(7)),
                PathExpression.with_child(Span::new(20, 21), SyntaxId(8)),
            ]
        );
    }

    #[test]
    fn division() {
        let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(test_cases::DIVISION));
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 23), SyntaxId(13)),
                FunctionItem.with_binary_children(Span::new(0, 23), SyntaxId(2), SyntaxId(12)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 23), SyntaxId(4), SyntaxId(11)),
                BlockExpression.with_child(Span::new(10, 23), SyntaxId(10)),
                PathSegment.empty(Span::new(16, 17)),
                Path.with_child(Span::new(16, 17), SyntaxId(5)),
                DivisionExpression.with_binary_children(Span::new(16, 21), SyntaxId(6), SyntaxId(9)),
                PathSegment.empty(Span::new(20, 21)),
                Path.with_child(Span::new(20, 21), SyntaxId(7)),
                PathExpression.with_child(Span::new(20, 21), SyntaxId(8)),
            ]
        );
    }

    #[test]
    fn modulo() {
        let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(test_cases::MODULO));
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 23), SyntaxId(13)),
                FunctionItem.with_binary_children(Span::new(0, 23), SyntaxId(2), SyntaxId(12)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 23), SyntaxId(4), SyntaxId(11)),
                BlockExpression.with_child(Span::new(10, 23), SyntaxId(10)),
                PathSegment.empty(Span::new(16, 17)),
                Path.with_child(Span::new(16, 17), SyntaxId(5)),
                ModuloExpression.with_binary_children(Span::new(16, 21), SyntaxId(6), SyntaxId(9)),
                PathSegment.empty(Span::new(20, 21)),
                Path.with_child(Span::new(20, 21), SyntaxId(7)),
                PathExpression.with_child(Span::new(20, 21), SyntaxId(8)),
            ]
        );
    }

    #[test]
    fn power() {
        let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(test_cases::POWER));
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 23), SyntaxId(13)),
                FunctionItem.with_binary_children(Span::new(0, 23), SyntaxId(2), SyntaxId(12)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 23), SyntaxId(4), SyntaxId(11)),
                BlockExpression.with_child(Span::new(10, 23), SyntaxId(10)),
                PathSegment.empty(Span::new(16, 17)),
                Path.with_child(Span::new(16, 17), SyntaxId(5)),
                ExponentExpression.with_binary_children(Span::new(16, 21), SyntaxId(6), SyntaxId(9)),
                PathSegment.empty(Span::new(20, 21)),
                Path.with_child(Span::new(20, 21), SyntaxId(7)),
                PathExpression.with_child(Span::new(20, 21), SyntaxId(8)),
            ]
        );
    }

    #[test]
    fn equal() {
        let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(test_cases::EQUAL));
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 24), SyntaxId(13)),
                FunctionItem.with_binary_children(Span::new(0, 24), SyntaxId(2), SyntaxId(12)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 24), SyntaxId(4), SyntaxId(11)),
                BlockExpression.with_child(Span::new(10, 24), SyntaxId(10)),
                PathSegment.empty(Span::new(16, 17)),
                Path.with_child(Span::new(16, 17), SyntaxId(5)),
                EqualExpression.with_binary_children(Span::new(16, 22), SyntaxId(6), SyntaxId(9)),
                PathSegment.empty(Span::new(21, 22)),
                Path.with_child(Span::new(21, 22), SyntaxId(7)),
                PathExpression.with_child(Span::new(21, 22), SyntaxId(8)),
            ]
        );
    }

    #[test]
    fn not_equal() {
        let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(test_cases::NOT_EQUAL));
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 24), SyntaxId(13)),
                FunctionItem.with_binary_children(Span::new(0, 24), SyntaxId(2), SyntaxId(12)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 24), SyntaxId(4), SyntaxId(11)),
                BlockExpression.with_child(Span::new(10, 24), SyntaxId(10)),
                PathSegment.empty(Span::new(16, 17)),
                Path.with_child(Span::new(16, 17), SyntaxId(5)),
                NotEqualExpression.with_binary_children(Span::new(16, 22), SyntaxId(6), SyntaxId(9)),
                PathSegment.empty(Span::new(21, 22)),
                Path.with_child(Span::new(21, 22), SyntaxId(7)),
                PathExpression.with_child(Span::new(21, 22), SyntaxId(8)),
            ]
        );
    }

    #[test]
    fn less_than() {
        let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(test_cases::LESS_THAN));
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 23), SyntaxId(13)),
                FunctionItem.with_binary_children(Span::new(0, 23), SyntaxId(2), SyntaxId(12)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 23), SyntaxId(4), SyntaxId(11)),
                BlockExpression.with_child(Span::new(10, 23), SyntaxId(10)),
                PathSegment.empty(Span::new(16, 17)),
                Path.with_child(Span::new(16, 17), SyntaxId(5)),
                LessThanExpression.with_binary_children(Span::new(16, 21), SyntaxId(6), SyntaxId(9)),
                PathSegment.empty(Span::new(20, 21)),
                Path.with_child(Span::new(20, 21), SyntaxId(7)),
                PathExpression.with_child(Span::new(20, 21), SyntaxId(8)),
            ]
        );
    }

    #[test]
    fn less_than_or_equal() {
        let parser = Parser::new(
            SourceFileId::MAIN,
            Lexer::from_bytes(test_cases::LESS_THAN_OR_EQUAL),
        );
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 24), SyntaxId(13)),
                FunctionItem.with_binary_children(Span::new(0, 24), SyntaxId(2), SyntaxId(12)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 24), SyntaxId(4), SyntaxId(11)),
                BlockExpression.with_child(Span::new(10, 24), SyntaxId(10)),
                PathSegment.empty(Span::new(16, 17)),
                Path.with_child(Span::new(16, 17), SyntaxId(5)),
                LessThanOrEqualExpression.with_binary_children(Span::new(16, 22), SyntaxId(6), SyntaxId(9)),
                PathSegment.empty(Span::new(21, 22)),
                Path.with_child(Span::new(21, 22), SyntaxId(7)),
                PathExpression.with_child(Span::new(21, 22), SyntaxId(8)),
            ]
        );
    }

    #[test]
    fn greater_than() {
        let parser = Parser::new(
            SourceFileId::MAIN,
            Lexer::from_bytes(test_cases::GREATER_THAN),
        );
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 23), SyntaxId(13)),
                FunctionItem.with_binary_children(Span::new(0, 23), SyntaxId(2), SyntaxId(12)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 23), SyntaxId(4), SyntaxId(11)),
                BlockExpression.with_child(Span::new(10, 23), SyntaxId(10)),
                PathSegment.empty(Span::new(16, 17)),
                Path.with_child(Span::new(16, 17), SyntaxId(5)),
                GreaterThanExpression.with_binary_children(Span::new(16, 21), SyntaxId(6), SyntaxId(9)),
                PathSegment.empty(Span::new(20, 21)),
                Path.with_child(Span::new(20, 21), SyntaxId(7)),
                PathExpression.with_child(Span::new(20, 21), SyntaxId(8)),
            ]
        );
    }

    #[test]
    fn greater_than_or_equal() {
        let parser = Parser::new(
            SourceFileId::MAIN,
            Lexer::from_bytes(test_cases::GREATER_THAN_OR_EQUAL),
        );
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 24), SyntaxId(13)),
                FunctionItem.with_binary_children(Span::new(0, 24), SyntaxId(2), SyntaxId(12)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 24), SyntaxId(4), SyntaxId(11)),
                BlockExpression.with_child(Span::new(10, 24), SyntaxId(10)),
                PathSegment.empty(Span::new(16, 17)),
                Path.with_child(Span::new(16, 17), SyntaxId(5)),
                GreaterThanOrEqualExpression.with_binary_children(Span::new(16, 22), SyntaxId(6), SyntaxId(9)),
                PathSegment.empty(Span::new(21, 22)),
                Path.with_child(Span::new(21, 22), SyntaxId(7)),
                PathExpression.with_child(Span::new(21, 22), SyntaxId(8)),
            ]
        );
    }

    #[test]
    fn logical_and() {
        let parser = Parser::new(
            SourceFileId::MAIN,
            Lexer::from_bytes(test_cases::LOGICAL_AND),
        );
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 24), SyntaxId(13)),
                FunctionItem.with_binary_children(Span::new(0, 24), SyntaxId(2), SyntaxId(12)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 24), SyntaxId(4), SyntaxId(11)),
                BlockExpression.with_child(Span::new(10, 24), SyntaxId(10)),
                PathSegment.empty(Span::new(16, 17)),
                Path.with_child(Span::new(16, 17), SyntaxId(5)),
                AndExpression.with_binary_children(Span::new(16, 22), SyntaxId(6), SyntaxId(9)),
                PathSegment.empty(Span::new(21, 22)),
                Path.with_child(Span::new(21, 22), SyntaxId(7)),
                PathExpression.with_child(Span::new(21, 22), SyntaxId(8)),
            ]
        );
    }

    #[test]
    fn logical_or() {
        let parser = Parser::new(
            SourceFileId::MAIN,
            Lexer::from_bytes(test_cases::LOGICAL_OR),
        );
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 24), SyntaxId(13)),
                FunctionItem.with_binary_children(Span::new(0, 24), SyntaxId(2), SyntaxId(12)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 24), SyntaxId(4), SyntaxId(11)),
                BlockExpression.with_child(Span::new(10, 24), SyntaxId(10)),
                PathSegment.empty(Span::new(16, 17)),
                Path.with_child(Span::new(16, 17), SyntaxId(5)),
                OrExpression.with_binary_children(Span::new(16, 22), SyntaxId(6), SyntaxId(9)),
                PathSegment.empty(Span::new(21, 22)),
                Path.with_child(Span::new(21, 22), SyntaxId(7)),
                PathExpression.with_child(Span::new(21, 22), SyntaxId(8)),
            ]
        );
    }
}

#[test]
fn grouped_expression() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(GROUPED_EXPRESSION));
    let ParseResult {
        syntax_tree,
        errors,
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 25), SyntaxId(14)),
            FunctionItem.with_binary_children(Span::new(0, 25), SyntaxId(2), SyntaxId(13)),
            PathSegment.empty(Span::new(3, 7)),
            Path.with_child(Span::new(3, 7), SyntaxId(1)),
            ValueParameters.empty(Span::new(7, 9)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
            FunctionExpression.with_binary_children(Span::new(7, 25), SyntaxId(4), SyntaxId(12)),
            BlockExpression.with_child(Span::new(10, 25), SyntaxId(11)),
            GroupedExpression.with_child(Span::new(16, 23), SyntaxId(10)),
            PathSegment.empty(Span::new(17, 18)),
            Path.with_child(Span::new(17, 18), SyntaxId(5)),
            AdditionExpression.with_binary_children(Span::new(17, 22), SyntaxId(6), SyntaxId(9)),
            PathSegment.empty(Span::new(21, 22)),
            Path.with_child(Span::new(21, 22), SyntaxId(7)),
            PathExpression.with_child(Span::new(21, 22), SyntaxId(8)),
        ]
    );
}

mod block_expression {
    use super::*;
    use crate::tests::block_expression as test_cases;

    #[test]
    fn empty() {
        let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(test_cases::EMPTY));
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 20), SyntaxId(8)),
                FunctionItem.with_binary_children(Span::new(0, 20), SyntaxId(2), SyntaxId(7)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 20), SyntaxId(4), SyntaxId(6)),
                BlockExpression.with_child(Span::new(10, 20), SyntaxId(5)),
                BlockExpression.empty(Span::new(16, 18)),
            ]
        );
    }

    #[test]
    fn item() {
        let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(test_cases::ITEM));
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 33), SyntaxId(15)),
                FunctionItem.with_binary_children(Span::new(0, 33), SyntaxId(2), SyntaxId(14)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 33), SyntaxId(4), SyntaxId(13)),
                BlockExpression.with_child(Span::new(10, 33), SyntaxId(12)),
                BlockExpression.with_child(Span::new(16, 31), SyntaxId(11)),
                FunctionItem.with_binary_children(Span::new(18, 30), SyntaxId(6), SyntaxId(10)),
                PathSegment.empty(Span::new(21, 24)),
                Path.with_child(Span::new(21, 24), SyntaxId(5)),
                ValueParameters.empty(Span::new(24, 26)),
                FunctionSignature.with_child(Span::new(24, 26), SyntaxId(7)),
                FunctionExpression.with_binary_children(Span::new(24, 29), SyntaxId(8), SyntaxId(9)),
                BlockExpression.empty(Span::new(27, 29)),
            ]
        );
    }

    #[test]
    fn statement() {
        let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(test_cases::STATEMENT));
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 33), SyntaxId(12)),
                FunctionItem.with_binary_children(Span::new(0, 33), SyntaxId(2), SyntaxId(11)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 33), SyntaxId(4), SyntaxId(10)),
                BlockExpression.with_child(Span::new(10, 33), SyntaxId(9)),
                BlockExpression.with_child(Span::new(16, 31), SyntaxId(8)),
                LetStatement.with_binary_children(Span::new(18, 29), SyntaxId(6), SyntaxId(7)),
                PathSegment.empty(Span::new(22, 23)),
                Path.with_child(Span::new(22, 23), SyntaxId(5)),
                IntegerExpression.with_value(Span::new(26, 28), SyntaxPayload::encode_integer(42)),
            ]
        );
    }

    #[test]
    fn expression() {
        let parser = Parser::new(
            SourceFileId::MAIN,
            Lexer::from_bytes(test_cases::EXPRESSION),
        );
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 27), SyntaxId(14)),
                FunctionItem.with_binary_children(Span::new(0, 27), SyntaxId(2), SyntaxId(13)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 27), SyntaxId(4), SyntaxId(12)),
                BlockExpression.with_child(Span::new(10, 27), SyntaxId(11)),
                BlockExpression.with_child(Span::new(16, 25), SyntaxId(10)),
                PathSegment.empty(Span::new(18, 19)),
                Path.with_child(Span::new(18, 19), SyntaxId(5)),
                AdditionExpression.with_binary_children(Span::new(18, 23), SyntaxId(6), SyntaxId(9)),
                PathSegment.empty(Span::new(22, 23)),
                Path.with_child(Span::new(22, 23), SyntaxId(7)),
                PathExpression.with_child(Span::new(22, 23), SyntaxId(8)),
            ]
        );
    }

    #[test]
    fn mixed() {
        let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(test_cases::MIXED));
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 51), SyntaxId(25)),
                FunctionItem.with_binary_children(Span::new(0, 51), SyntaxId(2), SyntaxId(24)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 51), SyntaxId(4), SyntaxId(23)),
                BlockExpression.with_child(Span::new(10, 51), SyntaxId(22)),
                BlockExpression.with_multiple_children(Span::new(16, 49), 0, 3),
                FunctionItem.with_binary_children(Span::new(18, 30), SyntaxId(6), SyntaxId(10)),
                PathSegment.empty(Span::new(21, 24)),
                Path.with_child(Span::new(21, 24), SyntaxId(5)),
                ValueParameters.empty(Span::new(24, 26)),
                FunctionSignature.with_child(Span::new(24, 26), SyntaxId(7)),
                FunctionExpression.with_binary_children(Span::new(24, 29), SyntaxId(8), SyntaxId(9)),
                BlockExpression.empty(Span::new(27, 29)),
                LetStatement.with_binary_children(Span::new(30, 41), SyntaxId(13), SyntaxId(14)),
                PathSegment.empty(Span::new(34, 35)),
                Path.with_child(Span::new(34, 35), SyntaxId(12)),
                IntegerExpression.with_value(Span::new(38, 40), SyntaxPayload::encode_integer(42)),
                PathSegment.empty(Span::new(42, 43)),
                Path.with_child(Span::new(42, 43), SyntaxId(16)),
                AdditionExpression.with_binary_children(Span::new(42, 47), SyntaxId(17), SyntaxId(20)),
                PathSegment.empty(Span::new(46, 47)),
                Path.with_child(Span::new(46, 47), SyntaxId(18)),
                PathExpression.with_child(Span::new(46, 47), SyntaxId(19)),
            ]
        );
    }
}

mod if_expression {
    use super::*;
    use crate::tests::if_expression as test_cases;

    #[test]
    fn if_only() {
        let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(test_cases::IF));
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 40), SyntaxId(18)),
                FunctionItem.with_binary_children(Span::new(0, 40), SyntaxId(2), SyntaxId(17)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 40), SyntaxId(4), SyntaxId(16)),
                BlockExpression.with_child(Span::new(10, 40), SyntaxId(15)),
                IfExpression.with_binary_children(Span::new(16, 38), SyntaxId(7), SyntaxId(14)),
                PathSegment.empty(Span::new(19, 28)),
                Path.with_child(Span::new(19, 28), SyntaxId(5)),
                PathExpression.with_child(Span::new(19, 28), SyntaxId(6)),
                BlockExpression.with_child(Span::new(29, 38), SyntaxId(13)),
                PathSegment.empty(Span::new(31, 32)),
                Path.with_child(Span::new(31, 32), SyntaxId(8)),
                AdditionExpression.with_binary_children(Span::new(31, 36), SyntaxId(9), SyntaxId(12)),
                PathSegment.empty(Span::new(35, 36)),
                Path.with_child(Span::new(35, 36), SyntaxId(10)),
                PathExpression.with_child(Span::new(35, 36), SyntaxId(11)),
            ]
        );
    }

    #[test]
    fn if_else() {
        let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(test_cases::IF_ELSE));
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 55), SyntaxId(26)),
                FunctionItem.with_binary_children(Span::new(0, 55), SyntaxId(2), SyntaxId(25)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 55), SyntaxId(4), SyntaxId(24)),
                BlockExpression.with_child(Span::new(10, 55), SyntaxId(23)),
                IfExpression.with_multiple_children(Span::new(16, 53), 0, 3),
                PathSegment.empty(Span::new(19, 28)),
                Path.with_child(Span::new(19, 28), SyntaxId(5)),
                PathExpression.with_child(Span::new(19, 28), SyntaxId(6)),
                BlockExpression.with_child(Span::new(29, 38), SyntaxId(13)),
                PathSegment.empty(Span::new(31, 32)),
                Path.with_child(Span::new(31, 32), SyntaxId(8)),
                AdditionExpression.with_binary_children(Span::new(31, 36), SyntaxId(9), SyntaxId(12)),
                PathSegment.empty(Span::new(35, 36)),
                Path.with_child(Span::new(35, 36), SyntaxId(10)),
                PathExpression.with_child(Span::new(35, 36), SyntaxId(11)),
                ElseExpression.with_child(Span::new(39, 53), SyntaxId(21)),
                BlockExpression.with_child(Span::new(44, 53), SyntaxId(20)),
                PathSegment.empty(Span::new(46, 47)),
                Path.with_child(Span::new(46, 47), SyntaxId(15)),
                SubtractionExpression.with_binary_children(Span::new(46, 51), SyntaxId(16), SyntaxId(19)),
                PathSegment.empty(Span::new(50, 51)),
                Path.with_child(Span::new(50, 51), SyntaxId(17)),
                PathExpression.with_child(Span::new(50, 51), SyntaxId(18)),
            ]
        );
    }

    #[test]
    fn if_else_if() {
        let parser = Parser::new(
            SourceFileId::MAIN,
            Lexer::from_bytes(test_cases::IF_ELSE_IF),
        );
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");
        assert_eq!(
            syntax_tree.sorted_nodes(),
            [
                Root.with_child(Span::new(0, 74), SyntaxId(38)),
                FunctionItem.with_binary_children(Span::new(0, 74), SyntaxId(2), SyntaxId(37)),
                PathSegment.empty(Span::new(3, 7)),
                Path.with_child(Span::new(3, 7), SyntaxId(1)),
                ValueParameters.empty(Span::new(7, 9)),
                FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
                FunctionExpression.with_binary_children(Span::new(7, 74), SyntaxId(4), SyntaxId(36)),
                BlockExpression.with_child(Span::new(10, 74), SyntaxId(35)),
                IfExpression.with_multiple_children(Span::new(16, 72), 3, 3),
                PathSegment.empty(Span::new(19, 23)),
                Path.with_child(Span::new(19, 23), SyntaxId(5)),
                PathExpression.with_child(Span::new(19, 23), SyntaxId(6)),
                BlockExpression.with_child(Span::new(24, 33), SyntaxId(13)),
                PathSegment.empty(Span::new(26, 27)),
                Path.with_child(Span::new(26, 27), SyntaxId(8)),
                AdditionExpression.with_binary_children(Span::new(26, 31), SyntaxId(9), SyntaxId(12)),
                PathSegment.empty(Span::new(30, 31)),
                Path.with_child(Span::new(30, 31), SyntaxId(10)),
                PathExpression.with_child(Span::new(30, 31), SyntaxId(11)),
                ElseExpression.with_child(Span::new(34, 72), SyntaxId(33)),
                IfExpression.with_multiple_children(Span::new(39, 72), 0, 3),
                PathSegment.empty(Span::new(42, 47)),
                Path.with_child(Span::new(42, 47), SyntaxId(15)),
                PathExpression.with_child(Span::new(42, 47), SyntaxId(16)),
                BlockExpression.with_child(Span::new(48, 57), SyntaxId(23)),
                PathSegment.empty(Span::new(50, 51)),
                Path.with_child(Span::new(50, 51), SyntaxId(18)),
                SubtractionExpression.with_binary_children(Span::new(50, 55), SyntaxId(19), SyntaxId(22)),
                PathSegment.empty(Span::new(54, 55)),
                Path.with_child(Span::new(54, 55), SyntaxId(20)),
                PathExpression.with_child(Span::new(54, 55), SyntaxId(21)),
                ElseExpression.with_child(Span::new(58, 72), SyntaxId(31)),
                BlockExpression.with_child(Span::new(63, 72), SyntaxId(30)),
                PathSegment.empty(Span::new(65, 66)),
                Path.with_child(Span::new(65, 66), SyntaxId(25)),
                MultiplicationExpression.with_binary_children(Span::new(65, 70), SyntaxId(26), SyntaxId(29)),
                PathSegment.empty(Span::new(69, 70)),
                Path.with_child(Span::new(69, 70), SyntaxId(27)),
                PathExpression.with_child(Span::new(69, 70), SyntaxId(28)),
            ]
        );
    }
}
