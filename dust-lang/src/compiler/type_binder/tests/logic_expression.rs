use crate::compiler::tests::type_bind_function;

#[test]
fn and() {
    type_bind_function("fn foo() -> bool { true && false }");
}
