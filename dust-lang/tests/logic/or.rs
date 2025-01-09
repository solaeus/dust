use dust_lang::*;
use smallvec::smallvec;

#[test]
fn true_or_false() {
    let source = "true || false";

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
                Instruction::test(0, false),
                Instruction::jump(1, true),
                Instruction::load_boolean(1, false, false),
                Instruction::r#return(true),
            ],
            smallvec![
                Span(0, 4),
                Span(5, 7),
                Span(5, 7),
                Span(8, 13),
                Span(13, 13),
            ],
            smallvec![],
            smallvec![],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::boolean(true))));
}
