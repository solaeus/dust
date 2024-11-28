use dust_lang::*;

#[test]
fn panic() {
    let source = "panic(\"Goodbye world!\", 42)";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::None),
            },
            vec![
                (
                    Instruction::load_constant(0, 0, false),
                    Type::String,
                    Span(6, 22)
                ),
                (
                    Instruction::load_constant(1, 1, false),
                    Type::Integer,
                    Span(24, 26)
                ),
                (
                    Instruction::call_native(2, NativeFunction::Panic, 2),
                    Type::None,
                    Span(0, 27)
                ),
                (Instruction::r#return(false), Type::None, Span(27, 27))
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
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::String),
            },
            vec![
                (
                    Instruction::load_constant(0, 0, false),
                    Type::Integer,
                    Span(10, 12)
                ),
                (
                    Instruction::call_native(1, NativeFunction::ToString, 1),
                    Type::String,
                    Span(0, 13)
                ),
                (Instruction::r#return(true), Type::None, Span(13, 13))
            ],
            vec![ConcreteValue::Integer(42)],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::string("42"))))
}
