use crate::compiler::tests::type_bind_function;

#[test]
fn tail_expression() {
    type_bind_function("fn foo() -> i32 { 1.0; 2 }");
}

#[test]
fn nested_propagates_type() {
    type_bind_function("fn foo() -> i32 { { 1 } }");
}
