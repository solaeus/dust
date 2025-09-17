use crate::{
    Address, Chunk, ConstantTable, FunctionType, Instruction, OperandType, Type, compile_main,
    tests::local_cases,
};
use std::sync::Arc;

#[test]
fn local_boolean() {
    let source = local_cases::LOCAL_BOOLEAN;
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            name: Arc::new("main".to_string()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![Instruction::r#return(
                true,
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Default::default()
        }
    );
}

#[test]
fn local_byte() {
    let source = local_cases::LOCAL_BYTE;
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            name: Arc::new("main".to_string()),
            r#type: FunctionType::new([], [], Type::Byte),
            instructions: vec![Instruction::r#return(
                true,
                Address::encoded(42),
                OperandType::BYTE
            )],
            ..Default::default()
        }
    );
}

#[test]
fn local_character() {
    let source = local_cases::LOCAL_CHARACTER;
    let chunk = compile_main(source).unwrap();
    let mut constants = ConstantTable::new();

    constants.add_character('q');

    assert_eq!(
        chunk,
        Chunk {
            name: Arc::new("main".to_string()),
            r#type: FunctionType::new([], [], Type::Character),
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::CHARACTER
            )],
            constants,
            ..Default::default()
        }
    );
}

#[test]
fn local_float() {
    let source = local_cases::LOCAL_FLOAT;
    let chunk = compile_main(source).unwrap();
    let mut constants = ConstantTable::new();

    constants.add_float(42.0);

    assert_eq!(
        chunk,
        Chunk {
            name: Arc::new("main".to_string()),
            r#type: FunctionType::new([], [], Type::Float),
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::FLOAT
            )],
            constants,
            ..Default::default()
        }
    );
}

#[test]
fn local_integer() {
    let source = local_cases::LOCAL_INTEGER;
    let chunk = compile_main(source).unwrap();
    let mut constants = ConstantTable::new();

    constants.add_integer(42);

    assert_eq!(
        chunk,
        Chunk {
            name: Arc::new("main".to_string()),
            r#type: FunctionType::new([], [], Type::Integer),
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::INTEGER
            )],
            constants,
            ..Default::default()
        }
    );
}

#[test]
fn local_string() {
    let source = local_cases::LOCAL_STRING;
    let chunk = compile_main(source).unwrap();
    let mut constants = ConstantTable::new();

    constants.add_string("foobar");

    assert_eq!(
        chunk,
        Chunk {
            name: Arc::new("main".to_string()),
            r#type: FunctionType::new([], [], Type::String),
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::STRING
            )],
            constants,
            ..Default::default()
        }
    );
}
