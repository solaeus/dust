use dust_lang::*;
use smallvec::smallvec;

#[test]
fn true_and_true_and_true() {
    let source = "true && true && true";

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
                Instruction::load_encoded(0, true, false),
                Instruction::test(0, true),
                Instruction::jump(1, true),
                Instruction::load_encoded(1, true, false),
                Instruction::test(1, true),
                Instruction::jump(1, true),
                Instruction::load_encoded(2, true, false),
                Instruction::r#return(true),
            ],
            smallvec![
                Span(0, 4),
                Span(5, 7),
                Span(5, 7),
                Span(8, 12),
                Span(13, 15),
                Span(13, 15),
                Span(16, 20),
                Span(20, 20)
            ],
            smallvec![],
            smallvec![],
            vec![],
        ))
    );
}
