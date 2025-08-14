use crate::{
    Address, Chunk, FunctionType, Instruction, List, OperandType, Path, Program, Type, Value,
    compile, run,
};

#[test]
fn boolean_true() {
    let source = "true";
    let program = compile(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        program,
        Program {
            main_chunk: Chunk {
                name: Some(Path::new("main").unwrap()),
                r#type: FunctionType::new([], [], Type::Boolean),
                instructions: vec![Instruction::r#return(
                    true,
                    Address::encoded(true as u16),
                    OperandType::BOOLEAN
                )],
                prototype_index: u16::MAX,
                ..Default::default()
            },
            cell_count: 0,
            prototypes: Vec::new(),
        }
    );
    assert_eq!(return_value, Some(Value::Boolean(true)));
}

#[test]
fn boolean_false() {
    let source = "false";
    let program = compile(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        program,
        Program {
            main_chunk: Chunk {
                name: Some(Path::new("main").unwrap()),
                r#type: FunctionType::new([], [], Type::Boolean),
                instructions: vec![Instruction::r#return(
                    true,
                    Address::encoded(false as u16),
                    OperandType::BOOLEAN
                )],
                prototype_index: u16::MAX,
                ..Default::default()
            },
            cell_count: 0,
            prototypes: Vec::new(),
        }
    );
    assert_eq!(return_value, Some(Value::Boolean(false)));
}

#[test]
fn byte() {
    let source = "0xFF";
    let program = compile(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        program,
        Program {
            main_chunk: Chunk {
                name: Some(Path::new("main").unwrap()),
                r#type: FunctionType::new([], [], Type::Byte),
                instructions: vec![Instruction::r#return(
                    true,
                    Address::encoded(255),
                    OperandType::BYTE
                )],
                prototype_index: u16::MAX,
                ..Default::default()
            },
            cell_count: 0,
            prototypes: Vec::new(),
        }
    );
    assert_eq!(return_value, Some(Value::Byte(255)));
}

#[test]
fn character() {
    let source = "'a'";
    let program = compile(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        program,
        Program {
            main_chunk: Chunk {
                name: Some(Path::new("main").unwrap()),
                r#type: FunctionType::new([], [], Type::Character),
                instructions: vec![Instruction::r#return(
                    true,
                    Address::constant(0),
                    OperandType::CHARACTER
                )],
                constants: vec![Value::Character('a')],
                prototype_index: u16::MAX,
                ..Default::default()
            },
            cell_count: 0,
            prototypes: Vec::new(),
        }
    );
    assert_eq!(return_value, Some(Value::Character('a')));
}

#[test]
fn float() {
    let source = "42.0";
    let program = compile(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        program,
        Program {
            main_chunk: Chunk {
                name: Some(Path::new("main").unwrap()),
                r#type: FunctionType::new([], [], Type::Float),
                instructions: vec![Instruction::r#return(
                    true,
                    Address::constant(0),
                    OperandType::FLOAT
                )],
                constants: vec![Value::Float(42.0)],
                prototype_index: u16::MAX,
                ..Default::default()
            },
            cell_count: 0,
            prototypes: Vec::new(),
        }
    );
    assert_eq!(return_value, Some(Value::Float(42.0)));
}

#[test]
fn integer() {
    let source = "42";
    let program = compile(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        program,
        Program {
            main_chunk: Chunk {
                name: Some(Path::new("main").unwrap()),
                r#type: FunctionType::new([], [], Type::Integer),
                instructions: vec![Instruction::r#return(
                    true,
                    Address::constant(0),
                    OperandType::INTEGER
                )],
                constants: vec![Value::Integer(42)],
                prototype_index: u16::MAX,
                ..Default::default()
            },
            cell_count: 0,
            prototypes: Vec::new(),
        }
    );
    assert_eq!(return_value, Some(Value::Integer(42)));
}

#[test]
fn string() {
    let source = "\"foobar\"";
    let program = compile(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        program,
        Program {
            main_chunk: Chunk {
                name: Some(Path::new("main").unwrap()),
                r#type: FunctionType::new([], [], Type::String),
                instructions: vec![Instruction::r#return(
                    true,
                    Address::constant(0),
                    OperandType::STRING
                )],
                constants: vec![Value::String("foobar".to_string())],
                prototype_index: u16::MAX,
                ..Default::default()
            },
            cell_count: 0,
            prototypes: Vec::new(),
        }
    );
    assert_eq!(return_value, Some(Value::String("foobar".to_string())));
}

#[test]
fn boolean_list() {
    let source = "[true, false]";
    let program = compile(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        program,
        Program {
            main_chunk: Chunk {
                name: Some(Path::new("main").unwrap()),
                r#type: FunctionType::new([], [], Type::List(Box::new(Type::Boolean))),
                instructions: vec![
                    Instruction::new_list(Address::register(0), 2, OperandType::LIST_BOOLEAN),
                    Instruction::set_list(
                        Address::register(0),
                        Address::encoded(1),
                        0,
                        OperandType::BOOLEAN
                    ),
                    Instruction::set_list(
                        Address::register(0),
                        Address::encoded(0),
                        1,
                        OperandType::BOOLEAN
                    ),
                    Instruction::r#return(true, Address::register(0), OperandType::LIST_BOOLEAN)
                ],
                register_count: 1,
                prototype_index: u16::MAX,
                ..Default::default()
            },
            cell_count: 0,
            prototypes: Vec::new(),
        }
    );
    assert_eq!(
        return_value,
        Some(Value::List(List::boolean([true, false])))
    );
}

#[test]
fn byte_list() {
    let source = "[0x00, 0xFF]";
    let program = compile(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        program,
        Program {
            main_chunk: Chunk {
                name: Some(Path::new("main").unwrap()),
                r#type: FunctionType::new([], [], Type::List(Box::new(Type::Byte))),
                instructions: vec![
                    Instruction::new_list(Address::register(0), 2, OperandType::LIST_BYTE),
                    Instruction::set_list(
                        Address::register(0),
                        Address::encoded(0),
                        0,
                        OperandType::BYTE
                    ),
                    Instruction::set_list(
                        Address::register(0),
                        Address::encoded(255),
                        1,
                        OperandType::BYTE
                    ),
                    Instruction::r#return(true, Address::register(0), OperandType::LIST_BYTE)
                ],
                register_count: 1,
                prototype_index: u16::MAX,
                ..Default::default()
            },
            cell_count: 0,
            prototypes: Vec::new(),
        }
    );
    assert_eq!(return_value, Some(Value::List(List::byte([0, 255]))));
}

#[test]
fn character_list() {
    let source = "['a', 'b', 'c']";
    let program = compile(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        program,
        Program {
            main_chunk: Chunk {
                name: Some(Path::new("main").unwrap()),
                r#type: FunctionType::new([], [], Type::List(Box::new(Type::Character))),
                instructions: vec![
                    Instruction::new_list(Address::register(0), 3, OperandType::LIST_CHARACTER),
                    Instruction::set_list(
                        Address::register(0),
                        Address::constant(0),
                        0,
                        OperandType::CHARACTER
                    ),
                    Instruction::set_list(
                        Address::register(0),
                        Address::constant(1),
                        1,
                        OperandType::CHARACTER
                    ),
                    Instruction::set_list(
                        Address::register(0),
                        Address::constant(2),
                        2,
                        OperandType::CHARACTER
                    ),
                    Instruction::r#return(true, Address::register(0), OperandType::LIST_CHARACTER)
                ],
                constants: vec![
                    Value::Character('a'),
                    Value::Character('b'),
                    Value::Character('c')
                ],
                register_count: 1,
                prototype_index: u16::MAX,
                ..Default::default()
            },
            cell_count: 0,
            prototypes: Vec::new(),
        }
    );
    assert_eq!(
        return_value,
        Some(Value::List(List::character(['a', 'b', 'c'])))
    );
}

#[test]
fn float_list() {
    let source = "[1.0, 2.0, 3.0]";
    let program = compile(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        program,
        Program {
            main_chunk: Chunk {
                name: Some(Path::new("main").unwrap()),
                r#type: FunctionType::new([], [], Type::List(Box::new(Type::Float))),
                instructions: vec![
                    Instruction::new_list(Address::register(0), 3, OperandType::LIST_FLOAT),
                    Instruction::set_list(
                        Address::register(0),
                        Address::constant(0),
                        0,
                        OperandType::FLOAT
                    ),
                    Instruction::set_list(
                        Address::register(0),
                        Address::constant(1),
                        1,
                        OperandType::FLOAT
                    ),
                    Instruction::set_list(
                        Address::register(0),
                        Address::constant(2),
                        2,
                        OperandType::FLOAT
                    ),
                    Instruction::r#return(true, Address::register(0), OperandType::LIST_FLOAT)
                ],
                constants: vec![Value::Float(1.0), Value::Float(2.0), Value::Float(3.0)],
                register_count: 1,
                prototype_index: u16::MAX,
                ..Default::default()
            },
            cell_count: 0,
            prototypes: Vec::new(),
        }
    );
    assert_eq!(
        return_value,
        Some(Value::List(List::float([1.0, 2.0, 3.0])))
    );
}

#[test]
fn integer_list() {
    let source = "[1, 2, 3]";
    let program = compile(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        program,
        Program {
            main_chunk: Chunk {
                name: Some(Path::new("main").unwrap()),
                r#type: FunctionType::new([], [], Type::List(Box::new(Type::Integer))),
                instructions: vec![
                    Instruction::new_list(Address::register(0), 3, OperandType::LIST_INTEGER),
                    Instruction::set_list(
                        Address::register(0),
                        Address::constant(0),
                        0,
                        OperandType::INTEGER
                    ),
                    Instruction::set_list(
                        Address::register(0),
                        Address::constant(1),
                        1,
                        OperandType::INTEGER
                    ),
                    Instruction::set_list(
                        Address::register(0),
                        Address::constant(2),
                        2,
                        OperandType::INTEGER
                    ),
                    Instruction::r#return(true, Address::register(0), OperandType::LIST_INTEGER)
                ],
                constants: vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)],
                register_count: 1,
                prototype_index: u16::MAX,
                ..Default::default()
            },
            cell_count: 0,
            prototypes: Vec::new(),
        }
    );
    assert_eq!(return_value, Some(Value::List(List::integer([1, 2, 3]))));
}

// #[test]
// fn string_list() {
//     let source = "[\"foo\", \"bar\"]";
//     let program = compile(source).unwrap();
//     let return_value = run(source).unwrap();

//     assert_eq!(
//         program,
//         Program {
//             main_chunk: Chunk {
//                 name: Some(Path::new("main").unwrap()),
//                 r#type: FunctionType::new([], [], Type::List(Box::new(Type::String))),
//                 instructions: vec![
//                     Instruction::new_list(Address::register(0), 2, OperandType::STRING),
//                     Instruction::set_list(
//                         Address::register(0),
//                         Address::constant(0),
//                         0,
//                         OperandType::STRING
//                     ),
//                     Instruction::set_list(
//                         Address::register(0),
//                         Address::constant(1),
//                         1,
//                         OperandType::STRING
//                     ),
//                     Instruction::r#return(true, Address::register(0), OperandType::LIST_STRING)
//                 ],
//                 constants: vec![
//                     Value::String("foo".to_string()),
//                     Value::String("bar".to_string())
//                 ],
//                 prototype_index: u16::MAX,
//                 ..Default::default()
//             },
//             cell_count: 0,
//             prototypes: Vec::new(),
//         }
//     );
//     assert_eq!(
//         return_value,
//         Some(Value::List(List::string(vec![
//             "foo".to_string(),
//             "bar".to_string()
//         ])))
//     );
// }

// #[test]
// fn list_list() {
//     let source = "[[true], [false]]";
//     let program = compile(source).unwrap();
//     let return_value = run(source).unwrap();

//     assert_eq!(
//         program,
//         Program {
//             main_chunk: Chunk {
//                 name: Some(Path::new("main").unwrap()),
//                 r#type: FunctionType::new(
//                     [],
//                     [],
//                     Type::List(Box::new(Type::List(Box::new(Type::Boolean))))
//                 ),
//                 instructions: vec![
//                     Instruction::new_list(Address::register(0), 2, OperandType::LIST_BOOLEAN),
//                     Instruction::new_list(Address::register(1), 1, OperandType::BOOLEAN),
//                     Instruction::set_list(
//                         Address::register(1),
//                         Address::encoded(0),
//                         0,
//                         OperandType::BOOLEAN
//                     ),
//                     Instruction::set_list(
//                         Address::register(0),
//                         Address::register(1),
//                         0,
//                         OperandType::LIST_BOOLEAN
//                     ),
//                     Instruction::new_list(Address::register(1), 1, OperandType::BOOLEAN),
//                     Instruction::set_list(
//                         Address::register(1),
//                         Address::encoded(1),
//                         0,
//                         OperandType::BOOLEAN
//                     ),
//                     Instruction::set_list(
//                         Address::register(0),
//                         Address::register(1),
//                         1,
//                         OperandType::LIST_BOOLEAN
//                     ),
//                     Instruction::r#return(true, Address::register(0), OperandType::LIST_LIST)
//                 ],
//                 prototype_index: u16::MAX,
//                 ..Default::default()
//             },
//             cell_count: 0,
//             prototypes: Vec::new(),
//         }
//     );
//     assert_eq!(
//         return_value,
//         Some(Value::List(List::list(vec![
//             List::boolean([true]),
//             List::boolean([false])
//         ])))
//     );
// }
