use dust_lang::*;

#[test]
fn panic() {
    let source = "panic(\"Goodbye world!\", 42)";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(6, 22)),
                (Instruction::load_constant(1, 1, false), Span(24, 26)),
                (
                    Instruction::call_native(2, NativeFunction::Panic, 2),
                    Span(0, 27)
                ),
                (Instruction::r#return(true), Span(27, 27))
            ],
            vec![
                ConcreteValue::string("Goodbye world!"),
                ConcreteValue::Integer(42)
            ],
            vec![]
        )),
    );

    assert_eq!(
        run(source),
        Err(DustError::Runtime {
            error: VmError::NativeFunction(NativeFunctionError::Panic {
                message: Some("Goodbye world! 42".to_string()),
                position: Span(0, 27)
            }),
            source
        })
    )
}

#[test]
fn to_string() {
    let source = "to_string(42)";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(10, 12)),
                (
                    Instruction::call_native(1, NativeFunction::ToString, 1),
                    Span(0, 13)
                ),
                (Instruction::r#return(true), Span(13, 13))
            ],
            vec![ConcreteValue::Integer(42)],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::string("42"))))
}
