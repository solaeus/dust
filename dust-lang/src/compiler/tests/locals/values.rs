use crate::compiler::Symbol;
use crate::{
    compiler::compile_main_prototype,
    instruction::{Address, Instruction, OperandType},
    prototype::Prototype,
    tests::local_cases,
    r#type::{FunctionType, Type},
};

#[test]
fn local_boolean() {
    let source = local_cases::LOCAL_BOOLEAN;
    let prototype = compile_main_prototype(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            name: Symbol::MAIN,
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
    let source = local_cases::LOCAL_BYTE;
    let prototype = compile_main_prototype(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            name: Symbol::MAIN,
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
    let source = local_cases::LOCAL_CHARACTER;
    let prototype = compile_main_prototype(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            name: Symbol::MAIN,
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
    let source = local_cases::LOCAL_FLOAT;
    let prototype = compile_main_prototype(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            name: Symbol::MAIN,
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
    let source = local_cases::LOCAL_INTEGER;
    let prototype = compile_main_prototype(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            name: Symbol::MAIN,
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
    let source = local_cases::LOCAL_STRING;
    let prototype = compile_main_prototype(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            name: Symbol::MAIN,
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

#[test]
fn local_function() {
    let source = local_cases::LOCAL_FUNCTION;
    let prototype = compile_main_prototype(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            name: Symbol::MAIN,
            function_type: FunctionType::new([], [], Type::Integer),
            instructions: vec![
                Instruction::r#move(0, Address::constant(1), OperandType::FUNCTION),
                Instruction::call(Some(1), Address::register(0), 0, 1),
                Instruction::r#return(Address::register(1), OperandType::INTEGER),
            ],
            call_arguments: vec![(Address::constant(1), OperandType::INTEGER)],
            register_count: 2,
            ..Prototype::dummy()
        }
    );
}
