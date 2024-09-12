use dust_lang::*;

#[test]
fn boolean() {
    assert_eq!(run("true"), Ok(Some(Value::boolean(true))));
}

#[test]
fn byte() {
    assert_eq!(run("0xff"), Ok(Some(Value::byte(0xff))));
}

#[test]
fn float_simple() {
    assert_eq!(run("42.0"), Ok(Some(Value::float(42.0))));
}

#[test]
fn float_negative() {
    assert_eq!(run("-42.0"), Ok(Some(Value::float(-42.0))));
}

#[test]
fn float_exponential() {
    assert_eq!(run("4.2e1"), Ok(Some(Value::float(42.0))));
}

#[test]
fn float_exponential_negative() {
    assert_eq!(run("4.2e-1"), Ok(Some(Value::float(0.42))));
}

#[test]
fn float_infinity_and_nan() {
    assert_eq!(run("Infinity"), Ok(Some(Value::float(f64::INFINITY))));
    assert_eq!(run("-Infinity"), Ok(Some(Value::float(f64::NEG_INFINITY))));
    assert!(run("NaN").unwrap().unwrap().as_float().unwrap().is_nan());
}

#[test]
fn integer() {
    assert_eq!(run("42"), Ok(Some(Value::integer(42))));
}

#[test]
fn integer_negative() {
    assert_eq!(run("-42"), Ok(Some(Value::integer(-42))));
}

#[test]
fn string() {
    assert_eq!(
        run("\"Hello, world!\""),
        Ok(Some(Value::string("Hello, world!")))
    );
}
