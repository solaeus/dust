use smallvec::smallvec;

use crate::{
    compiler::emitter::tests::emit_function,
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

#[test]
fn ascii() {
    let prototype = emit_function("fn foo() -> char { 'a' }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::CHARACTER, MemoryKind::ENCODED, 97),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::CHARACTER],
            register_count: 1,
            argument_count: 0,
        }
    );
}
