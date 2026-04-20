use crate::{
    compiler::{resolver::declarations::Definition, tests::bind_declarations},
    source::{Source, SourceCode, SourceCodeId},
    syntax::{
        components::{StructExpression, StructExpressionStructFields, SyntaxComponent},
        node::SyntaxKind,
    },
};

#[test]
fn binds_field_name() {
    let mut source = Source::new();

    source.add_code(SourceCode::validated_borrowed(
        "test",
        "struct Foo { x: i64 } fn main() { Foo { x: 1 }; }",
    ));

    let (syntax, resolver, _crate_scope_id) = bind_declarations(&source);

    let tree = syntax.get_tree(SourceCodeId::MAIN).unwrap();
    let struct_expr = tree
        .iter()
        .find(|node| node.node.kind == SyntaxKind::StructExpression)
        .unwrap();

    let StructExpression { fields, .. } = StructExpression::from_reader(&struct_expr).unwrap();
    let StructExpressionStructFields {
        name_expression_pairs,
    } = StructExpressionStructFields::from_reader(&fields).unwrap();

    for (field_name, _) in name_expression_pairs {
        let declaration_id = resolver.get_declaration_binding(&field_name.id).unwrap();
        let declaration = resolver
            .declarations
            .get_declaration(*declaration_id)
            .unwrap();

        assert!(matches!(declaration.definition, Definition::Field { .. }));
    }
}
