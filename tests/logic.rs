use whale_lib::*;

#[test]
fn assert() {
    eval("assert(true)").unwrap();
    eval("assert(false)").unwrap_err();
}

#[test]
fn assert_equal() {
    eval("assert_equal(true, true)").unwrap();
    eval("assert_equal(true, false)").unwrap_err();
}

#[test]
fn r#if() {
    eval("if(true, assert(true))").unwrap();

    let value = eval("if(true, 1)").unwrap();
    assert_eq!(Value::Integer(1), value);

    let value = eval("if(false, 1)").unwrap();
    assert!(value.is_empty());
}

#[test]
fn r#if_else() {
    let value = eval("if(true, 1, 2)").unwrap();
    assert_eq!(Value::Integer(1), value);

    let value = eval("if(false, 1, 2)").unwrap();
    assert_eq!(Value::Integer(2), value);

    let value = eval("if(true, '1', '2')").unwrap();
    assert_eq!(Value::Integer(1), value);

    let value = eval("if(false, '1', '2')").unwrap();
    assert_eq!(Value::Integer(2), value);
}
