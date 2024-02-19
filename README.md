# Dust

High-level programming language with effortless concurrency, automatic memory management, type safety and strict error handling.

![Dust version of an example from The Rust Programming Language.](https://git.jeffa.io/jeff/dust/docs/assets/example_0.png)

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

![Example of syntax error output.](https://git.jeffa.io/jeff/dust/docs/assets/syntax_error.png)

## Static analysis

Your code is always validated for safety before it is run. Other interpreted languages can fail halfway through, but Dust is able to avoid runtime errors by analyzing the program *before* it is run

![Example of type error output.](https://git.jeffa.io/jeff/dust/docs/assets/type_error.png)

## Debugging

Just set the environment variable `DUST_LOG=info` and Dust will tell you exactly what your code is doing while it's doing it. If you set `DUST_LOG=trace`, it will output detailed logs about parsing, abstraction, validation, memory management and runtime.

![Example of debug output.](https://git.jeffa.io/jeff/dust/docs/assets/debugging.png)

## Automatic Memory Management

## Error Handling

## Installation and Usage
