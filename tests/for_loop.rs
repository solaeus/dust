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
fn modify_list() {
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
fn modify_map() {
    let result = interpret(
        "
        map = {}
        
        for i in [['x', 1] ['y', 2]] {
            map:(i:0) = i:1 
        }
        
        map
        ",
    );

    let mut map = Map::new();

    map.set(Identifier::new("x"), Value::Integer(1));
    map.set(Identifier::new("y"), Value::Integer(2));

    assert_eq!(Ok(Value::Map(map)), result);
}

#[test]
fn modify_list_values() {
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
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4),
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
