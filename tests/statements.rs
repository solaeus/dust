use dust_lang::*;

#[test]
fn loops_and_breaks() {
    assert_eq!(
        interpret(
            "
            i = 0
            loop {
                if i == 3 {
                    break
                } else {
                    i += 1
                }
            }
            i
            "
        ),
        Ok(Some(Value::integer(3)))
    )
}

#[test]
fn r#if() {
    assert_eq!(
        interpret("if true { 'foobar' }"),
        Ok(Some(Value::string("foobar".to_string())))
    )
}

#[test]
fn if_else() {
    assert_eq!(
        interpret("if false { 'foo' } else { 'bar' }"),
        Ok(Some(Value::string("bar".to_string())))
    )
}
