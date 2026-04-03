use crate::{
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

use super::emit_function;

#[test]
fn literal() {
    let prototype = emit_function("fn foo() -> Range<i32> { 1..10 }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::CONSTANT, 0),
                Instruction::r#move(1, OperandType::I_32, MemoryKind::CONSTANT, 1),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_32, OperandType::I_32],
            register_count: 2,
            argument_count: 0,
        }
    );
}
