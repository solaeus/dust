use dust_lang::*;

#[test]
fn match_value() {
    let test = interpret(
        "
        match 1 {
            3 -> false
            2 -> { false }
            1 -> true
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
            3 -> false
            2 -> { false }
            1 -> true
        }
        x
        ",
    )
    .unwrap();

    assert_eq!(Value::Boolean(true), test);
}

#[test]
fn match_enum() {
    let result = interpret(
        "
        foobar = Option::Some(true)
        
        match foobar {
            Option::None -> false,
            Option::Some(content) -> content,
        }
        ",
    );

    assert_eq!(result, Ok(Value::Boolean(true)));
}
