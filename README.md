# Dust

!!! Dust is an experimental project under active development. !!!

Dust is a general purpose programming language that emphasises concurrency and correctness.

A basic dust program:

```dust
output("Hello world!")
```

Dust can do two (or more) things at the same time with effortless concurrency:

```dust
async {
    output('will this one finish first?')
    output('or will this one?')
}
```

You can use Dust to run complex operations simply and safely, you can even invoke other programs, run them at the same time, capture their output, and pipe them together.

```dust
# Run each statment in this block in its own thread.
async { 
    # Invoke another program and capture its output.
    ip_info = ^ip address;

    # Pipe the output to another program.
    ^ls -1 --all --long docs/ | ^rg .md | ^echo;

    # This block is not async and the statements will be run in order.
    {
        file = fs:read_file('Cargo.toml')

        for line in str:lines(file) {
            if str:contains(line, 'author') {
                output(line)
                break
            }
        }
    }
}
```

Dust is an interpreted, strictly typed language with first class functions. It emphasises concurrency by allowing any group of statements to be executed in parallel. Dust includes built-in tooling to import and export data in a variety of formats, including JSON, TOML, YAML and CSV.

<!--toc:start-->
- [Dust](#dust)
  - [Features](#features)
  - [Usage](#usage)
  - [Dust Language](#dust-language)
  - [Installation](#installation)
  - [Benchmarks](#benchmarks)
  - [Implementation](#implementation)
  - [Acknowledgements](#acknowledgements)
<!--toc:end-->

## Features

- Simplicity: Dust is designed to be easy to learn.
- Speed: Dust is built on [Tree Sitter] and [Rust] to prioritize performance and correctness. See [Benchmarks] below.
- Concurrency: Safe, effortless parallel code using thread pools.
- Safety: Written in safe, stable Rust.
- Correctness: Type checking makes it easy to write good code.

## Installation

You must have the default rust toolchain installed and up-to-date. Install [rustup] if it is not already installed. Run `cargo install dust-lang` then run `dust` to start the interactive shell.

To build from source, clone the repository and build the parser. To do so, enter the `tree-sitter-dust` directory and run `tree-sitter-generate`. In the project root, run `cargo run` to start the shell. To see other command line options, use `cargo run -- --help`.

## Usage

After installation, the command line interpreter can be given source code to run or it can launch the command-line shell.

```sh
cargo install dust-lang
dust -c "output('Hello world!')"
# Output: Hello world!
```

Run `dust --help` to see the available commands and options.

```txt
General purpose programming language

Usage: dust [OPTIONS] [PATH]

Arguments:
  [PATH]  Location of the file to run

Options:
  -c, --command <COMMAND>        Dust source code to evaluate
  -i, --input <INPUT>            Data to assign to the "input" variable
  -p, --input-path <INPUT_PATH>  A path to file whose contents will be assigned to the "input" variable
  -t, --tree                     Show the syntax tree
  -h, --help                     Print help
  -V, --version                  Print version    
```

## Dust Language

See the [Language Reference](/docs/language.md) for more information.

## Benchmarks

Dust is at an early development stage and these tests are overly simple. Better benchmarks are needed to get a realistic idea of how Dust performs real work. For now, these tests are just for fun.
The examples given were tested using [Hyperfine] on a single-core cloud instance with 1024 MB RAM. Each test was run 1000 times. The test script is shown below. Each test asks the program to read a JSON file and count the objects. Dust is a command line shell, programming language and data manipulation tool so three appropriate targets were chosen for comparison: nushell, NodeJS and jq. The programs produced identical output with the exception that NodeJS printed in color.

For the first test, a file with four entries was used.

| Command | Mean [ms] | Min [ms] | Max [ms] 
|:---|---:|---:|---:|
| Dust | 3.1 ± 0.5 | 2.4 | 8.4 |
| jq | 33.7 ± 2.2 | 30.0 | 61.8 |
| NodeJS | 226.4 ± 13.1 | 197.6 | 346.2 |
| Nushell | 51.6 ± 3.7 | 45.4 | 104.3 |

The second set of data is from the GitHub API, it consists of 100 commits from the jq GitHub repo.

| Command | Mean [ms] | Min [ms] | Max [ms] |
|:---|---:|---:|---:|
| Dust | 6.8 ± 0.6 | 5.7 | 12.0 | 2.20 ± 0.40 |
| jq | 43.3 ± 3.6 | 37.6 | 81.6 | 13.95 ± 2.49 |
| NodeJS | 224.9 ± 12.3 | 194.8 | 298.5 |
| Nushell | 59.2 ± 5.7 | 49.7 | 125.0 | 19.11 ± 3.55 |

This data came from CERN, it is a massive file of 100,000 entries.

| Command | Mean [ms] | Min [ms] | Max [ms] |
|:---|---:|---:|---:|
| Dust | 1080.8 ± 38.7 | 975.3 | 1326.6 |
| jq | 1305.3 ± 64.3 | 1159.7 | 1925.1 |
| NodeJS | 1850.5 ± 72.5 | 1641.9 | 2395.1 |
| Nushell | 1850.5 ± 86.2 | 1625.5 | 2400.7 |

The tests were run after 5 warmup runs and the cache was cleared before each run.

```sh
hyperfine \
	--shell none \
	--warmup 5 \
	--prepare "rm -rf /root/.cache" \
	--runs 1000 \
	--parameter-list data_path seaCreatures.json,jq_data.json,dielectron.json \
	--export-markdown test_output.md \
	"dust -c '(length (from_json input))' -p {data_path}" \
	"jq 'length' {data_path}" \
	"node --eval \"require('node:fs').readFile('{data_path}',(err,data)=>{console.log(JSON.parse(data).length)})\"" \
	"nu -c 'open {data_path} | length'"
```

## Implementation

Dust is formally defined as a Tree Sitter grammar in the tree-sitter-dust directory. Tree sitter generates a parser, written in C, from a set of rules defined in JavaScript. Dust itself is a rust binary that calls the C parser using FFI.

Tests are written in three places: in the Rust library, in Dust as examples and in the Tree Sitter test format. Generally, features are added by implementing and testing the syntax in the tree-sitter-dust repository, then writing library tests to evaluate the new syntax. Implementation tests run the Dust files in the "examples" directory and should be used to demonstrate and verify that features work together.

Tree Sitter generates a concrete syntax tree, which Dust traverses to create an abstract syntax tree that can run the Dust code. The CST generation is an extra step but it allows easy testing of the parser, defining the language in one file and makes the syntax easy to modify and expand. Because it uses Tree Sitter, developer-friendly features like syntax highlighting and code navigation are already available in any text editor that supports Tree Sitter.

## Acknowledgements

Dust began as a fork of [evalexpr]. Some of the original code is still in place but the project has dramatically changed and no longer uses any of its parsing or interpreting.

[Tree Sitter]: https://tree-sitter.github.io/tree-sitter/
[Rust]: https://rust-lang.org
[evalexpr]: https://github.com/ISibboI/evalexpr
[rustup]: https://rustup.rs
[Hyperfine]: https://github.com/sharkdp/hyperfine
