use dust_lang::*;

#[test]
fn add_boolean_left() {
    let source = "true + 1";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotAddType {
                argument_type: Type::Boolean,
                position: Span(0, 4)
            },
            source,
        })
    );
}

#[test]
fn add_boolean_right() {
    let source = "1 + true";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotAddType {
                argument_type: Type::Boolean,
                position: Span(4, 8)
            },
            source,
        })
    );
}

#[test]
fn add_function_left() {
    let source = "fn(){} + 1";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotAddType {
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
fn add_function_right() {
    let source = "1 + fn(){}";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotAddType {
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
fn add_list_left() {
    let source = "[1, 2] + 1";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotAddType {
                argument_type: Type::List(Box::new(Type::Integer)),
                position: Span(0, 6)
            },
            source,
        })
    );
}

#[test]
fn add_list_right() {
    let source = "1 + [1, 2]";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotAddType {
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
//

#[test]
fn add_byte_and_character() {
    let source = "0xff + 'a'";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotAddArguments {
                left_type: Type::Byte,
                right_type: Type::Character,
                position: Span(0, 10)
            },
            source,
        })
    );
}

#[test]
fn add_byte_and_integer() {
    let source = "0xff + 1";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotAddArguments {
                left_type: Type::Byte,
                right_type: Type::Integer,
                position: Span(0, 8)
            },
            source,
        })
    );
}

#[test]
fn add_byte_and_string() {
    let source = "0xff + \"hello\"";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotAddArguments {
                left_type: Type::Byte,
                right_type: Type::String,
                position: Span(0, 14)
            },
            source,
        })
    );
}

#[test]
fn add_character_and_byte() {
    let source = "'a' + 0xff";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotAddArguments {
                left_type: Type::Character,
                right_type: Type::Byte,
                position: Span(0, 10)
            },
            source,
        })
    );
}

#[test]
fn add_character_and_float() {
    let source = "'a' + 1.0";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotAddArguments {
                left_type: Type::Character,
                right_type: Type::Float,
                position: Span(0, 9)
            },
            source,
        })
    );
}

#[test]
fn add_character_and_integer() {
    let source = "'a' + 1";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotAddArguments {
                left_type: Type::Character,
                right_type: Type::Integer,
                position: Span(0, 7)
            },
            source,
        })
    );
}

#[test]
fn add_float_and_byte() {
    let source = "1.0 + 0xff";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotAddArguments {
                left_type: Type::Float,
                right_type: Type::Byte,
                position: Span(0, 10)
            },
            source,
        })
    );
}

#[test]
fn add_float_and_character() {
    let source = "1.0 + 'a'";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotAddArguments {
                left_type: Type::Float,
                right_type: Type::Character,
                position: Span(0, 9)
            },
            source,
        })
    );
}

#[test]
fn add_float_and_integer() {
    let source = "1.0 + 1";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotAddArguments {
                left_type: Type::Float,
                right_type: Type::Integer,
                position: Span(0, 7)
            },
            source,
        })
    );
}

#[test]
fn add_float_and_string() {
    let source = "1.0 + \"hello\"";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotAddArguments {
                left_type: Type::Float,
                right_type: Type::String,
                position: Span(0, 13)
            },
            source,
        })
    );
}

#[test]
fn add_integer_and_byte() {
    let source = "1 + 0xff";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotAddArguments {
                left_type: Type::Integer,
                right_type: Type::Byte,
                position: Span(0, 8)
            },
            source,
        })
    );
}

#[test]
fn add_integer_and_character() {
    let source = "1 + 'a'";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotAddArguments {
                left_type: Type::Integer,
                right_type: Type::Character,
                position: Span(0, 7)
            },
            source,
        })
    );
}

#[test]
fn add_integer_and_float() {
    let source = "1 + 1.0";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotAddArguments {
                left_type: Type::Integer,
                right_type: Type::Float,
                position: Span(0, 7)
            },
            source,
        })
    );
}

#[test]
fn add_integer_and_string() {
    let source = "1 + \"hello\"";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotAddArguments {
                left_type: Type::Integer,
                right_type: Type::String,
                position: Span(0, 11)
            },
            source,
        })
    );
}

#[test]
fn add_string_and_byte() {
    let source = "\"hello\" + 0xff";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotAddArguments {
                left_type: Type::String,
                right_type: Type::Byte,
                position: Span(0, 14)
            },
            source,
        })
    );
}

#[test]
fn add_string_and_float() {
    let source = "\"hello\" + 1.0";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotAddArguments {
                left_type: Type::String,
                right_type: Type::Float,
                position: Span(0, 13)
            },
            source,
        })
    );
}

#[test]
fn add_string_and_integer() {
    let source = "\"hello\" + 1";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::CannotAddArguments {
                left_type: Type::String,
                right_type: Type::Integer,
                position: Span(0, 11)
            },
            source,
        })
    );
}
