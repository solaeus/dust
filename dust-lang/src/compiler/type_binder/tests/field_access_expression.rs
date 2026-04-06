use crate::compiler::tests::type_bind_function;

#[test]
fn struct_field() {
    type_bind_function("struct Bar { x: i32 } fn foo(b: Bar) -> i32 { b.x }");
}
