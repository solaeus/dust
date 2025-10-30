use crate::{
    chunk::Chunk,
    compiler::compile_main,
    instruction::{Address, Instruction, OperandType},
    tests::if_else_cases,
    r#type::{FunctionType, Type},
};

#[test]
fn if_else_true() {
    let source = if_else_cases::IF_ELSE_TRUE.to_string();
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            r#type: FunctionType::new([], [], Type::Integer),
            instructions: vec![
                // if true { 42 } else { 0 }
                Instruction::test(Address::encoded(true as u16), true),
                Instruction::jump(2, true),
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER, false), // 42
                Instruction::jump(1, true),
                Instruction::r#move(0, Address::constant(1), OperandType::INTEGER, false), // 0
                Instruction::r#return(true, Address::register(0), OperandType::INTEGER),
            ],
            register_count: 1,
            ..Default::default()
        }
    );
}

#[test]
fn if_else_false() {
    let source = if_else_cases::IF_ELSE_FALSE.to_string();
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            r#type: FunctionType::new([], [], Type::Integer),
            instructions: vec![
                // if false { 0 } else { 42 }
                Instruction::test(Address::encoded(false as u16), true),
                Instruction::jump(2, true),
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER, false), // 0
                Instruction::jump(1, true),
                Instruction::r#move(0, Address::constant(1), OperandType::INTEGER, false), // 42
                Instruction::r#return(true, Address::register(0), OperandType::INTEGER),
            ],
            register_count: 1,
            ..Default::default()
        }
    );
}

#[test]
fn if_else_equal() {
    let source = if_else_cases::IF_ELSE_EQUAL.to_string();
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            r#type: FunctionType::new([], [], Type::Integer),
            instructions: vec![
                // let a = 0;
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER, false),
                // let b = 0;
                Instruction::r#move(1, Address::constant(0), OperandType::INTEGER, false),
                // if a == b { 42 } else { 0 }
                Instruction::equal(
                    true,
                    Address::register(0),
                    Address::register(1),
                    OperandType::INTEGER
                ),
                Instruction::jump(2, true),
                Instruction::r#move(2, Address::constant(1), OperandType::INTEGER, false), // 42
                Instruction::jump(1, true),
                Instruction::r#move(2, Address::constant(0), OperandType::INTEGER, false), // 0
                Instruction::r#return(true, Address::register(2), OperandType::INTEGER),
            ],
            register_count: 3,
            ..Default::default()
        }
    );
}
