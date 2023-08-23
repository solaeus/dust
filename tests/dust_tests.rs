use std::fs::read_to_string;

use whale_lib::*;

#[test]
fn collections() {
    let file_contents = read_to_string("tests/collections.ds").unwrap();

    eval(&file_contents).unwrap();
}

#[test]
fn list() {
    let file_contents = read_to_string("tests/list.ds").unwrap();

    eval(&file_contents).unwrap();
}

#[test]
fn table() {
    let file_contents = read_to_string("tests/table.ds").unwrap();

    eval(&file_contents).unwrap();
}

#[test]
fn variables() {
    let file_contents = read_to_string("tests/variables.ds").unwrap();

    eval(&file_contents).unwrap();
}

#[test]
fn scope() {
    let file_contents = read_to_string("tests/scope.ds").unwrap();

    eval(&file_contents).unwrap();
}
