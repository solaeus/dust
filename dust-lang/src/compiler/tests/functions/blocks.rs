use crate::{
    compiler::compile,
    dust_type::DustType,
    instruction::{Address, Instruction, ByteType},
    prototype::Prototype,
    tests::{block_cases, create_function_case},
};

#[test]
fn empty_block() {
    let source = create_function_case(block_cases::EMPTY_BLOCK, ByteType::NONE);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::None,
            instructions: vec![Instruction::r#return(Address::default())],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn block_expression() {
    let source = create_function_case(block_cases::BLOCK_EXPRESSION, ByteType::INTEGER);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn block_statement() {
    let source = create_function_case(block_cases::BLOCK_STATEMENT, ByteType::NONE);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::None,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::r#return(Address::default()),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn block_statement_and_expression() {
    let source = create_function_case(
        block_cases::BLOCK_STATEMENT_AND_EXPRESSION,
        ByteType::INTEGER,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::add(1, Address::register(0), Address::constant(1)),
                Instruction::r#return(Address::register(1)),
            ],
            register_count: 2,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn parent_scope_access() {
    let source = create_function_case(block_cases::PARENT_SCOPE_ACCESS, ByteType::INTEGER);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::r#return(Address::register(0)),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn nested_parrent_scope_access() {
    let source = create_function_case(
        block_cases::NESTED_PARRENT_SCOPE_ACCESS,
        ByteType::INTEGER,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::r#move(1, Address::constant(1)),
                Instruction::add(2, Address::register(0), Address::register(1)),
                Instruction::r#return(Address::register(2)),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn scope_shadowing() {
    let source = create_function_case(block_cases::SCOPE_SHADOWING, ByteType::INTEGER);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::r#move(1, Address::constant(1)),
                Instruction::r#return(Address::register(1)),
            ],
            register_count: 2,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn scope_deshadowing() {
    let source = create_function_case(block_cases::SCOPE_DESHADOWING, ByteType::INTEGER);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::r#move(1, Address::constant(1)),
                Instruction::r#return(Address::register(0)),
            ],
            register_count: 2,
            ..Prototype::dummy()
        }
    );
}
