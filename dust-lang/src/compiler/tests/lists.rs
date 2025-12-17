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
                Instruction::new_list(0, Address::constant(3), OperandType::LIST_BOOLEAN),
                Instruction::set_list(
                    0,
                    Address::encoded(true as u16),
                    Address::constant(0),
                    OperandType::BOOLEAN
                ),
                Instruction::set_list(
                    0,
                    Address::encoded(false as u16),
                    Address::constant(1),
                    OperandType::BOOLEAN
                ),
                Instruction::set_list(
                    0,
                    Address::encoded(true as u16),
                    Address::constant(2),
                    OperandType::BOOLEAN
                ),
                Instruction::r#return(Address::register(0), OperandType::LIST_BOOLEAN),
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
                Instruction::new_list(0, Address::constant(3), OperandType::LIST_BYTE),
                Instruction::set_list(
                    0,
                    Address::encoded(42),
                    Address::constant(0),
                    OperandType::BYTE
                ),
                Instruction::set_list(
                    0,
                    Address::encoded(43),
                    Address::constant(1),
                    OperandType::BYTE
                ),
                Instruction::set_list(
                    0,
                    Address::encoded(44),
                    Address::constant(2),
                    OperandType::BYTE
                ),
                Instruction::r#return(Address::register(0), OperandType::LIST_BYTE),
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
                Instruction::new_list(0, Address::constant(6), OperandType::LIST_CHARACTER),
                Instruction::set_list(
                    0,
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::CHARACTER
                ),
                Instruction::set_list(
                    0,
                    Address::constant(2),
                    Address::constant(3),
                    OperandType::CHARACTER
                ),
                Instruction::set_list(
                    0,
                    Address::constant(4),
                    Address::constant(5),
                    OperandType::CHARACTER
                ),
                Instruction::r#return(Address::register(0), OperandType::LIST_CHARACTER),
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
                Instruction::new_list(0, Address::constant(6), OperandType::LIST_FLOAT),
                Instruction::set_list(
                    0,
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::FLOAT
                ),
                Instruction::set_list(
                    0,
                    Address::constant(2),
                    Address::constant(3),
                    OperandType::FLOAT
                ),
                Instruction::set_list(
                    0,
                    Address::constant(4),
                    Address::constant(5),
                    OperandType::FLOAT
                ),
                Instruction::r#return(Address::register(0), OperandType::LIST_FLOAT),
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
                Instruction::new_list(0, Address::constant(3), OperandType::LIST_INTEGER),
                Instruction::set_list(
                    0,
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::INTEGER
                ),
                Instruction::set_list(
                    0,
                    Address::constant(2),
                    Address::constant(0),
                    OperandType::INTEGER
                ),
                Instruction::set_list(
                    0,
                    Address::constant(3),
                    Address::constant(2),
                    OperandType::INTEGER
                ),
                Instruction::r#return(Address::register(0), OperandType::LIST_INTEGER),
            ],
            register_count: 1,
            ..Default::default()
        }
    );
}

#[test]
fn list_equal() {
    let source = list_cases::LIST_EQUAL.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::new_list(0, Address::constant(2), OperandType::LIST_BOOLEAN),
                Instruction::set_list(
                    0,
                    Address::encoded(true as u16),
                    Address::constant(0),
                    OperandType::BOOLEAN
                ),
                Instruction::set_list(
                    0,
                    Address::encoded(false as u16),
                    Address::constant(1),
                    OperandType::BOOLEAN
                ),
                Instruction::new_list(1, Address::constant(2), OperandType::LIST_BOOLEAN),
                Instruction::set_list(
                    1,
                    Address::encoded(true as u16),
                    Address::constant(0),
                    OperandType::BOOLEAN
                ),
                Instruction::set_list(
                    1,
                    Address::encoded(false as u16),
                    Address::constant(1),
                    OperandType::BOOLEAN
                ),
                Instruction::equal(
                    true,
                    Address::register(0),
                    Address::register(1),
                    OperandType::LIST_BOOLEAN
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    OperandType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), OperandType::BOOLEAN),
                Instruction::r#return(Address::register(2), OperandType::BOOLEAN),
            ],
            register_count: 3,
            ..Default::default()
        }
    );
}

#[test]
fn list_not_equal() {
    let source = list_cases::LIST_NOT_EQUAL.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::new_list(0, Address::constant(2), OperandType::LIST_BYTE),
                Instruction::set_list(
                    0,
                    Address::encoded(0x2A),
                    Address::constant(0),
                    OperandType::BYTE
                ),
                Instruction::set_list(
                    0,
                    Address::encoded(0x2B),
                    Address::constant(1),
                    OperandType::BYTE
                ),
                Instruction::new_list(1, Address::constant(2), OperandType::LIST_BYTE),
                Instruction::set_list(
                    1,
                    Address::encoded(0x2B),
                    Address::constant(0),
                    OperandType::BYTE
                ),
                Instruction::set_list(
                    1,
                    Address::encoded(0x2A),
                    Address::constant(1),
                    OperandType::BYTE
                ),
                Instruction::equal(
                    false,
                    Address::register(0),
                    Address::register(1),
                    OperandType::LIST_BYTE
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    OperandType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), OperandType::BOOLEAN),
                Instruction::r#return(Address::register(2), OperandType::BOOLEAN),
            ],
            register_count: 3,
            ..Default::default()
        }
    );
}

#[test]
fn list_greater_than() {
    let source = list_cases::LIST_GREATER_THAN.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::new_list(0, Address::constant(4), OperandType::LIST_CHARACTER),
                Instruction::set_list(
                    0,
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::CHARACTER
                ),
                Instruction::set_list(
                    0,
                    Address::constant(2),
                    Address::constant(3),
                    OperandType::CHARACTER
                ),
                Instruction::new_list(1, Address::constant(4), OperandType::LIST_CHARACTER),
                Instruction::set_list(
                    1,
                    Address::constant(2),
                    Address::constant(1),
                    OperandType::CHARACTER
                ),
                Instruction::set_list(
                    1,
                    Address::constant(0),
                    Address::constant(3),
                    OperandType::CHARACTER
                ),
                Instruction::less_equal(
                    false,
                    Address::register(0),
                    Address::register(1),
                    OperandType::LIST_CHARACTER
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    OperandType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), OperandType::BOOLEAN),
                Instruction::r#return(Address::register(2), OperandType::BOOLEAN),
            ],
            register_count: 3,
            ..Default::default()
        }
    );
}

#[test]
fn list_less_than() {
    let source = list_cases::LIST_LESS_THAN.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::new_list(0, Address::constant(4), OperandType::LIST_FLOAT),
                Instruction::set_list(
                    0,
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::FLOAT
                ),
                Instruction::set_list(
                    0,
                    Address::constant(2),
                    Address::constant(3),
                    OperandType::FLOAT
                ),
                Instruction::new_list(1, Address::constant(4), OperandType::LIST_FLOAT),
                Instruction::set_list(
                    1,
                    Address::constant(2),
                    Address::constant(1),
                    OperandType::FLOAT
                ),
                Instruction::set_list(
                    1,
                    Address::constant(0),
                    Address::constant(3),
                    OperandType::FLOAT
                ),
                Instruction::less(
                    true,
                    Address::register(0),
                    Address::register(1),
                    OperandType::LIST_FLOAT
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    OperandType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), OperandType::BOOLEAN),
                Instruction::r#return(Address::register(2), OperandType::BOOLEAN),
            ],
            register_count: 3,
            ..Default::default()
        }
    );
}

#[test]
fn list_greater_than_or_equal() {
    let source = list_cases::LIST_GREATER_THAN_OR_EQUAL.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::new_list(0, Address::constant(2), OperandType::LIST_INTEGER),
                Instruction::set_list(
                    0,
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::INTEGER
                ),
                Instruction::set_list(
                    0,
                    Address::constant(2),
                    Address::constant(0),
                    OperandType::INTEGER
                ),
                Instruction::new_list(1, Address::constant(2), OperandType::LIST_INTEGER),
                Instruction::set_list(
                    1,
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::INTEGER
                ),
                Instruction::set_list(
                    1,
                    Address::constant(2),
                    Address::constant(0),
                    OperandType::INTEGER
                ),
                Instruction::less(
                    false,
                    Address::register(0),
                    Address::register(1),
                    OperandType::LIST_INTEGER
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    OperandType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), OperandType::BOOLEAN),
                Instruction::r#return(Address::register(2), OperandType::BOOLEAN),
            ],
            register_count: 3,
            ..Default::default()
        }
    );
}

#[test]
fn list_less_than_or_equal() {
    let source = list_cases::LIST_LESS_THAN_OR_EQUAL.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::new_list(0, Address::constant(4), OperandType::LIST_STRING),
                Instruction::set_list(
                    0,
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::STRING
                ),
                Instruction::set_list(
                    0,
                    Address::constant(2),
                    Address::constant(3),
                    OperandType::STRING
                ),
                Instruction::new_list(1, Address::constant(4), OperandType::LIST_STRING),
                Instruction::set_list(
                    1,
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::STRING
                ),
                Instruction::set_list(
                    1,
                    Address::constant(2),
                    Address::constant(3),
                    OperandType::STRING
                ),
                Instruction::less_equal(
                    true,
                    Address::register(0),
                    Address::register(1),
                    OperandType::LIST_STRING
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    OperandType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), OperandType::BOOLEAN),
                Instruction::r#return(Address::register(2), OperandType::BOOLEAN),
            ],
            register_count: 3,
            ..Default::default()
        }
    );
}

#[test]
fn list_index_boolean() {
    let source = list_cases::LIST_INDEX_BOOLEAN.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::new_list(0, Address::constant(3), OperandType::LIST_BOOLEAN),
                Instruction::set_list(
                    0,
                    Address::encoded(true as u16),
                    Address::constant(0),
                    OperandType::BOOLEAN
                ),
                Instruction::set_list(
                    0,
                    Address::encoded(false as u16),
                    Address::constant(1),
                    OperandType::BOOLEAN
                ),
                Instruction::set_list(
                    0,
                    Address::encoded(true as u16),
                    Address::constant(2),
                    OperandType::BOOLEAN
                ),
                Instruction::get_list(
                    1,
                    Address::register(0),
                    Address::constant(0),
                    OperandType::BOOLEAN
                ),
                Instruction::r#return(Address::register(1), OperandType::BOOLEAN),
            ],
            register_count: 2,
            ..Default::default()
        }
    );
}

#[test]
fn list_index_byte() {
    let source = list_cases::LIST_INDEX_BYTE.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::Byte),
            instructions: vec![
                Instruction::new_list(0, Address::constant(3), OperandType::LIST_BYTE),
                Instruction::set_list(
                    0,
                    Address::encoded(0x2A),
                    Address::constant(0),
                    OperandType::BYTE
                ),
                Instruction::set_list(
                    0,
                    Address::encoded(0x2B),
                    Address::constant(1),
                    OperandType::BYTE
                ),
                Instruction::set_list(
                    0,
                    Address::encoded(0x2C),
                    Address::constant(2),
                    OperandType::BYTE
                ),
                Instruction::get_list(
                    1,
                    Address::register(0),
                    Address::constant(1),
                    OperandType::BYTE
                ),
                Instruction::r#return(Address::register(1), OperandType::BYTE),
            ],
            register_count: 2,
            ..Default::default()
        }
    );
}

#[test]
fn list_index_character() {
    let source = list_cases::LIST_INDEX_CHARACTER.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::Character),
            instructions: vec![
                Instruction::new_list(0, Address::constant(6), OperandType::LIST_CHARACTER),
                Instruction::set_list(
                    0,
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::CHARACTER
                ),
                Instruction::set_list(
                    0,
                    Address::constant(2),
                    Address::constant(3),
                    OperandType::CHARACTER
                ),
                Instruction::set_list(
                    0,
                    Address::constant(4),
                    Address::constant(5),
                    OperandType::CHARACTER
                ),
                Instruction::get_list(
                    1,
                    Address::register(0),
                    Address::constant(5),
                    OperandType::CHARACTER
                ),
                Instruction::r#return(Address::register(1), OperandType::CHARACTER),
            ],
            register_count: 2,
            ..Default::default()
        }
    );
}

#[test]
fn list_index_float() {
    let source = list_cases::LIST_INDEX_FLOAT.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::Float),
            instructions: vec![
                Instruction::new_list(0, Address::constant(6), OperandType::LIST_FLOAT),
                Instruction::set_list(
                    0,
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::FLOAT
                ),
                Instruction::set_list(
                    0,
                    Address::constant(2),
                    Address::constant(3),
                    OperandType::FLOAT
                ),
                Instruction::set_list(
                    0,
                    Address::constant(4),
                    Address::constant(5),
                    OperandType::FLOAT
                ),
                Instruction::get_list(
                    1,
                    Address::register(0),
                    Address::constant(3),
                    OperandType::FLOAT
                ),
                Instruction::r#return(Address::register(1), OperandType::FLOAT),
            ],
            register_count: 2,
            ..Default::default()
        }
    );
}

#[test]
fn list_index_integer() {
    let source = list_cases::LIST_INDEX_INTEGER.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::Integer),
            instructions: vec![
                Instruction::new_list(0, Address::constant(3), OperandType::LIST_INTEGER),
                Instruction::set_list(
                    0,
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::INTEGER
                ),
                Instruction::set_list(
                    0,
                    Address::constant(2),
                    Address::constant(0),
                    OperandType::INTEGER
                ),
                Instruction::set_list(
                    0,
                    Address::constant(3),
                    Address::constant(2),
                    OperandType::INTEGER
                ),
                Instruction::get_list(
                    1,
                    Address::register(0),
                    Address::constant(1),
                    OperandType::INTEGER
                ),
                Instruction::r#return(Address::register(1), OperandType::INTEGER),
            ],
            register_count: 2,
            ..Default::default()
        }
    );
}

#[test]
fn list_index_string() {
    let source = list_cases::LIST_INDEX_STRING.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::String),
            instructions: vec![
                Instruction::new_list(0, Address::constant(6), OperandType::LIST_STRING),
                Instruction::set_list(
                    0,
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::STRING
                ),
                Instruction::set_list(
                    0,
                    Address::constant(2),
                    Address::constant(3),
                    OperandType::STRING
                ),
                Instruction::set_list(
                    0,
                    Address::constant(4),
                    Address::constant(5),
                    OperandType::STRING
                ),
                Instruction::get_list(
                    1,
                    Address::register(0),
                    Address::constant(5),
                    OperandType::STRING
                ),
                Instruction::r#return(Address::register(1), OperandType::STRING),
            ],
            register_count: 2,
            ..Default::default()
        }
    );
}

#[test]
fn local_list_boolean() {
    let source = list_cases::LOCAL_LIST_BOOLEAN.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::list(Type::Boolean)),
            instructions: vec![
                Instruction::new_list(0, Address::constant(3), OperandType::LIST_BOOLEAN),
                Instruction::set_list(
                    0,
                    Address::encoded(true as u16),
                    Address::constant(0),
                    OperandType::BOOLEAN
                ),
                Instruction::set_list(
                    0,
                    Address::encoded(false as u16),
                    Address::constant(1),
                    OperandType::BOOLEAN
                ),
                Instruction::set_list(
                    0,
                    Address::encoded(true as u16),
                    Address::constant(2),
                    OperandType::BOOLEAN
                ),
                Instruction::r#return(Address::register(0), OperandType::LIST_BOOLEAN),
            ],
            register_count: 1,
            ..Default::default()
        }
    );
}

#[test]
fn local_list_equal() {
    let source = list_cases::LOCAL_LIST_EQUAL.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::new_list(0, Address::constant(2), OperandType::LIST_BOOLEAN),
                Instruction::set_list(
                    0,
                    Address::encoded(true as u16),
                    Address::constant(0),
                    OperandType::BOOLEAN
                ),
                Instruction::set_list(
                    0,
                    Address::encoded(false as u16),
                    Address::constant(1),
                    OperandType::BOOLEAN
                ),
                Instruction::new_list(1, Address::constant(2), OperandType::LIST_BOOLEAN),
                Instruction::set_list(
                    1,
                    Address::encoded(true as u16),
                    Address::constant(0),
                    OperandType::BOOLEAN
                ),
                Instruction::set_list(
                    1,
                    Address::encoded(false as u16),
                    Address::constant(1),
                    OperandType::BOOLEAN
                ),
                Instruction::equal(
                    true,
                    Address::register(0),
                    Address::register(1),
                    OperandType::LIST_BOOLEAN
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    OperandType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), OperandType::BOOLEAN),
                Instruction::r#return(Address::register(2), OperandType::BOOLEAN),
            ],
            register_count: 3,
            ..Default::default()
        }
    );
}

#[test]
fn local_list_not_equal() {
    let source = list_cases::LOCAL_LIST_NOT_EQUAL.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::new_list(0, Address::constant(2), OperandType::LIST_BYTE),
                Instruction::set_list(
                    0,
                    Address::encoded(0x2A),
                    Address::constant(0),
                    OperandType::BYTE
                ),
                Instruction::set_list(
                    0,
                    Address::encoded(0x2B),
                    Address::constant(1),
                    OperandType::BYTE
                ),
                Instruction::new_list(1, Address::constant(2), OperandType::LIST_BYTE),
                Instruction::set_list(
                    1,
                    Address::encoded(0x2B),
                    Address::constant(0),
                    OperandType::BYTE
                ),
                Instruction::set_list(
                    1,
                    Address::encoded(0x2A),
                    Address::constant(1),
                    OperandType::BYTE
                ),
                Instruction::equal(
                    false,
                    Address::register(0),
                    Address::register(1),
                    OperandType::LIST_BYTE
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    OperandType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), OperandType::BOOLEAN),
                Instruction::r#return(Address::register(2), OperandType::BOOLEAN),
            ],
            register_count: 3,
            ..Default::default()
        }
    );
}

#[test]
fn local_list_greater_than() {
    let source = list_cases::LOCAL_LIST_GREATER_THAN.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::new_list(0, Address::constant(4), OperandType::LIST_CHARACTER),
                Instruction::set_list(
                    0,
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::CHARACTER
                ),
                Instruction::set_list(
                    0,
                    Address::constant(2),
                    Address::constant(3),
                    OperandType::CHARACTER
                ),
                Instruction::new_list(1, Address::constant(4), OperandType::LIST_CHARACTER),
                Instruction::set_list(
                    1,
                    Address::constant(2),
                    Address::constant(1),
                    OperandType::CHARACTER
                ),
                Instruction::set_list(
                    1,
                    Address::constant(0),
                    Address::constant(3),
                    OperandType::CHARACTER
                ),
                Instruction::less_equal(
                    false,
                    Address::register(0),
                    Address::register(1),
                    OperandType::LIST_CHARACTER
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    OperandType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), OperandType::BOOLEAN),
                Instruction::r#return(Address::register(2), OperandType::BOOLEAN),
            ],
            register_count: 3,
            ..Default::default()
        }
    );
}

#[test]
fn local_list_less_than() {
    let source = list_cases::LOCAL_LIST_LESS_THAN.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::new_list(0, Address::constant(4), OperandType::LIST_FLOAT),
                Instruction::set_list(
                    0,
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::FLOAT
                ),
                Instruction::set_list(
                    0,
                    Address::constant(2),
                    Address::constant(3),
                    OperandType::FLOAT
                ),
                Instruction::new_list(1, Address::constant(4), OperandType::LIST_FLOAT),
                Instruction::set_list(
                    1,
                    Address::constant(2),
                    Address::constant(1),
                    OperandType::FLOAT
                ),
                Instruction::set_list(
                    1,
                    Address::constant(0),
                    Address::constant(3),
                    OperandType::FLOAT
                ),
                Instruction::less(
                    true,
                    Address::register(1),
                    Address::register(0),
                    OperandType::LIST_FLOAT
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    OperandType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), OperandType::BOOLEAN),
                Instruction::r#return(Address::register(2), OperandType::BOOLEAN),
            ],
            register_count: 3,
            ..Default::default()
        }
    );
}

#[test]
fn local_list_greater_than_or_equal() {
    let source = list_cases::LOCAL_LIST_GREATER_THAN_OR_EQUAL.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::new_list(0, Address::constant(2), OperandType::LIST_INTEGER),
                Instruction::set_list(
                    0,
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::INTEGER
                ),
                Instruction::set_list(
                    0,
                    Address::constant(2),
                    Address::constant(0),
                    OperandType::INTEGER
                ),
                Instruction::new_list(1, Address::constant(2), OperandType::LIST_INTEGER),
                Instruction::set_list(
                    1,
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::INTEGER
                ),
                Instruction::set_list(
                    1,
                    Address::constant(2),
                    Address::constant(0),
                    OperandType::INTEGER
                ),
                Instruction::less(
                    false,
                    Address::register(0),
                    Address::register(1),
                    OperandType::LIST_INTEGER
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    OperandType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), OperandType::BOOLEAN),
                Instruction::r#return(Address::register(2), OperandType::BOOLEAN),
            ],
            register_count: 3,
            ..Default::default()
        }
    );
}

#[test]
fn local_list_less_than_or_equal() {
    let source = list_cases::LOCAL_LIST_LESS_THAN_OR_EQUAL.to_string();
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::new_list(0, Address::constant(4), OperandType::LIST_STRING),
                Instruction::set_list(
                    0,
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::STRING
                ),
                Instruction::set_list(
                    0,
                    Address::constant(2),
                    Address::constant(3),
                    OperandType::STRING
                ),
                Instruction::new_list(1, Address::constant(4), OperandType::LIST_STRING),
                Instruction::set_list(
                    1,
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::STRING
                ),
                Instruction::set_list(
                    1,
                    Address::constant(2),
                    Address::constant(3),
                    OperandType::STRING
                ),
                Instruction::less_equal(
                    true,
                    Address::register(0),
                    Address::register(1),
                    OperandType::LIST_STRING
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    OperandType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), OperandType::BOOLEAN),
                Instruction::r#return(Address::register(2), OperandType::BOOLEAN),
            ],
            register_count: 3,
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
                Instruction::new_list(0, Address::constant(6), OperandType::LIST_STRING),
                Instruction::set_list(
                    0,
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::STRING
                ),
                Instruction::set_list(
                    0,
                    Address::constant(2),
                    Address::constant(3),
                    OperandType::STRING
                ),
                Instruction::set_list(
                    0,
                    Address::constant(4),
                    Address::constant(5),
                    OperandType::STRING
                ),
                Instruction::r#return(Address::register(0), OperandType::LIST_STRING),
            ],
            register_count: 1,
            ..Default::default()
        }
    );
}
