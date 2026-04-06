use crate::compiler::tests::type_bind_function;

#[test]
fn with_type_annotation() {
    type_bind_function("fn foo() -> i32 { let x: i32 = 1; x }");
}
