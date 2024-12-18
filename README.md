# Dust

A programming language that is **fast**, **safe** and **easy to use**.

Dust is **statically typed** to ensure that each program is valid before it is run. It offers
compile times of less than 100 microseconds on modern hardware. As a **bytecode interpreter** with a
**register-based virtual machine**, Dust leverages compile-time safety guarantees and optimizations
along with beautiful syntax to deliver a unique set of features rarely found in other languages. It
is designed to deliver the best qualities of two disparate categories of programming language: the
highly optimized but slow-to-compile languages like Rust and C++ and the quick-to-start but often
slow and error-prone languages like Python and JavaScript.

Dust's syntax, safety features and evaluation model are based on Rust. Its instruction set,
optimization strategies and virtual machine are based on Lua. Unlike Rust and other languages that
compile to machine code, Dust has a very low time to execution. Unlike Lua and most other
interpreted languages, Dust enforces static typing to improve clarity and prevent bugs. While some
languages currently offer high-level features and strict typing (e.g. TypeScript), Dust has a simple
approach to syntax that offers flexibility and expressiveness while still being *obvious*, even
those who know how to code but don't know the language. Dust is developed with an emphasis on
achieving foundational soundness before adding new features. Dust's planned features and design
favor programmers who prefer their code to be simple and clear rather than clever and complex.

**Dust is under active development and is not yet ready for general use.**

**Features discussed in this README may be unimplemented, partially implemented or temporarily
removed**

```rust
write_line("Enter your name...")

let name = read_line()

write_line("Hello " + name + "!")
```

```rust
fn fib (n: int) -> int {
    if n <= 0 {
        return 0
    } else if n == 1 {
        return 1
    }

    fib(n - 1) + fib(n - 2)
}

write_line(fib(25))
```

Dust uses a custom register-based virtual machine with its own set of instructions and a compiler
based on recursive descet to emit them. This should not be confused with a machine code compiler.
Despite having **compile-time guarantees**, Dust falls into the category of interpreted languages.
Competing with the runtime performance of Rust or C++ *is not* a goal. Competing with the
approachability and simplicity of those languages *is* a goal. On the other hand Dust *does* intend
to be faster than Python, Ruby and NodeJS while also offering a superior development experience and
more reliable code due to its static typing. Dust's development approach is informed by some
books[^1] and academic research[^4] as well as practical insight from papers[^2] written by language
authors. See the [Inspiration](README#Inspiration) section for more information or keep reading to
learn about Dust's features.

## Goals

This project's goal is to deliver a language that not only *works* but that offers genunine value
due to a unique combination of design choices and a high-quality implementation. As mentioned in the
first sentence, Dust's general aspirations are to be **fast**, **safe** and **easy**.

- **Easy**
  - **Simple Syntax** Dust should be easier to learn than most programming languages. Its syntax
    should be familiar to users of other C-like languages to the point that even a new user can read
    Dust code and understand what it does. Rather than being dumbed down by a lack of features, Dust
    should be powerful and elegant in its simplicity, seeking a maximum of capability with a minimum
    of complexity. When advanced features are added, they should never obstruct existing features,
    including readability. Even the advanced type system should be clear and unintimidating.
  - **Excellent Errors** Dust should provide helpful error messages that guide the user to the
    source of the problem and suggest a solution. Errors should be a helpful learning ressource for
    users rather than a source of frustration.
  - **Relevant Documentation** Users should have the resources they need to learn Dust and write
    code in it. They should know where to look for answers and how to reach out for help.
- **Safe**
  - **Static Types** Typing should prevent runtime errors and improve code quality, offering a
    superior development experience despite some additional constraints. Like any good statically
    typed language, users should feel confident in the type-consistency of their code and not want
    to go back to a dynamically typed language.
  - **Memory Safety** Dust should be free of memory bugs. Being implemented in Rust makes this easy
    but, to accomodate long-running programs, Dust still requires a memory management strategy.
    Dust's design is to use a separate thread for garbage collection, allowing the main thread to
    continue executing code while the garbage collector looks for unused memory.
- **Fast**
  - **Fast Compilation** Despite its compile-time abstractions, Dust should compile and start
    executing quickly. The compilation time should feel negligible to the user.
  - **Fast Execution** Dust should be generally faster than Python, Ruby and NodeJS. It should be
    competitive with highly optimized, modern, register-based VM languages like Lua. Dust should
    be benchmarked during development to inform decisions about performance.
  - **Low Resource Usage** Despite its performance, Dust's use of memory and CPU power should be
    conservative and predictable enough to accomodate a wide range of devices.

These are the project's general design goals. There are many more implementation goals. Among them
are:

  - Effortless Concurrency: Dust should offer an excellent experience for writing multi-threaded
    programs. The language's native functions should offer an API for spawning threads, sending
    messages and waiting for results. When using these features, Dust should be much faster than any
    single-threaded language. However, Dust should be fast even when running on a single thread.
    Single-threaded performce is the best predictor of multi-threaded performance so continuing to
    optimize how each thread executes instructions, accesses memory and moves pointers is the best
    way to ensure that Dust is fast in all scenarios.
  - Embeddability: The library should be easy to use so that Dust can be built into other
    applications. Dust should compile to WebAssembly and offer examples of how to use it in a web
    application. The user should be able to query the VM for information about the program's state
    and control the program's execution. It should be possible to view and modify the value of a
    variable and inspect the call stack.
  - Data Fluency: Dust's value type should support conversion to and from arbitrary data in formats
    like JSON, YAML, TOML and CSV. Pulling data into a Dust program should be easy, with built-in
    functions offering conversion for the most widely used formats.
  - Portability: Dust should run on as many architectures and operating systems as possible. Using
    fewer dependencies and avoiding platform-specific code will help Dust achieve this goal. The
    Dust library should be available as a WebAssembly module.
  - Developer Experience: Dust should be fun and easy to use. That implies easy installation and the
    availability of tutorials and how-to guides. The CLI should be predictable and feature-rich,
    with features that make it easy to write and debug Dust code like formatting, bytecode
    disassembly and logging.
  - Advanced Type System: Dust should implement composite types, aliases and generics. The type
    system should use a descriptive syntax that is easy to understand. Dust's type system should be
    static, meaning that types are checked before a program reaches the VM. Dust is not a
    graduallly typed language, its VM is and should remain type-agnostic.
  - Thorough Testing: Primarily, the output of Dust's compiler and VM should be tested with programs
    that cover all of the language's features. The tests should be actively maintained and should be
    changed frequently to reflect a growing project that is constantly discovering new optimizations
    and opportunities for improvement.

## Project Status

This project is maintained by a single developer. For now, its primary home is on a private git
server. The GitHub mirror is updated automatically and should carry the latest branches. There are
no other contributors at this time but the project is open to feedback and should eventually accept
contributions.

For now, both the library API and the implementation details are freely changed and the CLI has not
been published. Dust is both an ambitious project and a continuous experiment in language design.
Features may be redesigned and reimplemented at will when they do not meet the project's performance
or usability goals. This approach maximizes the development experience as a learning opportunity and
enforces a high standard of quality but slows down the process of delivering features to users.
Eventually, Dust will reach a stable release and will be ready for general use. As the project
approaches this milestone, the experimental nature of the project will be reduced and a replaced
with a focus on stability and improvement.

## Language Overview

This is a quick overview of Dust's syntax features. It skips over the aspects that are familiar to
most programmers such as creating variables, using binary operators and printing to the console.
Eventually there should be a complete reference for the syntax.

### Syntax and Evaluation

Dust belongs to the C-like family of languages[^5], with an imperative syntax that will be familiar
to many programmers. Dust code looks a lot like Ruby, JavaScript, TypeScript and other members of
the family but Rust is its primary point of reference for syntax. Rust was chosen as a syntax model
because its imperative code is *obvious by design* and *widely familiar*. Those qualities are
aligned with Dust's emphasis on usability.

However, some differences exist. Dust *evaluates* all of the code in the file while Rust only
initiates from a "main" function. Dust's execution model is more like one found in a scripting
language. If we put `42 + 42 == 84` into a file and run it, it will return `true` because the outer
context is, in a sense, the "main" function.

So while the syntax is by no means compatible, it is superficially similar, even to the point that
syntax highlighting for Rust code works well with Dust code. This is not a design goal but a happy
coincidence.

### Semicolons

Dust borrowed Rust's approach to semicolons and their effect on evaluation and relaxed the rules to
accomated different styles of coding. Rust, for example, isn't design for command lines or REPLs but
Dust could be well-suited to those applications. Dust needs to work in a source file or in an ad-hoc
one-liner sent to the CLI. Thus, semicolons are optional in most cases.

There are two things you need to know about semicolons in Dust:

- Semicolons suppress the value of whatever they follow. The preceding statement or expression will
  have the type `none` and will not evaluate to a value.
- If a semicolon does not change how the program runs, it is optional.

This example shows three statements with semicolons. The compiler knows that a `let` statement
cannot produce a value and will always have the type `none`. Thanks to static typing, it also knows
that the `write_line` function has no return value so the function call also has the type `none`.
Therefore, these semicolons are optional.

```rust
let a = 40;
let b = 2;

write_line("The answer is ", a + b);
```

Removing the semicolons does not alter the execution pattern or the return value.

```rust
let x = 10
let y = 3

write_line("The remainder is ", x % y)
```

The next example produces a compiler error because the `if` block returns a value of type `int` but
the `else` block does not return a value at all. Dust does not allow branches of the same `if/else`
statement to have different types. In this case, adding a semicolon after the `777` expression fixes
the error by supressing the value.

```rust
// !!! Compile Error !!!
let input = read_line()
let reward = if input == "42" {
    write_line("You got it! Here's your reward.")

    777 // <- We need a semicolon here
} else {
    write_line(input, " is not the answer.")
}
```

### Statements and Expressions

Dust is composed of statements and expressions. If a statement ends in an expression without a
trailing semicolon, the statement evaluates to the value produced by that expression. However, if
the expression's value is suppressed with a semicolon, the statement does not evaluate to a value.
This is identical to Rust's evaluation model. That means that the following code will not compile:

```rust
// !!! Compile Error !!!
let a = { 40 + 2; }
```

The `a` variable is assigned to the value produced by a block. The block contains an expression that
is suppressed by a semicolon, so the block does not evaluate to a value. Therefore, the `a` variable
would have to be uninitialized (which Dust does not allow) or result in a runtime error (which Dust
avoids at all costs). We can fix this code by moving the semicolon to the end of the block. In this
position it suppresses the value of the entire `let` statement. As we saw above, a `let` statement
never evaluates to a value, so the semicolon has no effect on the program's behavior and could be
omitted altogether.

```rust
let a = { 40 + 2 }; // This is fine
let a = { 40 + 2 }  // This is also fine
```

Only the final expression in a block is returned. When a `let` statement is combined with an
`if/else` statement, the program can perform side effects before evaluating the value that will be
assigned to the variable.

```rust
let random: int = random(0..100)
let is_even = if random == 99 {
    write_line("We got a 99!")

    false
} else {
    random % 2 == 0
}

is_even
```

If the above example were passed to Dust as a complete program, it would return a boolean value and
might print a message to the console (if the user is especially lucky). However, note that the
program could be modified to return no value by simply adding a semicolon at the end.

Compared to JavaScript, Dust's evaluation model is more predictable, less error-prone and will never
trap the user into a frustating hunt for a missing semicolon. Compared to Rust, Dust's evaluation
model is more accomodating without sacrificing expressiveness. In Rust, semicolons are *required*
and *meaningful*, which provides excellent consistency but lacks flexibility. In JavaScript,
semicolons are *required* and *meaningless*, which is a source of confusion for many developers.

### Control Flow

-- TODO --

### Functions

-- TODO --

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

#### Immutability by Default

TODO

#### Memory Safety

TODO

### Basic Values

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

### Composite Values

TODO

## Previous Implementations

Dust has gone through several iterations, each with its own design choices. It was originally
implemented with a syntax tree generated by an external parser, then a parser generator, and finally
a custom parser. Eventually the language was rewritten to use bytecode instructions and a virtual
machine. The current implementation: compiling to bytecode with custom lexing and parsing for a
register-based VM, is by far the most performant and the general design is unlikely to change,
although it has been optimized and refactored several times. For example, the VM was refactored to
manage multiple threads.

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
choose between the two. Some of the benchmarks described in the paper inspired similar benchmarks
used in this project to compare Dust to other languages.

## License

Dust is licensed under the GNU General Public License v3.0. See the `LICENSE` file for details.

## References

[^1]: [Crafting Interpreters](https://craftinginterpreters.com/)
[^2]: [The Implementation of Lua 5.0](https://www.lua.org/doc/jucs05.pdf)
[^3]: [A No-Frills Introduction to Lua 5.1 VM Instructions](https://www.mcours.net/cours/pdf/hasclic3/hasssclic818.pdf)
[^4]: [A Performance Survey on Stack-based and Register-based Virtual Machines](https://arxiv.org/abs/1611.00467)
[^5]: [List of C-family programming languages](https://en.wikipedia.org/wiki/List_of_C-family_programming_languages)
[^6]: [ripgrep is faster than {grep, ag, git grep, ucg, pt, sift}](https://blog.burntsushi.net/ripgrep/#mechanics)
