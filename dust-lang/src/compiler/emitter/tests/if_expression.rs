use crate::{
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

use super::emit_function;

#[test]
fn with_else() {
    let prototype = emit_function("fn foo() -> i32 { if true { 1 } else { 2 } }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::test(true, MemoryKind::ENCODED, 1, 2),
                Instruction::r#move(0, OperandType::I_32, MemoryKind::ENCODED, 1),
                Instruction::jump(1, true),
                Instruction::r#move(0, OperandType::I_32, MemoryKind::ENCODED, 2),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
        }
    );
}

#[test]
fn without_else() {
    let prototype = emit_function("fn foo() { let x: i32 = 1; if x == 1 {} }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::test(true, MemoryKind::ENCODED, 1, 0),
                Instruction::r#return(),
            ],
            return_types: vec![],
            register_count: 0,
            argument_count: 0,
        }
    );
}
