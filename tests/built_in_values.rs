use dust_lang::*;

#[test]
fn args() {
    assert!(interpret("args").is_ok_and(|value| value.is_list()));
}
