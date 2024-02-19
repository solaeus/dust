# Dust

High-level programming language with effortless concurrency, automatic memory management, type safety and strict error handling.

![Dust version of an example from The Rust Programming Language.](https://git.jeffa.io/jeff/dust/raw/branch/main/docs/assets/example_0.png)

<!--toc:start-->
- [Dust](#dust)
  - [Easy to Read and Write](#easy-to-read-and-write)
  - [Effortless Concurrency](#effortless-concurrency)
  - [Helpful Errors](#helpful-errors)
  - [Debugging](#debugging)
  - [Automatic Memory Management](#automatic-memory-management)
  - [Installation and Usage](#installation-and-usage)
<!--toc:end-->

## Easy to Read and Write

Dust has simple, easy-to-learn syntax.

```js
output('Hello world!')
```

## Effortless Concurrency

Write multi-threaded code as easily as you would write code for a single thread.

```js
async {
    output('Will this one print first?')
    output('Or will this one?')
    output('Who knows! Each "output" will run in its own thread!')
}
```

## Helpful Errors

Dust shows you exactly where your code went wrong and suggests changes.

![Example of syntax error output.](https://git.jeffa.io/jeff/dust/raw/branch/main/docs/assets/syntax_error.png)

## Static analysis

Your code is always validated for safety before it is run.

![Example of type error output.](https://git.jeffa.io/jeff/dust/raw/branch/main/docs/assets/type_error.png)

Dust

## Debugging

Just set the environment variable `DUST_LOG=info` and Dust will tell you exactly what your code is doing while it's doing it. If you set `DUST_LOG=trace`, it will output detailed logs about parsing, abstraction, validation, memory management and runtime. Here are some of the logs from the end of a simple [fizzbuzz example](https://git.jeffa.io/jeff/dust/src/branch/main/examples/fizzbuzz.ds).

![Example of debug output.](https://git.jeffa.io/jeff/dust/raw/branch/main/docs/assets/debugging.png)

## Automatic Memory Management

Thanks to static analysis, Dust knows exactly how many times each variable is used. This allows Dust to free memory as soon as the variable will no longer be used, without any help from the user.

## Error Handling

Runtime errors are no problem with Dust. The `Result` type represents the output of an operation that might fail. The user must decide what to do in the case of an error.

```dust
match io:stdin() {
    Result::Ok(input) -> output("We read this input: " + input)
    Result::Error(message) -> output("We got this error: " + message)
}
```

## Installation and Usage

There are two ways to compile Dust. **It is best to clone the repository and compile the latest code**, otherwise the program may be a different version than the one shown on GitHub. Either way, you must have `rustup`, `cmake` and a C compiler installed.

To install from the git repository:

```fish
git clone https://git.jeffa.io/jeff/dust
cd dust
cargo build --release
```

To install with cargo:

```fish
cargo install dust-lang
```
