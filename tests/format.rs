use dust_lang::*;

#[test]
fn format_simple_program() {
    let mut interpreter = Interpreter::new(Map::new());

    interpreter.run("x=1").unwrap();

    assert_eq!(interpreter.format(), "x = 1");
}

const FORMATTED_BLOCK: &str = "{
    1
    2
    3
}
";

#[test]
fn format_block() {
    let mut interpreter = Interpreter::new(Map::new());

    interpreter.run("{1 2 3}").unwrap();

    assert_eq!(FORMATTED_BLOCK, interpreter.format());
}

const FORMATTED_FUNCTION: &str = "(x <int>) <num> {
    x / 2
}
";

#[test]
fn format_function() {
    let mut interpreter = Interpreter::new(Map::new());

    interpreter.run("( x< int > )<num>{x/2}").unwrap();

    assert_eq!(FORMATTED_FUNCTION, interpreter.format());
}
