use dust_lang::{interpret, Value};

#[test]
fn simple_command() {
    assert_eq!(interpret("^echo hi"), Ok(Value::string("hi\n")))
}

#[test]
fn assign_command_output() {
    assert_eq!(
        interpret("x = ^ls; length(str:lines(x))"),
        Ok(Value::Integer(11))
    );
}
