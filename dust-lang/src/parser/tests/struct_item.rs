use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{SourceFileId, Span},
    syntax::{SyntaxId, SyntaxKind::*},
};

#[test]
fn empty() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(b"struct Foo {}"));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 13), SyntaxId(3)),
            StructItem.with_binary_children(Span::new(0, 13), SyntaxId(1), SyntaxId(2)),
            SimplePath.empty(Span::new(7, 10)),
            StructFieldsDeclaration.empty(Span::new(11, 13)),
        ]
    );
}

#[test]
fn tuple() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(b"struct Foo(i64, i64);"),
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
            Root.with_child(Span::new(0, 21), SyntaxId(5)),
            StructItem.with_binary_children(Span::new(0, 21), SyntaxId(1), SyntaxId(4)),
            SimplePath.empty(Span::new(7, 10)),
            TupleFieldsDeclaration.with_binary_children(
                Span::new(10, 20),
                SyntaxId(2),
                SyntaxId(3)
            ),
            I64Type.empty(Span::new(11, 14)),
            I64Type.empty(Span::new(16, 19)),
        ]
    );
}

#[test]
fn fields() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(b"struct Foo { x: i64, y: i64 }"),
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
            Root.with_child(Span::new(0, 29), SyntaxId(7)),
            StructItem.with_binary_children(Span::new(0, 29), SyntaxId(1), SyntaxId(6)),
            SimplePath.empty(Span::new(7, 10)),
            StructFieldsDeclaration.with_multiple_children(Span::new(11, 29), 0, 4),
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
        SourceFileId::MAIN,
        Lexer::from_bytes(b"struct Foo<A, B, C> {}"),
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
            Root.with_child(Span::new(0, 22), SyntaxId(7)),
            StructItem.with_multiple_children(Span::new(0, 22), 3, 3),
            SimplePath.empty(Span::new(7, 10)),
            TypeParameters.with_multiple_children(Span::new(10, 19), 0, 3),
            SimplePath.empty(Span::new(11, 12)),
            SimplePath.empty(Span::new(14, 15)),
            SimplePath.empty(Span::new(17, 18)),
            StructFieldsDeclaration.empty(Span::new(20, 22)),
        ]
    );
}
