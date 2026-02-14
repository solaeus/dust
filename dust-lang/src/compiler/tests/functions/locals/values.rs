use crate::{
    compiler::compile,
    dust_type::DustType,
    instruction::{Address, Instruction, ByteType},
    prototype::Prototype,
    tests::{create_function_case, local_cases},
};

#[test]
fn local_boolean() {
    let source = create_function_case(local_cases::LOCAL_BOOLEAN, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::encoded(true as u16)),
                Instruction::r#return(Address::register(0)),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_byte() {
    let source = create_function_case(local_cases::LOCAL_BYTE, ByteType::BYTE);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Byte,
            instructions: vec![
                Instruction::r#move(0, Address::encoded(42)),
                Instruction::r#return(Address::register(0)),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_character() {
    let source = create_function_case(local_cases::LOCAL_CHARACTER, ByteType::CHARACTER);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Character,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::r#return(Address::register(0)),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_float() {
    let source = create_function_case(local_cases::LOCAL_FLOAT, ByteType::FLOAT);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Float,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::r#return(Address::register(0)),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_integer() {
    let source = create_function_case(local_cases::LOCAL_INTEGER, ByteType::INTEGER);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::r#return(Address::register(0)),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_string() {
    let source = create_function_case(local_cases::LOCAL_STRING, ByteType::STRING);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::String,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::r#return(Address::register(0)),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}
