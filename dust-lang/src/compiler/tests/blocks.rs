use crate::{
    Address, Chunk, ConstantTable, FunctionType, Instruction, OperandType, Type, compile_main,
    tests::block_cases,
};

#[test]
fn empty_block() {
    let source = block_cases::EMPTY_BLOCK;
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::None),
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
    let mut constants = ConstantTable::new();

    constants.add_integer(42);

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::Integer),
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::INTEGER
            )],
            constants,
            ..Default::default()
        }
    );
}

#[test]
fn block_statement() {
    let source = block_cases::BLOCK_STATEMENT;
    let chunk = compile_main(source).unwrap();
    let mut constants = ConstantTable::new();

    constants.add_integer(42);

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::None),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::constant(0),
                    OperandType::INTEGER,
                    false
                ),
                Instruction::r#return(false, Address::default(), OperandType::NONE),
            ],
            constants,
            register_count: 1,
            ..Default::default()
        }
    );
}

#[test]
fn block_statement_and_expression() {
    let source = block_cases::BLOCK_STATEMENT_AND_EXPRESSION;
    let chunk = compile_main(source).unwrap();
    let mut constants = ConstantTable::new();

    constants.add_integer(42);
    constants.add_integer(1);

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::Integer),
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
                    Address::constant(1),
                    OperandType::INTEGER
                ),
                Instruction::r#return(true, Address::register(2), OperandType::INTEGER),
            ],
            constants,
            register_count: 3,
            ..Default::default()
        }
    );
}
