use crate::{
    chunk::Chunk,
    compiler::compile_main,
    instruction::{Address, Instruction, OperandType},
    tests::local_cases,
    r#type::{FunctionType, Type},
};

#[test]
fn local_boolean() {
    let source = local_cases::LOCAL_BOOLEAN.to_string();
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::r#move(
                    0,
                    Address::encoded(true as u16),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::r#return(true, Address::register(0), OperandType::BOOLEAN),
            ],
            register_count: 1,
            ..Default::default()
        }
    );
}

#[test]
fn local_byte() {
    let source = local_cases::LOCAL_BYTE.to_string();
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            r#type: FunctionType::new([], [], Type::Byte),
            instructions: vec![
                Instruction::r#move(0, Address::encoded(42), OperandType::BYTE, false),
                Instruction::r#return(true, Address::register(0), OperandType::BYTE),
            ],
            register_count: 1,
            ..Default::default()
        }
    );
}

#[test]
fn local_character() {
    let source = local_cases::LOCAL_CHARACTER.to_string();
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            r#type: FunctionType::new([], [], Type::Character),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::CHARACTER, false),
                Instruction::r#return(true, Address::register(0), OperandType::CHARACTER),
            ],
            register_count: 1,
            ..Default::default()
        }
    );
}

#[test]
fn local_float() {
    let source = local_cases::LOCAL_FLOAT.to_string();
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            r#type: FunctionType::new([], [], Type::Float),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::FLOAT, false),
                Instruction::r#return(true, Address::register(0), OperandType::FLOAT),
            ],
            register_count: 1,
            ..Default::default()
        }
    );
}

#[test]
fn local_integer() {
    let source = local_cases::LOCAL_INTEGER.to_string();
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            r#type: FunctionType::new([], [], Type::Integer),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER, false),
                Instruction::r#return(true, Address::register(0), OperandType::INTEGER),
            ],
            register_count: 1,
            ..Default::default()
        }
    );
}

#[test]
fn local_string() {
    let source = local_cases::LOCAL_STRING.to_string();
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            r#type: FunctionType::new([], [], Type::String),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::STRING, false),
                Instruction::r#return(true, Address::register(0), OperandType::STRING),
            ],
            register_count: 1,
            ..Default::default()
        }
    );
}
