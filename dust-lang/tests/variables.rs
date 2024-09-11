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

#[test]
fn lots_of_variables() {
    env_logger::builder().is_test(true).try_init().unwrap();

    let source = "
        let foo = 1;
        let bar = 2;
        let baz = 3;
        let qux = 4;
        let quux = 5;
        foo + bar + baz + qux + quux";
    let result = run(source);

    assert_eq!(result, Ok(Some(Value::integer(15))));
}
