use crate::compiler::Symbol;
use crate::constant_table::ConstantId;
use crate::{
    compiler::compile_prototypes,
    instruction::{Address, Instruction, OperandType},
    prototype::Prototype,
    tests::{create_function_case, local_cases},
    r#type::{FunctionType, Type},
};

#[test]
fn local_boolean() {
    let source = create_function_case(local_cases::LOCAL_BOOLEAN, OperandType::BOOLEAN);
    let prototypes = compile_prototypes(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            index: 1,
            name: Symbol::Constant {
                constant_id: ConstantId(0),
            },
            function_type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::r#move(0, Address::encoded(true as u16), OperandType::BOOLEAN),
                Instruction::r#return(Address::register(0), OperandType::BOOLEAN),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_byte() {
    let source = create_function_case(local_cases::LOCAL_BYTE, OperandType::BYTE);
    let prototypes = compile_prototypes(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            index: 1,
            name: Symbol::Constant {
                constant_id: ConstantId(0),
            },
            function_type: FunctionType::new([], [], Type::Byte),
            instructions: vec![
                Instruction::r#move(0, Address::encoded(42), OperandType::BYTE),
                Instruction::r#return(Address::register(0), OperandType::BYTE),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_character() {
    let source = create_function_case(local_cases::LOCAL_CHARACTER, OperandType::CHARACTER);
    let prototypes = compile_prototypes(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            index: 1,
            name: Symbol::Constant {
                constant_id: ConstantId(0),
            },
            function_type: FunctionType::new([], [], Type::Character),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::CHARACTER),
                Instruction::r#return(Address::register(0), OperandType::CHARACTER),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_float() {
    let source = create_function_case(local_cases::LOCAL_FLOAT, OperandType::FLOAT);
    let prototypes = compile_prototypes(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            index: 1,
            name: Symbol::Constant {
                constant_id: ConstantId(0),
            },
            function_type: FunctionType::new([], [], Type::Float),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::FLOAT),
                Instruction::r#return(Address::register(0), OperandType::FLOAT),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_integer() {
    let source = create_function_case(local_cases::LOCAL_INTEGER, OperandType::INTEGER);
    let prototypes = compile_prototypes(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            index: 1,
            name: Symbol::Constant {
                constant_id: ConstantId(0),
            },
            function_type: FunctionType::new([], [], Type::Integer),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER),
                Instruction::r#return(Address::register(0), OperandType::INTEGER),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_string() {
    let source = create_function_case(local_cases::LOCAL_STRING, OperandType::STRING);
    let prototypes = compile_prototypes(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            index: 1,
            name: Symbol::Constant {
                constant_id: ConstantId(0),
            },
            function_type: FunctionType::new([], [], Type::String),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::STRING),
                Instruction::r#return(Address::register(0), OperandType::STRING),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}
