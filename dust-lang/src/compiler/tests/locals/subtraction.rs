use crate::{
    Address, Chunk, Instruction, OperandType, compile_main,
    resolver::{DeclarationId, TypeId},
    tests::local_cases,
};

#[test]
fn local_byte_subtraction() {
    let source = local_cases::LOCAL_BYTE_SUBTRACTION.to_string();
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            declaration_id: DeclarationId::MAIN,
            type_id: TypeId(2),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::encoded(44),
                    OperandType::BYTE,
                    false
                ),
                Instruction::load(
                    Address::register(1),
                    Address::encoded(2),
                    OperandType::BYTE,
                    false
                ),
                Instruction::subtract(
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
fn local_float_subtraction() {
    let source = local_cases::LOCAL_FLOAT_SUBTRACTION.to_string();
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            declaration_id: DeclarationId::MAIN,
            type_id: TypeId(2),
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
                Instruction::subtract(
                    Address::register(2),
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
fn local_integer_subtraction() {
    let source = local_cases::LOCAL_INTEGER_SUBTRACTION.to_string();
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            declaration_id: DeclarationId::MAIN,
            type_id: TypeId(2),
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
                Instruction::subtract(
                    Address::register(2),
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
