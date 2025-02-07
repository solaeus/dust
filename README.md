# Dust

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
- Static typing with extensive type inference
- Multi-threaded register-based virtual machine
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

- **Fast** 🚀
  - **Fast Compilation** Despite its compile-time abstractions, Dust should compile and start
    executing quickly. The compilation time should feel negligible to the user.
  - **Fast Execution** Dust should be competitive with highly optimized, modern, register-based VM
    languages like Lua. Dust should be bench tested during development to inform decisions about
    performance.
  - **Low Resource Usage** Memory and CPU power should be used conservatively and predictably.
- **Safe** 🛡️
  - **Static Types** Typing should prevent runtime errors and improve code quality, offering a
    superior development experience despite some additional constraints.
  - **Null-Free** Dust has no "null" or "undefined" values. All values are initialized and have a
    type. This eliminates a whole class of bugs that are common in other languages.
  - **Memory Safety** Dust should be free of memory bugs, using both safe Rust and sound, correct
    "unsafe" Rust to maximize performance. Dust should employ a concurrent mark-and-sweep garbage
    collecter, allowing other threads to continue executing instructions while the garbage collector
    looks for freeable memory.
- **Easy** 🎂
  - **Simple Syntax** Dust should be easier to learn than most programming languages. Its syntax
    should be familiar to users of other C-like languages to the point that even a new user can read
    Dust code and understand what it does. Rather than being held back by a lack of features, Dust
    should be powerful and elegant in its simplicity, seeking a maximum of capability with a minimum
    of complexity.
  - **Practical Tooling** Shipped as a single binary, Dust should provide logging and tools for
    disassembly and tokenization that make the lexer, compiler and runtime as transparent as
    possible. Dust should also include an official formatter through the same binary. Additional
    tools such as a language server and linter should be adopted when possible.
  - **Excellent Errors** Dust should provide helpful error messages that guide the user to the
    source of the problem and suggest a solution. Errors should be a helpful learning resource for
    users rather than a source of frustration.

### Author

I'm Jeff 🦀 and I started this project as a simple expession evaluator. Initially, the project used
an external parser and a tree-walking interpreter. After several books, a few papers, countless
articles and a lot of experimentation, Dust has evolved to an ambitious project that aims to
implement lucrative features with a high-quality implementation that competes with established
languages.

## Usage

**Dust is under active development and is not yet ready for general use.**

### CLI

The Dust CLI has commands to run, disassemble or tokenize Dust code. It can also provide logging at
different levels and measure the time taken for compilation and execution.

```text
Usage: dust [OPTIONS] [FILE]
       dust {run|-r} [OPTIONS] [FILE]
       dust {disassemble|-d} [OPTIONS] [FILE]
       dust {tokenize|-t} [OPTIONS] [FILE]
       dust help [COMMAND]...

Modes:
  run, -r          Compile and run the program (default)
  disassemble, -d  Compile and print the bytecode disassembly
  tokenize, -t     Lex the source code and print the tokens
  help             Print this message or the help of the given subcommand(s)

Options:
  -l, --log-level <LOG_LEVEL>  Overrides the DUST_LOG environment variable
      --time                   Print the time taken for compilation and execution
      --no-output              Do not print the program's return value
      --name <NAME>            Custom program name, overrides the file name
  -c, --command <INPUT>        Source code to run instead of a file
      --stdin                  Read source code from stdin
  -h, --help                   Print help
  -V, --version                Print version
```

#### Running a program

If not specified, the CLI will use `run` mode. This mode compiles and executes the Dust program,
printing the return value to the console. You can also run Dust code directly from the command line
using the `--command` or `-c` flag.

```sh
dust foobar.ds
dust -c 'let x = 42; x'
```

#### Disassembly

Dust's disassembly output is a detailed, human-readable representation of the internal
representation of the Dust program. It shows every piece of information that the compiler sends to
the virtual machine and explains what each instruction does and what data it uses.

```sh
dust -d example.ds
```

<details>
    <summary>Show disassembly</summary>

```text
╭──────────────────────────────────────────────────────────────────────────────────╮
│                                    example.ds               <---- file name      │
│                                                                                  │
│                     let mut i = 0; while i < 10 { i += 1 };    <---- source code │
│                                                                                  │
│               6 instructions, 4 constants, 1 locals, returns none  <---- summary │
│                                                                                  │
│                                   Instructions                                   │
│ ╭─────┬────────────┬─────────────────┬─────────────────────────────────────────╮ │
│ │  i  │  POSITION  │    OPERATION    │                  INFO                   │ │
│ ├─────┼────────────┼─────────────────┼─────────────────────────────────────────┤ │
│ │  0  │  (12, 13)  │  LOAD_CONSTANT  │            R_INT_0 = C_INT_0            │ │
│ │  1  │  (23, 24)  │      LESS       │    if R_INT_0 < C_INT_1 { JUMP +1 }     │ │
│ │  2  │  (38, 39)  │      JUMP       │                 JUMP +2                 │ │
│ │  3  │  (32, 34)  │       ADD       │       R_INT_0 = R_INT_0 + C_INT_2       │ │
│ │  4  │  (38, 39)  │      JUMP       │                 JUMP -3                 │ │
│ │  5  │  (39, 39)  │     RETURN      │                 RETURN                  │ │
│ ╰─────┴────────────┴─────────────────┴─────────────────────────────────────────╯ │
│                                      Locals                                      │
│       ╭─────┬────────────────┬────────────────┬──────────┬───────┬───────╮       │
│       │  i  │   identifier   │      type      │ register │ scope │mutable│       │
│       ├─────┼────────────────┼────────────────┼──────────┼───────┼───────┤       │
│       │  0  │       i        │      int       │ R_INT_0  │  0.0  │ true  │       │
│       ╰─────┴────────────────┴────────────────┴──────────┴───────┴───────╯       │
│                                    Constants                                     │
│         ╭────────┬──────────────────────────┬──────────────────────────╮         │
│         │   i    │           TYPE           │          VALUE           │         │
│         ├────────┼──────────────────────────┼──────────────────────────┤         │
│         │ INT_0  │           int            │            0             │         │
│         │ INT_1  │           int            │            10            │         │
│         │ INT_2  │           int            │            1             │         │
│         │ STR_0  │           str            │            i             │         │
│         ╰────────┴──────────────────────────┴──────────────────────────╯         │
╰──────────────────────────────────────────────────────────────────────────────────╯
```

</details>

The instruction notation reflects the Dust VM's register-based architecture. Values are referred to
by their address in the register or constant table. For example, `R_INT_42` refers to the
forty-second integer register, and `C_INT_0` refers to the first integer constant.

```text
R_INT_0 = R_INT_0 + C_INT_2
```

The info section for the ADD instruction shows what the instruction does: it adds the value at
`R_INT_0` to the value at `C_INT_2` and stores the result in `R_INT_0`. As the "Constants" section
shows, `C_INT_2` is the integer constant `1`. This means that this add instruction increments the
value in `R_INT_0` by `1`. In the "Locals" section, we can see that `R_INT_0` is the register used
by the `i` variable.


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
instructions. This paper was recommended by Bob Nystrom in *Crafting Interpreters*.

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

*Writing a Compiler in Go*[^6] by Thorsten Ball is a lot like *Crafting Interpreters*, they are the
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
