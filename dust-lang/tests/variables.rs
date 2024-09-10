use dust_lang::*;

#[test]
fn add_variables() {
    let source = "let foo = 21; let bar = 21; foo + bar";
    let result = run(source);

    assert_eq!(result, Ok(Some(Value::integer(42))));
}

#[test]
fn variable() {
    let source = "let foo = 42; foo";
    let result = run(source);

    assert_eq!(result, Ok(Some(Value::integer(42))));
}
