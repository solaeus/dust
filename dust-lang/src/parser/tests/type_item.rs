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
fn simple() {
    let parser = Parser::new(
        FileId::MAIN,
        Lexer::with_unvalidated_source(b"type Foo = i64;"),
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
            Root.with_single_child(Span::new(0, 15), SyntaxId(3)),
            TypeItem.with_binary_children(Span::new(0, 15), SyntaxId(1), SyntaxId(2)),
            SimplePath.empty(Span::new(5, 8)),
            I64Type.empty(Span::new(11, 14)),
        ]
    );
}

#[test]
fn with_type_parameters() {
    let parser = Parser::new(
        FileId::MAIN,
        Lexer::with_unvalidated_source(b"type Foo<T> = T;"),
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
            Root.with_single_child(Span::new(0, 16), SyntaxId(7)),
            TypeItem.with_children(Span::new(0, 16), SyntaxChildren::new(0, 3)),
            SimplePath.empty(Span::new(5, 8)),
            TypePath.with_single_child(Span::new(14, 15), SyntaxId(5)),
            PathSegment.empty(Span::new(14, 15)),
            TypeParameters.with_single_child(Span::new(8, 11), SyntaxId(3)),
            TypeParameter.with_single_child(Span::new(9, 10), SyntaxId(2)),
            SimplePath.empty(Span::new(9, 10)),
        ]
    );
}
