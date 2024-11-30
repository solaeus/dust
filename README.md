# Dust

Dust is a high-level interpreted programming language with static types that focuses on ease of use,
performance and correctness.

## Features

- Values
  - [x] no `null` or `undefined`
  - [x] boolean
  - [x] byte
  - [x] character
  - [x] integer
  - [x] float
  - [x] function
  - [x] list
  - [x] range
  - [x] string
- Types
  - [x] basic types: `bool`, `byte`, `char`, `int`, `float`, functions, lists, ranges, `str`
  - [O] generalized types: `num`, `any`

## Implementation

Dust is implemented in Rust and is divided into several parts, primarily the lexer, compiler, and
virtual machine. All of Dust's components are designed with performance in mind and the codebase
uses as few dependencies as possible.

### Lexer

The lexer emits tokens from the source code. Dust makes extensive use of Rust's zero-copy
capabilities to avoid unnecessary allocations when creating tokens. A token, depending on its type,
may contain a reference to some data from the source code. The data is only copied in the case of an
error, because it improves the usability of the codebase for errors to own their data when possible.
In a successfully executed program, no part of the source code is copied unless it is a string
literal or identifier.

### Compiler

The compiler creates a chunk, which contains all of the data needed by the virtual machine to run a
Dust program. It does so by emitting bytecode instructions, constants and locals while parsing the
tokens, which are generated one at a time by the lexer.

#### Parsing

Dust's compiler uses a custom Pratt parser, a kind of recursive descent parser, to translate a
sequence of tokens into a chunk.

#### Optimizing

When generating instructions for a register-based virtual machine, there are opportunities to
optimize the generated code, usually by consolidating register use or reusing registers within an
expression. While it is best to output optimal code in the first place, it is not always possible.
Dust's compiler has a simple peephole optimizer that can be used to modify isolated sections of the
instruction list through a mutable reference.

### Instructions

### Virtual Machine

## Previous Implementations

## Inspiration

- [The Implementation of Lua 5.0](https://www.lua.org/doc/jucs05.pdf)
- [A No-Frills Introduction to Lua 5.1 VM Instructions](https://www.mcours.net/cours/pdf/hasclic3/hasssclic818.pdf)
- [Crafting Interpreters](https://craftinginterpreters.com/)
