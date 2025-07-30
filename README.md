# Dust

[![Build Status](https://github.com/solaeus/dust/actions/workflows/rust.yml/badge.svg)](https://github.com/solaeus/dust/actions)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust Version](https://img.shields.io/badge/rust-1.90.0--nightly-orange?logo=rust)](https://www.rust-lang.org/)

**High-performance programming language focused on correctness, speed, and ease of use.**

Dust enforces static typing, has no null or undefined values, and emits helpful errors that guide users to correct syntax. Compiling to 64-bit encoded bytecode in a single pass before JIT compilation enables powerful runtime optimizations and fast startup times. Dust is designed to combine the best features of register-based virtual machines, JIT compilation, and static typing to deliver a language that never compromises on correctness or speed while remaining delightfully easy to read and write.

```rust
// An interactive "Hello, world" using Dust's built-in I/O functions
write_line("Enter your name...")

let name = read_line()

write_line("Hello " + name + "!")
```

```rust
// The classic, unoptimized Fibonacci sequence
fn fib (n: int) -> int {
    if n <= 0 {
        0
    } else if n == 1 {
        1
    } else {
        fib(n - 1) + fib(n - 2)
    }
}

write_line(fib(25))
```

> [!IMPORTANT]
> 🧪 💡 ⚗️
> Dust is still experimental. Currently, development is more focused on exploring ideas for optimization and performance than on stability or feature completeness.

## Highlights

- Easy to read and write
- Static typing with extensive type inference
- Multi-threaded register-based virtual machine
- JIT compilation for fast execution
- Beautiful, helpful error messages from the compiler
- Safe execution, runtime errors are treated as bugs

## Overview

Dust's syntax, safety features and evaluation model are based on Rust. Its instruction set and compiler optimization strategies are based on Lua. Unlike Rust and other languages that compile ahead-of-time, Dust has a very low time to execution. Unlike Lua and most other interpreted languages, Dust enforces static typing to improve clarity and prevent bugs.

### Project Goals

This project's goal is to deliver a language with features that stand out due to a combination of design choices and a high-quality implementation. As mentioned in the first sentence, Dust's general aspirations are to be **fast**, **safe** and **easy**.

- **Fast** 🚀
  - **Fast Compilation** Despite its compile-time abstractions, Dust should compile and start executing quickly. The compilation time should feel negligible to the user.
  - **Fast Execution** Dust should be competitive with highly optimized, modern, register-based VM languages like Lua. Dust should be bench tested during development to inform decisions about performance.
  - **Low Resource Usage** Memory and CPU power should be used conservatively and predictably.
- **Safe** 🛡️
  - **Static Types** Typing should prevent runtime errors and improve code quality, offering a superior development experience despite some additional constraints.
  - **Null-Free** Dust has no "null" or "undefined" values. All values are initialized and have a type. This eliminates a whole class of bugs that are common in other languages.
  - **Memory Safety** Dust should be free of memory bugs, using both safe Rust and sound, correct "unsafe" Rust to maximize performance. Dust should employ a concurrent mark-and-sweep garbage collecter, allowing other threads to continue executing instructions while the garbage collector looks for freeable memory.
- **Easy** 🎂
  - **Simple Syntax** Dust should be easier to learn than most programming languages. Its syntax should be familiar to users of other C-like languages to the point that even a new user can read Dust code and understand what it does. Rather than being held back by a lack of features, Dust should be powerful and elegant in its simplicity, seeking a maximum of capability with a minimum of complexity.
  - **Practical Tooling** Shipped as a single binary, Dust should provide logging and tools for disassembly and tokenization that make the lexer, compiler and runtime as transparent as possible. Dust should also include an official formatter through the same binary. Additional tools such as a language server and linter should be adopted when possible.
  - **Excellent Errors** Dust should provide helpful error messages that guide the user to the source of the problem and suggest a solution. Errors should be a helpful learning resource for users rather than a source of frustration.

### Author

I'm Jeff 🦀 and I started this project as a simple expession evaluator. Initially, the project used an external parser and a tree-walking interpreter. After several books, a few papers, countless articles and a lot of experimentation, Dust has evolved to an ambitious project that aims to implement lucrative features with a high-quality implementation that competes with established languages.

## Usage

**Dust is under active development and is not yet ready for general use.**
The Dust CLI has commands to run, disassemble or tokenize Dust code. It can also provide logging at different levels and measure the time taken for compilation and execution.

If not specified, the CLI will use the `run` command. This mode compiles and executes the Dust program, printing the return value to the console. You can also run Dust code directly from the command line using the `--eval` or `-e` flag.

```sh
dust foobar.ds
dust -e 'let x = 42; x'
```

## Benchmarks

> [!IMPORTANT]
> Anything shown here is a preliminary benchmark to guage the performance of Dust as it is being developed. Benchmarks at this point in development are not intended to be rigorous.

The following benchmarks were run on a machine with the following specifications:

|               |                                               |
|---------------|-----------------------------------------------|
| CPU           | AMD Ryzen 9 7900X3D 12-Core Processor         |
| Memory        | 32 GB                                         |
| Rust Version  | 1.90.0-nightly (ba7e63b63 2025-07-29)         |

The languages used in the benchmarks were chosen because they are invoked in a single command, i.e. they are "interpreted" languages that run directly from source code, rather than being compiled to an executable file.

### Addictive Addition

See the `bench/addictive_addition` directory for the code used in this benchmark.
This is a simple iterative loop that increments a counter by 1 until it reaches 10,000,000. The benchmark was taken from a similar benchmark used in a paper[^3] on stack-based and register-based virtual machines. This benchmark favors languages that have a fast startup time and efficient execution of simple loops.

| Language | Mean [ms] | Relative |
|----------|-----------|----------|
| dust     |     5.5   |  1.00    |
| luajit   |     8.7   |  1.57    |
| bun      |    12.5   |  2.26    |
| pypy     |    21.2   |  3.83    |
| php      |    41.7   |  7.54    |
| lua      |    43.5   |  7.87    |
| node     |    54.6   |  9.87    |
| deno     |    84.8   |  15.34   |
| julia    |   106.3   |  19.22   |
| ruby     |   119.5   |  21.62   |
| perl     |   192.3   |  34.78   |
| java     |   210.9   |  38.15   |
| Rscript  |   249.8   |  45.17   |
| python   |   439.3   |  79.44   |
| clojure  |   1283    |  232.06  |

## Inspiration

*Crafting Interpreters*[^0] by Bob Nystrom was a great resource for writing the compiler, especially the Pratt parser. The book is a great introduction to writing interpreters. Had it been discovered sooner, some early implementations of Dust would have been both simpler in design and more ambitious in scope.

*The Implementation of Lua 5.0*[^1] by Roberto Ierusalimschy, Luiz Henrique de Figueiredo, and Waldemar Celes was a great resource for understanding register-based virtual machines and their instructions. This paper was recommended by Bob Nystrom in *Crafting Interpreters*.

*A No-Frills Introduction to Lua 5.1 VM Instructions*[^2] by Kein-Hong Man has a wealth of detailed information on how Lua uses terse instructions to create dense chunks that execute quickly. This was essential in the design of Dust's instructions. Dust uses compile-time optimizations that are based on Lua optimizations covered in this paper.

"A Performance Survey on Stack-based and Register-based Virtual Machines"[^3] by Ruijie Fang and Siqi Liup was helpful for a quick yet efficient primer on getting stack-based and register-based virtual machines up and running. The included code examples show how to implement both types of VMs in C. The performance comparison between the two types of VMs is worth reading for anyone who is trying to choose between the two. Some of the benchmarks described in the paper inspired similar benchmarks used in this project to compare Dust to other languages and inform design decisions.

*Writing a Compiler in Go*[^6] by Thorsten Ball is a lot like *Crafting Interpreters*, they are the where I look for a generalized approach to solving a problem. Filled with code examples, this book helps the reader make the turn from evaluating a syntax tree to thinking about how problems are solved on physical hardware and how that informs the design of a virtual machine.

> Let me get straight to the point: a virtual machine is a computer built with software.
> -- Thorsten Ball, *Writing a Compiler in Go*

*Structure and Interpretation of Computer Programs, Second Edition*[^7] by Harold Abelson and Gerald Jay Sussman with Julie Sussman is a classic text on computer science. It encourages an abstract view of programming, sometimes using diagrams to explain programs as though they were physical devices. It requires more effort than the books that immediately show you how to write a program, but the takeaway is a deep understanding of the the process a computer (or a VM) goes through to execute a program.

## License

Dust is licensed under the GNU General Public License v3.0. See the `LICENSE` file for details.

[^0]: [Crafting Interpreters](https://craftinginterpreters.com/)
[^1]: [The Implementation of Lua 5.0](https://www.lua.org/doc/jucs05.pdf)
[^2]: [A No-Frills Introduction to Lua 5.1 VM Instructions](https://www.mcours.net/cours/pdf/hasclic3/hasssclic818.pdf)
[^3]: [A Performance Survey on Stack-based and Register-based Virtual Machines](https://arxiv.org/abs/1611.00467)
[^4]: [List of C-family programming languages](https://en.wikipedia.org/wiki/List_of_C-family_programming_languages)
[^5]: [ripgrep is faster than {grep, ag, git grep, ucg, pt, sift}](https://blog.burntsushi.net/ripgrep/#mechanics)
[^6]: [Writing a Compiler in Go](https://compilerbook.com/)
[^7]: [Structure and Interpretation of Computer Programs, Second Edition](https://mitpress.mit.edu/9780262510875/structure-and-interpretation-of-computer-programs/)
