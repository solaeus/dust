use dust_lang::*;

#[test]
fn multiply_boolean_left() {
    let source = "true * 1";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotMultiplyType {
                argument_type: Type::Boolean,
                position: Span(0, 4)
            },
            source,
        })
    );
}

#[test]
fn multiply_boolean_right() {
    let source = "1 * true";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotMultiplyType {
                argument_type: Type::Boolean,
                position: Span(4, 8)
            },
            source,
        })
    );
}

#[test]
fn multiply_character_left() {
    let source = "'a' * 1";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotMultiplyType {
                argument_type: Type::Character,
                position: Span(0, 3)
            },
            source,
        })
    );
}

#[test]
fn multiply_character_right() {
    let source = "1 * 'a'";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotMultiplyType {
                argument_type: Type::Character,
                position: Span(4, 7)
            },
            source,
        })
    );
}

#[test]
fn multiply_float_and_character() {
    let source = "1.0 * 'a'";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotMultiplyType {
                argument_type: Type::Character,
                position: Span(6, 9)
            },
            source,
        })
    );
}

#[test]
fn multiply_float_and_integer() {
    let source = "1.0 * 1";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotMultiplyArguments {
                left_type: Type::Float,
                right_type: Type::Integer,
                position: Span(0, 7)
            },
            source,
        })
    );
}

#[test]
fn multiply_function_left() {
    let source = "fn(){} * 1";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotMultiplyType {
                argument_type: Type::Function(FunctionType {
                    type_parameters: None,
                    value_parameters: None,
                    return_type: Box::new(Type::None)
                }),
                position: Span(0, 6)
            },
            source,
        })
    );
}

#[test]
fn multiply_function_right() {
    let source = "1 * fn(){}";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotMultiplyType {
                argument_type: Type::Function(FunctionType {
                    type_parameters: None,
                    value_parameters: None,
                    return_type: Box::new(Type::None)
                }),
                position: Span(4, 10)
            },
            source,
        })
    );
}

#[test]
fn multiply_integer_and_float() {
    let source = "1 * 1.0";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotMultiplyArguments {
                left_type: Type::Integer,
                right_type: Type::Float,
                position: Span(0, 7)
            },
            source,
        })
    );
}

#[test]
fn multiply_list_left() {
    let source = "[1, 2] * 1";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotMultiplyType {
                argument_type: Type::List(Box::new(Type::Integer)),
                position: Span(0, 6)
            },
            source,
        })
    );
}

#[test]
fn multiply_list_right() {
    let source = "1 * [1, 2]";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotMultiplyType {
                argument_type: Type::List(Box::new(Type::Integer)),
                position: Span(4, 10)
            },
            source,
        })
    );
}

// #[test]
// fn add_range_left() {
//     todo!("Add ranges")
// }

// #[test]
// fn add_range_right() {
//     todo!("Add ranges")
// }

#[test]
fn multiply_string_left() {
    let source = "\"hello\" * 1";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotMultiplyType {
                argument_type: Type::String,
                position: Span(0, 7)
            },
            source,
        })
    );
}

#[test]
fn multiply_string_right() {
    let source = "1 * \"hello\"";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotMultiplyType {
                argument_type: Type::String,
                position: Span(4, 11)
            },
            source,
        })
    );
}
