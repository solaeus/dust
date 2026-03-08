use crate::source::SourceFileId;
use crate::syntax::SyntaxKind;

use super::{bind_declarations, find_declaration};

#[test]
fn resolves_field_paths() {
    let (syntax, mut resolver) =
        bind_declarations("struct Foo { x: i64 } fn main() { Foo { x: 1 } }");
    let (foo_id, _) = find_declaration(&mut resolver, "Foo").unwrap();
    let (x_id, _) = find_declaration(&mut resolver, "x").unwrap();

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let struct_expr = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::StructExpression)
        .unwrap();
    let (path, fields) = struct_expr.binary_children().unwrap();
    let field = fields.children().unwrap().next().unwrap();
    let (field_path, _) = field.binary_children().unwrap();

    assert_eq!(
        *resolver.get_declaration_binding(&path.id).unwrap(),
        foo_id
    );
    assert_eq!(
        *resolver.get_declaration_binding(&field_path.id).unwrap(),
        x_id
    );
}
