use crate::{jit_vm::run_main, tests::local_cases, value::Value};

#[test]
fn local_boolean_or() {
    let source = local_cases::LOCAL_BOOLEAN_OR.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}
