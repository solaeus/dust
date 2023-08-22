use std::fs::read_to_string;

use whale_lib::*;

#[test]
fn collections() {
    let file_contents = read_to_string("tests/collections.ds").unwrap();

    eval(&file_contents).unwrap();
}
