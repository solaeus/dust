use dust_lang::*;

#[test]
fn allow_access_to_parent_scope() {
    let source = r#"
        let x = 1;
        {
            x
        }
    "#;

    assert_eq!(run(source), Ok(Some(Value::integer(1))));
}

#[test]
fn block_scope() {
    let source = "
        let a = 0;
        {
            let b = 42;
            {
                let c = 1;
            }
            let d = 2;
        }
        let e = 1;
    ";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(17, 18)),
                (Instruction::define_local(0, 0, false), Span(13, 14)),
                (Instruction::load_constant(1, 2, false), Span(50, 52)),
                (Instruction::define_local(1, 1, false), Span(46, 47)),
                (Instruction::load_constant(2, 4, false), Span(92, 93)),
                (Instruction::define_local(2, 2, false), Span(88, 89)),
                (Instruction::load_constant(3, 6, false), Span(129, 130)),
                (Instruction::define_local(3, 3, false), Span(125, 126)),
                (Instruction::load_constant(4, 4, false), Span(158, 159)),
                (Instruction::define_local(4, 4, false), Span(154, 155)),
                (Instruction::r#return(false), Span(165, 165))
            ],
            vec![
                Value::integer(0),
                Value::string("a"),
                Value::integer(42),
                Value::string("b"),
                Value::integer(1),
                Value::string("c"),
                Value::integer(2),
                Value::string("d"),
                Value::string("e"),
            ],
            vec![
                Local::new(1, Type::Integer, false, Scope::new(0, 0)),
                Local::new(3, Type::Integer, false, Scope::new(1, 1)),
                Local::new(5, Type::Integer, false, Scope::new(2, 2)),
                Local::new(7, Type::Integer, false, Scope::new(1, 1)),
                Local::new(8, Type::Integer, false, Scope::new(0, 0)),
            ]
        )),
    );

    assert_eq!(run(source), Ok(None));
}

#[test]
fn multiple_block_scopes() {
    let source = "
        let a = 0;
        {
            let b = 42;
            {
                let c = 1;
            }
            let d = 2;
        }
        let q = 42;
        {
            let b = 42;
            {
                let c = 1;
            }
            let d = 2;
        }
        let e = 1;
    ";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(17, 18)),
                (Instruction::define_local(0, 0, false), Span(13, 14)),
                (Instruction::load_constant(1, 2, false), Span(50, 52)),
                (Instruction::define_local(1, 1, false), Span(46, 47)),
                (Instruction::load_constant(2, 4, false), Span(92, 93)),
                (Instruction::define_local(2, 2, false), Span(88, 89)),
                (Instruction::load_constant(3, 6, false), Span(129, 130)),
                (Instruction::define_local(3, 3, false), Span(125, 126)),
                (Instruction::load_constant(4, 2, false), Span(158, 160)),
                (Instruction::define_local(4, 4, false), Span(154, 155)),
                (Instruction::load_constant(5, 2, false), Span(192, 194)),
                (Instruction::define_local(5, 5, false), Span(188, 189)),
                (Instruction::load_constant(6, 4, false), Span(234, 235)),
                (Instruction::define_local(6, 6, false), Span(230, 231)),
                (Instruction::load_constant(7, 6, false), Span(271, 272)),
                (Instruction::define_local(7, 7, false), Span(267, 268)),
                (Instruction::load_constant(8, 4, false), Span(300, 301)),
                (Instruction::define_local(8, 8, false), Span(296, 297)),
                (Instruction::r#return(false), Span(307, 307))
            ],
            vec![
                Value::integer(0),
                Value::string("a"),
                Value::integer(42),
                Value::string("b"),
                Value::integer(1),
                Value::string("c"),
                Value::integer(2),
                Value::string("d"),
                Value::string("q"),
                Value::string("e"),
            ],
            vec![
                Local::new(1, Type::Integer, false, Scope::new(0, 0)),
                Local::new(3, Type::Integer, false, Scope::new(1, 1)),
                Local::new(5, Type::Integer, false, Scope::new(2, 2)),
                Local::new(7, Type::Integer, false, Scope::new(1, 1)),
                Local::new(8, Type::Integer, false, Scope::new(0, 0)),
                Local::new(3, Type::Integer, false, Scope::new(1, 3)),
                Local::new(5, Type::Integer, false, Scope::new(2, 4)),
                Local::new(7, Type::Integer, false, Scope::new(1, 3)),
                Local::new(9, Type::Integer, false, Scope::new(0, 0)),
            ]
        )),
    );

    assert_eq!(run(source), Ok(None));
}

#[test]
fn disallow_access_to_child_scope() {
    let source = r#"
        {
            let x = 1;
        }
        x
    "#;

    assert_eq!(
        run(source),
        Err(DustError::Compile {
            error: CompileError::VariableOutOfScope {
                identifier: "x".to_string(),
                position: Span(52, 53),
                variable_scope: Scope::new(1, 1),
                access_scope: Scope::new(0, 0),
            },
            source
        })
    );
}

#[test]
fn disallow_access_to_child_scope_nested() {
    let source = r#"
        {
            {
                let x = 1;
            }
            x
        }
    "#;

    assert_eq!(
        run(source),
        Err(DustError::Compile {
            error: CompileError::VariableOutOfScope {
                identifier: "x".to_string(),
                position: Span(78, 79),
                variable_scope: Scope::new(2, 2),
                access_scope: Scope::new(1, 1),
            },
            source
        })
    );
}

#[test]
fn disallow_access_to_sibling_scope() {
    let source = r#"
        {
            let x = 1;
        }
        {
            x
        }
    "#;

    assert_eq!(
        run(source),
        Err(DustError::Compile {
            error: CompileError::VariableOutOfScope {
                identifier: "x".to_string(),
                variable_scope: Scope::new(1, 1),
                access_scope: Scope::new(1, 2),
                position: Span(66, 67),
            },
            source
        })
    );
}

#[test]
fn disallow_access_to_sibling_scope_nested() {
    let source = r#"
        {
            {
                let x = 1;
            }
            {
                x
            }
        }
    "#;

    assert_eq!(
        run(source),
        Err(DustError::Compile {
            error: CompileError::VariableOutOfScope {
                identifier: "x".to_string(),
                variable_scope: Scope::new(2, 2),
                access_scope: Scope::new(2, 3),
                position: Span(96, 97),
            },
            source
        })
    );
}
