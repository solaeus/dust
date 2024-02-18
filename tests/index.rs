use dust_lang::*;

#[test]
fn list_index() {
    let test = interpret("x = [1 [2] 3] x:1:0");

    assert_eq!(Ok(Value::Integer(2)), test);
}

#[test]
fn map_index() {
    let test = interpret("x = {y = {z = 2}} x:y:z");

    assert_eq!(Ok(Value::Integer(2)), test);
}

#[test]
fn index_function_calls() {
    assert_eq!(
        interpret(
            "
                x = [1 2 3]
                y = () <int> { 2 }
                x:(y())
                ",
        ),
        Ok(Value::Integer(3))
    );

    assert_eq!(
        interpret(
            "
                x = {
                    y = () <int> { 2 }
                }
                x:y()
                ",
        ),
        Ok(Value::Integer(2))
    );
}
