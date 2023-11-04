use std::fs::read_to_string;

use dust_lang::*;

#[test]
fn clue_solver() {
    let file_contents = read_to_string("examples/clue_solver.ds").unwrap();

    evaluate(&file_contents).unwrap();
}

#[test]
#[ignore]
fn download_async() {
    let file_contents = read_to_string("examples/download_async.ds").unwrap();

    evaluate(&file_contents).unwrap();
}

#[test]
#[ignore]
fn fetch() {
    let file_contents = read_to_string("examples/fetch.ds").unwrap();

    evaluate(&file_contents).unwrap();
}

#[test]
fn fibonacci() {
    let file_contents = read_to_string("examples/fibonacci.ds").unwrap();

    evaluate(&file_contents).unwrap();
}

#[test]
fn find_loop() {
    let file_contents = read_to_string("examples/find_loop.ds").unwrap();

    evaluate(&file_contents).unwrap();
}

#[test]
fn fizzbuzz() {
    let file_contents = read_to_string("examples/fizzbuzz.ds").unwrap();

    evaluate(&file_contents).unwrap();
}

#[test]
fn for_loop() {
    let file_contents = read_to_string("examples/for_loop.ds").unwrap();

    evaluate(&file_contents).unwrap();
}

#[test]
fn hello_world() {
    let file_contents = read_to_string("examples/hello_world.ds").unwrap();

    evaluate(&file_contents).unwrap();
}

#[test]
fn remove_loop() {
    let file_contents = read_to_string("examples/remove_loop.ds").unwrap();

    evaluate(&file_contents).unwrap();
}

#[test]
fn select() {
    let file_contents = read_to_string("examples/select.ds").unwrap();

    evaluate(&file_contents).unwrap();
}

#[test]
fn table() {
    let file_contents = read_to_string("examples/table.ds").unwrap();

    evaluate(&file_contents).unwrap();
}

#[test]
fn transform_loop() {
    let file_contents = read_to_string("examples/transform_loop.ds").unwrap();

    evaluate(&file_contents).unwrap();
}

#[test]
fn variables() {
    let file_contents = read_to_string("examples/variables.ds").unwrap();

    evaluate(&file_contents).unwrap();
}

#[test]
fn while_loop() {
    let file_contents = read_to_string("examples/while_loop.ds").unwrap();

    evaluate(&file_contents).unwrap();
}
