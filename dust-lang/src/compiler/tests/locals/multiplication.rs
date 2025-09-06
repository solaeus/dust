use crate::{
    Address, Chunk, FunctionType, Instruction, OperandType, Type, compile_main, tests::local_cases,
};

#[test]
fn local_byte_multiplication() {
    let source = local_cases::LOCAL_BYTE_MULTIPLICATION;
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::Byte),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::encoded(14),
                    OperandType::BYTE,
                    false
                ),
                Instruction::load(
                    Address::register(1),
                    Address::encoded(3),
                    OperandType::BYTE,
                    false
                ),
                Instruction::multiply(
                    Address::register(2),
                    Address::register(0),
                    Address::register(1),
                    OperandType::BYTE
                ),
                Instruction::r#return(true, Address::register(2), OperandType::BYTE)
            ],
            register_count: 3,
            ..Default::default()
        }
    );
}

#[test]
fn local_float_multiplication() {
    let source = local_cases::LOCAL_FLOAT_MULTIPLICATION;
    let chunk = compile_main(source).unwrap();
    let mut constants = chunk.constants.clone();

    constants.add_float(14.0);
    constants.add_float(3.0);

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::Float),
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
                Instruction::multiply(
                    Address::register(2),
                    Address::register(0),
                    Address::register(1),
                    OperandType::FLOAT
                ),
                Instruction::r#return(true, Address::register(2), OperandType::FLOAT)
            ],
            constants,
            register_count: 3,
            ..Default::default()
        }
    );
}

#[test]
fn local_integer_multiplication() {
    let source = local_cases::LOCAL_INTEGER_MULTIPLICATION;
    let chunk = compile_main(source).unwrap();
    let mut constants = chunk.constants.clone();

    constants.add_integer(14);
    constants.add_integer(3);

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::Integer),
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
                Instruction::multiply(
                    Address::register(2),
                    Address::register(0),
                    Address::register(1),
                    OperandType::INTEGER
                ),
                Instruction::r#return(true, Address::register(2), OperandType::INTEGER)
            ],
            constants,
            register_count: 3,
            ..Default::default()
        }
    );
}
