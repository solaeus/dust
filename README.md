# Dust

Dust is a high-level interpreted programming language with static types that focuses on ease of use,
performance and correctness. The syntax, safety features and evaluation model are inspired by Rust.
Due to being interpreted, Dust's total time to execution is much lower than Rust's. Unlike other
interpreted languages, Dust is type-safe, with a simple yet powerful type system that enhances the
clarity and correctness of a program.

## Feature Progress

Dust is still in development. This list may change as the language evolves.

- [X] Lexer
- [X] Compiler
- [X] VM
- [ ] Formatter
- CLI
  - [X] Run source
  - [X] Compile to chunk and show disassembly
  - [ ] Tokenize using the lexer and show token list
  - [ ] Format using the formatter and display the output
  - [ ] Compile to and run from intermediate formats
    - [ ] JSON
    - [ ] Postcard
- Values
  - [X] Basic values: booleans, bytes, characters, integers, floats, UTF-8 strings
  - [X] No `null` or `undefined`
  - [ ] Enums
  - [X] Functions
  - [X] Lists
  - [ ] Maps
  - [ ] Ranges
  - [ ] Structs
  - [ ] Tuples
  - [ ] Runtime-efficient abstract values for lists and maps
- Types
  - [X] Basic types for each kind of value
  - [X] Generalized types: `num`, `any`
  - [ ] `struct` types
  - [ ] `enum` types
  - [ ] Type aliases
  - [ ] Type arguments
  - [ ] Compile-time type checking
    - [ ] Function returns
    - [X] If/Else branches
    - [ ] Instruction arguments
  - [ ] Runtime type checking for debug compilation modes
- Variables
  - [X] Immutable by default
  - [X] Block scope
  - [X] Statically typed
  - [X] Copy-free identifiers are stored in the chunk as string constants
- Functions
  - [X] First-class value
  - [X] Statically typed arguments and returns
  - [X] Pure (no "closure" of local variables, arguments are the only input)
  - [ ] Type arguments
- Control Flow
  - [X] If/Else
  - [ ] Loops
    - [ ] `for`
    - [ ] `loop`
    - [X] `while`
  - [ ] Match
- Instructions
  - [X] Arithmetic
  - [X] Boolean
  - [X] Call
  - [X] Constant
  - [X] Control flow
  - [X] Load
  - [X] Store
  - [X] Return
  - [X] Stack
  - [X] Unar

## Implementation

Dust is implemented in Rust and is divided into several parts, most importantly the lexer, compiler,
and virtual machine. All of Dust's components are designed with performance in mind and the codebase
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

Types are checked during parsing and each emitted instruction is associated with a type.

#### Parsing

Dust's compiler uses a custom Pratt parser, a kind of recursive descent parser, to translate a
sequence of tokens into a chunk. Each token is given a precedence and may have a prefix and/or infix
parser. The parsers are just functions that modify the compiler and its output. For example, when
the compiler encounters a boolean token, its prefix parser is the `parse_boolean` function, which
emits a `LoadBoolean` instruction. An integer token's prefix parser is `parse_integer`, which emits
a `LoadConstant` instruction and adds the integer to the constant list. Tokens with infix parsers
include the math operators, which emit `Add`, `Subtract`, `Multiply`, `Divide`, and `Modulo`
instructions.

Functions are compiled into their own chunks, which are stored in the constant list. A function's
arguments are stored in the locals list. The VM must later bind the arguments to runtime values by
assigning each argument a register and associating the register with the local.

#### Optimizing

When generating instructions for a register-based virtual machine, there are opportunities to
optimize the generated code by using fewer instructions or fewer registers. While it is best to
output optimal code in the first place, it is not always possible. Dust's compiler uses simple
functions that modify isolated sections of the instruction list through a mutable reference.

### Instructions

Dust's virtual machine is register-based and uses 64-bit instructions, which encode nine pieces of
information:

Bit   | Description
----- | -----------
0-8   | The operation code.
9     | Boolean flag indicating whether the B argument is a constant.
10    | Boolean flag indicating whether the C argument is a constant.
11    | Boolean flag indicating whether the A argument is a local.
12    | Boolean flag indicating whether the B argument is a local.
13    | Boolean flag indicating whether the C argument is a local.
17-32 | The A argument,
33-48 | The B argument.
49-63 | The C argument.

### Virtual Machine

## Previous Implementations

Dust has gone through several iterations, each with its own unique features and design choices. It
was originally implemented with a syntax tree generated by an external parser, then a parser
generator, and finally a custom parser. Eventually the language was rewritten to use bytecode
instructions and a virtual machine. The current implementation is by far the most performant and the
general design is unlikely to change.

Dust previously had a more complex type system with type arguments (or "generics") and a simple
model for asynchronous execution of statements. Both of these features were removed to simplify the
language when it was rewritten to use bytecode instructions. Both features are planned to be
reintroduced in the future.

## Inspiration

[Crafting Interpreters] by Bob Nystrom was a major inspiration for rewriting Dust to use bytecode
instructions. It was also a great resource for writing the compiler, especially the Pratt parser.

[A No-Frills Introduction to Lua 5.1 VM Instructions] by Kein-Hong Man was a great resource for the
design of Dust's instructions and operation codes. The Lua VM is simple and efficient, and Dust's VM
attempts to be the same, though it is not as optimized for different platforms. Dust's instructions
were originally 32-bit like Lua's, but were changed to 64-bit to allow for more complex information
about the instruction's arguments.

[The Implementation of Lua 5.0] by Roberto Ierusalimschy, Luiz Henrique de Figueiredo, and Waldemar
Celes was a great resource for understanding how a compiler and VM tie together. Dust's compiler's
optimization functions were inspired by Lua optimizations covered in this paper.

[Crafting Interpreters]: https://craftinginterpreters.com/
[The Implementation of Lua 5.0]: https://www.lua.org/doc/jucs05.pdf
[A No-Frills Introduction to Lua 5.1 VM Instructions]: https://www.mcours.net/cours/pdf/hasclic3/hasssclic818.pdf
