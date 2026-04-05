use crate::{
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

use super::emit_function;

#[test]
fn unused_binding() {
    let prototype = emit_function("fn foo() -> i32 { let x: i32 = 1; let y: i32 = 2; x }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(1, OperandType::I_32, MemoryKind::ENCODED, 1),
                Instruction::r#move(2, OperandType::I_32, MemoryKind::ENCODED, 2),
                Instruction::r#move(0, OperandType::I_32, MemoryKind::REGISTER, 1),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_32],
            register_count: 3,
            argument_count: 0,
        }
    );
}
