# Dust

[![Build Status](https://github.com/solaeus/dust/actions/workflows/rust.yml/badge.svg)](https://github.com/solaeus/dust/actions)

**Programming language focused on correctness, performance and ease of use.**

```rust
fn fib(n: u32) -> u32 {
    if n == 0 {
        0
    } else if n <= 2 {
        1
    } else {
        fib(n - 1) + fib(n - 2)
    }
}

fn main() -> u32 {
    fib(10)
}
```

> [!IMPORTANT]
> 🧪 💡 ⚗️
>
> Dust is still experimental.

Development is active and, while many aspects of the implementation are stable, research is ongoing
into design optimizations and performance improvements.

## Features

- [X] Basic values and types:
  - [X] Booleans: `bool`
  - [X] Integers: `i8`, `i16`, `i32`, `i64`, `isize`, `u8`, `u16`, `u32`, `u64` and `usize`
  - [X] Floats: `f32` and `f64`
  - [X] Unicode scalars: `char`
  - [X] Tuples: `(bool, i32, f64)`
  - [X] Arrays: `[i32; 4]`
  - [_] Vectors: `Vec<i32>`
  - [_] Slices: `[i32]`
  - [_] Strings: `String`
  - [_] String slices: `str`
- [X] `struct` types
  - Unit structs
  - Tuple structs
  - Field structs
- [X] `enum` types
  - Unit variants
  - Tuple variants
  - Field variants
- [X] `impl` blocks
- [X] `trait` definitions and implementations
- [X] Generics
- [_] Trait constraints

## Design

### Ease-of-use

Your mental energy should be invested in your program's data structures and algorithms, not in
learning new syntax.

If you know Rust, you already know Dust. If you know another C-family language, you already know
most of Dust. Because Dust is an interpreted language that runs in a virtual machine, some Rust
concepts like references and lifetimes do not exist in Dust. In other words, **Dust's syntax is like
Rust but without the hard parts**.

Dust is designed with the philosophy that "advanced" syntax is an anti-feature. Dynamically-typed
languages have to introduce new syntax to circumvent the limitations of runtime type resolution and
the lack of abstract data types. Dust doesn't have those limitations. Features like custom iterators
and operator overloading are exposed through the type system using the same familiar syntax.

### Correctness

The value proposition of Dust's type system is that the increase in code quantity is minimal or
negligible while the increase in code quality is significant. Dust has no null type or undefined
values. It has algebraic types (tuples, structs and enums), abstract types (traits), generics, trait
bounds and Hindley-Milner type inference.

Type systems like Hindley-Milner are a mathematical approach to guaranteeing a program's behavior by
explicitly declaring the input and output types and ensuring that all possible code paths are
compliant through type "unification". In practice, that means that the compiler knows more about
your code and the VM knows less. In some cases it requires extra syntax but the result is better
errors from the compiler, better performance at runtime, functions that behave as expected and
programs that validate their input and are guaranteed to produce their intended output.

Because Dust runs in a VM, it is able to go beyond the Rust type system that inspired it. Effects, a
type-adjacent concept not found in Rust, define which side effects a function is allowed to perform.
For example, the `Fs` effect indicates that a function can access the filesystem, so a function with
the `!Fs` effect bound will fail to compile if the function touches anything on-disk. Effects allow
the user to make guarantees about individual functions or entire Dust programs.

Rust values, including functions, can be passed to the VM as program inputs. Rust code could
theoretically do anything, so Rust functions passed as input values automatically have effects like
`Fs`, `StdIn` and `StdOut` to prevent circumventing effect bounds. Arbitrary Rust code also carries
the risk of undefined behavior via `unsafe`. For this reason, Rust values can only be passed as
inputs to a program. It is impossible to declare a Rust function as a Dust value in a library and
break the effect system for downstream users. It is also impossible to inject unwanted inputs into a
program because they must be explicitly declared on the `main` function.

### Performance

Performance is a key consideration for the VM and every phase of the compilation pipeline.

As an interpreted language, Dust must compile quickly. Rapid development should be a natural
consequence of choosing Dust for your project. Even a large project should compile and begin to run
so fast that a human is unable to notice the compilation time. Realistically, it won't feel
instantaneous to the user unless the time to execution is under ~100 milliseconds. Despite its
robust type system and trait solving, Dust currently exceeds this metric by a huge margin, even on
large code samples that intentionally stress the compiler. This is likely due to a novel compiler
architecture that uses cache-efficient data structures instead of intermediate representations.

Unlike traditional stack-based VMs, Dust uses a register-based approach that has solid research
backing[^3] and has achieved real-world success in Lua. Register-based implementations are rare and
Lua is incredibly simple, so Lua's instruction format[^2] was a major inspiration for Dust's.
However, Dust's design has one major deviation: its instruction format uses 64 bits. Python's uses a
16-bit instruction format while Lua and many others use a 32-bit format.

The benefits of wider instructions are numerous. Lua's 32-bit instructions can only operate on
registers, a special instruction must load a constant into a register before any other instruction
can operate on it. But Dust instructions have room to encode whether each operand is a register or a
constant, so any operand in any instruction can be a constant. In fact, because there are 16 bits
available for each operand index, constant values that fit within 16 bits can be encoded directly
into the instruction. If the compiler needs to communicate `let foo: i32 = bar + 42;` to the VM, it
can emit a single instruction with the ADD operation that will contain the register indices for
`foo` and `bar` as well as the `42` value. 2-bit fields tell the VM whether each index represents a
register, constant or encoded value. The `i32` type is encoded in a 4-bit field so the VM knows how
to interpret the operands. There is no need to use a separate instruction to load the constant, the
constants table is only used for strings or scalars with more than 16 significant bits. This is a
simple example but there are many optimizations in the instruction set that are only possible due to
the 64-bit width.

As far as the author is aware, no other language uses a 64-bit packed instruction format. For 64-bit
platforms this is a natural choice but even on 32-bit platforms the width is justified; the
decreased cache performance is mitigated by the fact that fewer instructions are needed to perform
the same operation. In other words, the increase in instruction size and complexity of decoding more
fields is more than offset by the decrease in instruction count.

## Inspiration

_Crafting Interpreters_[^0] by Bob Nystrom was a great resource for writing the compiler, especially
the Pratt parser. The book is a great introduction to writing interpreters. Had it been discovered
sooner, some early implementations of Dust would have been both simpler in design and more ambitious
in scope.

_Writing a Compiler in Go_[^1] by Thorsten Ball is filled with code examples and helps the reader
make the turn from evaluating a syntax tree to thinking about how problems are solved on physical
hardware and how that informs the design of a virtual machine.

_The Implementation of Lua 5.0_[^2] by Roberto Ierusalimschy, Luiz Henrique de Figueiredo, and
Waldemar Celes was a great resource for understanding register-based virtual machines and their
instructions. This paper was recommended by Bob Nystrom in _Crafting Interpreters_.

_A No-Frills Introduction to Lua 5.1 VM Instructions_[^3] by Kein-Hong Man has a wealth of detailed
information on how Lua uses terse instructions to create dense prototypes that execute quickly. This
was essential in the design of Dust's instructions. Dust uses compile-time optimizations that are
based on Lua optimizations covered in this paper.

"A Performance Survey on Stack-based and Register-based Virtual Machines"[^4] by Ruijie Fang and
Siqi Liu was a useful analysis with informative results that also functions as a primer on getting
stack-based and register-based virtual machines up and running. The included code examples show how
to implement both types of VMs in C. Some of the benchmarks described in the paper inspired similar
benchmarks used in this project to compare Dust to other languages and inform design decisions.

## Contributing

This project's goal is to deliver a delightful new language by combining a thoughtful selection of
features, novel design concepts and a high-quality implementation. In order to innovate, it is
necessary to have both a deep understanding of the algorithms at work and a close familiarity with
the code itself. That can only be gained by actually writing it. This project has found success in
using LLM tools to write and maintain tests based on human-written examples. Beyond test generation
and edit predictions, using LLM tools would hinder innovation. This is currently a solo project but
any future contributors would be expected to respect that reasoning and make their edits personally.
It is not a goal of the project to churn out lots of features right away. Good languages are built
on a solid foundation and carefully maintained by the humans who know them best.

## License

Dust is licensed under the GNU General Public License v3.0. See the `LICENSE` file for details.

[^0]: [Crafting Interpreters](https://craftinginterpreters.com/)

[^1]: [The Implementation of Lua 5.0](https://www.lua.org/doc/jucs05.pdf)

[^2]: [A No-Frills Introduction to Lua 5.1 VM Instructions](https://www.mcours.net/cours/pdf/hasclic3/hasssclic818.pdf)

[^3]: [A Performance Survey on Stack-based and Register-based Virtual Machines](https://arxiv.org/abs/1611.00467)

[^4]: [Writing a Compiler in Go](https://compilerbook.com/)
