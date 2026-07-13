#![allow(clippy::disallowed_methods)]

use super::*;

fn create_test_table(add_constant: fn(&mut ConstantsBuilder) -> ConstantId) -> (Constants, u16) {
    let mut table = ConstantsBuilder::new();

    table.add_character('q');
    table.add_u32(666);
    table.add_i32(-666);
    table.add_u64(666);
    table.add_i64(666);
    table.add_u128(666);
    table.add_i128(666);
    table.add_f32(666.0);
    table.add_f64(666.0);
    table.add_string("666");

    let id = add_constant(&mut table);

    (table.build().0, id.0)
}

#[test]
fn interns() {
    let mut table = ConstantsBuilder::new();

    let first_id = table.add_character('a');
    let second_id = table.add_character('a');

    assert_eq!(first_id, second_id);

    let first_id = table.add_u32(42);
    let second_id = table.add_u32(42);

    assert_eq!(first_id, second_id);

    let first_id = table.add_i32(-42);
    let second_id = table.add_i32(-42);

    assert_eq!(first_id, second_id);

    let first_id = table.add_u64(42);
    let second_id = table.add_u64(42);

    assert_eq!(first_id, second_id);

    let first_id = table.add_i64(42);
    let second_id = table.add_i64(42);

    assert_eq!(first_id, second_id);

    let first_id = table.add_u128(42);
    let second_id = table.add_u128(42);

    assert_eq!(first_id, second_id);

    let first_id = table.add_i128(42);
    let second_id = table.add_i128(42);

    assert_eq!(first_id, second_id);

    let first_id = table.add_string("foobar");
    let first_pool_length = table.string_pool.len();
    let second_id = table.add_string("foobar");
    let second_pool_length = table.string_pool.len();

    assert_eq!(first_id, second_id);
    assert_eq!(first_pool_length, second_pool_length);
}

#[test]
fn character() {
    let (table, index) = create_test_table(|table| table.add_character('q'));
    let retrieved = table.get_character(index).unwrap();

    assert_eq!(retrieved, 'q');
}

#[test]
fn u32() {
    let (table, index) = create_test_table(|table| table.add_u32(666));
    let retrieved = table.get_u32(index).unwrap();

    assert_eq!(retrieved, 666);
}

#[test]
fn i32() {
    let (table, index) = create_test_table(|table| table.add_i32(-666));
    let retrieved = table.get_i32(index).unwrap();

    assert_eq!(retrieved, -666);
}

#[test]
fn u64() {
    let (table, index) = create_test_table(|table| table.add_u64(666));
    let retrieved = table.get_u64(index).unwrap();

    assert_eq!(retrieved, 666);
}

#[test]
fn i64() {
    let (table, index) = create_test_table(|table| table.add_i64(666));
    let retrieved = table.get_i64(index).unwrap();

    assert_eq!(retrieved, 666);
}

#[test]
fn u128() {
    let (table, index) = create_test_table(|table| table.add_u128(666));
    let retrieved = table.get_u128(index).unwrap();

    assert_eq!(retrieved, 666);
}

#[test]
fn i128() {
    let (table, index) = create_test_table(|table| table.add_i128(666));
    let retrieved = table.get_i128(index).unwrap();

    assert_eq!(retrieved, 666);
}

#[test]
fn f32() {
    let (table, index) = create_test_table(|table| table.add_f32(666.0));
    let retrieved = table.get_f32(index).unwrap();

    assert_eq!(retrieved, 666.0);
}

#[test]
fn f64() {
    let (table, index) = create_test_table(|table| table.add_f64(666.0));
    let retrieved = table.get_f64(index).unwrap();

    assert_eq!(retrieved, 666.0);
}

#[test]
fn string() {
    let (table, index) = create_test_table(|table| table.add_string("666"));
    let retrieved = table.get_string(index).unwrap();

    assert_eq!(retrieved, "666");
}
