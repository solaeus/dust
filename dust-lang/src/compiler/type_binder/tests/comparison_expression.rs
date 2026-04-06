use crate::compiler::tests::type_bind_function;

#[test]
fn less_than() {
    type_bind_function("fn foo() -> bool { 1 < 2 }");
}
