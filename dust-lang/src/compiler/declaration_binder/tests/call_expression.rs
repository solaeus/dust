use crate::source::SourceFileId;
use crate::syntax::SyntaxKind;

use super::{bind_declarations, find_declaration};

#[test]
fn resolves_callee() {
    let (syntax, mut resolver) = bind_declarations("fn foo() {} fn main() { foo() }");
    let (foo_id, _) = find_declaration(&mut resolver, "foo").unwrap();

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let call = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::CallExpression)
        .unwrap();
    let (callee, _) = call.binary_children().unwrap();

    assert_eq!(
        *resolver.get_declaration_binding(&callee.id).unwrap(),
        foo_id
    );
}
