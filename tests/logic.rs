use dust_lang::*;

#[test]
fn complex_logic_sequence() {
    let result = interpret("(length([0]) == 1) && (length([0 0]) == 2) && (length([0 0 0]) == 3)");

    assert_eq!(Ok(Value::Boolean(true)), result);
}
