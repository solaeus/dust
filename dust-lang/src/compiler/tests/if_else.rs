use crate::{
    compiler::compile_main,
    dust_type::{DustFunctionType, DustType},
    instruction::{Address, Instruction, OperandType},
    prototype::Prototype,
    symbol_table::SymbolId,
    tests::if_else_cases,
};

#[test]
fn if_else_true() {
    let source = if_else_cases::IF_ELSE_TRUE;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: DustFunctionType::new([], [], DustType::Integer),
            instructions: vec![
                Instruction::test(Address::encoded(true as u16), true, 1),
                Instruction::jump(1, true),
                Instruction::move_with_jump(0, Address::constant(0), OperandType::INTEGER, 1, true),
                Instruction::r#move(0, Address::constant(1), OperandType::INTEGER),
                Instruction::r#return(Address::register(0), OperandType::INTEGER),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn if_else_logical_and() {
    let source = if_else_cases::IF_ELSE_LOGICAL_AND;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: DustFunctionType::new([], [], DustType::Integer),
            instructions: vec![
                Instruction::r#move(0, Address::encoded(true as u16), OperandType::BOOLEAN),
                Instruction::r#move(1, Address::encoded(true as u16), OperandType::BOOLEAN),
                Instruction::test(Address::register(0), false, 1),
                Instruction::test(Address::register(1), false, 1),
                Instruction::move_with_jump(2, Address::constant(0), OperandType::INTEGER, 1, true),
                Instruction::r#move(2, Address::constant(1), OperandType::INTEGER),
                Instruction::r#return(Address::register(2), OperandType::INTEGER),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn if_else_logical_or() {
    let source = if_else_cases::IF_ELSE_LOGICAL_OR;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: DustFunctionType::new([], [], DustType::Integer),
            instructions: vec![
                Instruction::r#move(0, Address::encoded(false as u16), OperandType::BOOLEAN),
                Instruction::r#move(1, Address::encoded(true as u16), OperandType::BOOLEAN),
                Instruction::test(Address::register(0), true, 1),
                Instruction::test(Address::register(1), false, 1),
                Instruction::move_with_jump(2, Address::constant(0), OperandType::INTEGER, 1, true),
                Instruction::r#move(2, Address::constant(1), OperandType::INTEGER),
                Instruction::r#return(Address::register(2), OperandType::INTEGER),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn if_else_false() {
    let source = if_else_cases::IF_ELSE_FALSE;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: DustFunctionType::new([], [], DustType::Integer),
            instructions: vec![
                Instruction::test(Address::encoded(false as u16), true, 1),
                Instruction::jump(1, true),
                Instruction::move_with_jump(0, Address::constant(0), OperandType::INTEGER, 1, true),
                Instruction::r#move(0, Address::constant(1), OperandType::INTEGER),
                Instruction::r#return(Address::register(0), OperandType::INTEGER),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn if_else_equal() {
    let source = if_else_cases::IF_ELSE_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: DustFunctionType::new([], [], DustType::Integer),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER),
                Instruction::r#move(1, Address::constant(0), OperandType::INTEGER),
                Instruction::equal(
                    true,
                    Address::register(0),
                    Address::register(1),
                    OperandType::INTEGER
                ),
                Instruction::jump(1, true),
                Instruction::move_with_jump(2, Address::constant(1), OperandType::INTEGER, 1, true),
                Instruction::r#move(2, Address::constant(0), OperandType::INTEGER),
                Instruction::r#return(Address::register(2), OperandType::INTEGER),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn if_else_not_equal() {
    let source = if_else_cases::IF_ELSE_NOT_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: DustFunctionType::new([], [], DustType::Integer),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER),
                Instruction::r#move(1, Address::constant(1), OperandType::INTEGER),
                Instruction::equal(
                    false,
                    Address::register(0),
                    Address::register(1),
                    OperandType::INTEGER
                ),
                Instruction::jump(1, true),
                Instruction::move_with_jump(2, Address::constant(2), OperandType::INTEGER, 1, true),
                Instruction::r#move(2, Address::constant(0), OperandType::INTEGER),
                Instruction::r#return(Address::register(2), OperandType::INTEGER),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn if_else_less_than() {
    let source = if_else_cases::IF_ELSE_LESS_THAN;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: DustFunctionType::new([], [], DustType::Integer),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER),
                Instruction::r#move(1, Address::constant(1), OperandType::INTEGER),
                Instruction::less(
                    true,
                    Address::register(0),
                    Address::register(1),
                    OperandType::INTEGER
                ),
                Instruction::jump(1, true),
                Instruction::move_with_jump(2, Address::constant(2), OperandType::INTEGER, 1, true),
                Instruction::r#move(2, Address::constant(0), OperandType::INTEGER),
                Instruction::r#return(Address::register(2), OperandType::INTEGER),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn if_else_greater_than() {
    let source = if_else_cases::IF_ELSE_GREATER_THAN;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: DustFunctionType::new([], [], DustType::Integer),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER),
                Instruction::r#move(1, Address::constant(1), OperandType::INTEGER),
                Instruction::less_equal(
                    false,
                    Address::register(0),
                    Address::register(1),
                    OperandType::INTEGER
                ),
                Instruction::jump(1, true),
                Instruction::move_with_jump(2, Address::constant(2), OperandType::INTEGER, 1, true),
                Instruction::r#move(2, Address::constant(1), OperandType::INTEGER),
                Instruction::r#return(Address::register(2), OperandType::INTEGER),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn if_else_less_than_equal() {
    let source = if_else_cases::IF_ELSE_LESS_THAN_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: DustFunctionType::new([], [], DustType::Integer),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER),
                Instruction::r#move(1, Address::constant(0), OperandType::INTEGER),
                Instruction::less_equal(
                    true,
                    Address::register(0),
                    Address::register(1),
                    OperandType::INTEGER
                ),
                Instruction::jump(1, true),
                Instruction::move_with_jump(2, Address::constant(1), OperandType::INTEGER, 1, true),
                Instruction::r#move(2, Address::constant(0), OperandType::INTEGER),
                Instruction::r#return(Address::register(2), OperandType::INTEGER),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn if_else_greater_than_equal() {
    let source = if_else_cases::IF_ELSE_GREATER_THAN_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: DustFunctionType::new([], [], DustType::Integer),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER),
                Instruction::r#move(1, Address::constant(1), OperandType::INTEGER),
                Instruction::less(
                    false,
                    Address::register(0),
                    Address::register(1),
                    OperandType::INTEGER
                ),
                Instruction::jump(1, true),
                Instruction::move_with_jump(2, Address::constant(2), OperandType::INTEGER, 1, true),
                Instruction::r#move(2, Address::constant(1), OperandType::INTEGER),
                Instruction::r#return(Address::register(2), OperandType::INTEGER),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn if_else_if_chain_end() {
    let source = if_else_cases::IF_ELSE_IF_CHAIN_END;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: DustFunctionType::new([], [], DustType::Integer),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER),
                Instruction::r#move(1, Address::constant(1), OperandType::INTEGER),
                Instruction::less(
                    true,
                    Address::register(0),
                    Address::register(1),
                    OperandType::INTEGER
                ),
                Instruction::jump(1, true),
                Instruction::move_with_jump(2, Address::constant(2), OperandType::INTEGER, 4, true),
                Instruction::equal(
                    true,
                    Address::register(0),
                    Address::register(1),
                    OperandType::INTEGER
                ),
                Instruction::jump(1, true),
                Instruction::move_with_jump(2, Address::constant(1), OperandType::INTEGER, 1, true),
                Instruction::r#move(2, Address::constant(3), OperandType::INTEGER),
                Instruction::r#return(Address::register(2), OperandType::INTEGER),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn if_else_if_chain_middle() {
    let source = if_else_cases::IF_ELSE_IF_CHAIN_MIDDLE;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: DustFunctionType::new([], [], DustType::Integer),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER),
                Instruction::r#move(1, Address::constant(0), OperandType::INTEGER),
                Instruction::less(
                    true,
                    Address::register(0),
                    Address::register(1),
                    OperandType::INTEGER
                ),
                Instruction::jump(1, true),
                Instruction::move_with_jump(2, Address::constant(1), OperandType::INTEGER, 4, true),
                Instruction::equal(
                    true,
                    Address::register(0),
                    Address::register(1),
                    OperandType::INTEGER
                ),
                Instruction::jump(1, true),
                Instruction::move_with_jump(2, Address::constant(2), OperandType::INTEGER, 1, true),
                Instruction::r#move(2, Address::constant(0), OperandType::INTEGER),
                Instruction::r#return(Address::register(2), OperandType::INTEGER),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn if_else_nested() {
    let source = if_else_cases::IF_ELSE_NESTED;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: DustFunctionType::new([], [], DustType::Integer),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER),
                Instruction::r#move(1, Address::constant(1), OperandType::INTEGER),
                Instruction::less(
                    true,
                    Address::register(0),
                    Address::register(1),
                    OperandType::INTEGER
                ),
                Instruction::jump(4, true),
                Instruction::less_equal(
                    false,
                    Address::register(1),
                    Address::register(0),
                    OperandType::INTEGER
                ),
                Instruction::jump(1, true),
                Instruction::move_with_jump(2, Address::constant(2), OperandType::INTEGER, 2, true),
                Instruction::move_with_jump(2, Address::constant(3), OperandType::INTEGER, 1, true),
                Instruction::r#move(2, Address::constant(3), OperandType::INTEGER),
                Instruction::r#return(Address::register(2), OperandType::INTEGER),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn if_else_double_nested() {
    let source = if_else_cases::IF_ELSE_DOUBLE_NESTED;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: DustFunctionType::new([], [], DustType::Integer),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER),
                Instruction::r#move(1, Address::constant(1), OperandType::INTEGER),
                Instruction::less(
                    true,
                    Address::register(0),
                    Address::register(1),
                    OperandType::INTEGER
                ),
                Instruction::jump(7, true),
                Instruction::less_equal(
                    false,
                    Address::register(1),
                    Address::register(0),
                    OperandType::INTEGER
                ),
                Instruction::jump(4, true),
                Instruction::equal(
                    false,
                    Address::register(0),
                    Address::constant(2),
                    OperandType::INTEGER
                ),
                Instruction::jump(1, true),
                Instruction::move_with_jump(2, Address::constant(3), OperandType::INTEGER, 3, true),
                Instruction::move_with_jump(2, Address::constant(2), OperandType::INTEGER, 2, true),
                Instruction::move_with_jump(2, Address::constant(2), OperandType::INTEGER, 1, true),
                Instruction::r#move(2, Address::constant(2), OperandType::INTEGER),
                Instruction::r#return(Address::register(2), OperandType::INTEGER),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}
