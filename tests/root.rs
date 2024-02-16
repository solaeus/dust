use dust_lang::*;

#[test]
fn returns_final_statement() {
    assert_eq!(
        interpret(
            "
                1
                1 + 1
                3
                "
        ),
        Ok(Value::Integer(3))
    );
}

#[test]
fn return_statement() {
    assert_eq!(
        interpret(
            "
                return 1
                1 + 1
                3
                "
        ),
        Ok(Value::Integer(1))
    );
}
