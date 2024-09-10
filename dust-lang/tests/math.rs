use dust_lang::*;

#[test]
fn addition() {
    let source = "21 + 21";
    let result = run(source);

    assert_eq!(result, Ok(Some(Value::integer(42))));
}
