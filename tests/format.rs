use dust_lang::*;

#[test]
fn format_simple_program() {
    let mut interpreter = Interpreter::new(Context::default());

    assert_eq!(interpreter.format("x=1"), Ok("x = 1\n".to_string()));
}

const FORMATTED_BLOCK: &str = "{
    1
    2
    3
}
";

#[test]
fn format_block() {
    let mut interpreter = Interpreter::new(Context::default());

    assert_eq!(
        interpreter.format("{1 2 3}"),
        Ok(FORMATTED_BLOCK.to_string())
    );
}

const FORMATTED_MAP: &str = "{
    {
        x = 1
        y <int> = 2
    }
}
";

#[test]
fn format_map() {
    let mut interpreter = Interpreter::new(Context::default());

    assert_eq!(
        interpreter.format("{{x=1 y   <int>     = 2}}"),
        Ok(FORMATTED_MAP.to_string())
    );
}

const FORMATTED_FUNCTION: &str = "(x <int>) <num> {
    x / 2
}
";

#[test]
fn format_function() {
    let mut interpreter = Interpreter::new(Context::default());
    assert_eq!(
        interpreter.format("( x< int > )<num>{x/2}"),
        Ok(FORMATTED_FUNCTION.to_string())
    );
}
