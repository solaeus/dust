use crate::{
    Address, DebugChunk, DustString, FunctionType, Instruction, List, OperandType, Path, Type,
    Value, compile, run,
};

#[test]
fn boolean_true() {
    let source = "true";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![Instruction::r#return(
                true,
                Address::encoded(true as usize),
                OperandType::BOOLEAN
            )],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Boolean(true)));
}

#[test]
fn boolean_false() {
    let source = "false";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![Instruction::r#return(
                true,
                Address::encoded(false as usize),
                OperandType::BOOLEAN
            )],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Boolean(false)));
}

#[test]
fn byte() {
    let source = "0x64";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Byte),
            instructions: vec![Instruction::r#return(
                true,
                Address::encoded(0x64),
                OperandType::BYTE
            )],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Byte(0x64)));
}

#[test]
fn character() {
    let source = "'a'";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Character),
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::CHARACTER
            )],
            constants: vec![Value::character('a')],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Character('a')));
}

#[test]
fn float() {
    let source = "42.0";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Float),
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::FLOAT
            )],
            constants: vec![Value::Float(42.0)],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Float(42.0)));
}

#[test]
fn integer() {
    let source = "42";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Integer),
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::INTEGER
            )],
            constants: vec![Value::Integer(42)],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Integer(42)));
}

#[test]
fn string() {
    let source = "\"Hello, World!\"";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::String),
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::STRING
            )],
            constants: vec![Value::string("Hello, World!")],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::string("Hello, World!")));
}

#[test]
fn list_of_booleans() {
    let source = "[true, false]";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::List(Box::new(Type::Boolean))),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::encoded(true as usize),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::load(
                    Address::register(1),
                    Address::encoded(false as usize),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::list(
                    Address::register(2),
                    Address::register(0),
                    Address::register(1),
                    OperandType::LIST_BOOLEAN,
                ),
                Instruction::r#return(true, Address::register(2), OperandType::LIST_BOOLEAN)
            ],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::boolean_list([true, false])));
}

#[test]
fn list_of_bytes() {
    let source = "[0x64, 0x65]";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::List(Box::new(Type::Byte))),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::encoded(0x64),
                    OperandType::BYTE,
                    false
                ),
                Instruction::load(
                    Address::register(1),
                    Address::encoded(0x65),
                    OperandType::BYTE,
                    false
                ),
                Instruction::list(
                    Address::register(2),
                    Address::register(0),
                    Address::register(1),
                    OperandType::LIST_BYTE,
                ),
                Instruction::r#return(true, Address::register(2), OperandType::LIST_BYTE)
            ],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::byte_list([0x64, 0x65])));
}

#[test]
fn list_of_characters() {
    let source = "['a', 'b']";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::List(Box::new(Type::Character))),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::constant(0),
                    OperandType::CHARACTER,
                    false
                ),
                Instruction::load(
                    Address::register(1),
                    Address::constant(1),
                    OperandType::CHARACTER,
                    false
                ),
                Instruction::list(
                    Address::register(2),
                    Address::register(0),
                    Address::register(1),
                    OperandType::LIST_CHARACTER,
                ),
                Instruction::r#return(true, Address::register(2), OperandType::LIST_CHARACTER)
            ],
            constants: vec![Value::character('a'), Value::character('b')],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::character_list(['a', 'b'])));
}

#[test]
fn list_of_floats() {
    let source = "[42.0, 0.42]";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::List(Box::new(Type::Float))),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::constant(0),
                    OperandType::FLOAT,
                    false
                ),
                Instruction::load(
                    Address::register(1),
                    Address::constant(1),
                    OperandType::FLOAT,
                    false
                ),
                Instruction::list(
                    Address::register(2),
                    Address::register(0),
                    Address::register(1),
                    OperandType::LIST_FLOAT,
                ),
                Instruction::r#return(true, Address::register(2), OperandType::LIST_FLOAT)
            ],
            constants: vec![Value::Float(42.0), Value::Float(0.42)],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::float_list([42.0, 0.42])));
}

#[test]
fn list_of_integers() {
    let source = "[42, 100]";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::List(Box::new(Type::Integer))),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::constant(0),
                    OperandType::INTEGER,
                    false
                ),
                Instruction::load(
                    Address::register(1),
                    Address::constant(1),
                    OperandType::INTEGER,
                    false
                ),
                Instruction::list(
                    Address::register(2),
                    Address::register(0),
                    Address::register(1),
                    OperandType::LIST_INTEGER,
                ),
                Instruction::r#return(true, Address::register(2), OperandType::LIST_INTEGER)
            ],
            constants: vec![Value::Integer(42), Value::Integer(100)],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::integer_list([42, 100])));
}

#[test]
fn list_of_strings() {
    let source = "[\"Hello\", \"World\"]";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::List(Box::new(Type::String))),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::constant(0),
                    OperandType::STRING,
                    false
                ),
                Instruction::load(
                    Address::register(1),
                    Address::constant(1),
                    OperandType::STRING,
                    false
                ),
                Instruction::list(
                    Address::register(2),
                    Address::register(0),
                    Address::register(1),
                    OperandType::LIST_STRING,
                ),
                Instruction::r#return(true, Address::register(2), OperandType::LIST_STRING)
            ],
            constants: vec![Value::string("Hello"), Value::string("World")],
            ..Default::default()
        }
    );
    assert_eq!(
        return_value,
        Some(Value::string_list([
            DustString::from("Hello"),
            DustString::from("World")
        ]))
    );
}

#[test]
fn list_of_lists() {
    let source = "[[1, 2], [3, 4]]";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new(
                [],
                [],
                Type::List(Box::new(Type::List(Box::new(Type::Integer))))
            ),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::constant(0),
                    OperandType::LIST_INTEGER,
                    false
                ),
                Instruction::load(
                    Address::register(1),
                    Address::constant(1),
                    OperandType::LIST_INTEGER,
                    false
                ),
                Instruction::list(
                    Address::register(2),
                    Address::register(0),
                    Address::register(1),
                    OperandType::LIST_LIST,
                ),
                Instruction::r#return(true, Address::register(2), OperandType::LIST_LIST)
            ],
            constants: vec![Value::integer_list([1, 2]), Value::integer_list([3, 4])],
            ..Default::default()
        }
    );
    assert_eq!(
        return_value,
        Some(Value::list_list([
            List::integer([1, 2]),
            List::integer([3, 4])
        ]))
    );
}
