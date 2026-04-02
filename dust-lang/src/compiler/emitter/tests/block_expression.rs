use crate::{
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
    resolver::symbols::SymbolId,
    source::{Position, SourceFileId, Span},
};

use super::emit_function;

#[test]
fn block_with_tail_expression() {
    let prototype = emit_function("fn foo() -> i32 { let x: i32 = 1; { let y: i32 = 2; x + y } }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::add(
                    0,
                    OperandType::I_32,
                    MemoryKind::CONSTANT,
                    0,
                    MemoryKind::CONSTANT,
                    1
                ),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
            debug_symbol_id: Some(SymbolId(14)),
            debug_position: Position::new(SourceFileId::MAIN, Span::new(0, 61)),
        }
    );
}
