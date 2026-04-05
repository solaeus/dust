use crate::{
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

use super::emit_function;

#[test]
fn two_fields() {
    let prototype =
        emit_function("struct Point { x: i32, y: i32 } fn foo() -> Point { Point { x: 1, y: 2 } }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::ENCODED, 1),
                Instruction::r#move(1, OperandType::I_32, MemoryKind::ENCODED, 2),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_32, OperandType::I_32],
            register_count: 2,
            argument_count: 0,
        }
    );
}
