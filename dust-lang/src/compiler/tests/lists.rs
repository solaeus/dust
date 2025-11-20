use crate::{
    compiler::compile_main,
    instruction::{Address, Instruction, OperandType},
    prototype::Prototype,
    tests::list_cases,
    r#type::{FunctionType, Type},
};

#[test]
fn list_boolean() {
    let source = list_cases::LIST_BOOLEAN.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::list(Type::Boolean)),
            instructions: vec![
                Instruction::new_list(0, 3, OperandType::LIST_BOOLEAN),
                Instruction::set_list(0, Address::encoded(true as u16), 0, OperandType::BOOLEAN),
                Instruction::set_list(0, Address::encoded(false as u16), 1, OperandType::BOOLEAN),
                Instruction::set_list(0, Address::encoded(true as u16), 2, OperandType::BOOLEAN),
                Instruction::r#return(true, Address::register(0), OperandType::LIST_BOOLEAN),
            ],
            register_count: 1,
            ..Default::default()
        }
    );
}

#[test]
fn list_byte() {
    let source = list_cases::LIST_BYTE.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::list(Type::Byte)),
            instructions: vec![
                Instruction::new_list(0, 3, OperandType::LIST_BYTE),
                Instruction::set_list(0, Address::encoded(42), 0, OperandType::BYTE),
                Instruction::set_list(0, Address::encoded(43), 1, OperandType::BYTE),
                Instruction::set_list(0, Address::encoded(44), 2, OperandType::BYTE),
                Instruction::r#return(true, Address::register(0), OperandType::LIST_BYTE),
            ],
            register_count: 1,
            ..Default::default()
        }
    );
}

#[test]
fn list_character() {
    let source = list_cases::LIST_CHARACTER.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::list(Type::Character)),
            instructions: vec![
                Instruction::new_list(0, 3, OperandType::LIST_CHARACTER),
                Instruction::set_list(0, Address::constant(0), 0, OperandType::CHARACTER),
                Instruction::set_list(0, Address::constant(1), 1, OperandType::CHARACTER),
                Instruction::set_list(0, Address::constant(2), 2, OperandType::CHARACTER),
                Instruction::r#return(true, Address::register(0), OperandType::LIST_CHARACTER),
            ],
            register_count: 1,
            ..Default::default()
        }
    );
}

#[test]
fn list_float() {
    let source = list_cases::LIST_FLOAT.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::list(Type::Float)),
            instructions: vec![
                Instruction::new_list(0, 3, OperandType::LIST_FLOAT),
                Instruction::set_list(0, Address::constant(0), 0, OperandType::FLOAT),
                Instruction::set_list(0, Address::constant(1), 1, OperandType::FLOAT),
                Instruction::set_list(0, Address::constant(2), 2, OperandType::FLOAT),
                Instruction::r#return(true, Address::register(0), OperandType::LIST_FLOAT),
            ],
            register_count: 1,
            ..Default::default()
        }
    );
}

#[test]
fn list_integer() {
    let source = list_cases::LIST_INTEGER.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::list(Type::Integer)),
            instructions: vec![
                Instruction::new_list(0, 3, OperandType::LIST_INTEGER),
                Instruction::set_list(0, Address::constant(0), 0, OperandType::INTEGER),
                Instruction::set_list(0, Address::constant(1), 1, OperandType::INTEGER),
                Instruction::set_list(0, Address::constant(2), 2, OperandType::INTEGER),
                Instruction::r#return(true, Address::register(0), OperandType::LIST_INTEGER),
            ],
            register_count: 1,
            ..Default::default()
        }
    );
}

#[test]
fn list_string() {
    let source = list_cases::LIST_STRING.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::list(Type::String)),
            instructions: vec![
                Instruction::new_list(0, 3, OperandType::LIST_STRING),
                Instruction::set_list(0, Address::constant(0), 0, OperandType::STRING),
                Instruction::set_list(0, Address::constant(1), 1, OperandType::STRING),
                Instruction::set_list(0, Address::constant(2), 2, OperandType::STRING),
                Instruction::r#return(true, Address::register(0), OperandType::LIST_STRING),
            ],
            register_count: 1,
            ..Default::default()
        }
    );
}
