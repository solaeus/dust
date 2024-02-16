use dust_lang::*;

#[test]
fn simple() {
    assert_eq!(interpret("{ 1 }"), Ok(Value::Integer(1)));
}

#[test]
fn nested() {
    assert_eq!(interpret("{ 1 { 1 + 1 } }"), Ok(Value::Integer(2)));
}

#[test]
fn with_return() {
    assert_eq!(interpret("{ return 1; 1 + 1; }"), Ok(Value::Integer(1)));
}

#[test]
fn async_with_return() {
    assert_eq!(
        interpret(
            "
                async {
                    return 1
                    1 + 1
                    3
                }
                "
        ),
        Ok(Value::Integer(1))
    );
}
