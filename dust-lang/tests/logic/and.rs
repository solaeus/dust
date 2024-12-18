use dust_lang::*;
use smallvec::smallvec;

#[test]
fn true_and_true() {
    let source = "true && true";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Type::Boolean,
            },
            smallvec![
                Instruction::load_boolean(0, true, false),
                Instruction::test(0, true),
                Instruction::jump(1, true),
                Instruction::load_boolean(1, true, false),
                Instruction::r#return(true),
            ],
            smallvec![
                Span(0, 4),
                Span(5, 7),
                Span(5, 7),
                Span(8, 12),
                Span(12, 12),
            ],
            smallvec![],
            smallvec![],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::boolean(true))));
}

#[test]
fn false_and_false() {
    let source = "false && false";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Type::Boolean,
            },
            smallvec![
                Instruction::load_boolean(0, false, false),
                Instruction::test(0, true),
                Instruction::jump(1, true),
                Instruction::load_boolean(1, false, false),
                Instruction::r#return(true),
            ],
            smallvec![
                Span(0, 5),
                Span(6, 8),
                Span(6, 8),
                Span(9, 14),
                Span(14, 14),
            ],
            smallvec![],
            smallvec![],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::boolean(false))));
}

#[test]
fn false_and_true() {
    let source = "false && true";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Type::Boolean,
            },
            smallvec![
                Instruction::load_boolean(0, false, false),
                Instruction::test(0, true),
                Instruction::jump(1, true),
                Instruction::load_boolean(1, true, false),
                Instruction::r#return(true),
            ],
            smallvec![
                Span(0, 5),
                Span(6, 8),
                Span(6, 8),
                Span(9, 13),
                Span(13, 13)
            ],
            smallvec![],
            smallvec![],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::boolean(false))));
}

#[test]
fn true_and_false() {
    let source = "true && false";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Type::Boolean,
            },
            smallvec![
                Instruction::load_boolean(0, true, false),
                Instruction::test(0, true),
                Instruction::jump(1, true),
                Instruction::load_boolean(1, false, false),
                Instruction::r#return(true),
            ],
            smallvec![
                Span(0, 4),
                Span(5, 7),
                Span(5, 7),
                Span(8, 13),
                Span(13, 13)
            ],
            smallvec![],
            smallvec![],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::boolean(false))));
}
