use crate::source::SourceFileId;
use crate::syntax::SyntaxKind;

use super::{bind_declarations, find_declaration};

#[test]
fn binds_to_declaration() {
    let (syntax, mut resolver) =
        bind_declarations("mod foo { pub fn bar() {} } use foo::bar; fn main() { bar }");
    let (bar_id, _) = find_declaration(&mut resolver, "bar").unwrap();

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let use_item = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::UseItem)
        .unwrap();

    assert_eq!(
        *resolver.get_declaration_binding(&use_item.id).unwrap(),
        bar_id
    );
}
