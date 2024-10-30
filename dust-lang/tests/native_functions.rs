use dust_lang::*;

#[test]
fn panic() {
    let source = "panic(\"Goodbye world!\", 42)";

    assert_eq!(
        parse(source),
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
            vec![Value::string("Goodbye world!"), Value::integer(42)],
            vec![]
        )),
    );

    assert_eq!(
        run(source),
        Err(DustError::Runtime {
            error: VmError::Panic {
                message: Some("Goodbye world! 42".to_string()),
                position: Span(0, 27)
            },
            source
        })
    )
}
