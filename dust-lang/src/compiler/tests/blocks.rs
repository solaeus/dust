use crate::{
    compiler::compile_main,
    dust_type::DustType,
    instruction::{Address, Instruction, ByteType},
    prototype::Prototype,
    tests::block_cases,
};

#[test]
fn empty_block() {
    let source = block_cases::EMPTY_BLOCK;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::None,
            instructions: vec![Instruction::r#return(Address::default(), ByteType::NONE)],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn block_expression() {
    let source = block_cases::BLOCK_EXPRESSION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![Instruction::r#return(
                Address::constant(0),
                ByteType::INTEGER
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn block_statement() {
    let source = block_cases::BLOCK_STATEMENT;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::None,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), ByteType::INTEGER),
                Instruction::r#return(Address::default(), ByteType::NONE),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn block_statement_and_expression() {
    let source = block_cases::BLOCK_STATEMENT_AND_EXPRESSION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), ByteType::INTEGER),
                Instruction::add(
                    1,
                    Address::register(0),
                    Address::constant(1),
                    ByteType::INTEGER
                ),
                Instruction::r#return(Address::register(1), ByteType::INTEGER),
            ],
            register_count: 2,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn parent_scope_access() {
    let source = block_cases::PARENT_SCOPE_ACCESS;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), ByteType::INTEGER),
                Instruction::r#return(Address::register(0), ByteType::INTEGER),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn nested_parrent_scope_access() {
    let source = block_cases::NESTED_PARRENT_SCOPE_ACCESS;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), ByteType::INTEGER),
                Instruction::r#move(1, Address::constant(1), ByteType::INTEGER),
                Instruction::add(
                    2,
                    Address::register(0),
                    Address::register(1),
                    ByteType::INTEGER
                ),
                Instruction::r#return(Address::register(2), ByteType::INTEGER),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn scope_shadowing() {
    let source = block_cases::SCOPE_SHADOWING;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), ByteType::INTEGER),
                Instruction::r#move(1, Address::constant(1), ByteType::INTEGER),
                Instruction::r#return(Address::register(1), ByteType::INTEGER),
            ],
            register_count: 2,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn scope_deshadowing() {
    let source = block_cases::SCOPE_DESHADOWING;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), ByteType::INTEGER),
                Instruction::r#move(1, Address::constant(1), ByteType::INTEGER),
                Instruction::r#return(Address::register(0), ByteType::INTEGER),
            ],
            register_count: 2,
            ..Prototype::dummy()
        }
    );
}
