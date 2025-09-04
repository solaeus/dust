use crate::{
    Address, Chunk, ConstantTable, FunctionType, Instruction, OperandType, Type,
    compile_main, tests::local_cases,
};

#[test]
fn local_declaration() {
    let source = local_cases::LOCAL_DECLARATION;
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
            register_count: 1,
            constants,
            ..Default::default()
        }
    );
}

#[test]
fn local_mut_declaration() {
    let source = local_cases::LOCAL_MUT_DECLARATION;
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
            register_count: 1,
            constants,
            ..Default::default()
        }
    );
}

#[test]
fn local_evalutation() {
    let source = local_cases::LOCAL_EVALUATION;
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
            ),],
            constants,
            ..Default::default()
        }
    );
}
