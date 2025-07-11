use crate::{
    Address, DebugChunk, FunctionType, Instruction, OperandType, Path, Type, Value, compile, run,
};

#[test]
fn true_or_true_and_true() {
    let source = "true || true && true";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::encoded(1),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::test(Address::register(0), false),
                Instruction::jump(2, true),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(1),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::test(Address::register(0), true),
                Instruction::jump(1, true),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(1),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::r#return(true, Address::register(0), OperandType::BOOLEAN),
            ],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Boolean(true)));
}

// Repeat for all combinations:
#[test]
fn true_or_true_and_false() {
    let source = "true || true && false";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();
    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::encoded(1),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::test(Address::register(0), false),
                Instruction::jump(2, true),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(1),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::test(Address::register(0), true),
                Instruction::jump(1, true),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(0),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::r#return(true, Address::register(0), OperandType::BOOLEAN),
            ],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Boolean(true)));
}

#[test]
fn true_or_false_and_true() {
    let source = "true || false && true";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();
    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::encoded(1),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::test(Address::register(0), false),
                Instruction::jump(2, true),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(0),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::test(Address::register(0), true),
                Instruction::jump(1, true),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(1),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::r#return(true, Address::register(0), OperandType::BOOLEAN),
            ],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Boolean(true)));
}

#[test]
fn true_or_false_and_false() {
    let source = "true || false && false";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();
    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::encoded(1),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::test(Address::register(0), false),
                Instruction::jump(2, true),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(0),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::test(Address::register(0), true),
                Instruction::jump(1, true),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(0),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::r#return(true, Address::register(0), OperandType::BOOLEAN),
            ],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Boolean(true)));
}

#[test]
fn false_or_true_and_true() {
    let source = "false || true && true";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();
    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::encoded(0),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::test(Address::register(0), false),
                Instruction::jump(2, true),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(1),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::test(Address::register(0), true),
                Instruction::jump(1, true),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(1),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::r#return(true, Address::register(0), OperandType::BOOLEAN),
            ],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Boolean(true)));
}

#[test]
fn false_or_true_and_false() {
    let source = "false || true && false";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();
    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::encoded(0),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::test(Address::register(0), false),
                Instruction::jump(2, true),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(1),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::test(Address::register(0), true),
                Instruction::jump(1, true),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(0),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::r#return(true, Address::register(0), OperandType::BOOLEAN),
            ],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Boolean(false)));
}

#[test]
fn false_or_false_and_true() {
    let source = "false || false && true";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();
    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::encoded(0),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::test(Address::register(0), false),
                Instruction::jump(2, true),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(0),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::test(Address::register(0), true),
                Instruction::jump(1, true),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(1),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::r#return(true, Address::register(0), OperandType::BOOLEAN),
            ],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Boolean(false)));
}

#[test]
fn false_or_false_and_false() {
    let source = "false || false && false";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();
    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::encoded(0),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::test(Address::register(0), false),
                Instruction::jump(2, true),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(0),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::test(Address::register(0), true),
                Instruction::jump(1, true),
                Instruction::load(
                    Address::register(0),
                    Address::encoded(0),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::r#return(true, Address::register(0), OperandType::BOOLEAN),
            ],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Boolean(false)));
}
