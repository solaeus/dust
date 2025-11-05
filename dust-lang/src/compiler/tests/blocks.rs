use crate::{
    chunk::Chunk,
    compiler::compile_main,
    instruction::{Address, Instruction, OperandType},
    tests::block_cases,
    r#type::{FunctionType, Type},
};

#[test]
fn empty_block() {
    let source = block_cases::EMPTY_BLOCK.to_string();
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            function_type: FunctionType::new([], [], Type::None),
            instructions: vec![Instruction::r#return(
                false,
                Address::default(),
                OperandType::NONE
            )],
            ..Default::default()
        }
    );
}

#[test]
fn block_expression() {
    let source = block_cases::BLOCK_EXPRESSION.to_string();
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            function_type: FunctionType::new([], [], Type::Integer),
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::INTEGER
            )],
            ..Default::default()
        }
    );
}

#[test]
fn block_statement() {
    let source = block_cases::BLOCK_STATEMENT.to_string();
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            function_type: FunctionType::new([], [], Type::None),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER, false),
                Instruction::r#return(false, Address::default(), OperandType::NONE),
            ],
            register_count: 1,
            ..Default::default()
        }
    );
}

#[test]
fn block_statement_and_expression() {
    let source = block_cases::BLOCK_STATEMENT_AND_EXPRESSION.to_string();
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            function_type: FunctionType::new([], [], Type::Integer),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER, false),
                Instruction::add(
                    1,
                    Address::register(0),
                    Address::constant(1),
                    OperandType::INTEGER
                ),
                Instruction::r#return(true, Address::register(1), OperandType::INTEGER),
            ],
            register_count: 2,
            ..Default::default()
        }
    );
}

#[test]
fn parent_scope_access() {
    let source = block_cases::PARENT_SCOPE_ACCESS.to_string();
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            function_type: FunctionType::new([], [], Type::Integer),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER, false),
                Instruction::r#return(true, Address::register(0), OperandType::INTEGER),
            ],
            register_count: 1,
            ..Default::default()
        }
    );
}

#[test]
fn nested_parrent_scope_access() {
    let source = block_cases::NESTED_PARRENT_SCOPE_ACCESS.to_string();
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            function_type: FunctionType::new([], [], Type::Integer),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER, false),
                Instruction::r#move(1, Address::constant(1), OperandType::INTEGER, false),
                Instruction::add(
                    2,
                    Address::register(0),
                    Address::register(1),
                    OperandType::INTEGER
                ),
                Instruction::r#return(true, Address::register(2), OperandType::INTEGER),
            ],
            register_count: 3,
            ..Default::default()
        }
    );
}

#[test]
fn scope_shadowing() {
    let source = block_cases::SCOPE_SHADOWING.to_string();
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            function_type: FunctionType::new([], [], Type::Integer),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER, false),
                Instruction::r#move(1, Address::constant(1), OperandType::INTEGER, false),
                Instruction::r#return(true, Address::register(1), OperandType::INTEGER),
            ],
            register_count: 2,
            ..Default::default()
        }
    );
}

#[test]
fn scope_deshadowing() {
    let source = block_cases::SCOPE_DESHADOWING.to_string();
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            function_type: FunctionType::new([], [], Type::Integer),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER, false),
                Instruction::r#move(1, Address::constant(1), OperandType::INTEGER, false),
                Instruction::r#return(true, Address::register(0), OperandType::INTEGER),
            ],
            register_count: 2,
            ..Default::default()
        }
    );
}
