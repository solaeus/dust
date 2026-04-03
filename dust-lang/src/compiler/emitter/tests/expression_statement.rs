use crate::{
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

use super::emit_function;

#[test]
fn reassignment() {
    let prototype = emit_function("fn foo() { let mut x: i32 = 0; x = 1; }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::CONSTANT, 0),
                Instruction::r#return(),
            ],
            return_types: vec![],
            register_count: 1,
            argument_count: 0,
        }
    );
}
