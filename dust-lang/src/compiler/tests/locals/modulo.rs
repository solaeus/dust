use crate::{
    chunk::Chunk,
    compiler::compile_main,
    instruction::{Address, Instruction, OperandType},
    tests::local_cases,
    r#type::{FunctionType, Type},
};

#[test]
fn local_byte_modulo() {
    let source = local_cases::LOCAL_BYTE_MODULO.to_string();
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            function_type: FunctionType::new([], [], Type::Byte),
            instructions: vec![
                Instruction::r#move(0, Address::encoded(84), OperandType::BYTE, false),
                Instruction::r#move(1, Address::encoded(5), OperandType::BYTE, false),
                Instruction::modulo(
                    2,
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
fn local_float_modulo() {
    let source = local_cases::LOCAL_FLOAT_MODULO.to_string();
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            function_type: FunctionType::new([], [], Type::Float),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::FLOAT, false),
                Instruction::r#move(1, Address::constant(1), OperandType::FLOAT, false),
                Instruction::modulo(
                    2,
                    Address::register(0),
                    Address::register(1),
                    OperandType::FLOAT
                ),
                Instruction::r#return(true, Address::register(2), OperandType::FLOAT)
            ],
            register_count: 3,
            ..Default::default()
        }
    );
}

#[test]
fn local_integer_modulo() {
    let source = local_cases::LOCAL_INTEGER_MODULO.to_string();
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            function_type: FunctionType::new([], [], Type::Integer),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER, false),
                Instruction::r#move(1, Address::constant(1), OperandType::INTEGER, false),
                Instruction::modulo(
                    2,
                    Address::register(0),
                    Address::register(1),
                    OperandType::INTEGER
                ),
                Instruction::r#return(true, Address::register(2), OperandType::INTEGER)
            ],
            register_count: 3,
            ..Default::default()
        }
    );
}
