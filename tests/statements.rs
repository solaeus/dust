use dust_lang::*;

#[test]
fn loops_and_breaks() {
    assert_eq!(
        interpret(
            "
            i = 0;
            loop {
                if i == 3 {
                    break 'foobar'
                } else {
                    i += 1
                }
            }
            "
        ),
        Ok(Some(Value::string("foobar")))
    )
}
