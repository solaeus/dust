use crate::compiler::tests::type_bind_function;

#[test]
fn array_index() {
    type_bind_function("fn foo() -> i32 { let arr:[i32; 3] = [10, 20, 30]; arr[1] }");
}

#[test]
fn range_slice() {
    type_bind_function("fn foo() { let arr: [i32; 5] = [1, 2, 3, 4, 5]; arr[1..3]; }");
}
