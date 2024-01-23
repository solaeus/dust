use std::fs::read_to_string;

use dust_lang::*;

#[test]
fn r#async() {
    let file_contents = read_to_string("examples/async.ds").unwrap();

    interpret(&file_contents).unwrap();
}

#[test]
#[ignore]
fn async_download() {
    let file_contents = read_to_string("examples/async_download.ds").unwrap();

    interpret(&file_contents).unwrap();
}

#[test]
#[ignore]
fn clue_solver() {
    let file_contents = read_to_string("examples/clue_solver.ds").unwrap();

    interpret(&file_contents).unwrap();
}

#[test]
#[ignore]
fn fetch() {
    let file_contents = read_to_string("examples/fetch.ds").unwrap();

    interpret(&file_contents).unwrap();
}

#[test]
#[ignore]
fn fibonacci() {
    let file_contents = read_to_string("examples/fibonacci.ds").unwrap();

    interpret(&file_contents).unwrap();
}

#[test]
fn fizzbuzz() {
    let file_contents = read_to_string("examples/fizzbuzz.ds").unwrap();

    interpret(&file_contents).unwrap();
}

#[test]
fn for_loop() {
    let file_contents = read_to_string("examples/for_loop.ds").unwrap();

    interpret(&file_contents).unwrap();
}

#[test]
fn hello_world() {
    let file_contents = read_to_string("examples/hello_world.ds").unwrap();

    interpret(&file_contents).unwrap();
}

#[test]
fn jq_data() {
    let file_contents = read_to_string("examples/jq_data.ds").unwrap();

    interpret(&file_contents).unwrap();
}

#[test]
fn list() {
    let file_contents = read_to_string("examples/list.ds").unwrap();

    interpret(&file_contents).unwrap();
}

#[test]
fn map() {
    let file_contents = read_to_string("examples/map.ds").unwrap();

    interpret(&file_contents).unwrap();
}

#[test]
fn r#match() {
    let file_contents = read_to_string("examples/match.ds").unwrap();

    interpret(&file_contents).unwrap();
}

#[test]
fn random() {
    let file_contents = read_to_string("examples/random.ds").unwrap();

    interpret(&file_contents).unwrap();
}

#[test]
fn sea_creatures() {
    let file_contents = read_to_string("examples/sea_creatures.ds").unwrap();

    interpret(&file_contents).unwrap();
}

#[test]
fn variables() {
    let file_contents = read_to_string("examples/variables.ds").unwrap();

    interpret(&file_contents).unwrap();
}

#[test]
fn while_loop() {
    let file_contents = read_to_string("examples/while_loop.ds").unwrap();

    interpret(&file_contents).unwrap();
}

#[test]
fn r#yield() {
    let file_contents = read_to_string("examples/yield.ds").unwrap();

    interpret(&file_contents).unwrap();
}
