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
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::LIST_BOOLEAN
            )],
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
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::LIST_BYTE
            )],
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
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::LIST_CHARACTER
            )],
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
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::LIST_FLOAT
            )],
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
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::LIST_INTEGER
            )],
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
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::LIST_STRING
            )],
            ..Default::default()
        }
    );
}
