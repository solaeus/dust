use crate::{
    parser::parse,
    parser::syntax::{SyntaxId, SyntaxKind, SyntaxNode, SyntaxPayload},
    source::Span,
    tests::local_cases,
};

#[test]
fn local_byte_addition() {
    let source = local_cases::LOCAL_BYTE_ADDITION;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::child_indices(10, 3),
                span: Span(0, 47)
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::child_indices(1, 3),
                span: Span(1, 20)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(0, 1),
                span: Span(5, 6)
            },
            SyntaxNode {
                kind: SyntaxKind::ByteType,
                payload: SyntaxPayload::empty(),
                span: Span(8, 12)
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::encode_byte(40),
                span: Span(15, 19)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(4)),
                span: Span(15, 20)
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::child_indices(5, 3),
                span: Span(21, 40)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(25, 26)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(4, 1),
                span: Span(25, 26)
            },
            SyntaxNode {
                kind: SyntaxKind::ByteType,
                payload: SyntaxPayload::empty(),
                span: Span(28, 32)
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::encode_byte(2),
                span: Span(35, 39)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(10)),
                span: Span(35, 40)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(41, 42)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(8, 1),
                span: Span(41, 42)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(14)),
                span: Span(41, 42)
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                payload: SyntaxPayload::children(SyntaxId(15), SyntaxId(18)),
                span: Span(41, 46)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(45, 46)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(9, 1),
                span: Span(45, 46)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(17)),
                span: Span(45, 46)
            },
        ]
    );
}

#[test]
fn local_float_addition() {
    let source = local_cases::LOCAL_FLOAT_ADDITION;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::child_indices(10, 3),
                span: Span(0, 48)
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::child_indices(1, 3),
                span: Span(1, 21)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(0, 1),
                span: Span(5, 6)
            },
            SyntaxNode {
                kind: SyntaxKind::FloatType,
                payload: SyntaxPayload::empty(),
                span: Span(8, 13)
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(40.0),
                span: Span(16, 20)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(4)),
                span: Span(16, 21)
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::child_indices(5, 3),
                span: Span(22, 41)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(26, 27)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(4, 1),
                span: Span(26, 27)
            },
            SyntaxNode {
                kind: SyntaxKind::FloatType,
                payload: SyntaxPayload::empty(),
                span: Span(29, 34)
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(2.0),
                span: Span(37, 40)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(10)),
                span: Span(37, 41)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(42, 43)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(8, 1),
                span: Span(42, 43)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(14)),
                span: Span(42, 43)
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                payload: SyntaxPayload::children(SyntaxId(15), SyntaxId(18)),
                span: Span(42, 47)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(46, 47)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(9, 1),
                span: Span(46, 47)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(17)),
                span: Span(46, 47)
            },
        ]
    );
}

#[test]
fn local_integer_addition() {
    let source = local_cases::LOCAL_INTEGER_ADDITION;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::child_indices(10, 3),
                span: Span(0, 40)
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::child_indices(1, 3),
                span: Span(1, 17)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(0, 1),
                span: Span(5, 6)
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                payload: SyntaxPayload::empty(),
                span: Span(8, 11)
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(40),
                span: Span(14, 16)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(4)),
                span: Span(14, 17)
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::child_indices(5, 3),
                span: Span(18, 33)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(22, 23)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(4, 1),
                span: Span(22, 23)
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                payload: SyntaxPayload::empty(),
                span: Span(25, 28)
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(2),
                span: Span(31, 32)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(10)),
                span: Span(31, 33)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(34, 35)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(8, 1),
                span: Span(34, 35)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(14)),
                span: Span(34, 35)
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                payload: SyntaxPayload::children(SyntaxId(15), SyntaxId(18)),
                span: Span(34, 39)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(38, 39)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(9, 1),
                span: Span(38, 39)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(17)),
                span: Span(38, 39)
            },
        ]
    );
}

#[test]
fn local_string_concatenation() {
    let source = local_cases::LOCAL_STRING_CONCATENATION;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    println!("sorted_nodes: {:?}", syntax_tree.sorted_nodes());
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::child_indices(10, 3),
                span: Span(0, 47)
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::child_indices(1, 3),
                span: Span(1, 20)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(0, 1),
                span: Span(5, 6)
            },
            SyntaxNode {
                kind: SyntaxKind::StringType,
                payload: SyntaxPayload::empty(),
                span: Span(8, 11)
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::encode_string(b"foo"),
                span: Span(14, 19)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(4)),
                span: Span(14, 20)
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::child_indices(5, 3),
                span: Span(21, 40)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(25, 26)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(4, 1),
                span: Span(25, 26)
            },
            SyntaxNode {
                kind: SyntaxKind::StringType,
                payload: SyntaxPayload::empty(),
                span: Span(28, 31)
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::encode_string(b"bar"),
                span: Span(34, 39)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(10)),
                span: Span(34, 40)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(41, 42)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(8, 1),
                span: Span(41, 42)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(14)),
                span: Span(41, 42)
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                payload: SyntaxPayload::children(SyntaxId(15), SyntaxId(18)),
                span: Span(41, 46)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(45, 46)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(9, 1),
                span: Span(45, 46)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(17)),
                span: Span(45, 46)
            },
        ]
    );
}

#[test]
fn local_character_concatenation() {
    let source = local_cases::LOCAL_CHARACTER_CONCATENATION;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::child_indices(10, 3),
                span: Span(0, 45)
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::child_indices(1, 3),
                span: Span(1, 19)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(0, 1),
                span: Span(5, 6)
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterType,
                payload: SyntaxPayload::empty(),
                span: Span(8, 12)
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::encode_character('q'),
                span: Span(15, 18)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(4)),
                span: Span(15, 19)
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::child_indices(5, 3),
                span: Span(20, 38)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(24, 25)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(4, 1),
                span: Span(24, 25)
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterType,
                payload: SyntaxPayload::empty(),
                span: Span(27, 31)
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::encode_character('q'),
                span: Span(34, 37)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(10)),
                span: Span(34, 38)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(39, 40)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(8, 1),
                span: Span(39, 40)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(14)),
                span: Span(39, 40)
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                payload: SyntaxPayload::children(SyntaxId(15), SyntaxId(18)),
                span: Span(39, 44)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(43, 44)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(9, 1),
                span: Span(43, 44)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(17)),
                span: Span(43, 44)
            },
        ]
    );
}

#[test]
fn local_string_character_concatenation() {
    let source = local_cases::LOCAL_STRING_CHARACTER_CONCATENATION;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    println!("sorted_nodes: {:?}", syntax_tree.sorted_nodes());
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::child_indices(10, 3),
                span: Span(0, 46)
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::child_indices(1, 3),
                span: Span(1, 20)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(0, 1),
                span: Span(5, 6)
            },
            SyntaxNode {
                kind: SyntaxKind::StringType,
                payload: SyntaxPayload::empty(),
                span: Span(8, 11)
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::encode_string(b"foo"),
                span: Span(14, 19)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(4)),
                span: Span(14, 20)
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::child_indices(5, 3),
                span: Span(21, 39)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(25, 26)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(4, 1),
                span: Span(25, 26)
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterType,
                payload: SyntaxPayload::empty(),
                span: Span(28, 32)
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::encode_character('q'),
                span: Span(35, 38)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(10)),
                span: Span(35, 39)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(40, 41)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(8, 1),
                span: Span(40, 41)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(14)),
                span: Span(40, 41)
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                payload: SyntaxPayload::children(SyntaxId(15), SyntaxId(18)),
                span: Span(40, 45)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(44, 45)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(9, 1),
                span: Span(44, 45)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(17)),
                span: Span(44, 45)
            },
        ]
    );
}

#[test]
fn local_character_string_concatenation() {
    let source = local_cases::LOCAL_CHARACTER_STRING_CONCATENATION;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::child_indices(10, 3),
                span: Span(0, 46)
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::child_indices(1, 3),
                span: Span(1, 19)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(0, 1),
                span: Span(5, 6)
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterType,
                payload: SyntaxPayload::empty(),
                span: Span(8, 12)
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::encode_character('q'),
                span: Span(15, 18)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(4)),
                span: Span(15, 19)
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::child_indices(5, 3),
                span: Span(20, 39)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(24, 25)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(4, 1),
                span: Span(24, 25)
            },
            SyntaxNode {
                kind: SyntaxKind::StringType,
                payload: SyntaxPayload::empty(),
                span: Span(27, 30)
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::encode_string(b"foo"),
                span: Span(33, 38)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(10)),
                span: Span(33, 39)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(40, 41)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(8, 1),
                span: Span(40, 41)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(14)),
                span: Span(40, 41)
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                payload: SyntaxPayload::children(SyntaxId(15), SyntaxId(18)),
                span: Span(40, 45)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(44, 45)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(9, 1),
                span: Span(44, 45)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(17)),
                span: Span(44, 45)
            },
        ]
    );
}

#[test]
fn local_mut_byte_addition() {
    let source = local_cases::LOCAL_MUT_BYTE_ADDITION;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::child_indices(6, 3),
                span: Span(0, 38)
            },
            SyntaxNode {
                kind: SyntaxKind::LetMutStatement,
                payload: SyntaxPayload::child_indices(1, 3),
                span: Span(1, 24)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(9, 10)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(0, 1),
                span: Span(9, 10)
            },
            SyntaxNode {
                kind: SyntaxKind::ByteType,
                payload: SyntaxPayload::empty(),
                span: Span(12, 16)
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::encode_byte(40),
                span: Span(19, 23)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(4)),
                span: Span(19, 24)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(25, 26)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(4, 1),
                span: Span(25, 26)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(8)),
                span: Span(25, 26)
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionAssignmentStatement,
                payload: SyntaxPayload::children(SyntaxId(9), SyntaxId(10)),
                span: Span(25, 35)
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::encode_byte(2),
                span: Span(30, 34)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(36, 37)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(5, 1),
                span: Span(36, 37)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(13)),
                span: Span(36, 37)
            },
        ]
    );
}

#[test]
fn local_mut_float_addition() {
    let source = local_cases::LOCAL_MUT_FLOAT_ADDITION;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::child_indices(6, 3),
                span: Span(0, 38)
            },
            SyntaxNode {
                kind: SyntaxKind::LetMutStatement,
                payload: SyntaxPayload::child_indices(1, 3),
                span: Span(1, 25)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(9, 10)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(0, 1),
                span: Span(9, 10)
            },
            SyntaxNode {
                kind: SyntaxKind::FloatType,
                payload: SyntaxPayload::empty(),
                span: Span(12, 17)
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(40.0),
                span: Span(20, 24)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(4)),
                span: Span(20, 25)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(26, 27)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(4, 1),
                span: Span(26, 27)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(8)),
                span: Span(26, 27)
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionAssignmentStatement,
                payload: SyntaxPayload::children(SyntaxId(9), SyntaxId(10)),
                span: Span(26, 35)
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(2.0),
                span: Span(31, 34)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(36, 37)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(5, 1),
                span: Span(36, 37)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(13)),
                span: Span(36, 37)
            },
        ]
    );
}

#[test]
fn local_mut_integer_addition() {
    let source = local_cases::LOCAL_MUT_INTEGER_ADDITION;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::child_indices(6, 3),
                span: Span(0, 32)
            },
            SyntaxNode {
                kind: SyntaxKind::LetMutStatement,
                payload: SyntaxPayload::child_indices(1, 3),
                span: Span(1, 21)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(9, 10)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(0, 1),
                span: Span(9, 10)
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                payload: SyntaxPayload::empty(),
                span: Span(12, 15)
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(40),
                span: Span(18, 20)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(4)),
                span: Span(18, 21)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(22, 23)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(4, 1),
                span: Span(22, 23)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(8)),
                span: Span(22, 23)
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionAssignmentStatement,
                payload: SyntaxPayload::children(SyntaxId(9), SyntaxId(10)),
                span: Span(22, 29)
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(2),
                span: Span(27, 28)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(30, 31)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(5, 1),
                span: Span(30, 31)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(13)),
                span: Span(30, 31)
            },
        ]
    );
}

#[test]
fn local_mut_string_concatenation() {
    let source = local_cases::LOCAL_MUT_STRING_CONCATENATION;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    println!("sorted_nodes: {:?}", syntax_tree.sorted_nodes());
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::child_indices(6, 3),
                span: Span(0, 39)
            },
            SyntaxNode {
                kind: SyntaxKind::LetMutStatement,
                payload: SyntaxPayload::child_indices(1, 3),
                span: Span(1, 24)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(9, 10)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(0, 1),
                span: Span(9, 10)
            },
            SyntaxNode {
                kind: SyntaxKind::StringType,
                payload: SyntaxPayload::empty(),
                span: Span(12, 15)
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::encode_string(b"foo"),
                span: Span(18, 23)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(4)),
                span: Span(18, 24)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(25, 26)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(4, 1),
                span: Span(25, 26)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(8)),
                span: Span(25, 26)
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionAssignmentStatement,
                payload: SyntaxPayload::children(SyntaxId(9), SyntaxId(10)),
                span: Span(25, 36)
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::encode_string(b"bar"),
                span: Span(30, 35)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(37, 38)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(5, 1),
                span: Span(37, 38)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(13)),
                span: Span(37, 38)
            },
        ]
    );
}

#[test]
fn local_mut_string_character_concatenation() {
    let source = local_cases::LOCAL_MUT_STRING_CHARACTER_CONCATENATION;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    println!("sorted_nodes: {:?}", syntax_tree.sorted_nodes());
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::child_indices(6, 3),
                span: Span(0, 37)
            },
            SyntaxNode {
                kind: SyntaxKind::LetMutStatement,
                payload: SyntaxPayload::child_indices(1, 3),
                span: Span(1, 24)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(9, 10)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(0, 1),
                span: Span(9, 10)
            },
            SyntaxNode {
                kind: SyntaxKind::StringType,
                payload: SyntaxPayload::empty(),
                span: Span(12, 15)
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::encode_string(b"foo"),
                span: Span(18, 23)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(4)),
                span: Span(18, 24)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(25, 26)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(4, 1),
                span: Span(25, 26)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(8)),
                span: Span(25, 26)
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionAssignmentStatement,
                payload: SyntaxPayload::children(SyntaxId(9), SyntaxId(10)),
                span: Span(25, 34)
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::encode_character('q'),
                span: Span(30, 33)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(35, 36)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(5, 1),
                span: Span(35, 36)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(13)),
                span: Span(35, 36)
            },
        ]
    );
}
