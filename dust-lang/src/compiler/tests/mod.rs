use crate::{
    Address, Chunk, ConstantTable, FunctionType, Instruction, OperandType, Type, compile,
    tests::cases,
};

#[test]
fn boolean() {
    let source = cases::BOOLEAN;
    let chunk = compile(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![Instruction::load(
                Address::register(0),
                Address::encoded(true as u16),
                OperandType::BOOLEAN,
                false
            )],
            register_count: 1,
            ..Default::default()
        }
    );
}

#[test]
fn byte() {
    let source = cases::BYTE;
    let chunk = compile(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::Byte),
            instructions: vec![Instruction::load(
                Address::register(0),
                Address::encoded(42),
                OperandType::BYTE,
                false
            )],
            register_count: 1,
            ..Default::default()
        }
    );
}

#[test]
fn character() {
    let source = cases::CHARACTER;
    let chunk = compile(source).unwrap();
    let mut constants = ConstantTable::new();

    constants.add_character('q');

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::Character),
            instructions: vec![Instruction::load(
                Address::register(0),
                Address::constant(0),
                OperandType::CHARACTER,
                false
            )],
            register_count: 1,
            constants,
            ..Default::default()
        }
    );
}

#[test]
fn float() {
    let source = cases::FLOAT;
    let chunk = compile(source).unwrap();
    let mut constants = ConstantTable::new();

    constants.add_float(42.0);

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::Float),
            instructions: vec![Instruction::load(
                Address::register(0),
                Address::constant(0),
                OperandType::FLOAT,
                false
            )],
            register_count: 1,
            constants,
            ..Default::default()
        }
    );
}

#[test]
fn integer() {
    let source = cases::INTEGER;
    let chunk = compile(source).unwrap();
    let mut constants = ConstantTable::new();

    constants.add_integer(42);

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::Integer),
            instructions: vec![Instruction::load(
                Address::register(0),
                Address::constant(0),
                OperandType::INTEGER,
                false
            )],
            register_count: 1,
            constants,
            ..Default::default()
        }
    );
}

#[test]
fn string() {
    let source = cases::STRING;
    let chunk = compile(source).unwrap();
    let mut constants = ConstantTable::new();

    constants.add_string("foobar");

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::String),
            instructions: vec![Instruction::load(
                Address::register(0),
                Address::constant(0),
                OperandType::STRING,
                false
            )],
            register_count: 1,
            constants,
            ..Default::default()
        }
    );
}
