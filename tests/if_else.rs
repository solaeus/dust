use dust_lang::*;

#[test]
fn r#if() {
    assert_eq!(
        interpret("if true { 'true' }"),
        Ok(Value::string("true".to_string()))
    );
}

#[test]
fn if_else() {
    assert_eq!(
        interpret("if false { 1 } else { 2 }"),
        Ok(Value::Integer(2))
    );
    assert_eq!(
        interpret("if true { 1.0 } else { 42.0 }"),
        Ok(Value::Float(1.0))
    );
}

#[test]
fn if_else_else_if_else() {
    assert_eq!(
        interpret(
            "
                    if false {
                        'no'
                    } else if 1 + 1 == 3 {
                        'nope'
                    } else {
                        'ok'
                    }
                "
        ),
        Ok(Value::string("ok".to_string()))
    );
}

#[test]
fn if_else_if_else_if_else_if_else() {
    assert_eq!(
        interpret(
            "
                    if false {
                        'no'
                    } else if 1 + 1 == 1 {
                        'nope'
                    } else if 9 / 2 == 4 {
                        'nope'
                    } else if 'foo' == 'bar' {
                        'nope'
                    } else {
                        'ok'
                    }
                "
        ),
        Ok(Value::string("ok".to_string()))
    );
}
