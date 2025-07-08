use std::sync::Arc;

use indexmap::IndexMap;

use crate::{
    Address, Chunk, DustString, FullChunk, FunctionType, Instruction, OperandType, Path, Span,
    Type, Value, compile, run,
};

#[test]
fn boolean() {
    let chunk = compile::<FullChunk>("false").unwrap();
    let return_value = run("false").unwrap();

    assert_eq!(
        chunk,
        FullChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![Instruction::r#return(
                true,
                Address::encoded(0),
                OperandType::BOOLEAN
            )],
            constants: vec![],
            locals: IndexMap::new(),
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Boolean(false)));
}
