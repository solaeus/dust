use std::collections::BTreeMap;

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

    map.set("bar".to_string(), Value::Integer(0));
    map.set("baz".to_string(), Value::String("hiya".to_string()));

    let expected = Ok(Value::Struct(StructInstance::new("Foo".to_string(), map)));

    assert_eq!(result, expected);
}

#[test]
fn nested_struct() {
    let _result = interpret(
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

    let mut map = BTreeMap::new();

    map.insert("x".to_string(), (Some(Value::Integer(0)), Type::Integer));

    // let expected = Value::Map(Map::from_structure(Structure::new(map)));

    // assert_eq!(Ok(expected), result);

    todo!()
}
