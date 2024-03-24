use dust_lang::*;

#[test]
fn async_block() {
    assert_eq!(
        interpret(
            "test",
            "
                x = 41
                async {
                    x += 1
                    5
                }
                x
            "
        ),
        Ok(Some(Value::integer(42)))
    )
}

#[test]
fn loops_and_breaks() {
    assert_eq!(
        interpret(
            "test",
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
    );
    assert_eq!(
        interpret(
            "test",
            "
            foobar = {
                while true {
                    break
                }
                'foobar'
            }

            foobar
            "
        ),
        Ok(Some(Value::string("foobar".to_string())))
    );
}

#[test]
fn r#if() {
    assert_eq!(
        interpret("test", "if true { 'foobar'  }"),
        Ok(Some(Value::string("foobar".to_string())))
    )
}

#[test]
fn if_else() {
    assert_eq!(
        interpret("test", "if false { 'foo' } else { 'bar' }"),
        Ok(Some(Value::string("bar".to_string())))
    )
}
