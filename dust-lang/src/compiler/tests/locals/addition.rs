use crate::{
    compiler::compile_main,
    instruction::{Address, Instruction, OperandType},
    prototype::Prototype,
    tests::local_cases,
    r#type::{FunctionType, Type},
};

#[test]
fn local_byte_addition() {
    let source = local_cases::LOCAL_BYTE_ADDITION.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::Byte),
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
            ..Default::default()
        }
    );
}

#[test]
fn local_float_addition() {
    let source = local_cases::LOCAL_FLOAT_ADDITION.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::Float),
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
            ..Default::default()
        }
    );
}

#[test]
fn local_integer_addition() {
    let source = local_cases::LOCAL_INTEGER_ADDITION.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::Integer),
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
            ..Default::default()
        }
    );
}

#[test]
fn local_string_concatenation() {
    let source = local_cases::LOCAL_STRING_CONCATENATION.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::String),
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
            ..Default::default()
        }
    );
}

#[test]
fn local_character_concatenation() {
    let source = local_cases::LOCAL_CHARACTER_CONCATENATION.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::String),
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
            ..Default::default()
        }
    );
}

#[test]
fn local_string_character_concatenation() {
    let source = local_cases::LOCAL_STRING_CHARACTER_CONCATENATION.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::String),
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
            ..Default::default()
        }
    );
}

#[test]
fn local_character_string_concatenation() {
    let source = local_cases::LOCAL_CHARACTER_STRING_CONCATENATION.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::String),
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
            ..Default::default()
        }
    );
}

#[test]
fn local_mut_byte_addition() {
    let source = local_cases::LOCAL_MUT_BYTE_ADDITION.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::Byte),
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
            ..Default::default()
        }
    );
}

#[test]
fn local_mut_float_addition() {
    let source = local_cases::LOCAL_MUT_FLOAT_ADDITION.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::Float),
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
            ..Default::default()
        }
    );
}

#[test]
fn local_mut_integer_addition() {
    let source = local_cases::LOCAL_MUT_INTEGER_ADDITION.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::Integer),
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
            ..Default::default()
        }
    );
}

#[test]
fn local_mut_string_concatenation() {
    let source = local_cases::LOCAL_MUT_STRING_CONCATENATION.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::String),
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
            ..Default::default()
        }
    );
}

#[test]
fn local_mut_string_character_concatenation() {
    let source = local_cases::LOCAL_MUT_STRING_CHARACTER_CONCATENATION.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::String),
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
            ..Default::default()
        }
    );
}
