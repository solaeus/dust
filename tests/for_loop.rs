use dust_lang::*;

#[test]
fn list_for_loop() {
    let result = interpret("for i in [1 2 3] { output(i) }");

    assert_eq!(Ok(Value::none()), result);
}

#[test]
fn range_for_loop() {
    let result = interpret(
        "
        numbers = []
        
        for i in 1..3 {
            numbers += i
        }

        numbers
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
fn mutate_list() {
    let result = interpret(
        "
        list = []
        for i in [1 2 3] { list += i }
        list
        ",
    );

    assert_eq!(
        result,
        Ok(Value::List(List::with_items(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]))),
    );
}

#[test]
fn do_not_mutate_list_items() {
    let result = interpret(
        "
        list = [1 2 3]
        for i in list { i += i }
        list
        ",
    );

    assert_eq!(
        result,
        Ok(Value::List(List::with_items(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]))),
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
