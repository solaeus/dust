use std::sync::Arc;

use crate::{
    Address, Chunk, ConcreteValue, DustString, FunctionType, Instruction, Local, Scope, Span, Type,
    Value, compile,
    instruction::{AddressKind, Destination},
    run,
};

#[test]
fn load_boolean_true() {
    let source = "true";
    let chunk = Chunk {
        name: Some(DustString::from("anonymous")),
        r#type: FunctionType::new([], [], Type::Boolean),
        instructions: vec![
            Instruction::load_encoded(
                Destination::register(0),
                true as u16,
                AddressKind::BOOLEAN_MEMORY,
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::BOOLEAN_REGISTER)),
        ],
        positions: vec![Span(0, 4), Span(4, 4)],
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_boolean_false() {
    let source = "false";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_byte() {
    let source = "0x2a";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_character() {
    let source = "'a'";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_float() {
    let source = "42.42";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_integer() {
    let source = "42";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_string() {
    let source = "\"Hello, World!\"";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_boolean_list() {
    let source = "[true, false]";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_byte_list() {
    let source = "[0x2a, 0x42]";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_character_list() {
    let source = "['a', 'b']";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_float_list() {
    let source = "[42.42, 24.24]";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_integer_list() {
    let source = "[1, 2, 3]";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_string_list() {
    let source = "[\"Hello\", \"World\"]";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_nested_list() {
    let source = "[[1, 2], [3, 4]]";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_deeply_nested_list() {
    let source = "[[[1, 2], [3, 4]], [[5, 6], [7, 8]]]";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_function() {
    let source = "fn () {}";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_boolean_in_function() {
    let source = "fn () { true }";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_integer_in_function() {
    let source = "fn () { 42 }";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_string_in_function() {
    let source = "fn () { \"Hello\" }";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_list_in_function() {
    let source = "fn () { [1, 2, 3] }";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_byte_in_function() {
    let source = "fn () { 0x2a }";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_character_in_function() {
    let source = "fn () { 'a' }";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_float_in_function() {
    let source = "fn () { 42.42 }";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_nested_list_in_function() {
    let source = "fn () { [[1, 2], [3, 4]] }";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_deeply_nested_list_in_function() {
    let source = "fn () { [[[1, 2], [3, 4]], [[5, 6], [7, 8]]] }";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_function_in_function() {
    let source = "fn outer() { fn inner() -> int { 42 } }";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}
