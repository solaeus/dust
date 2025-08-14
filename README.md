# Dust

[![Build Status](https://github.com/solaeus/dust/actions/workflows/rust.yml/badge.svg)](https://github.com/solaeus/dust/actions)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust Version](https://img.shields.io/badge/rust-1.91.0--nightly-orange?logo=rust)](https://www.rust-lang.org/)

**High-performance programming language focused on correctness, performance and ease of use.**

Dust enforces static typing, has no null or undefined values and emits helpful errors that guide users to correct syntax. Compiling to 64-bit encoded bytecode in a single pass before JIT compilation enables powerful runtime optimizations and fast startup times. Dust is designed to combine the best features of register-based virtual machines, JIT compilation and static typing to deliver a language that never compromises on correctness or speed while remaining delightfully easy to read and write.

An interactive "Hello, world" using Dust's built-in I/O functions:

```rust
write_line("Enter your name...")

let name = read_line()

write_line("Hello " + name + "!")
```

The classic, unoptimized Fibonacci sequence:

```rust
fn fib (n: int) -> int {
    if n <= 0 {
        0
    } else if n == 1 {
        1
    } else {
        fib(n - 1) + fib(n - 2)
    }
}

fib(25)
```

## Project Status

> [!IMPORTANT]
> 🧪 💡 ⚗️
>
> Dust is still experimental.

Development is active and, while many aspects of the implementation are stable, research is ongoing into optimizations and performance improvements. JIT compilation is the latest feature to be added and is still being refined. Before a 1.0 release, the JIT VM needs to be fully implemented along with a bytecode interpreter for environments where JIT compilation is not possible.

## Overview

This project's goal is to deliver a language with features that stand out due to a combination of design choices and a high-quality implementation, providing **correctness**, **performance** and **ease-of-use**.

- **Correctness**
  - Statically typed with type inference
  - No null or undefined values
  - Potential runtime errors are caught at compile time
- **Performance**
  - JIT compilation with execution times that rival or exceed established languages
  - Fast compilation for negligible startup times
  - Low memory usage, especially at runtime
- **Ease-of-use**
  - Simple syntax that is easy to read and write and resembles other C-family languages
  - Helpful error messages that guide users to correct syntax
  - Batteries included in a standard library that is always available

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
> Anything shown here is a preliminary benchmark to guage the performance of Dust as it is being developed.

The following benchmarks were run on a machine with the following specifications:

|               |                                       |
|---------------|---------------------------------------|
| CPU           | AMD Ryzen 9 7900X3D 12-Core Processor |
| Memory        | 32 GB                                 |
| Rust Version  | 1.91.0-nightly                        |

The languages used in the benchmarks were chosen because they are invoked in a single command, i.e. they are "interpreted" languages that run directly from source code, rather than being compiled to an executable file. See the `bench/addictive_addition` and `bench/addictive_calling` directories for the code used.

**Addictive Addition** increments a counter from 0 to 10,000,000 using a loop and an operator.

**Addictive Calling** performs the same logic as "Addictive Addition" but it increments by calling a function rather than using an operator directly.

|  Runtime  | Addictive Addition (ms) | Addictive Calling (ms) |
|-----------|-------------------------|------------------------|
| **Dust**  | **8.5**                 | **14.4**               |
| LuaJIT    | 8.7                     | 8.7                    |
| Bun       | 12.6                    | 14.4                   |
| PyPy      | 21.4                    | 22.0                   |
| PHP       | 42.8                    | 98.6                   |
| Lua       | 43.2                    | 149.9                  |
| Node      | 55.3                    | 56.2                   |
| Deno      | 87.6                    | 87.9                   |
| Julia     | 103.8                   | 105.1                  |
| Ruby      | 120.6                   | 245.9                  |
| Perl      | 193.8                   | 647.0                  |
| Java      | 210.2                   | 214.0                  |
| R         | 250.6                   | 1533.0                 |
| Python    | 461.2                   | 594.9                  |
| Clojure   | 1266.0                  | 1352.0                 |

The results of this benchmark show that Dust is performing very well in simple arithmetic operations. Languages like LuaJIT and Bun are clearly using function inlining due to the nearly identical times for both benchmarks. Dust does not yet perform function inlining, hence the slower time for "Addictive Calling".

As this project matures, more benchmarks will be added to cover a wider range of use cases.

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
