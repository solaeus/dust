# ✭ Dust Programming Language

**Fast**, **safe** and **easy-to-use** general-purpose programming language.

```rust
// An interactive "Hello, world" using Dust's built-in I/O functions
write_line("Enter your name...")

let name = read_line()

write_line("Hello " + name + "!")
```

```rust
// The classic, unoptimized Fibonacci sequence
fn fib (n: int) -> int {
    if n <= 0 { return 0 }
    if n == 1 { return 1 }

    fib(n - 1) + fib(n - 2)
}

write_line(fib(25))
```

## Highlights

- Easy to read and write
- Single-pass, self-optimizing compiler
- Static typing with extensive type inference
- Multi-threaded register-based virtual machine with concurrent garbage collection
- Beautiful, helpful error messages from the compiler
- Safe execution, runtime errors are treated as bugs

## Overview

Dust's syntax, safety features and evaluation model are based on Rust. Its instruction set
and optimization strategies are based on Lua. Unlike Rust and other languages that compile to
machine code, Dust has a very low time to execution. Unlike Lua and most other interpreted
languages, Dust enforces static typing to improve clarity and prevent bugs.

### Project Goals

This project's goal is to deliver a language with features that stand out due to a combination of
design choices and a high-quality implementation. As mentioned in the first sentence, Dust's general
aspirations are to be **fast**, **safe** and **easy**.

- **Fast**
  - **Fast Compilation** Despite its compile-time abstractions, Dust should compile and start
    executing quickly. The compilation time should feel negligible to the user.
  - **Fast Execution** Dust should be competitive with highly optimized, modern, register-based VM
    languages like Lua. Dust should be bench tested during development to inform decisions about
    performance.
  - **Low Resource Usage** Memory and CPU power should be used conservatively and predictably.
- **Safe**
  - **Static Types** Typing should prevent runtime errors and improve code quality, offering a
    superior development experience despite some additional constraints. Like any good statically
    typed language, users should feel confident in the type-consistency of their code and not want
    to go back to a dynamically typed language.
  - **Null-Free** Dust has no "null" or "undefined" values. All values are initialized and have a
    type. This eliminates a whole class of bugs that are common in other languages.
  - **Memory Safety** Dust should be free of memory bugs. Being implemented in Rust makes this easy
    but, to accommodate long-running programs, Dust still requires a memory management strategy.
    Dust's design is to use a separate thread for garbage collection, allowing other threads to
    continue executing instructions while the garbage collector looks for unused memory.
- **Easy**
  - **Simple Syntax** Dust should be easier to learn than most programming languages. Its syntax
    should be familiar to users of other C-like languages to the point that even a new user can read
    Dust code and understand what it does. Rather than being held back by a lack of features, Dust
    should be powerful and elegant in its simplicity, seeking a maximum of capability with a minimum
    of complexity.
  - **Excellent Errors** Dust should provide helpful error messages that guide the user to the
    source of the problem and suggest a solution. Errors should be a helpful learning resource for
    users rather than a source of frustration.
  - **Relevant Documentation** Users should have the resources they need to learn Dust and write
    code in it. They should know where to look for answers and how to reach out for help.

### Author

I'm Jeff 🦀 and I started this project as simple expession evaluator. Initially, the project used an
external parser and a tree-walking interpreter. After several books, a few papers, countless
articles and a lot of experimentation, Dust has evolved to an ambitious project that aims to
implement lucrative features with a high-quality implementation that competes with established
languages.

## Usage

**Dust is under active development and is not yet ready for general use.**

## Installation

Eventually, Dust should be available via package managers and as an embeddable library. For now,
the only way to use Dust is to clone the repository and build it from source.

## Inspiration

*Crafting Interpreters*[^0] by Bob Nystrom was a great resource for writing the compiler, especially
the Pratt parser. The book is a great introduction to writing interpreters. Had it been discovered
sooner, some early implementations of Dust would have been both simpler in design and more ambitious
in scope.

*The Implementation of Lua 5.0*[^1] by Roberto Ierusalimschy, Luiz Henrique de Figueiredo, and
Waldemar Celes was a great resource for understanding register-based virtual machines and their
instructions. This paper was recommended by Bob Nystrom in [Crafting Interpreters].

*A No-Frills Introduction to Lua 5.1 VM Instructions*[^2] by Kein-Hong Man has a wealth of detailed
information on how Lua uses terse instructions to create dense chunks that execute quickly. This was
essential in the design of Dust's instructions. Dust uses compile-time optimizations that are based
on Lua optimizations covered in this paper.

"A Performance Survey on Stack-based and Register-based Virtual Machines"[^3] by Ruijie Fang and Siqi
Liup was helpful for a quick yet efficient primer on getting stack-based and register-based virtual
machines up and running. The included code examples show how to implement both types of VMs in C.
The performance comparison between the two types of VMs is worth reading for anyone who is trying to
choose between the two. Some of the benchmarks described in the paper inspired similar benchmarks
used in this project to compare Dust to other languages and inform design decisions.

*Writing a Compiler in Go*[^6] by Thorsten Ball is a lot like [Crafting Interpreters][], they are the
where I look for a generalized approach to solving a problem. Filled with code examples, this book
helps the reader make the turn from evaluating a syntax tree to thinking about how problems are
solved on physical hardware and how that informs the design of a virtual machine.

> Let me get straight to the point: a virtual machine is a computer built with software.
> -- Thorsten Ball, *Writing a Compiler in Go*

*Structure and Interpretation of Computer Programs, Second Edition*[^7] by Harold Abelson and Gerald
Jay Sussman with Julie Sussman is a classic text on computer science. It encourages an abstract
view of programming, sometimes using diagrams to explain programs as though they were physical
devices. It requires more effort than the books that immediately show you how to write a program,
but the takeaway is a deep understanding of the the process a computer (or a VM) goes through to
execute a program.

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
