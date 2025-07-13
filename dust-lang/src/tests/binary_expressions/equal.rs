use crate::{
    Address, DebugChunk, FunctionType, Instruction, OperandType, Path, Type, Value, compile, run,
};

#[test]
fn equal_bytes_true() {
    let source = "0x21 == 0x21";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::equal(
                    true,
                    Address::encoded(0x21),
                    Address::encoded(0x21),
                    OperandType::BYTE
                ),
                Instruction::jump(1, true),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(true as usize),
                    OperandType::BOOLEAN,
                    true,
                ),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(false as usize),
                    OperandType::BOOLEAN,
                    false,
                ),
                Instruction::r#return(true, Address::register(0), OperandType::BOOLEAN),
            ],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Boolean(true)));
}

#[test]
fn equal_bytes_false() {
    let source = "0x21 == 0x22";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::equal(
                    true,
                    Address::encoded(0x21),
                    Address::encoded(0x22),
                    OperandType::BYTE
                ),
                Instruction::jump(1, true),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(true as usize),
                    OperandType::BOOLEAN,
                    true,
                ),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(false as usize),
                    OperandType::BOOLEAN,
                    false,
                ),
                Instruction::r#return(true, Address::register(0), OperandType::BOOLEAN),
            ],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Boolean(false)));
}

#[test]
fn equal_characters_true() {
    let source = "'a' == 'a'";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::equal(
                    true,
                    Address::constant(0),
                    Address::constant(0),
                    OperandType::CHARACTER
                ),
                Instruction::jump(1, true),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(true as usize),
                    OperandType::BOOLEAN,
                    true
                ),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(false as usize),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::r#return(true, Address::register(0), OperandType::BOOLEAN),
            ],
            constants: vec![Value::character('a')],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Boolean(true)));
}

#[test]
fn equal_characters_false() {
    let source = "'a' == 'b'";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::equal(
                    true,
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::CHARACTER
                ),
                Instruction::jump(1, true),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(true as usize),
                    OperandType::BOOLEAN,
                    true
                ),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(false as usize),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::r#return(true, Address::register(0), OperandType::BOOLEAN),
            ],
            constants: vec![Value::character('a'), Value::character('b')],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Boolean(false)));
}

#[test]
fn equal_floats_true() {
    let source = "42.0 == 42.0";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::equal(
                    true,
                    Address::constant(0),
                    Address::constant(0),
                    OperandType::FLOAT
                ),
                Instruction::jump(1, true),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(true as usize),
                    OperandType::BOOLEAN,
                    true
                ),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(false as usize),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::r#return(true, Address::register(0), OperandType::BOOLEAN),
            ],
            constants: vec![Value::Float(42.0)],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Boolean(true)));
}

#[test]
fn equal_floats_false() {
    let source = "42.0 == 43.0";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::equal(
                    true,
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::FLOAT
                ),
                Instruction::jump(1, true),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(true as usize),
                    OperandType::BOOLEAN,
                    true
                ),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(false as usize),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::r#return(true, Address::register(0), OperandType::BOOLEAN),
            ],
            constants: vec![Value::Float(42.0), Value::Float(43.0)],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Boolean(false)));
}

#[test]
fn equal_integers_true() {
    let source = "42 == 42";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::equal(
                    true,
                    Address::constant(0),
                    Address::constant(0),
                    OperandType::INTEGER
                ),
                Instruction::jump(1, true),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(true as usize),
                    OperandType::BOOLEAN,
                    true
                ),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(false as usize),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::r#return(true, Address::register(0), OperandType::BOOLEAN),
            ],
            constants: vec![Value::Integer(42)],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Boolean(true)));
}

#[test]
fn equal_integers_false() {
    let source = "42 == 43";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::equal(
                    true,
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::INTEGER
                ),
                Instruction::jump(1, true),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(true as usize),
                    OperandType::BOOLEAN,
                    true
                ),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(false as usize),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::r#return(true, Address::register(0), OperandType::BOOLEAN),
            ],
            constants: vec![Value::Integer(42), Value::Integer(43)],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Boolean(false)));
}

#[test]
fn equal_strings_true() {
    let source = "\"abc\" == \"abc\"";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::equal(
                    true,
                    Address::constant(0),
                    Address::constant(0),
                    OperandType::STRING
                ),
                Instruction::jump(1, true),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(true as usize),
                    OperandType::BOOLEAN,
                    true
                ),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(false as usize),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::r#return(true, Address::register(0), OperandType::BOOLEAN),
            ],
            constants: vec![Value::string("abc")],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Boolean(true)));
}

#[test]
fn equal_strings_false() {
    let source = "\"abc\" == \"def\"";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::equal(
                    true,
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::STRING
                ),
                Instruction::jump(1, true),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(true as usize),
                    OperandType::BOOLEAN,
                    true
                ),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(false as usize),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::r#return(true, Address::register(0), OperandType::BOOLEAN),
            ],
            constants: vec![Value::string("abc"), Value::string("def")],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Boolean(false)));
}

#[test]
fn equal_lists_true() {
    let source = "[1, 2] == [1, 2]";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::equal(
                    true,
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::LIST_INTEGER
                ),
                Instruction::jump(1, true),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(true as usize),
                    OperandType::BOOLEAN,
                    true
                ),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(false as usize),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::r#return(true, Address::register(0), OperandType::BOOLEAN),
            ],
            constants: vec![Value::integer_list([1, 2]), Value::integer_list([1, 2])],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Boolean(true)));
}

#[test]
fn equal_lists_false() {
    let source = "[1, 2] == [2, 1]";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::equal(
                    true,
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::LIST_INTEGER
                ),
                Instruction::jump(1, true),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(true as usize),
                    OperandType::BOOLEAN,
                    true
                ),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(false as usize),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::r#return(true, Address::register(0), OperandType::BOOLEAN),
            ],
            constants: vec![Value::integer_list([1, 2]), Value::integer_list([2, 1])],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Boolean(false)));
}
