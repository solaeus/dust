use std::fs::read_to_string;

use dust_lang::*;

#[test]
fn r#async() {
    let file_contents = read_to_string("examples/async.ds").unwrap();

    interpret(&file_contents).unwrap();
}

#[test]
fn async_commands() {
    let file_contents = read_to_string("examples/async_commands.ds").unwrap();

    interpret(&file_contents).unwrap();
}

#[test]
#[ignore]
fn async_download() {
    let file_contents = read_to_string("examples/async_download.ds").unwrap();

    interpret(&file_contents).unwrap();
}

#[test]
fn clue_solver() {
    let file_contents = read_to_string("examples/clue_solver.ds").unwrap();

    interpret(&file_contents).unwrap();
}

#[test]
#[ignore]
fn download() {
    let file_contents = read_to_string("examples/download.ds").unwrap();

    interpret(&file_contents).unwrap();
}

#[test]
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
fn random() {
    let file_contents = read_to_string("examples/random.ds").unwrap();

    interpret(&file_contents).unwrap();
}

#[test]
fn sea_creatures() {
    let file_contents = read_to_string("examples/sea_creatures.ds").unwrap();

    interpret(&file_contents).unwrap();
}
