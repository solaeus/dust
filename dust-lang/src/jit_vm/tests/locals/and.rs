use crate::{jit_vm::run_main, tests::local_cases, value::Value};

#[test]
fn local_boolean_and() {
    let source = local_cases::LOCAL_BOOLEAN_AND.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(false)));
}
