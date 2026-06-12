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

## Introduction

Programmers have been conditioned to believe that, while most languages deliver only one of the
following, the really great ones deliver two:

- a fast compiler
- a fast runtime
- correctness guarantees

There are projects that have taken existing languages and added a new item from this list. JIT
compilers have been written for Java, Python and Lua[^1] to speed them up at runtime. TypeScript is
a less incorrect addendum to JavaScript. Correctness guarantees are harder to come by and languages
that embrace correctness (Haskell, ML, Rust) come with a steep learning curve and are implemented as
slow compilers that produce fast native executables.

Dust's goal is to overcome that pattern, delivering a language that empowers and delights its users
by focusing on the three factors that no language should ignore, but many do.

### Ease-of-use

If you know Rust, you already know Dust. If you know another C-family language, you already know
most of Dust. *There are no lifetimes or borrow checker in Dust.* Heap-allocated values are passed
by reference and garbage-collected. Statically-sized values can be passed by value or by reference.

Unlike dynamically typed languages that ignore many details of your program until runtime, Dust's
statically typed compiler can see every mechanical error in your code and provide helpful error
messages. By assuming this cognitive load, the compiler allows you to focus on logic and code
quality.

### Correctness

Dust has a static, nominal, compile-time type system, including:

- Null safety
- Abstract data types and ad-hoc polymorphism
- Generic associated types
- Zero-sized types
- Dynamically-sized types
- Hindley-Milner inference
- Projected types

No gradual typing. No duck typing. Dust uses a from-scratch reimplementation of the Rust type system
that was largely influenced by parts of the `rustc` code, but built with the same cache-friendly
structure as the rest of the compiler.

### Performance

Thanks to a novel compiler architecture based on cache-friendly data structures, Dust can offer
robust type solving at previously-unseen speeds. In addition to its bytecode VM that can compile to
WebAssembly or be embedded in any Rust application, Dust includes a bytecode-to-native compiler
based on cranelift[^1], meaning it can also compile to a native executable.

TODO

## Inspiration

_Crafting Interpreters_[^2] by Bob Nystrom was a great resource for writing the compiler, especially
the Pratt parser. The book is a great introduction to writing interpreters. Had it been discovered
sooner, some early implementations of Dust would have been both simpler in design and more ambitious
in scope.

_Writing a Compiler in Go_[^3] by Thorsten Ball is filled with code examples and helps the reader
make the turn from evaluating a syntax tree to thinking about how problems are solved on physical
hardware and how that informs the design of a virtual machine.

_The Implementation of Lua 5.0_[^4] by Roberto Ierusalimschy, Luiz Henrique de Figueiredo, and
Waldemar Celes was a great resource for understanding register-based virtual machines and their
instructions. This paper was recommended by Bob Nystrom in _Crafting Interpreters_.

_A No-Frills Introduction to Lua 5.1 VM Instructions_[^5] by Kein-Hong Man has a wealth of detailed
information on how Lua uses terse instructions to create dense prototypes that execute quickly. This
was essential in the design of Dust's instructions. Dust uses compile-time optimizations that are
based on Lua optimizations covered in this paper.

"A Performance Survey on Stack-based and Register-based Virtual Machines"[^6] by Ruijie Fang and
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

[^0]: Lua deserves an honorable mention for having a very fast compiler and runtime even without JIT
compilation. Lua uses a simple C implementation, highly optimizable bytecode and a register-based
VM. LuaJIT takes it to another level, but Lua succeeded on its own through solid research and earned
a reputation as the performant embeddable language and for handling logic in video games (in some
cases, both). See [^2] and [^3] below.

[^1]: [cranelift](https://cranelift.dev)

[^2]: [Crafting Interpreters](https://craftinginterpreters.com/)

[^3]: [The Implementation of Lua 5.0](https://www.lua.org/doc/jucs05.pdf)

[^4]: [A No-Frills Introduction to Lua 5.1 VM Instructions](https://www.mcours.net/cours/pdf/hasclic3/hasssclic818.pdf)

[^5]: [A Performance Survey on Stack-based and Register-based Virtual Machines](https://arxiv.org/abs/1611.00467)

[^6]: [Writing a Compiler in Go](https://compilerbook.com/)
