# Dust

A programming language that is **fast**, **safe** and **easy to use**.

Dust's syntax, safety features and evaluation model are inspired by Rust. The instruction set,
optimization strategies and virtual machine are inspired by Lua and academic research in the field
(see the [Inspiration](README#Inspiration). Unlike Rust and most other compiled languages, Dust has
a very low time to execution. Unlike Lua and most other interpreted languages, Dust enforces static
typing during compilation, with a simple yet powerful type system that enhances clarity and prevents
bugs.

```rust
write_line("Enter your name...")

let name = read_line()

write_line("Hello " + name + "!")
```

```rust
fn fib (n: int) -> int {
    if n <= 0 { return 0 }
    if n == 1 { return 1 }

    fib(n - 1) + fib(n - 2)
}

write_line(fib(25))
```

Dust uses the same library for error reporting as Rust, which provides ample opportunities to show
the user where they went wrong and how to fix it. Helpful error messages are a high priority and the
language will not be considered stable until they are consistently informative and actionable.

```
error: Compilation Error: Cannot add these types
  |
1 | 40 + 2.0
  | -- info: A value of type "int" was used here.
  |
1 | 40 + 2.0
  |      --- info: A value of type "float" was used here.
  |
1 | 40 + 2.0
  | -------- help: Type "int" cannot be added to type "float". Try converting one of the values to the other type.
  |
```

## Project Status

**Dust is under active development and is not yet ready for general use.**

**Features discussed in this README may be unimplemented, partially implemented, temporarily removed
or only available on a seperate branch.**

Dust is an ambitious project that acts as a continuous experiment in language design. Features may
be redesigned and reimplemented at will when they do not meet the project's performance or
usability goals. This approach maximizes the development experience as a learning opportunity and
enforces a high standard of quality but slows down the process of delivering features to users.
Eventually, Dust will reach a stable release and will be ready for general use. As the project
approaches this milestone, the experimental nature of the project will be reduced and a replaced
with a focus on stability and improvement.

## Language Overview

### Syntax

Dust belongs to the C-like family of languages, with an imperative syntax that will be familiar to
many programmers. Dust code looks a lot like Ruby, JavaScript, TypeScript and other members of the
family but Rust is its primary point of reference for syntax. Rust was chosen as a syntax model
because its imperative code is *obvious* and *familiar*. Those qualities are aligned with Dust's
emphasis on safety and usability. However, some differences exist because Dust is a simpler language
that can tolerate more relaxed syntax. For example, Dust has more relaxed rules about semicolons:
they can be used to suppress values (like in Rust) but are not required at the end of every
statement.

In this example, these semicolons are optional. Because these `let` statements do not return a
value, the semicolons have nothing to suppress and are ignored.

```rust
let a = 40;
let b = 2;

write_line("The answer is ", a + b);
```

One could write the above program without any semicolons at all.

```rust
let x = 10
let y = 3

write_line("The remainder is ", x % y)
```

The next example produces a compiler error because the `if` block returns a value of type `int` but
the `else` block does not return a value at all. Dust does not allow branches of the same `if/else`
statement to return different types of values. In this case, adding a semicolon after the `777`
expression fixes the error by supressing the value.

```rust
let input = read_line()

if input == "42" {
    write_line("You got it! Here's your reward.")

    777
} else {
    write_line("That is not the answer.")
}
```

Remember that even if some syntax is optional, that does not mean it should always be omitted or is
not useful. Aside from their practical use, semicolons provide a visual barrier between statements
written on the same line. Dust's design philosophy is to provide a balance between strictness and
expressiveness so that the language is applicable to a wide range of use cases. A web server with a
team of developers may prefer a more long-form style of code with lots of line breaks while a user
writing Dust on the command line may prefer a more terse style without sacrificing readability.

```rust
let a = 0; let b = 1; let c = 2; let list = [a, b, c];

write_line("Here's our list: ", list)
```

### Safety

#### Type System

All variables have a type that is established when the variable is declared. This usually does not
require that the type be explicitly stated, Dust can infer the type from the value. Types are also
associated with the arms of `if/else` statements and the return values of functions, which prevents
different runtime scenarios from producing different types of values.

#### Null-Free

There is no `null` or `undefined` value in Dust. All values and variables must be initialized to one
of the supported value types. This eliminates a whole class of bugs that permeate many other
languages. "I call it my billion-dollar mistake. It was the invention of the null reference in
1965." - Tony Hoare

Dust *does* have a `none` type, which should not be confused for being `null`-like. Like the `()` or
"unit" type in Rust, `none` exists as a type but not as a value. It indicates the lack of a value
from a function, expression or statement. A variable cannot be assigned to `none`.

#### Memory Safety

<!-- TODO: Introduce Dust's approach to memory management and garbage collection. -->

### Values, Variables and Types

Dust supports the following basic values:

- Boolean: `true` or `false`
- Byte: An unsigned 8-bit integer
- Character: A Unicode scalar value
- Float: A 64-bit floating-point number
- Function: An executable chunk of code
- Integer: A signed 64-bit integer
- String: A UTF-8 encoded byte sequence

Dust's "basic" values are conceptually similar because they are singular as opposed to composite.
Most of these values are stored on the stack but some are heap-allocated. A Dust string is a
sequence of bytes that are encoded in UTF-8. Even though it could be seen as a composite of byte
values, strings are considered "basic" because they are parsed directly from tokens and behave as
singular values. Shorter strings are stored on the stack while longer strings are heap-allocated.
Dust offers built-in native functions that can manipulate strings by accessing their bytes or
reading them as a sequence of characters.

<!-- TODO: Describe Dust's composite values -->

## Feature Progress

This list is a rough outline of the features that are planned to be implemented as soon as possible.
*This is not an exhaustive list of all planned features.* This list is updated and rearranged to
maintain a docket of what is being worked on, what is coming next and what can be revisited later.

- [X] Lexer
- [X] Compiler
- [X] VM
- [X] Disassembler (for chunk debugging)
- [ ] Formatter
- [ ] REPL
- CLI
  - [X] Run source
  - [X] Compile source to a chunk and show disassembly
  - [X] Tokenize using the lexer and show token list
  - [ ] Format using a built-in formatter
  - [ ] Compile to and run from intermediate formats
    - [ ] JSON
    - [ ] Postcard
  - [ ] Integrated REPL
- Basic Values
  - [X] No `null` or `undefined` values
  - [X] Booleans
  - [X] Bytes (unsigned 8-bit)
  - [X] Characters (Unicode scalar value)
  - [X] Floats (64-bit)
  - [X] Functions
  - [X] Integers (signed 64-bit)
  - [X] Strings (UTF-8)
- Composite Values
  - [X] Concrete lists
  - [X] Abstract lists (optimization)
  - [ ] Concrete maps
  - [ ] Abstract maps (optimization)
  - [ ] Ranges
  - [ ] Tuples (fixed-size constant lists)
  - [ ] Structs
  - [ ] Enums
- Types
  - [X] Basic types for each kind of basic value
  - [X] Generalized types: `num`, `any`, `none`
  - [ ] Type conversion (safe, explicit and coercion-free)
  - [ ] `struct` types
  - [ ] `enum` types
  - [ ] Type aliases
  - [ ] Type arguments
  - [ ] Compile-time type checking
    - [ ] Function returns
    - [X] If/Else branches
    - [ ] Instruction arguments
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
  - [ ] Match
  - [ ] Loops
    - [ ] `for`
    - [ ] `loop`
    - [X] `while`
- Native Functions
  - Assertions
    - [X] `assert`
    - [ ] `assert_eq`
    - [ ] `assert_ne`
    - [ ] `panic`
  - I/O
    - [ ] `read`
    - [X] `read_line`
    - [X] `write`
    - [X] `write_line`
  - String Functions
  - List Functions
  - Map Functions
  - Math Functions
  - Filesystem Functions
  - Network Functions
  - System Functions

## Implementation

Dust is implemented in Rust and is divided into several parts, most importantly the lexer, compiler,
and virtual machine. All of Dust's components are designed with performance in mind and the codebase
uses as few dependencies as possible. The code is tested by integration tests that compile source
code and check the compiled chunk, then run the source and check the output of the virtual machine.
It is important to maintain a high level of quality by writing meaningful tests and preferring to
compile and run programs in an optimal way before adding new features.

### Command Line Interface

Dust's command line interface and developer experience are inspired by tools like Bun and especially
Cargo, the Rust package manager that includes everything from project creation to documentation
generation to code formatting to much more. Dust's CLI has started by exposing the most imporant
features for debugging and developing the language itself. Tokenization, compiling, disassembling
and running Dust code are currently supported. The CLI will eventually support a REPL, code
formatting, linting and other features that enhance the development experience and make Dust more
fun and easy to use.

### Lexer and Tokens

The lexer emits tokens from the source code. Dust makes extensive use of Rust's zero-copy
capabilities to avoid unnecessary allocations when creating tokens. A token, depending on its type,
may contain a reference to some data from the source code. The data is only copied in the case of an
error. In a successfully executed program, no part of the source code is copied unless it is a
string literal or identifier.

### Compiler

The compiler creates a chunk, which contains all of the data needed by the virtual machine to run a
Dust program. It does so by emitting bytecode instructions, constants and locals while parsing the
tokens, which are generated one at a time by the lexer.

#### Parsing

Dust's compiler uses a custom Pratt parser, a kind of recursive descent parser, to translate a
sequence of tokens into a chunk. Each token is given a precedence and may have a prefix and/or infix
parser. The parsers are just functions that modify the compiler and its output. For example, when
the compiler encounters a boolean token, its prefix parser is the `parse_boolean` function, which
emits a `LoadBoolean` instruction. An integer token's prefix parser is `parse_integer`, which emits
a `LoadConstant` instruction and adds the integer to the constants list. Tokens with infix parsers
include the math operators, which emit `Add`, `Subtract`, `Multiply`, `Divide`, `Modulo` and `Power`
instructions.

Functions are compiled into their own chunks, which are stored in the constant list. A function's
arguments are stored in its locals list. Before the function is run, the VM must bind the arguments
to values by filling locals' corresponding registers. Instead of copying the arguments, the VM uses
a pointer to one of the parent's registers or constants.

#### Optimizing

When generating instructions for a register-based virtual machine, there are opportunities to
optimize the generated code by using fewer instructions or fewer registers. While it is best to
output optimal code in the first place, it is not always possible. Dust's uses a single-pass
compiler and therefore applies optimizations immeadiately after the opportunity becomes available.
There is no separate optimization pass and the compiler cannot be run in a mode that disables
optimizations.

#### Type Checking

Dust's compiler associates each emitted instruction with a type. This allows the compiler to enforce
compatibility when values are used in expressions. For example, the compiler will not allow a string
to be added to an integer, but it will allow either to be added to another of the same type. Aside
from instruction arguments, the compiler also checks the types of function arguments and the blocks
of `if`/`else` statements.

The compiler always checks types on the fly, so there is no need for a separate type-checking pass.
Type information is removed from the instructions list before the chunk is created, so the VM (which
is entirely type-agnostic) never sees it.

### Instructions

Dust's virtual machine uses 32-bit instructions, which encode seven pieces of information:

Bit   | Description
----- | -----------
0-4   | Operation code
5     | Flag indicating if the B field is a constant
6     | Flag indicating if the C field is a constant
7     | D field (boolean)
8-15  | A field (unsigned 8-bit integer)
16-23 | B field (unsigned 8-bit integer)
24-31 | C field (unsigned 8-bit integer)

#### Operations

The 1.0 version of Dust will have more than the current number of operations but cannot exceed 32
because of the 5 bit format.

##### Stack manipulation

- MOVE: Makes a register's value available in another register by using a pointer. This avoids
  copying the value or invalidating the original register.
- CLOSE: Sets a range of registers to the "empty" state.

##### Value loaders

- LOAD_BOOLEAN: Loads a boolean to a register. Booleans known at compile-time are not stored in the
  constant list. Instead, they are encoded in the instruction itself.
- LOAD_CONSTANT: Loads a constant from the constant list to a register. The VM avoids copying the
  constant by using a pointer with the constant's index.
- LOAD_LIST: Creates a list abstraction from a range of registers and loads it to a register.
- LOAD_MAP: Creates a map abstraction from a range of registers and loads it to a register.
- LOAD_SELF: Creates an abstraction that represents the current function and loads it to a register.

##### Variable operations

- GET_LOCAL: Loads a variable's value to a register by using a pointer to point to the variable's
  canonical register (i.e. the register whose index is stored in the locals list).
- SET_LOCAL: Changes a variable's register to a pointer to another register, effectively changing
  the variable's value.

##### Arithmetic

Arithmetic instructions use the A, B and C fields. The A field is the destination register, the B
and C fields are the arguments, and the flags indicate whether the arguments are constants.

- ADD: Adds two values and stores the result in a register. Unlike the other arithmetic operations,
  the ADD instruction can also be used to concatenate strings and/or characters. Characters are the
  only type of value that can perform a kind of implicit conversion. Although the character itself
  is not converted, its underlying bytes are concatenated to the string.
- SUBTRACT: Subtracts one argument from another and stores the result in a register.
- MULTIPLY: Multiplies one argument by another and stores the result in a register.
- DIVIDE: Divides one value by another and stores the result in a register.
- MODULO: Calculates the division remainder of two values and stores the result in a register.
- POWER: Raises one value to the power of another and stores the result in a register.

##### Logic and Control Flow

Logic instructions work differently from arithmetic and comparison instructions, but they are still
essentially binary operations with a left and a right argument. These areguments, however, are other
instructions. This is reminiscent of a stack-based virtual machine in which the arguments are found
in the stack rather than having their location encoded in the instruction. The logic instructions
perform a check on the left-hand argument and, based on the result, either skip the right-hand
argument or allow it to be executed. A `TEST` is always followed by a `JUMP`. If the left argument
passes the test (a boolean equality check), the `JUMP` instruction is skipped and the right argument
is executed. If the left argument fails the test, the `JUMP` is not skipped and it jumps past the
right argument.

- TEST
- TEST_SET

<!-- TODO: Discuss control flow using TEST -->

##### Comparison

<!-- TODO -->

- EQUAL
- LESS
- LESS_EQUAL

##### Unary operations

<!-- TODO -->

- NEGATE
- NOT

##### Execution

<!-- TODO -->

- CALL
- CALL_NATIVE
- JUMP
- RETURN

### Virtual Machine

The virtual machine is simple and efficient. It uses a stack of registers, which can hold values or
pointers. Pointers can point to values in the constant list or the stack itself.

While the compiler has multiple responsibilities that warrant more complexity, the VM is simple
enough to use a very straightforward design. The VM's `run` function uses a simple `while` loop with
a `match` statement to execute instructions. When it reaches a `Return` instruction, it breaks the
loop and optionally returns a value.

## Previous Implementations

Dust has gone through several iterations, each with its own design choices. It was originally
implemented with a syntax tree generated by an external parser, then a parser generator, and finally
a custom parser. Eventually the language was rewritten to use bytecode instructions and a virtual
machine. The current implementation is by far the most performant and the general design is unlikely
to change.

Dust previously had a more complex type system with type arguments (or "generics") and a simple
model for asynchronous execution of statements. Both of these features were removed to simplify the
language when it was rewritten to use bytecode instructions. Both features are planned to be
reintroduced in the future.

## Inspiration

[Crafting Interpreters] by Bob Nystrom was a great resource for writing the compiler, especially the
Pratt parser. The book is a great introduction to writing interpreters. Had it been discovered
sooner, some early implementations of Dust would have been both simpler in design and more ambitious
in scope.

[The Implementation of Lua 5.0] by Roberto Ierusalimschy, Luiz Henrique de Figueiredo, and Waldemar
Celes was a great resource for understanding register-based virtual machines and their instructions.
This paper was recommended by Bob Nystrom in [Crafting Interpreters].

[A No-Frills Introduction to Lua 5.1 VM Instructions] by Kein-Hong Man has a wealth of detailed
information on how Lua uses terse instructions to create dense chunks that execute quickly. This was
essential in the design of Dust's instructions. Dust uses compile-time optimizations that are based
on Lua optimizations covered in this paper.

[A Performance Survey on Stack-based and Register-based Virtual Machines] by Ruijie Fang and Siqi
Liup was helpful for a quick yet efficient primer on getting stack-based and register-based virtual
machines up and running. The included code examples show how to implement both types of VMs in C.
The performance comparison between the two types of VMs is worth reading for anyone who is trying to
choose between the two[^1]. Some of the benchmarks described in the paper inspired similar benchmarks
used in this project to compare Dust to other languages.

## License

Dust is licensed under the GNU General Public License v3.0. See the `LICENSE` file for details.

## References

[^1]: [Crafting Interpreters](https://craftinginterpreters.com/)
[^2]: [The Implementation of Lua 5.0](https://www.lua.org/doc/jucs05.pdf)
[^3]: [A No-Frills Introduction to Lua 5.1 VM Instructions](https://www.mcours.net/cours/pdf/hasclic3/hasssclic818.pdf)
[^4]: [A Performance Survey on Stack-based and Register-based Virtual Machines](https://arxiv.org/abs/1611.00467)
[^5]: [List of C-family programming languages](https://en.wikipedia.org/wiki/List_of_C-family_programming_languages)
