use dust_lang::*;
use smallvec::smallvec;

#[test]
fn add_assign() {
    let source = "let mut a = 1; a += 2; a";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Type::Integer,
            },
            smallvec![
                Instruction::load_constant(0, 0, false),
                Instruction::add(0, Argument::Register(0), Argument::Constant(2)),
                Instruction::get_local(1, 0),
                Instruction::r#return(true)
            ],
            smallvec![Span(12, 13), Span(17, 19), Span(23, 24), Span(24, 24)],
            smallvec![Value::integer(1), Value::string("a"), Value::integer(2)],
            smallvec![Local::new(1, 0, true, Scope::default())],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::integer(3))));
}
