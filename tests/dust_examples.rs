use std::fs::read_to_string;

use dust_lang::*;

#[test]
fn r#async() {
    let file_contents = read_to_string("examples/async.ds").unwrap();

    evaluate(&file_contents).unwrap();
}

#[test]
#[ignore]
fn async_download() {
    let file_contents = read_to_string("examples/async_download.ds").unwrap();

    evaluate(&file_contents).unwrap();
}

#[test]
fn clue_solver() {
    let file_contents = read_to_string("examples/clue_solver.ds").unwrap();

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
fn jq_data() {
    let file_contents = read_to_string("examples/jq_data.ds").unwrap();

    evaluate(&file_contents).unwrap();
}

#[test]
fn list() {
    let file_contents = read_to_string("examples/list.ds").unwrap();

    evaluate(&file_contents).unwrap();
}

#[test]
fn map() {
    let file_contents = read_to_string("examples/map.ds").unwrap();

    evaluate(&file_contents).unwrap();
}

#[test]
fn random() {
    let file_contents = read_to_string("examples/random.ds").unwrap();

    evaluate(&file_contents).unwrap();
}

#[test]
fn sea_creatures() {
    let file_contents = read_to_string("examples/sea_creatures.ds").unwrap();

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
fn variables() {
    let file_contents = read_to_string("examples/variables.ds").unwrap();

    evaluate(&file_contents).unwrap();
}

#[test]
fn while_loop() {
    let file_contents = read_to_string("examples/while_loop.ds").unwrap();

    evaluate(&file_contents).unwrap();
}

#[test]
fn r#yield() {
    let file_contents = read_to_string("examples/yield.ds").unwrap();

    evaluate(&file_contents).unwrap();
}
