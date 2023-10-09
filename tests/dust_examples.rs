use std::fs::read_to_string;

use dust::*;

#[test]
fn collections() {
    let file_contents = read_to_string("examples/collections.ds").unwrap();

    evaluate(&file_contents).unwrap();
}

#[test]
fn list() {
    let file_contents = read_to_string("examples/list.ds").unwrap();

    evaluate(&file_contents).unwrap();
}

#[test]
fn table() {
    let file_contents = read_to_string("examples/table.ds").unwrap();

    evaluate(&file_contents).unwrap();
}

#[test]
fn variables() {
    let file_contents = read_to_string("examples/variables.ds").unwrap();

    evaluate(&file_contents).unwrap();
}

#[test]
fn scope() {
    let file_contents = read_to_string("examples/scope.ds").unwrap();

    evaluate(&file_contents).unwrap();
}

#[test]
fn data_formats() {
    let file_contents = read_to_string("examples/data_formats.ds").unwrap();

    evaluate(&file_contents).unwrap();
}
