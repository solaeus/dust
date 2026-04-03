use crate::{
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

use super::emit_function;

#[test]
fn variable() {
    let prototype = emit_function("fn foo() -> bool { let x: bool = true; !x }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::negate(0, OperandType::BOOLEAN, MemoryKind::CONSTANT, 0),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::BOOLEAN],
            register_count: 1,
            argument_count: 0,
        }
    );
}
