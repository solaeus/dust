use dust_lang::*;

#[test]
fn true_and_true() {
    let source = "true && true";

    assert_eq!(
        compile(source),
        Ok(Chunk {
            name: None,
            r#type: FunctionType {
                return_type: Type::Boolean,
                ..FunctionType::default()
            },
            instructions: vec![
                Instruction::load_boolean(0, true, false),
                Instruction::test(0, true),
                Instruction::jump(1, true),
                Instruction::load_boolean(1, true, false),
                Instruction::r#return(true, TypeCode::BOOLEAN, 1),
            ],
            positions: vec![
                Span(0, 4),
                Span(5, 7),
                Span(5, 7),
                Span(8, 12),
                Span(12, 12),
            ],
            constants: ConstantTable::new(),
            ..Chunk::default()
        })
    );

    assert_eq!(run(source), Ok(Some(Value::Boolean(true))));
}

#[test]
fn false_and_false() {
    let source = "false && false";

    assert_eq!(
        compile(source),
        Ok(Chunk {
            name: None,
            r#type: FunctionType {
                return_type: Type::Boolean,
                ..FunctionType::default()
            },
            instructions: vec![
                Instruction::load_boolean(0, false, false),
                Instruction::test(0, true),
                Instruction::jump(1, true),
                Instruction::load_boolean(1, false, false),
                Instruction::r#return(true, TypeCode::BOOLEAN, 1),
            ],
            positions: vec![
                Span(0, 5),
                Span(6, 8),
                Span(6, 8),
                Span(9, 14),
                Span(14, 14),
            ],
            constants: ConstantTable::new(),
            ..Chunk::default()
        })
    );

    assert_eq!(run(source), Ok(Some(Value::Boolean(false))));
}

#[test]
fn false_and_true() {
    let source = "false && true";

    assert_eq!(
        compile(source),
        Ok(Chunk {
            name: None,
            r#type: FunctionType {
                return_type: Type::Boolean,
                ..FunctionType::default()
            },
            instructions: vec![
                Instruction::load_boolean(0, false, false),
                Instruction::test(0, true),
                Instruction::jump(1, true),
                Instruction::load_boolean(1, true, false),
                Instruction::r#return(true, TypeCode::BOOLEAN, 1),
            ],
            positions: vec![
                Span(0, 5),
                Span(6, 8),
                Span(6, 8),
                Span(9, 13),
                Span(13, 13),
            ],
            constants: ConstantTable::new(),
            ..Chunk::default()
        })
    );

    assert_eq!(run(source), Ok(Some(Value::Boolean(false))));
}

#[test]
fn true_and_false() {
    let source = "true && false";

    assert_eq!(
        compile(source),
        Ok(Chunk {
            name: None,
            r#type: FunctionType {
                return_type: Type::Boolean,
                ..FunctionType::default()
            },
            instructions: vec![
                Instruction::load_boolean(0, true, false),
                Instruction::test(0, true),
                Instruction::jump(1, true),
                Instruction::load_boolean(1, false, false),
                Instruction::r#return(true, TypeCode::BOOLEAN, 1),
            ],
            positions: vec![
                Span(0, 4),
                Span(5, 8),
                Span(5, 8),
                Span(9, 14),
                Span(14, 14),
            ],
            constants: ConstantTable::new(),
            ..Chunk::default()
        })
    );

    assert_eq!(run(source), Ok(Some(Value::Boolean(false))));
}
