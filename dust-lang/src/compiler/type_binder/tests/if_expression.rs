use crate::compiler::tests::type_bind_function;

#[test]
fn with_else() {
    type_bind_function("fn foo() -> i32 { if true { 1 } else { 2 } }");
}
