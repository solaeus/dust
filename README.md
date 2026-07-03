# Dust

[![Build Status](https://github.com/solaeus/dust/actions/workflows/rust.yml/badge.svg)](https://github.com/solaeus/dust/actions)

**Programming language focused on correctness, performance and ease of use.**

> [!IMPORTANT]
> 🧪 💡 ⚗️
>
> Dust is still experimental.

Development is active and, while many aspects of the implementation are stable, research is ongoing
into design optimizations and performance improvements.

## Overview

### Ease-of-use

Dust's type system is based on Rust's, but *there are no lifetimes or borrow checker in Dust.*
Heap-allocated values are passed by reference and garbage-collected. Statically-sized values can be
passed by value or by reference.

Unlike dynamically typed languages that ignore many details of your program until runtime, Dust's
statically typed compiler can see every mechanical error in your code and provide helpful error
messages. 

### Correctness

Dust has a static, nominal type system, including:

- Null safety (i.e. null does not exist)
- Abstract data types with ad-hoc polymorphism
- Generic associated types
- Zero-sized types
- Dynamically-sized types
- Hindley-Milner inference
- Projected types

No gradual typing. No duck typing. Dust uses a from-scratch reimplementation of the Rust type system
that was largely influenced by parts of the `rustc` code, but built specifically for Dust.

### Performance

Thanks to a novel compiler architecture based on cache-friendly data structures, Dust can offer
robust type solving at previously-unseen speeds. TODO

While Dust is a great interpreted language, when it comes to ease of distribution and raw
performance, there is no competing with an ahead-of-time compiler that produces a native executable
file for the target platform. Dust includes a native compiler based on [cranelift][^1] for exactly
that reason. TODO

## Inspiration

[*Crafting Interpreters*][^2] by Bob Nystrom is a great resource for getting started with parsers
and bytecode compilers. The information on Pratt parsering is especially useful.

[*The Implementation of Lua 5.0*][^3] by Roberto Ierusalimschy, Luiz Henrique de Figueiredo, and
Waldemar Celes is a great resource for understanding register-based virtual machines and their
instructions. This paper is recommended by Bob Nystrom in *Crafting Interpreters*.

[*A No-Frills Introduction to Lua 5.1 VM Instructions*][^4] by Kein-Hong Man has detailed
information on the design of Lua's function prototypes and instruction format. Dust's compiler
output is directly based on these. Many of Dust's bytecode instructions mirror Lua's.

["A Performance Survey on Stack-based and Register-based Virtual Machines"][^5] by Ruijie Fang and
Siqi Liu is a study with notable results that also functions as a primer on getting stack-based and
register-based virtual machines up and running. The included code examples show how to implement
both types of VMs in C. Some of the benchmarks described in the paper inspired benchmarks used in
this project.

## Contributing

In order to innovate, it is necessary to have both a deep understanding of the algorithms at work
and a close familiarity with the code itself. That can only be gained by actually writing it. This
project has found success in using LLM tools to write and maintain tests based on human-written
examples. Beyond test generation and edit predictions, using LLM tools would hinder innovation. This
is currently a solo project but any future contributors would be expected to respect that reasoning
and make their edits personally. It is not a goal of the project to churn out lots of features right
away. Good languages are built on a solid foundation and carefully maintained by the humans who know
them best.

## License

Dust is licensed under the GNU General Public License v3.0. See the `LICENSE` file for details.

## Refernces

[^0]: Lua deserves an honorable mention for having a very fast compiler and runtime well before
LuaJIT. Lua uses a simple C implementation, highly optimizable bytecode and a register-based VM.
LuaJIT takes it to another level, but Lua succeeded on its own through solid research and earned a
reputation as the performant embeddable language, and did so without native compilation. See [^3]
and [^4] below.

[^1]: [cranelift](https://cranelift.dev)

[^2]: [Crafting Interpreters](https://craftinginterpreters.com/)

[^3]: [The Implementation of Lua 5.0](https://www.lua.org/doc/jucs05.pdf)

[^4]: [A No-Frills Introduction to Lua 5.1 VM Instructions](https://www.mcours.net/cours/pdf/hasclic3/hasssclic818.pdf)

[^5]: [A Performance Survey on Stack-based and Register-based Virtual Machines](https://arxiv.org/abs/1611.00467)
