use crate::{
    instruction::{Instruction, MemoryKind},
    prototype::Prototype,
    resolver::symbols::SymbolId,
    source::{Position, SourceFileId, Span},
};

use super::emit_function;

#[test]
fn while_loop() {
    let prototype = emit_function("fn foo() { while true {} }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::test(false, MemoryKind::CONSTANT, 0, 1),
                Instruction::jump(1, false),
                Instruction::r#return(),
            ],
            return_types: vec![],
            register_count: 0,
            argument_count: 0,
            debug_symbol_id: Some(SymbolId(14)),
            debug_position: Position::new(SourceFileId::MAIN, Span::new(0, 25)),
        }
    );
}
