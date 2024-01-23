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
