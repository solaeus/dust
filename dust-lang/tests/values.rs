use dust_lang::*;

#[test]
fn integer() {
    let source = "42";
    let result = run(source);

    assert_eq!(result, Ok(Some(Value::integer(42))));
}

#[test]
fn float() {
    let source = "42.0";
    let result = run(source);

    assert_eq!(result, Ok(Some(Value::float(42.0))));
}

#[test]
fn string() {
    let source = "\"Hello, World!\"";
    let result = run(source);

    assert_eq!(result, Ok(Some(Value::string("Hello, World!"))));
}

#[test]
fn boolean() {
    let source = "true";
    let result = run(source);

    assert_eq!(result, Ok(Some(Value::boolean(true))));
}

#[test]
fn byte() {
    let source = "0x42";
    let result = run(source);

    assert_eq!(result, Ok(Some(Value::byte(0x42))));
}

#[test]
fn character() {
    let source = "'a'";
    let result = run(source);

    assert_eq!(result, Ok(Some(Value::character('a'))));
}
