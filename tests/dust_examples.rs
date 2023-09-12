use std::fs::read_to_string;

use dust_lib::*;

#[test]
fn collections() {
    let file_contents = read_to_string("examples/collections.ds").unwrap();

    eval(&file_contents).unwrap();
}

#[test]
fn list() {
    let file_contents = read_to_string("examples/list.ds").unwrap();

    eval(&file_contents).unwrap();
}

#[test]
fn table() {
    let file_contents = read_to_string("examples/table.ds").unwrap();

    eval(&file_contents).unwrap();
}

#[test]
fn variables() {
    let file_contents = read_to_string("examples/variables.ds").unwrap();

    eval(&file_contents).unwrap();
}

#[test]
fn scope() {
    let file_contents = read_to_string("examples/scope.ds").unwrap();

    eval(&file_contents).unwrap();
}

#[test]
fn data_formats() {
    let file_contents = read_to_string("examples/data_formats.ds").unwrap();

    eval(&file_contents).unwrap();
}
