use crate::{
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

use super::emit_function;

#[test]
fn constant_index() {
    let prototype = emit_function("fn foo() -> i32 { let arr: [i32; 3] = [10, 20, 30]; arr[1] }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::CONSTANT, 0),
                Instruction::r#move(1, OperandType::I_32, MemoryKind::CONSTANT, 1),
                Instruction::r#move(2, OperandType::I_32, MemoryKind::CONSTANT, 2),
                Instruction::r#move(3, OperandType::I_32, MemoryKind::REGISTER, 1),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_32],
            register_count: 4,
            argument_count: 0,
        }
    );
}

#[test]
fn out_of_bounds() {
    let prototype = emit_function("fn foo() -> i32 { let arr: [i32; 3] = [10, 20, 30]; arr[5] }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::CONSTANT, 0),
                Instruction::r#move(1, OperandType::I_32, MemoryKind::CONSTANT, 1),
                Instruction::r#move(2, OperandType::I_32, MemoryKind::CONSTANT, 2),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_32],
            register_count: 3,
            argument_count: 0,
        }
    );
}
