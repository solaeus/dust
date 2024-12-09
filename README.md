# Dust

Dust is a high-level interpreted programming language with static types that focuses on ease of use,
performance and correctness. The syntax, safety features and evaluation model are inspired by Rust.
The instruction set, optimization strategies and virtual machine are inspired by Lua. Unlike Rust
and other compiled languages, Dust has a very low time to execution. Simple programs compile in
under a millisecond on a modern processor. Unlike Lua and most other interpreted languages, Dust is
type-safe, with a simple yet powerful type system that enhances clarity and prevent bugs.

```dust
write_line("Enter your name...")

let name = read_line()

write_line("Hello " + name + "!")
```

## Feature Progress

Dust is still in development. This list may change as the language evolves.

- [X] Lexer
- [X] Compiler
- [X] VM
- [ ] Formatter
- [X] Disassembler (for chunk debugging)
- CLI
  - [X] Run source
  - [X] Compile to chunk and show disassembly
  - [X] Tokenize using the lexer and show token list
  - [ ] Format using the formatter and display the output
  - [ ] Compile to and run from intermediate formats
    - [ ] JSON
    - [ ] Postcard
- Basic Values
  - [X] No `null` or `undefined` values
  - [X] Booleans
  - [X] Bytes (unsigned 8-bit)
  - [X] Characters (Unicode scalar value)
  - [X] Floats (64-bit)
  - [X] Functions
  - [X] Integers (signed 64-bit)
  - [ ] Ranges
  - [X] Strings (UTF-8)
- Composite Values
  - [X] Concrete lists
  - [X] Abstract lists (optimization)
  - [ ] Concrete maps
  - [ ] Abstract maps (optimization)
  - [ ] Tuples (fixed-size constant lists)
  - [ ] Structs
  - [ ] Enums
- Types
  - [X] Basic types for each kind of basic value
  - [X] Generalized types: `num`, `any`, `none`
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
  - [ ] Loops
    - [ ] `for`
    - [ ] `loop`
    - [X] `while`
  - [ ] Match

## Implementation

Dust is implemented in Rust and is divided into several parts, most importantly the lexer, compiler,
and virtual machine. All of Dust's components are designed with performance in mind and the codebase
uses as few dependencies as possible. The code is tested by integration tests that compile source
code and check the compiled chunk, then run the source and check the output of the virtual machine.
It is important to maintain a high level of quality by writing meaningful tests and preferring to
compile and run programs in an optimal way before adding new features.

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
a `LoadConstant` instruction and adds the integer to the constant list. Tokens with infix parsers
include the math operators, which emit `Add`, `Subtract`, `Multiply`, `Divide`, and `Modulo`
instructions.

Functions are compiled into their own chunks, which are stored in the constant list. A function's
arguments are stored in the locals list. The VM must later bind the arguments to runtime values by
assigning each argument a register and associating the register with the local.

#### Optimizing

When generating instructions for a register-based virtual machine, there are opportunities to
optimize the generated code by using fewer instructions or fewer registers. While it is best to
output optimal code in the first place, it is not always possible. Dust's compiler modifies the
instruction list during parsing to apply optimizations before the chunk is completed. There is no
separate optimization pass, and the compiler cannot be run in a mode that disables optimizations.

#### Type Checking

Dust's compiler associates each emitted instruction with a type. This allows the compiler to enforce
compatibility when values are used in expressions. For example, the compiler will not allow a string
to be added to an integer, but it will allow either to be added to another of the same type. Aside
from instruction arguments, the compiler also checks the types of function arguments and the blocks
of `if`/`else` statements.

The compiler always checks types on the fly, so there is no need for a separate type-checking pass.

### Instructions

Dust's virtual machine uses 32-bit instructions, which encode seven pieces of information:

Bit   | Description
----- | -----------
0-4   | Operation code
5     | Flag indicating if the B argument is a constant
6     | Flag indicating if the C argument is a constant
7     | D field (boolean)
8-15  | A field (unsigned 8-bit integer)
16-23 | B field (unsigned 8-bit integer)
24-31 | C field (unsigned 8-bit integer)

#### Operations

Five bits are used for the operation, which allows for up to 32 operations.

##### Stack manipulation

- MOVE: Makes a register's value available in another register by using a pointer. This avoids
  copying the value or invalidating the original register.
- CLOSE: Sets a range of registers to the "empty" state.

##### Value loaders

- LOAD_BOOLEAN: Loads a boolean, the value of which is encoded in the instruction, to a register.
- LOAD_CONSTANT: Loads a constant from the constant list to a register.
- LOAD_LIST: Creates a list abstraction from a range of registers and loads it to a register.
- LOAD_MAP: Creates a map abstraction from a range of registers and loads it to a register.
- LOAD_SELF: Creates an abstraction that represents the current function and loads it to a register.

##### Variable operations

- GET_LOCAL: Loads a variable's value to a register by using a pointer to point to the variable's
  canonical register (i.e. the register whose index is stored in the locals list).
- SET_LOCAL: Changes a variable's register to a pointer to another register, effectively changing
  the variable's value.

##### Arithmetic

Arithmetic instructions use every field except for D. The A field is the destination register, the B
and C fields are the arguments, and the flags indicate whether the arguments are constants.

- ADD: Adds two values and stores the result in a register. Unlike the other arithmetic operations,
  the ADD instruction can also be used to concatenate strings and characters.
- SUBTRACT: Subtracts one argument from another and stores the result in a register.
- MULTIPLY: Multiplies two arguments and stores the result in a register.
- DIVIDE: Divides one value by another and stores the result in a register.
- MODULO: Calculates the division remainder of two values and stores the result in a register.
- POWER: Raises one value to the power of another and stores the result in a register.

##### Logic

Logic instructions work differently from arithmetic and comparison instructions, but they are still
essentially binary operations with a left and a right argument. Rather than performing some
calculation and storing a result, the logic instructions perform a check on the left-hand argument
and, based on the result, either skip the right-hand argument or allow it to be executed. A `TEST`
is always followed by a `JUMP`. If the left argument passes the test (a boolean equality check), the
`JUMP` instruction is skipped and the right argument is executed. If the left argument fails the
test, the `JUMP` is not skipped and it jumps past the right argument.

- TEST
- TEST_SET

##### Comparison

- EQUAL
- LESS
- LESS_EQUAL

##### Unary operations

- NEGATE
- NOT

##### Execution

- CALL
- CALL_NATIVE
- JUMP
- RETURN


The A, B, and C
fields are used for usually used as indexes into the constant list or stack, but they can also hold
other information, like the number of arguments for a function call.

### Virtual Machine

The virtual machine is simple and efficient. It uses a stack of registers, which can hold values or
pointers. Pointers can point to values in the constant list, locals list, or the stack itself.

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
Pratt parser. The book is a great introduction to writing interpreters.

[A No-Frills Introduction to Lua 5.1 VM Instructions] by Kein-Hong Man was a great resource for the
design of Dust's instructions and operation codes. The Lua VM is simple and efficient, and Dust's VM
attempts to be the same, though it is not as optimized for different platforms. Dust's instructions
were originally 32-bit like Lua's, but were changed to 64-bit to allow for more complex information
about the instruction's arguments. Dust's compile-time optimizations are inspired by Lua
optimizations covered in this paper.

[The Implementation of Lua 5.0] by Roberto Ierusalimschy, Luiz Henrique de Figueiredo, and Waldemar
Celes was a great resource for understanding register-based virtual machines and their instructions.
This paper is a great resource when designing new features.

[Crafting Interpreters]: https://craftinginterpreters.com/
[The Implementation of Lua 5.0]: https://www.lua.org/doc/jucs05.pdf
[A No-Frills Introduction to Lua 5.1 VM Instructions]: https://www.mcours.net/cours/pdf/hasclic3/hasssclic818.pdf
