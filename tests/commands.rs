use dust_lang::{interpret, Value};

use std::fs::{remove_file, write};

#[test]
fn simple_command() {
    assert_eq!(interpret("^echo hi"), Ok(Value::String("".to_string())))
}

#[test]
fn assign_command_output() {
    write("target/test.txt", "123").unwrap();

    assert_eq!(
        interpret("x = ^cat target/test.txt; x"),
        Ok(Value::String("123".to_string()))
    );

    remove_file("target/test.txt").unwrap();
}
