use crate::{
    compiler::compile,
    dust_type::DustType,
    instruction::{Address, Instruction, ByteType},
    prototype::Prototype,
    tests::{create_function_case, local_cases},
};

#[test]
fn local_byte_subtraction() {
    let source = create_function_case(local_cases::LOCAL_BYTE_SUBTRACTION, ByteType::BYTE);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Byte,
            instructions: vec![
                Instruction::r#move(0, Address::encoded(44), ByteType::BYTE),
                Instruction::r#move(1, Address::encoded(2), ByteType::BYTE),
                Instruction::subtract(
                    2,
                    Address::register(0),
                    Address::register(1),
                    ByteType::BYTE
                ),
                Instruction::r#return(Address::register(2), ByteType::BYTE)
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_float_subtraction() {
    let source = create_function_case(local_cases::LOCAL_FLOAT_SUBTRACTION, ByteType::FLOAT);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Float,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), ByteType::FLOAT),
                Instruction::r#move(1, Address::constant(1), ByteType::FLOAT),
                Instruction::subtract(
                    2,
                    Address::register(0),
                    Address::register(1),
                    ByteType::FLOAT
                ),
                Instruction::r#return(Address::register(2), ByteType::FLOAT)
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_integer_subtraction() {
    let source = create_function_case(local_cases::LOCAL_INTEGER_SUBTRACTION, ByteType::INTEGER);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), ByteType::INTEGER),
                Instruction::r#move(1, Address::constant(1), ByteType::INTEGER),
                Instruction::subtract(
                    2,
                    Address::register(0),
                    Address::register(1),
                    ByteType::INTEGER
                ),
                Instruction::r#return(Address::register(2), ByteType::INTEGER)
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_mut_byte_subtraction() {
    let source = create_function_case(local_cases::LOCAL_MUT_BYTE_SUBTRACTION, ByteType::BYTE);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Byte,
            instructions: vec![
                Instruction::r#move(0, Address::encoded(44), ByteType::BYTE),
                Instruction::subtract(
                    0,
                    Address::register(0),
                    Address::encoded(2),
                    ByteType::BYTE
                ),
                Instruction::r#return(Address::register(0), ByteType::BYTE)
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_mut_float_subtraction() {
    let source = create_function_case(local_cases::LOCAL_MUT_FLOAT_SUBTRACTION, ByteType::FLOAT);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Float,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), ByteType::FLOAT),
                Instruction::subtract(
                    0,
                    Address::register(0),
                    Address::constant(1),
                    ByteType::FLOAT
                ),
                Instruction::r#return(Address::register(0), ByteType::FLOAT)
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_mut_integer_subtraction() {
    let source = create_function_case(
        local_cases::LOCAL_MUT_INTEGER_SUBTRACTION,
        ByteType::INTEGER,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), ByteType::INTEGER),
                Instruction::subtract(
                    0,
                    Address::register(0),
                    Address::constant(1),
                    ByteType::INTEGER
                ),
                Instruction::r#return(Address::register(0), ByteType::INTEGER)
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}
