use crate::{
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
    resolver::symbols::SymbolId,
    source::{Position, SourceFileId, Span},
};

use super::emit_function;

#[test]
fn function_with_argument() {
    let prototype =
        emit_function("fn foo() -> i32 { fn add_one(x: i32) -> i32 { x + 1 } add_one(5) }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(1, OperandType::I_32, MemoryKind::CONSTANT, 0),
                Instruction::call(0, MemoryKind::CONSTANT, 1, 1),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
            debug_symbol_id: Some(SymbolId(14)),
            debug_position: Position::new(SourceFileId::MAIN, Span::new(0, 66)),
        }
    );
}
