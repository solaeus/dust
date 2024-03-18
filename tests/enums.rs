use dust_lang::{
    abstract_tree::Type,
    error::{Error, TypeConflict, ValidationError},
    *,
};

#[test]
fn simple_enum_type_check() {
    assert_eq!(
        interpret(
            "
            enum FooBar {
                Foo(int),
                Bar,    
            }

            foo = FooBar::Foo('yo')
            foo
        ",
        ),
        Err(vec![Error::Validation {
            error: ValidationError::TypeCheck {
                conflict: TypeConflict {
                    actual: Type::String,
                    expected: Type::Integer,
                },
                actual_position: (123, 127).into(),
                expected_position: (47, 50).into()
            },
            position: (105, 141).into()
        }])
    )
}

#[test]
fn simple_enum() {
    interpret(
        "
        enum FooBar {
            Foo(int),
            Bar,    
        }

        foo = FooBar::Foo(1)
        foo
        ",
    )
    .unwrap();
}

#[test]
fn simple_enum_with_type_argument() {
    interpret(
        "
        enum FooBar(F) {
            Foo(F),
            Bar,    
        }

        foo = FooBar(int)::Foo(1)
        foo
        ",
    )
    .unwrap();
}

#[test]
fn complex_enum_with_type_arguments() {
    interpret(
        "
        enum FooBar(F, B) {
            Foo(F),
            Bar(B),    
        }

        bar = FooBar(int, str)::Bar('bar')
        bar
        ",
    )
    .unwrap();
}
