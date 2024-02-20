use dust_lang::*;

#[test]
fn simple_struct() {
    let result = interpret(
        "
        struct Foo {
            bar <int> = 0
            baz <str>
        }

        Foo::{
            baz = 'hiya'
        }
        ",
    );

    let mut map = Map::new();

    map.set(Identifier::new("bar"), Value::Integer(0));
    map.set(Identifier::new("baz"), Value::string("hiya"));

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

        Foo::{
            bar = Bar::{}
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
