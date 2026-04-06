use crate::compiler::tests::type_bind_function;

#[test]
fn boolean() {
    type_bind_function("fn foo() -> bool { !true }");
}
