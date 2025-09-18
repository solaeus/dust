use crate::{
    Address, Chunk, Instruction, OperandType, compile_main, resolver::TypeId, tests::block_cases,
};
use std::sync::Arc;

#[test]
fn empty_block() {
    let source = block_cases::EMPTY_BLOCK;
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            name: Arc::new("main".to_string()),
            r#type: TypeId(2),
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
    let source = block_cases::BLOCK_EXPRESSION;
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            name: Arc::new("main".to_string()),
            r#type: TypeId(2),
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
    let source = block_cases::BLOCK_STATEMENT;
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            name: Arc::new("main".to_string()),
            r#type: TypeId(2),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::constant(0),
                    OperandType::INTEGER,
                    false
                ),
                Instruction::r#return(false, Address::default(), OperandType::NONE),
            ],
            register_count: 1,
            ..Default::default()
        }
    );
}

#[test]
fn block_statement_and_expression() {
    let source = block_cases::BLOCK_STATEMENT_AND_EXPRESSION;
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            name: Arc::new("main".to_string()),
            r#type: TypeId(2),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::constant(0),
                    OperandType::INTEGER,
                    false
                ),
                Instruction::add(
                    Address::register(1),
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
    let source = block_cases::PARENT_SCOPE_ACCESS;
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            name: Arc::new("main".to_string()),
            r#type: TypeId(2),
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::INTEGER
            ),],
            register_count: 0,
            ..Default::default()
        }
    );
}

#[test]
fn nested_parrent_scope_access() {
    let source = block_cases::NESTED_PARRENT_SCOPE_ACCESS;
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            name: Arc::new("main".to_string()),
            r#type: TypeId(2),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::constant(0),
                    OperandType::INTEGER,
                    false
                ),
                Instruction::load(
                    Address::register(1),
                    Address::constant(1),
                    OperandType::INTEGER,
                    false
                ),
                Instruction::add(
                    Address::register(2),
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
    let source = block_cases::SCOPE_SHADOWING;
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            name: Arc::new("main".to_string()),
            r#type: TypeId(2),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::constant(0),
                    OperandType::INTEGER,
                    false
                ),
                Instruction::r#return(true, Address::constant(1), OperandType::INTEGER),
            ],
            register_count: 1,
            ..Default::default()
        }
    );
}

#[test]
fn scope_deshadowing() {
    let source = block_cases::SCOPE_DESHADOWING;
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            name: Arc::new("main".to_string()),
            r#type: TypeId(2),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::constant(0),
                    OperandType::INTEGER,
                    false
                ),
                Instruction::load(
                    Address::register(1),
                    Address::constant(1),
                    OperandType::INTEGER,
                    false
                ),
                Instruction::r#return(true, Address::register(0), OperandType::INTEGER),
            ],
            register_count: 2,
            ..Default::default()
        }
    );
}
