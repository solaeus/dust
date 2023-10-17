# Dust

Dust is a programming language and interactive shell. Dust can be used as a replacement for a traditional command line shell, as a scripting language and as a data format. Dust is fast, efficient and easy to learn.

A basic dust program:

```dust
(output  "Hello world!")
```

Dust can do two (or more) things at the same time with effortless concurrency:

```dust
async {
    (output 'will this one finish first?')
    (output 'or will this one?')
}
```

Dust is an interpreted, general purpose language with first class functions. It is *data-oriented*, with extensive tools to manage structured and relational data. Dust also includes built-in tooling to import and export data in a variety of formats, including JSON, TOML, YAML and CSV.

<!--toc:start-->
- [Dust](#dust)
  - [Features](#features)
  - [Usage](#usage)
  - [Installation](#installation)
  - [The Dust Programming Language](#the-dust-programming-language)
    - [Declaring Variables](#declaring-variables)
    - [Lists](#lists)
    - [Maps](#maps)
    - [Tables](#tables)
    - [Functions](#functions)
    - [Concurrency](#concurrency)
  - [Implementation](#implementation)
<!--toc:end-->

## Features

- Simplicity: Dust is designed to be easy to learn.
- Speed: Dust is built on [Tree Sitter] and [Rust] to prioritize performance and correctness.
- Data format: Dust is data-oriented, making it a great language for defining data.
- Format conversion: Effortlessly convert between dust and formats like JSON, CSV and TOML.
- Structured data: Dust can represent data with more than just strings. Lists, maps and tables are easy to make and manage.

## Usage

Dust is an experimental project under active development. At this stage, features come and go and the API is always changing. It should not be considered for serious use yet.

To get help with the shell you can use the "help" tool.

```dust
(help)          # Returns a table will all tool info.
(help "random") # Returns a table with info on tools in the specified group.
```

## Installation

You must have the default rust toolchain installed and up-to-date. Install [rustup] if it is not already installed. Run `cargo install dust-lang` then run `dust` to start the interactive shell. Use `dust --help` to see the full command line options.

To build from source, clone the repository and build the parser. To do so, enter the `tree-sitter-dust` directory and run `tree-sitter-generate`. In the project root, run `cargo run` to start the shell. To see other command line options, use `cargo run -- --help`.

## The Dust Programming Language

It should not take long for a new user to learn the language, especially with the assistance of the shell. If your editor supports tree sitter, you can use [tree-sitter-dust] for syntax highlighting and completion support. Aside from this guide, the best way to learn dust is to read the examples and tests to get a better idea of what dust can do.

### Declaring Variables

Variables have two parts: a key and a value. The key is always a string. The value can be any of the following data types:

- string
- integer
- floating point value
- boolean
- list
- map
- table
- function

Here are some examples of variables in dust.

```dust
string = "The answer is 42."
integer = 42
float = 42.42
list = [1 2 string integer float] # Commas are optional when writing lists.
map = {
    key = 'value'
}
```

Note that strings can be wrapped with any kind of quote: single, double or backticks. Numbers are always integers by default. Floats are declared by adding a decimal. If you divide integers or do any kind of math with a float, you will create a float value.

### Lists

Lists are sequential collections. They can be built by grouping values with square brackets. Commas are optional. Values can be indexed by their position using dot notation with an integer. Dust lists are zero-indexed.

```dust
list = [true 41 "Ok"]

(assert_equal list.0 true)

the_answer = list.1 + 1

(assert_equal the_answer, 42) # You can also use commas when passing values to
                              # a function. 
```

### Maps

Maps are flexible collections with arbitrary key-value pairs, similar to JSON objects. Under the hood, all of dust's runtime variables are stored in a map so. A map is created with a pair of curly braces and its entries and just variables declared inside those braces. Map contents can be accessed using dot notation and a value's key.

```dust
reminder = {
    message = "Buy milk"
    tags = ["groceries", "home"]
}

(output reminder.message)
```

### Loops

A **while** loop continues until a predicate is false.

```dust
i = 0
while i < 10 {
    (output i)
    i += 1
}
```

A **for** loop operates on a list without mutating it or the items inside. It does not return a value.

```dust
list = [ 1, 2, 3 ]

for number in list {
    number += 1 # This modifies x *only* in this block scope
}

(output list)
# Output: [ 1 2 3 ]
# The original list is left unchanged.
```

To mutate the values in a list, use a **transform** loop, which returns a new modified list.

```dust
list = transform number in [1 2 3] {
    number - 1
}

(output list)
# Output: [ 0 1 2 ]
```

To filter out some of the values in a list, use a **filter** loop.

```dust
list = filter number in [1 2 3] {
    number >= 2
}

(output list)
# Output: [ 2 3 ]
```

A **find** loop will return a single value, the first item that satisfies the predicate.

```dust
found = find number in [1 2 1] {
    number != 1
}

(output found)
# Output: 2
```

### Tables

Tables are strict collections, each row must have a value for each column. If a value is "missing" it should be set to an appropriate value for that type. For example, a string can be empty and a number can be set to zero. Dust table declarations consist of a list of column names, which are identifiers enclosed in pointed braces. The column names are followed by a pair of curly braces filled with list values. Each list will become a row in the new table.

```dust
animals = table <name species age> {
    ["rover" "cat" 14]
    ["spot" "snake" 9]
    ["bob" "giraffe" 2]
}
```

Querying a table is similar to SQL.

```dust
names = select name from animals
youngins = select species from animals where age <= 10
```

The keywords `table` and `insert` make sure that all of the memory used to hold the rows is allocated at once, so it is good practice to group your rows together instead of using a call for each row.

```dust
insert into animals {
    ["eliza" "ostrich" 4]
    ["pat" "white rhino" 7]
    ["jim" "walrus" 9]
}

(assert_equal 6 (length animals))
```

### Functions

Functions are first-class values in dust, so they are assigned to variables like any other value. The function body is wrapped in single parentheses. To create a function, use the "function" keyword. The function's arguments are identifiers inside of a set of pointed braces and the function body is enclosed in curly braces. To call a fuction, invoke its variable name and use a set of curly braces to pass arguments (or leave them empty to pass nothing). You don't need commas when listing arguments and you don't need to add whitespace inside the function body but doing so may make your code easier to read.

```dust
say_hi = function <> {
    (output "hi")
}

add_one = function <number> {
    (number + 1)
}

(say_hi)
(assert_equal (add_one 3), 4)
```

This function simply passes the input to the shell's standard output.

```dust
print = function <input> {
    (output input)
}
```

### Concurrency

As a language written in Rust, Dust features effortless concurrency anywhere in your code.

```dust
async {
    await 1 + 1
}
```

The **await** keyword can be used in an asnyc block to indicate what value the async block should evaluate to. In this case, we want "data" to be read from a file.

```dust
data = async {
    (output "Reading a file...")
    (read "examples/assets/faithful.csv")
}

(output data)
```

## Implementation

Dust is formally defined as a Tree Sitter grammar in the tree-sitter-dust module. Tree sitter generates a parser, written in C, from a set of rules defined in Javascript. Dust itself is a rust binary that calls the C parser using FFI.

Tests are written in the Rust library, in Dust as implementation tests and in the Tree Sitter test format. Generally, features are added by implementing and testing the syntax in the tree-sitter-dust repository, then writing library tests to evaluate the new syntax. Implementation tests run the Dust files in the "examples" directory and should be used to demonstrate and verify that features work together.

Tree Sitter generates a concrete syntax tree, which dust traverses to create an abstract syntax tree that can run the dust code. The CST generation is an extra step but it allows easy testing of the parser, defining the language in one file and makes the syntax easy to modify and expand.

[dnf]: https://dnf.readthedocs.io/en/latest/index.html
[evalexpr]: https://github.com/ISibboI/evalexpr
[rustup]: https://rustup.rs
