use crate::{parser::parse_main, syntax_tree::SyntaxKind};

#[test]
fn unclosed_block() {
    let source = "{";
    let (syntax_tree, error) = parse_main(source.to_string());

    assert!(error.is_some());

    let nodes = syntax_tree.sorted_nodes();
    assert!(!nodes.is_empty());
    assert_eq!(nodes[0].kind, SyntaxKind::MainFunctionItem);
}

#[test]
fn unclosed_list() {
    let source = "[";
    let (syntax_tree, _error) = parse_main(source.to_string());

    let nodes = syntax_tree.sorted_nodes();
    assert!(!nodes.is_empty());
    assert_eq!(nodes[0].kind, SyntaxKind::MainFunctionItem);
}

#[test]
fn unclosed_call() {
    let source = "test(";
    let (syntax_tree, _error) = parse_main(source.to_string());

    let nodes = syntax_tree.sorted_nodes();
    assert!(!nodes.is_empty());
    assert_eq!(nodes[0].kind, SyntaxKind::MainFunctionItem);
}

#[test]
fn unclosed_function_parameters() {
    let source = "fn test(";
    let (syntax_tree, error) = parse_main(source.to_string());

    assert!(error.is_some());

    let nodes = syntax_tree.sorted_nodes();
    assert!(!nodes.is_empty());
    assert_eq!(nodes[0].kind, SyntaxKind::MainFunctionItem);
}

#[test]
fn nested_unclosed_blocks() {
    let source = "{ { {";
    let (syntax_tree, error) = parse_main(source.to_string());

    assert!(error.is_some());

    let nodes = syntax_tree.sorted_nodes();
    assert!(!nodes.is_empty());
    assert_eq!(nodes[0].kind, SyntaxKind::MainFunctionItem);
}

#[test]
fn mixed_unclosed_delimiters() {
    let source = "{ [ (";
    let (syntax_tree, error) = parse_main(source.to_string());

    assert!(error.is_some());

    let nodes = syntax_tree.sorted_nodes();
    assert!(!nodes.is_empty());
    assert_eq!(nodes[0].kind, SyntaxKind::MainFunctionItem);
}

#[test]
fn unclosed_block_with_content() {
    let source = "{ let x = 42";
    let (syntax_tree, error) = parse_main(source.to_string());

    assert!(error.is_some());

    let nodes = syntax_tree.sorted_nodes();
    assert!(!nodes.is_empty());
    assert_eq!(nodes[0].kind, SyntaxKind::MainFunctionItem);
}
