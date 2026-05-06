use crate::{
    compiler::{resolver::declarations::Definition, tests::bind_declarations},
    source::{Source, SourceCode, SourceCodeId},
    syntax::node::SyntaxKind,
};

#[test]
fn binds_declaration() {
    let mut source = Source::new();

    source.add_code(SourceCode::validated_borrowed(
        "test",
        "fn main() { let x = 1; x; }",
    ));

    let (syntax, resolver, _crate_scope_id) = bind_declarations(&source);

    let tree = syntax.get_tree(SourceCodeId::MAIN).unwrap();
    let path_expr = tree
        .iter()
        .find(|node| node.node.kind == SyntaxKind::PathExpression)
        .unwrap();

    let declaration_id = resolver.get_declaration_binding(&path_expr.id).unwrap();
    let declaration = resolver
        .declarations
        .get_declaration(*declaration_id);

    assert!(matches!(declaration.definition, Definition::Local { .. }));
}
