use dust_lang::*;

#[test]
fn r#match() {
    let test = interpret(
        "
                match 1 {
                    3 => false
                    2 => { false }
                    1 => true
                }
            ",
    )
    .unwrap();

    assert_eq!(Value::Boolean(true), test);
}

#[test]
fn match_assignment() {
    let test = interpret(
        "
                x = match 1 {
                    3 => false
                    2 => { false }
                    1 => true
                }
                x
            ",
    )
    .unwrap();

    assert_eq!(Value::Boolean(true), test);
}
