use crate::{
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

use super::emit_function;

#[test]
fn empty() {
    let prototype = emit_function("fn foo() { while true {} }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::test(false, MemoryKind::CONSTANT, 0, 1),
                Instruction::jump(1, false),
                Instruction::r#return(),
            ],
            return_types: vec![],
            register_count: 0,
            argument_count: 0,
        }
    );
}

#[test]
fn with_body() {
    let prototype =
        emit_function("fn foo() { let mut x: i32 = 0; while x < 10 { x += 1; } }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::CONSTANT, 0),
                Instruction::less(
                    true,
                    OperandType::I_32,
                    MemoryKind::REGISTER,
                    0,
                    MemoryKind::CONSTANT,
                    1
                ),
                Instruction::add(
                    0,
                    OperandType::I_32,
                    MemoryKind::REGISTER,
                    0,
                    MemoryKind::CONSTANT,
                    2
                ),
                Instruction::jump(3, false),
                Instruction::r#return(),
            ],
            return_types: vec![],
            register_count: 1,
            argument_count: 0,
        }
    );
}
