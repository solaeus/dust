# Dust

[![Build Status](https://github.com/solaeus/dust/actions/workflows/rust.yml/badge.svg)](https://github.com/solaeus/dust/actions)

**Programming language focused on correctness, performance and ease of use.**

Dust is an ongoing research project into a language implementation that enforces static typing, has
no null or undefined values and emits helpful errors that guide users to correct syntax. Compiling
to 64-bit encoded bytecode in a single pass before JIT compilation enables powerful runtime
optimizations and fast startup times. Dust is designed to combine the best features of
register-based virtual machines, JIT compilation and static typing to deliver a language that never
compromises on correctness or speed while remaining delightfully easy to read and write.

An interactive "Hello, world" using Dust's built-in I/O functions:

```rust
write_line("Enter your name...");

let name = read_line();

write_line("Hello " + name + "!");
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

Development is active and, while many aspects of the implementation are stable, research is ongoing
into design optimizations and performance improvements.

## Overview

This project's goal is to deliver a language with features that stand out due to a combination of
design choices and a high-quality implementation, providing **correctness**, **performance** and
**ease-of-use**.

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

The Dust CLI has commands to run, disassemble or tokenize Dust code. It can also provide logging at
different levels and measure the time taken for compilation and execution.

If not specified, the CLI will use the `run` command. This mode compiles and executes the Dust
program, printing the return value to the console. You can also run Dust code directly from the
command line using the `--eval` or `-e` flag.

```sh
dust foobar.ds
dust -e 'let x = 42; x'
```

## Benchmarks

> [!IMPORTANT]
> Anything shown here is a preliminary benchmark to guage the performance of Dust as it is being developed.

The following benchmarks were run on a machine with the following specifications:

|              |                                       |
| ------------ | ------------------------------------- |
| CPU          | AMD Ryzen 9 7900X3D 12-Core Processor |
| Memory       | 32 GB                                 |
| Rust Version | 1.91.0-nightly                        |

The languages used in the benchmarks were chosen because they are invoked in a single command, i.e.
they are "interpreted" languages that run directly from source code, rather than being compiled to
an executable file. See the `bench/addictive_addition` and `bench/addictive_calling` directories for
the code used.

**Fibonacci** computes the 25th Fibonacci number using a naive recursive implementation.

**Addictive Addition** increments a counter from 0 to 10,000,000 using a loop and an operator.

**Addictive Calling** performs the same logic as "Addictive Addition" but it increments by calling a
function rather than using an operator directly.

### Fibonacci

| Rank  | Language | Mean Time           | Relative to Dust |
| ----- | -------- | ------------------- | ---------------- |
| 1     | LuaJIT   | 1.3 ms ± 0.1 ms     | 2.53× faster     |
| **2** | **Dust** | **3.2 ms ± 0.3 ms** | **baseline**     |
| 3     | Lua      | 3.9 ms ± 0.2 ms     | 1.22× slower     |
| 4     | Bun      | 8.4 ms ± 0.4 ms     | 2.63× slower     |
| 5     | Python   | 14.5 ms ± 0.5 ms    | 4.53× slower     |
| 6     | PHP      | 16.7 ms ± 0.7 ms    | 5.22× slower     |
| 7     | PyPy     | 25.2 ms ± 1.2 ms    | 7.88× slower     |
| 8     | Perl     | 27.5 ms ± 0.8 ms    | 8.59× slower     |
| 9     | Ruby     | 46.6 ms ± 1.4 ms    | 14.56× slower    |
| 10    | Node.js  | 48.0 ms ± 1.9 ms    | 15.00× slower    |
| 11    | Deno     | 77.4 ms ± 1.8 ms    | 24.19× slower    |
| 12    | Julia    | 100.9 ms ± 2.7 ms   | 31.53× slower    |
| 13    | R        | 139.0 ms ± 3.9 ms   | 43.44× slower    |
| 14    | Java     | 198.2 ms ± 2.7 ms   | 61.94× slower    |
| 15    | Clojure  | 1234 ms ± 12 ms     | 385.63× slower   |

### Addictive Addition

| Rank  | Language | Mean Time            | Relative to Dust |
| ----- | -------- | -------------------- | ---------------- |
| 1     | LuaJIT   | 8.7 ms ± 0.2 ms      | 1.18× faster     |
| 2     | Bun      | 10.0 ms ± 0.4 ms     | 1.03× faster     |
| **3** | **Dust** | **10.3 ms ± 0.3 ms** | **baseline**     |
| 4     | PyPy     | 20.4 ms ± 0.8 ms     | 1.98× slower     |
| 5     | PHP      | 41.8 ms ± 1.4 ms     | 4.06× slower     |
| 6     | Lua      | 43.6 ms ± 0.8 ms     | 4.23× slower     |
| 7     | Node.js  | 50.4 ms ± 1.8 ms     | 4.89× slower     |
| 8     | Deno     | 80.0 ms ± 1.7 ms     | 7.77× slower     |
| 9     | Julia    | 99.8 ms ± 2.0 ms     | 9.69× slower     |
| 10    | Ruby     | 124.2 ms ± 10.9 ms   | 12.06× slower    |
| 11    | Perl     | 192.3 ms ± 7.7 ms    | 18.67× slower    |
| 12    | Java     | 199.1 ms ± 2.8 ms    | 19.33× slower    |
| 13    | R        | 252.2 ms ± 13.4 ms   | 24.49× slower    |
| 14    | Python   | 399.8 ms ± 36.1 ms   | 38.82× slower    |
| 15    | Clojure  | 1223 ms ± 12 ms      | 118.74× slower   |

### Addictive Calling

| Rank  | Language | Mean Time            | Relative to Dust |
| ----- | -------- | -------------------- | ---------------- |
| 1     | LuaJIT   | 8.7 ms ± 0.2 ms      | 1.85× faster     |
| 2     | Bun      | 11.7 ms ± 0.9 ms     | 1.38× faster     |
| **3** | **Dust** | **16.1 ms ± 0.8 ms** | **baseline**     |
| 4     | PyPy     | 20.7 ms ± 1.1 ms     | 1.29× slower     |
| 5     | Node.js  | 50.0 ms ± 1.7 ms     | 3.11× slower     |
| 6     | Deno     | 80.1 ms ± 1.8 ms     | 4.98× slower     |
| 7     | PHP      | 95.4 ms ± 0.9 ms     | 5.93× slower     |
| 8     | Julia    | 102.4 ms ± 3.1 ms    | 6.36× slower     |
| 9     | Lua      | 145.4 ms ± 3.9 ms    | 9.03× slower     |
| 10    | Java     | 200.3 ms ± 4.8 ms    | 12.44× slower    |
| 11    | Ruby     | 245.1 ms ± 2.7 ms    | 15.22× slower    |
| 12    | Python   | 584.4 ms ± 42.2 ms   | 36.30× slower    |
| 13    | Perl     | 645.5 ms ± 8.0 ms    | 40.09× slower    |
| 14    | Clojure  | 1300 ms ± 6 ms       | 80.75× slower    |
| 15    | R        | 1431 ms ± 14 ms      | 88.88× slower    |

The results of this benchmark show that Dust is performing very well in simple arithmetic
operations. Languages like LuaJIT and Bun are clearly using function inlining due to the nearly
identical times for both benchmarks. Dust does not yet perform function inlining, hence the slower
time for "Addictive Calling".

As this project matures, more benchmarks will be added to cover a wider range of use cases.

## Inspiration

_Crafting Interpreters_[^0] by Bob Nystrom was a great resource for writing the compiler, especially
the Pratt parser. The book is a great introduction to writing interpreters. Had it been discovered
sooner, some early implementations of Dust would have been both simpler in design and more ambitious
in scope.

_The Implementation of Lua 5.0_[^1] by Roberto Ierusalimschy, Luiz Henrique de Figueiredo, and
Waldemar Celes was a great resource for understanding register-based virtual machines and their
instructions. This paper was recommended by Bob Nystrom in _Crafting Interpreters_.

_A No-Frills Introduction to Lua 5.1 VM Instructions_[^2] by Kein-Hong Man has a wealth of detailed
information on how Lua uses terse instructions to create dense prototypes that execute quickly. This
was essential in the design of Dust's instructions. Dust uses compile-time optimizations that are
based on Lua optimizations covered in this paper.

"A Performance Survey on Stack-based and Register-based Virtual Machines"[^3] by Ruijie Fang and
Siqi Liup was helpful for a quick yet efficient primer on getting stack-based and register-based
virtual machines up and running. The included code examples show how to implement both types of VMs
in C. The performance comparison between the two types of VMs is worth reading for anyone who is
trying to choose between the two. Some of the benchmarks described in the paper inspired similar
benchmarks used in this project to compare Dust to other languages and inform design decisions.

_Writing a Compiler in Go_[^6] by Thorsten Ball is a lot like _Crafting Interpreters_, they are the
where I look for a generalized approach to solving a problem. Filled with code examples, this book
helps the reader make the turn from evaluating a syntax tree to thinking about how problems are
solved on physical hardware and how that informs the design of a virtual machine.

> Let me get straight to the point: a virtual machine is a computer built with software.
> -- Thorsten Ball, _Writing a Compiler in Go_

## License

Dust is licensed under the GNU General Public License v3.0. See the `LICENSE` file for details.

[^0]: [Crafting Interpreters](https://craftinginterpreters.com/)

[^1]: [The Implementation of Lua 5.0](https://www.lua.org/doc/jucs05.pdf)

[^2]: [A No-Frills Introduction to Lua 5.1 VM Instructions](https://www.mcours.net/cours/pdf/hasclic3/hasssclic818.pdf)

[^3]: [A Performance Survey on Stack-based and Register-based Virtual Machines](https://arxiv.org/abs/1611.00467)

[^4]: [List of C-family programming languages](https://en.wikipedia.org/wiki/List_of_C-family_programming_languages)

[^6]: [Writing a Compiler in Go](https://compilerbook.com/)
