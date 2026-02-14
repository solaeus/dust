use crate::{
    compiler::compile,
    dust_type::DustType,
    instruction::{Address, Instruction, ByteType},
    prototype::Prototype,
    tests::{create_function_case, local_cases},
};

#[test]
fn local_mut_byte_multiplication() {
    let source = create_function_case(
        local_cases::LOCAL_MUT_BYTE_MULTIPLICATION,
        ByteType::BYTE,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Byte,
            instructions: vec![
                Instruction::r#move(0, Address::encoded(14)),
                Instruction::multiply(0, Address::register(0), Address::encoded(3)),
                Instruction::r#return(Address::register(0))
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_mut_float_multiplication() {
    let source = create_function_case(
        local_cases::LOCAL_MUT_FLOAT_MULTIPLICATION,
        ByteType::FLOAT,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Float,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::multiply(0, Address::register(0), Address::constant(1)),
                Instruction::r#return(Address::register(0))
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_mut_integer_multiplication() {
    let source = create_function_case(
        local_cases::LOCAL_MUT_INTEGER_MULTIPLICATION,
        ByteType::INTEGER,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::multiply(0, Address::register(0), Address::constant(1)),
                Instruction::r#return(Address::register(0))
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_byte_multiplication() {
    let source = create_function_case(local_cases::LOCAL_BYTE_MULTIPLICATION, ByteType::BYTE);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Byte,
            instructions: vec![
                Instruction::r#move(0, Address::encoded(14)),
                Instruction::r#move(1, Address::encoded(3)),
                Instruction::multiply(2, Address::register(0), Address::register(1)),
                Instruction::r#return(Address::register(2))
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_float_multiplication() {
    let source = create_function_case(local_cases::LOCAL_FLOAT_MULTIPLICATION, ByteType::FLOAT);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Float,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::r#move(1, Address::constant(1)),
                Instruction::multiply(2, Address::register(0), Address::register(1)),
                Instruction::r#return(Address::register(2))
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_integer_multiplication() {
    let source = create_function_case(
        local_cases::LOCAL_INTEGER_MULTIPLICATION,
        ByteType::INTEGER,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::r#move(1, Address::constant(1)),
                Instruction::multiply(2, Address::register(0), Address::register(1)),
                Instruction::r#return(Address::register(2))
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}
