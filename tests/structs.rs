use dust_lang::*;

#[test]
fn simple_struct() {
    let result = interpret(
        "
        struct Foo {
            bar <int> = 0
            baz <str>
        }

        new Foo {
            baz = 'hiya'
        }
        ",
    );

    let mut map = Map::new();

    map.set(Identifier::new("bar"), Value::Integer(0));
    map.set(Identifier::new("baz"), Value::String("hiya".to_string()));

    let expected = Ok(Value::Struct(StructInstance::new(
        Identifier::new("Foo"),
        map,
    )));

    assert_eq!(result, expected);
}

#[test]
fn nested_struct() {
    let result = interpret(
        "
        struct Foo {
            bar <Bar>
        }
        struct Bar {}

        new Foo {
            bar = new Bar {}
        }
        ",
    );
    let mut foo_map = Map::new();

    foo_map.set(
        Identifier::new("bar"),
        Value::Struct(StructInstance::new(Identifier::new("Bar"), Map::new())),
    );

    let expected = Ok(Value::Struct(StructInstance::new(
        Identifier::new("Foo"),
        foo_map,
    )));

    assert_eq!(result, expected)
}
