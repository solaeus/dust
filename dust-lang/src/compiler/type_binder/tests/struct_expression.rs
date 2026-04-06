use crate::compiler::tests::type_bind_function;

#[test]
fn empty() {
    type_bind_function("struct Bar {} fn foo() -> Bar { Bar {} }");
}

#[test]
fn field_types() {
    type_bind_function("struct Foo { x: i32, y: bool } fn foo() -> Foo { Foo { x: 1, y: true } }");
}
