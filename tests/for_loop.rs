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

#[test]
fn modify_iteration_values() {
    let result = interpret(
        "
            list = [1 2 3]
            for i in list { i += i }
            list
            ",
    );

    assert_eq!(
        Ok(Value::List(List::with_items(vec![
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4),
        ]))),
        result
    );
}

#[test]
fn r#break() {
    let result = interpret(
        "
            list = []
            for i in [1 2 3] {
                if i > 2 {
                    break
                } else {
                    list += i
                }
            }
            list
            ",
    );

    assert_eq!(
        Ok(Value::List(List::with_items(vec![
            Value::Integer(1),
            Value::Integer(2),
        ]))),
        result
    );
}
