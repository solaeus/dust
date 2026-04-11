use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{FileId, Span},
    syntax::{
        SyntaxId,
        node::{SyntaxKind::*, SyntaxChildren},
    },
};

#[test]
fn empty() {
    let parser = Parser::new(
        FileId::MAIN,
        Lexer::with_unvalidated_source(b"struct Foo {}"),
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
            Root.with_single_child(Span::new(0, 13), SyntaxId(3)),
            StructItem.with_binary_children(Span::new(0, 13), SyntaxId(1), SyntaxId(2)),
            SimplePath.empty(Span::new(7, 10)),
            StructItemStructFields.empty(Span::new(11, 13)),
        ]
    );
}

#[test]
fn tuple() {
    let parser = Parser::new(
        FileId::MAIN,
        Lexer::with_unvalidated_source(b"struct Foo(i64, i64);"),
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
            Root.with_single_child(Span::new(0, 21), SyntaxId(5)),
            StructItem.with_binary_children(Span::new(0, 21), SyntaxId(1), SyntaxId(4)),
            SimplePath.empty(Span::new(7, 10)),
            StructItemTupleFields.with_binary_children(Span::new(10, 20), SyntaxId(2), SyntaxId(3)),
            I64Type.empty(Span::new(11, 14)),
            I64Type.empty(Span::new(16, 19)),
        ]
    );
}

#[test]
fn fields() {
    let parser = Parser::new(
        FileId::MAIN,
        Lexer::with_unvalidated_source(b"struct Foo { x: i64, y: i64 }"),
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
            Root.with_single_child(Span::new(0, 29), SyntaxId(7)),
            StructItem.with_binary_children(Span::new(0, 29), SyntaxId(1), SyntaxId(6)),
            SimplePath.empty(Span::new(7, 10)),
            StructItemStructFields.with_children(Span::new(11, 29), SyntaxChildren::new(0, 4)),
            SimplePath.empty(Span::new(13, 14)),
            I64Type.empty(Span::new(16, 19)),
            SimplePath.empty(Span::new(21, 22)),
            I64Type.empty(Span::new(24, 27)),
        ]
    );
}

#[test]
fn type_parameters() {
    let parser = Parser::new(
        FileId::MAIN,
        Lexer::with_unvalidated_source(b"struct Foo<A, B, C> {}"),
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
            Root.with_single_child(Span::new(0, 22), SyntaxId(10)),
            StructItem.with_children(Span::new(0, 22), SyntaxChildren::new(3, 6)),
            SimplePath.empty(Span::new(7, 10)),
            StructItemStructFields.empty(Span::new(20, 22)),
            TypeParameters.with_children(Span::new(10, 19), SyntaxChildren::new(0, 3)),
            TypeParameter.with_single_child(Span::new(11, 12), SyntaxId(2)),
            SimplePath.empty(Span::new(11, 12)),
            TypeParameter.with_single_child(Span::new(14, 15), SyntaxId(4)),
            SimplePath.empty(Span::new(14, 15)),
            TypeParameter.with_single_child(Span::new(17, 18), SyntaxId(6)),
            SimplePath.empty(Span::new(17, 18)),
        ]
    );
}
