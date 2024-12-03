use dust_lang::*;

#[test]
fn divide_boolean_left() {
    let source = "true / 1";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotDivideType {
                argument_type: Type::Boolean,
                position: Span(0, 4)
            },
            source,
        })
    );
}

#[test]
fn divide_boolean_right() {
    let source = "1 / true";
}

#[test]
fn divide_character_left() {
    let source = "'a' / 1";
}

#[test]
fn divide_character_right() {
    let source = "1 / 'a'";
}

#[test]
fn divide_function_left() {
    let source = "fn(){} / 1";
}

#[test]
fn divide_function_right() {
    let source = "1 / fn(){}";
}

#[test]
fn divide_list_left() {
    let source = "[1, 2] / 1";
}

#[test]
fn divide_list_right() {
    let source = "1 / [1, 2]";
}

// #[test]
// fn add_range_left() {
//     todo!("Add ranges")
// }

// #[test]
// fn add_range_right() {
//     todo!("Add ranges")
// }

#[test]
fn divide_string_left() {
    let source = "\"hello\" / 1";
}

#[test]
fn divide_string_right() {
    let source = "1 / \"hello\"";
}

#[test]
fn divide_float_and_character() {
    let source = "1.0 / 'a'";
}

#[test]
fn divide_float_and_integer() {
    let source = "1.0 / 1";
}

#[test]
fn divide_integer_and_float() {
    let source = "1 / 1.0";
}
