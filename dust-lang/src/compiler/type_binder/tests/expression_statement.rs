use crate::compiler::tests::type_bind_function;

#[test]
fn produces_unit_block() {
    type_bind_function("fn foo() { 1; }");
}
