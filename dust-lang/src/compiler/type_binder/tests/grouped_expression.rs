use crate::compiler::tests::type_bind_function;

#[test]
fn simple() {
    type_bind_function("fn foo() -> i32 { (1) }");
}
