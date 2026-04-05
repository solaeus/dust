use crate::{
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

use super::emit_function;

#[test]
fn ascii() {
    let prototype = emit_function("fn foo() -> u8 { 0x60 }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::U_8, MemoryKind::ENCODED, 96),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::U_8],
            register_count: 1,
            argument_count: 0,
        }
    );
}
