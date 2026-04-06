use crate::compiler::tests::type_bind_function;

#[test]
fn addition() {
    type_bind_function("fn foo() -> i32 { 1 + 2 }");
}

#[test]
fn xor() {
    type_bind_function("fn foo() -> i32 { 2 ^ 3 }");
}
