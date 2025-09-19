use crate::{
    Address, Chunk, Instruction, OperandType, compile_main,
    resolver::{DeclarationId, TypeId},
    tests::local_cases,
};

#[test]
fn local_boolean() {
    let source = local_cases::LOCAL_BOOLEAN.to_string();
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            declaration_id: DeclarationId::MAIN,
            type_id: TypeId(2),
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
    let source = local_cases::LOCAL_BYTE.to_string();
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            declaration_id: DeclarationId::MAIN,
            type_id: TypeId(2),
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
    let source = local_cases::LOCAL_CHARACTER.to_string();
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            declaration_id: DeclarationId::MAIN,
            type_id: TypeId(2),
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::CHARACTER
            )],
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
            declaration_id: DeclarationId::MAIN,
            type_id: TypeId(2),
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::FLOAT
            )],
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
            declaration_id: DeclarationId::MAIN,
            type_id: TypeId(2),
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::INTEGER
            )],
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
            declaration_id: DeclarationId::MAIN,
            type_id: TypeId(0),
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::STRING
            )],
            ..Default::default()
        }
    );
}
