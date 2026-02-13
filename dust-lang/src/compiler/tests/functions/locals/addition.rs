use crate::compiler::Symbol;

use crate::{
    compiler::compile,
    dust_type::{DustType, FunctionDustType},
    instruction::{Address, Instruction, OperandType},
    prototype::{Prototype, PrototypeId},
    tests::{create_function_case, local_cases},
};

#[test]
fn local_byte_addition() {
    let source = create_function_case(local_cases::LOCAL_BYTE_ADDITION, OperandType::BYTE);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Byte),
            instructions: vec![
                Instruction::r#move(0, Address::encoded(40), OperandType::BYTE),
                Instruction::r#move(1, Address::encoded(2), OperandType::BYTE),
                Instruction::add(
                    2,
                    Address::register(0),
                    Address::register(1),
                    OperandType::BYTE
                ),
                Instruction::r#return(Address::register(2), OperandType::BYTE)
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_float_addition() {
    let source = create_function_case(local_cases::LOCAL_FLOAT_ADDITION, OperandType::FLOAT);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Float),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::FLOAT),
                Instruction::r#move(1, Address::constant(1), OperandType::FLOAT),
                Instruction::add(
                    2,
                    Address::register(0),
                    Address::register(1),
                    OperandType::FLOAT
                ),
                Instruction::r#return(Address::register(2), OperandType::FLOAT)
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_integer_addition() {
    let source = create_function_case(local_cases::LOCAL_INTEGER_ADDITION, OperandType::INTEGER);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Integer),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER),
                Instruction::r#move(1, Address::constant(1), OperandType::INTEGER),
                Instruction::add(
                    2,
                    Address::register(0),
                    Address::register(1),
                    OperandType::INTEGER
                ),
                Instruction::r#return(Address::register(2), OperandType::INTEGER)
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_string_concatenation() {
    let source = create_function_case(local_cases::LOCAL_STRING_CONCATENATION, OperandType::STRING);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::String),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::STRING),
                Instruction::r#move(1, Address::constant(1), OperandType::STRING),
                Instruction::add(
                    2,
                    Address::register(0),
                    Address::register(1),
                    OperandType::STRING
                ),
                Instruction::r#return(Address::register(2), OperandType::STRING)
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_character_concatenation() {
    let source = create_function_case(
        local_cases::LOCAL_CHARACTER_CONCATENATION,
        OperandType::STRING,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::String),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::CHARACTER),
                Instruction::r#move(1, Address::constant(0), OperandType::CHARACTER),
                Instruction::add(
                    2,
                    Address::register(0),
                    Address::register(1),
                    OperandType::CHARACTER
                ),
                Instruction::r#return(Address::register(2), OperandType::STRING)
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_string_character_concatenation() {
    let source = create_function_case(
        local_cases::LOCAL_STRING_CHARACTER_CONCATENATION,
        OperandType::STRING,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::String),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::STRING),
                Instruction::r#move(1, Address::constant(1), OperandType::CHARACTER),
                Instruction::add(
                    2,
                    Address::register(0),
                    Address::register(1),
                    OperandType::STRING_CHARACTER
                ),
                Instruction::r#return(Address::register(2), OperandType::STRING)
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_character_string_concatenation() {
    let source = create_function_case(
        local_cases::LOCAL_CHARACTER_STRING_CONCATENATION,
        OperandType::STRING,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::String),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::CHARACTER),
                Instruction::r#move(1, Address::constant(1), OperandType::STRING),
                Instruction::add(
                    2,
                    Address::register(0),
                    Address::register(1),
                    OperandType::CHARACTER_STRING
                ),
                Instruction::r#return(Address::register(2), OperandType::STRING)
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_mut_byte_addition() {
    let source = create_function_case(local_cases::LOCAL_MUT_BYTE_ADDITION, OperandType::BYTE);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Byte),
            instructions: vec![
                Instruction::r#move(0, Address::encoded(40), OperandType::BYTE),
                Instruction::add(
                    0,
                    Address::register(0),
                    Address::encoded(2),
                    OperandType::BYTE
                ),
                Instruction::r#return(Address::register(0), OperandType::BYTE)
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_mut_float_addition() {
    let source = create_function_case(local_cases::LOCAL_MUT_FLOAT_ADDITION, OperandType::FLOAT);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Float),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::FLOAT),
                Instruction::add(
                    0,
                    Address::register(0),
                    Address::constant(1),
                    OperandType::FLOAT
                ),
                Instruction::r#return(Address::register(0), OperandType::FLOAT)
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_mut_integer_addition() {
    let source = create_function_case(
        local_cases::LOCAL_MUT_INTEGER_ADDITION,
        OperandType::INTEGER,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Integer),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER),
                Instruction::add(
                    0,
                    Address::register(0),
                    Address::constant(1),
                    OperandType::INTEGER
                ),
                Instruction::r#return(Address::register(0), OperandType::INTEGER)
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_mut_string_concatenation() {
    let source = create_function_case(
        local_cases::LOCAL_MUT_STRING_CONCATENATION,
        OperandType::STRING,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::String),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::STRING),
                Instruction::add(
                    0,
                    Address::register(0),
                    Address::constant(1),
                    OperandType::STRING
                ),
                Instruction::r#return(Address::register(0), OperandType::STRING)
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_mut_string_character_concatenation() {
    let source = create_function_case(
        local_cases::LOCAL_MUT_STRING_CHARACTER_CONCATENATION,
        OperandType::STRING,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::String),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::STRING),
                Instruction::add(
                    0,
                    Address::register(0),
                    Address::constant(1),
                    OperandType::STRING_CHARACTER
                ),
                Instruction::r#return(Address::register(0), OperandType::STRING)
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}
