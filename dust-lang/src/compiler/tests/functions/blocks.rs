use crate::{
    compiler::compile,
    instruction::{Address, Instruction, OperandType},
    prototype::Prototype,
    source::{Position, SourceFileId, Span},
    tests::{block_cases, create_function_case},
    r#type::{FunctionType, Type},
};

#[test]
fn empty_block() {
    let source = create_function_case(block_cases::EMPTY_BLOCK);
    let prototypes = compile(source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            index: 1,
            name_position: Some(Position {
                file_id: SourceFileId(0),
                span: Span(16, 22)
            }),
            function_type: FunctionType::new([], [], Type::None),
            instructions: vec![Instruction::r#return(Address::default(), OperandType::NONE)],
            ..Default::default()
        }
    );
}

#[test]
fn block_expression() {
    let source = create_function_case(block_cases::BLOCK_EXPRESSION);
    let prototypes = compile(source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            index: 1,
            name_position: Some(Position {
                file_id: SourceFileId(0),
                span: Span(16, 22)
            }),
            function_type: FunctionType::new([], [], Type::Integer),
            instructions: vec![Instruction::r#return(
                Address::constant(0),
                OperandType::INTEGER
            )],
            ..Default::default()
        }
    );
}

#[test]
fn block_statement() {
    let source = create_function_case(block_cases::BLOCK_STATEMENT);
    let prototypes = compile(source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            index: 1,
            name_position: Some(Position {
                file_id: SourceFileId(0),
                span: Span(16, 22)
            }),
            function_type: FunctionType::new([], [], Type::None),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER),
                Instruction::r#return(Address::default(), OperandType::NONE),
            ],
            register_count: 1,
            ..Default::default()
        }
    );
}

#[test]
fn block_statement_and_expression() {
    let source = create_function_case(block_cases::BLOCK_STATEMENT_AND_EXPRESSION);
    let prototypes = compile(source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            index: 1,
            name_position: Some(Position {
                file_id: SourceFileId(0),
                span: Span(16, 22)
            }),
            function_type: FunctionType::new([], [], Type::Integer),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER),
                Instruction::add(
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
fn parent_scope_access() {
    let source = create_function_case(block_cases::PARENT_SCOPE_ACCESS);
    let prototypes = compile(source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            index: 1,
            name_position: Some(Position {
                file_id: SourceFileId(0),
                span: Span(16, 22)
            }),
            function_type: FunctionType::new([], [], Type::Integer),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER),
                Instruction::r#return(Address::register(0), OperandType::INTEGER),
            ],
            register_count: 1,
            ..Default::default()
        }
    );
}

#[test]
fn nested_parrent_scope_access() {
    let source = create_function_case(block_cases::NESTED_PARRENT_SCOPE_ACCESS);
    let prototypes = compile(source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            index: 1,
            name_position: Some(Position {
                file_id: SourceFileId(0),
                span: Span(16, 22)
            }),
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
                Instruction::r#return(Address::register(2), OperandType::INTEGER),
            ],
            register_count: 3,
            ..Default::default()
        }
    );
}

#[test]
fn scope_shadowing() {
    let source = create_function_case(block_cases::SCOPE_SHADOWING);
    let prototypes = compile(source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            index: 1,
            name_position: Some(Position {
                file_id: SourceFileId(0),
                span: Span(16, 22)
            }),
            function_type: FunctionType::new([], [], Type::Integer),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER),
                Instruction::r#move(1, Address::constant(1), OperandType::INTEGER),
                Instruction::r#return(Address::register(1), OperandType::INTEGER),
            ],
            register_count: 2,
            ..Default::default()
        }
    );
}

#[test]
fn scope_deshadowing() {
    let source = create_function_case(block_cases::SCOPE_DESHADOWING);
    let prototypes = compile(source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            index: 1,
            name_position: Some(Position {
                file_id: SourceFileId(0),
                span: Span(16, 22)
            }),
            function_type: FunctionType::new([], [], Type::Integer),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER),
                Instruction::r#move(1, Address::constant(1), OperandType::INTEGER),
                Instruction::r#return(Address::register(0), OperandType::INTEGER),
            ],
            register_count: 2,
            ..Default::default()
        }
    );
}
