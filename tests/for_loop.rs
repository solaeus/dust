use dust_lang::*;

#[test]
fn simple_for_loop() {
    let result = interpret("for i in [1 2 3] { output(i) }");

    assert_eq!(Ok(Value::none()), result);
}

#[test]
fn modify_value() {
    let result = interpret(
        "
            list = []
            for i in [1 2 3] { list += i }
            list
            ",
    );

    assert_eq!(
        Ok(Value::List(List::with_items(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]))),
        result
    );
}
