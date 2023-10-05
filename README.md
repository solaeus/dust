# Dust

Dust is a data-oriented programming language and interactive shell. Dust can be used as a replacement for a traditional command line shell, as a scripting language and as a tool create or manage data. Dust is expression-based, has first-class functions, lexical scope and lightweight syntax. Dust's grammar is formally defined in code and its minimalism is in large part due to its tree sitter parser, which is lightning-fast, accurate and thoroughly tested.

A basic dust program:

```dust
output <"Hello world!">
```

Dust can do two (or more) things at the same time with effortless concurrency:

```dust
run <
    function { output 'will this one finish first?' }
    function { output 'or will this one?' }
>
```

Dust can do amazing things with data. To load CSV data, isolate a column and render it as a line plot in a GUI window:

```dust
read_file("examples/assets/faithful.csv")
    -> from_csv
    -> rows
    -> transform <{item.1}>
    -> plot
```

<!--toc:start-->
- [Dust](#dust)
  - [Features](#features)
  - [Usage](#usage)
  - [Installation](#installation)
  - [Implementation](#implementation)
  - [Contributing](#contributing)
  - [The Dust Programming Language](#the-dust-programming-language)
    - [Declaring Variables](#declaring-variables)
    - [Integers and Floats](#integers-and-floats)
    - [Lists](#lists)
    - [Maps](#maps)
    - [Tables](#tables)
    - [The Yield Operator](#the-yield-operator)
    - [Functions](#functions)
    - [Empty Values](#empty-values)
<!--toc:end-->

## Features

- Simplicity: Dust is designed to be easy to learn and powerful to use, without compromising either.
- Speed: Dust is built on [Tree Sitter] and [Rust] to prioritize performance and correctness.
- Data format: Dust is data-oriented, so first and foremost it makes a great language for defining data.
- Pipelines: Like a pipe in bash, dust features the yield `->` operator.
- Format conversion: Effortlessly convert between dust and formats like JSON, CSV and TOML.
- Structured data: Dust can represent data with more than just strings. Lists, maps and tables are easy to make and manage.

## Usage

Dust is an experimental project under active development. At this stage, features come and go and the API is always changing. It should not be considered for serious use yet.

To get help with the shell you can use the "help" tool.

```dust
help            # Returns a table will all tool info.
help <"random"> # Returns a table with info on tools in the specified group.
# The above is simply a shorthand for this:
help -> where(input, function <tool> { tool == "random" })    
```

## Installation

You must have the default rust toolchain installed and up-to-date. Install [rustup] if it is not already installed. Run `cargo install dust-lang` then run `dust` to start the interactive shell. Use `dust --help` to see the full command line options.

To build from source, clone the repository and run `cargo run` to start the shell. To see other command line options, use `cargo run -- --help`.

## Implementation

Dust is formally defined as a Tree Sitter grammar in the tree-sitter-dust module. Tree sitter generates a parser, written in C, from a set of rules defined in JavaScript. Dust itself is a rust binary that calls the C parser using FFI. Dust does not use Javascript at runtime.

Tree Sitter generates a concrete syntax tree, which the Rust code maps to an abstract syntax tree by traversing each node once. Tree sitter is fast enough to be updated on every keystroke which is perfect for a data-oriented language like Dust because it allows only the relevant sections to be re-evaluated and the result displayed instantly.

## Contributing

Please submit any thoughts or suggestions for this project. For instructions on the internal API, see the library documentation. Implementation tests are written in dust and are run by a corresponding rust test so dust tests will be run when `cargo test` is called.

## The Dust Programming Language

It should not take long for a new user to learn the language, especially with the assistance of the shell. If your editor supports tree sitter, you can use [tree-sitter-dust] for syntax highlighting and completion support. Aside from this guide, the best way to learn dust is to read the examples and tests to get a better idea of what dust can do.

### Declaring Variables

Variables have two parts: a key and a value. The key is always a text string. The value can be any of the following data types:

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
list = (1 2 string integer float)
map = {
    key = `value`
}
```

Note that strings can be wrapped with any kind of quote: single, double or backticks. Numbers are always integers by default. And commas are optional in lists.

### Integers and Floats

Integer and floating point values are dust's numeric types. Any whole number (i.e. without a decimal) is an integer. Floats are declared by adding a single decimal to or number. If you divide integers or do any kind of math with a float, you will create a float value.

### Lists

Lists are sequential collections. They can be built by grouping values with square brackets. Commas are optional. Values can be indexed by their position to access their contents. Their contents can be indexed using dot notation with an integer. Dust lists are zero-indexed.

```dust
list = [true 41 "Ok"]

assert_equal <list.0 true>

the_answer = list.1 + 1

assert_equal <the_answer, 42>
```

### Maps

Maps are flexible collections with arbitrary key-value pairs, similar to JSON objects. Under the hood, all of dust's runtime variables are stored in a map, so, as with variables, the key is always a string. A map is created with a pair of curly braces and its entries and just variables declared inside those braces. Map contents can be accessed using dot notation and a value's key.

```dust
reminder = {
    message = "Buy milk"
    tags = ("groceries", "home")
}

output <reminder.message>
```

### Tables

Tables are strict collections, each row must have a value for each column. If a values is "missing" it should be set to an appropriate value for that type. For example a string can be empty and a number can be set to zero. Dust table declarations consist of a list of column names, which are identifiers enclosed in pointed braces. The column names are followed by a pair of curly braces filled with list values. Each list will become a row in the new table.

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

The commands `create_table` and `insert` make sure that all of the memory used to hold the rows is allocated at once, so it is good practice to group your rows together instead of using a call for each row.

```dust
insert into animals
    ["eliza" "ostrich" 4]
    ["pat" "white rhino" 7]
    ["jim" "walrus" 9]

assert_equal(count(animals.all), 6);

sorted = sort(animals);
```

### The Yield Operator

Like a pipe in bash, zsh or fish, the yield operator evaluates the expression on the left and passes it as input to the expression on the right. That input is always assigned to the **`input` variable** for that context. These expressions may simply contain a value or they can call a command or function that returns a value.

```dust
"Hello dust!" -> output <input>
```

This can be useful when working on the command line but to make a script easier to read or to avoid fetching the same resource multiple times, we can also declare variables. You should use `->` and variables together to write efficient, elegant scripts.

```dust
json = download("https://api.sampleapis.com/futurama/characters");
from_json(json)
    -> select(input, "name");
    -> input.4
```

### Functions

Functions are first-class values in dust, so they are assigned to variables like any other value. The function body is wrapped in single parentheses. To create a function, use the "function" keyword. The function's arguments are identifiers inside of a set of pointed braces and the function body is enclosed in curly braces. To call a fuction, invoke its variable name and use a set of pointed braces to pass arguments (unless it has no arguments).

```dust
say_hi = function <> {
    output <"hi">
}

add_one = function <number> {
    number + 1
}

say_hi
assert_equal <add_one(3), 4>
```

This function simply passes the input to the shell's standard output.

```dust
print = function <input> {
    output<input>
}
```

### Empty Values

Dust does not have a null type. Instead, it uses the "empty" type to represent a lack of any other value. There is no syntax to create this value: it is only used by the interpreter. Note that Dust does have the NaN value, which is a floating point value that must exist in order for floats to work as intended. Integers will never be NaN and no value will ever be null or undefined.

[dnf]: https://dnf.readthedocs.io/en/latest/index.html
[evalexpr]: https://github.com/ISibboI/evalexpr
[rustup]: https://rustup.rs
