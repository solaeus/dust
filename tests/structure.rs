use std::collections::BTreeMap;

use dust_lang::*;

#[test]
fn simple_structure() {
    let result = interpret("struct { x <int> = 0 }");

    let mut btree_map = BTreeMap::new();

    btree_map.insert("x".to_string(), (Some(Value::Integer(0)), Type::Integer));

    let expected = Ok(Value::TypeDefinition(TypeDefintion::Structure(
        Structure::new(btree_map),
    )));

    assert_eq!(expected, result);
}

#[test]
fn new_structure() {
    let result = interpret(
        "
        Coords = struct {
            x <float> = 0.0
            x <float> = 0.0
        }

        new Coords {
            x = 1.5
            y = 4.2
        }
        ",
    );

    let mut map = BTreeMap::new();

    map.insert("x".to_string(), (Some(Value::Integer(0)), Type::Integer));

    let expected = Value::Map(Map::from_structure(Structure::new(map)));

    assert_eq!(Ok(expected), result);
}
