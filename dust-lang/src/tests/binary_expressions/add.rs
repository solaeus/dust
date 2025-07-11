use crate::{
    Address, DebugChunk, FunctionType, Instruction, OperandType, Path, Type, Value, compile, run,
};

#[test]
fn add_bytes() {
    let source = "0x40 + 0x02";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Byte),
            instructions: vec![
                Instruction::add(
                    Address::register(0),
                    Address::encoded(0x40),
                    Address::encoded(0x02),
                    OperandType::BYTE
                ),
                Instruction::r#return(true, Address::register(0), OperandType::BYTE),
            ],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Byte(0x42)));
}

#[test]
fn add_characters() {
    let source = "'4' + '2'";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::String),
            instructions: vec![
                Instruction::add(
                    Address::register(0),
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::CHARACTER
                ),
                Instruction::r#return(true, Address::register(0), OperandType::STRING),
            ],
            constants: vec![Value::character('4'), Value::character('2')],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::string("42")));
}

#[test]
fn add_floats() {
    let source = "40.0 + 2.0";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Float),
            instructions: vec![
                Instruction::add(
                    Address::register(0),
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::FLOAT
                ),
                Instruction::r#return(true, Address::register(0), OperandType::FLOAT),
            ],
            constants: vec![Value::Float(40.0), Value::Float(2.0)],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Float(42.0)));
}

#[test]
fn add_integers() {
    let source = "40 + 2";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Integer),
            instructions: vec![
                Instruction::add(
                    Address::register(0),
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::INTEGER
                ),
                Instruction::r#return(true, Address::register(0), OperandType::INTEGER),
            ],
            constants: vec![Value::Integer(40), Value::Integer(2),],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Integer(42)));
}

#[test]
fn add_strings() {
    let source = "\"Hello, \" + \"World!\"";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::String),
            instructions: vec![
                Instruction::add(
                    Address::register(0),
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::STRING
                ),
                Instruction::r#return(true, Address::register(0), OperandType::STRING),
            ],
            constants: vec![Value::string("Hello, "), Value::string("World!")],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::string("Hello, World!")));
}
