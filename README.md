# Dust

<!--toc:start-->
- [Dust](#dust)
  - [Features](#features)
  - [Usage](#usage)
  - [Installation](#installation)
  - [The Dust Programming Language](#the-dust-programming-language)
    - [Variables and Data Types](#variables-and-data-types)
    - [Commands](#commands)
    - [Lists](#lists)
    - [Maps](#maps)
    - [Tables](#tables)
    - [The Yield Operator](#the-yield-operator)
    - [Functions](#functions)
    - [Time](#time)
<!--toc:end-->

Dust is a data-oriented programming language and interactive shell. Dust can be used as a replacement for a traditional command line shell, as a scripting language and as a tool create or manage data. Dust is expression-based, has first-class functions, lexical scope and lightweight syntax.

A basic dust program:

```dust
output "Hello world!"
```

Dust can do two (or more) things at the same time with effortless concurrency:

```dust
run(
    'output "will this one finish first?"',
    'output "or will this one?"'
)
```

Dust can do amazing things with data. To load CSV data, isolate a column and render it as a line plot in a GUI window:

```dust
read("examples/assets/faithful.csv")
    -> from_csv(input)
    -> get_rows(input)
    -> transform(input, 'input.1')
    -> plot(input)
```

## Features

- Data visualization: GUI (not TUI) plots, graphs and charts are available from directly within dust. No external tools are needed.
- Powerful tooling: Built-in commands reduce complex tasks to plain, simple code. You can even partition disks or install software.
- Pipelines: Like a pipe in bash, dust features the yield `->` operator.
- Format conversion: Effortlessly convert between dust and formats like JSON, CSV and TOML.
- Structured data: Dust can represent data with more than just strings. Lists, maps and tables are easy to make and manage.
- Developer tools: Dust has a complete tree sitter grammar, allowing syntax highlighting and completion in most code editors.

## Usage

Dust is an experimental project under active development. At this stage, features come and go and the API is always changing. It should not be considered for serious use yet.

## Installation

You must have the default rust toolchain installed and up-to-date. Clone the repository and run `cargo run` to start the interactive shell. To see other command line options, use `cargo run -- --help`.

## The Dust Programming Language

Dust is a hard fork of [evalexpr]; a simple expression language. Dust's core language features maintain this simplicity. But it can manage large, complex sets of data and perform complicated tasks through commands. It should not take long for a new user to learn the language, especially with the assistance of the shell.

If your editor supports tree sitter, you can use [tree-sitter-dust] for syntax highlighting and completion support.

### Variables and Data Types

Variables have two parts: a key and a value. The key is always a text string. The value can be any of the following data types:

- string
- integer
- floating point value
- boolean
- list
- map
- table
- function
- time
- empty

Here are some examples of variables in dust.

```dust
string = "The answer is 42.";
integer = 42;
float = 42.42;
list = (1, 2, string, integer, float);
map.key = "value";
empty = ();
```

### Commands

**Commands** are dust's built-in tools. Some of them can reconfigure your whole system while others are do very little. They may accept different inputs, or none at all. commands in the `random` group can be run without input, but the `random_integer` command can optionally take two numbers as in inclusive range.

```dust
die_roll = random_integer(1, 6);
d20_roll = random_integer(1, 20);
coin_flip = random_boolean();
```

Other commands can be found by pressing TAB in the interactive shell.

```dust
message = "I hate dust.";
replace(message, "hate", "love");
```

### Lists

Lists are sequential collections. They can be built by grouping values with parentheses and separating them with commas. Values can be indexed by their position to access their contents. Lists are used to represent rows in tables and most commands take a list as an argument.

```dust
list = (true, 42, "Ok");

assert_eq(list.0, true);
```

### Maps

Maps are flexible collections with arbitrary key-value pairs, similar to JSON objects. Under the hood, all of dust's runtime variables are stored in a map, so, as with variables, the key is always a string.

```dust
reminder.message = "Buy milk";
reminder.tags = ("groceries", "home");

json = to_json(reminder);
append_to_file(json, "info.txt");
```

### Tables

Tables are strict collections, each row must have a value for each column. Empty cells must be explicitly set to an empty value.

```dust
animals = create_table (
    ("name", "species", "age"),
    (
        ("rover", "cat", 14),
        ("spot", "snake", 9),
        ("bob", "giraffe", 2),
    )
);
```

Querying a table is similar to SQL.
  
```dust
names = select(animals, "name");
youngins = where(animals, 'age < 5');
old_species = select_where(animals, "species", 'age > 5')
```

The commands `create_table` and `insert` make sure that all of the memory used to hold the rows is allocated at once, so it is good practice to group your rows together instead of using a call for each row.

```dust
insert(
    animals,
    ("eliza", "ostrich", 4),
    ("pat", "white rhino", 7),
    ("jim", "walrus", 9)
);

assert_eq(length(animals.all), 6);

animals.by_name = sort_by(animals.all, "name");
```

### The Yield Operator

Like a pipe in bash, zsh or fish, the yield operator evaluates the expression on the left and passes it as input to the expression on the right. That input is always assigned to the **`input` variable** for that context. These expressions may simply contain a value or they can call a command or function that returns a value.

```dust
"Hello dust!" -> output(input)
```

This can be useful when working on the command line but to make a script easier to read or to avoid fetching the same resource multiple times, we can also declare variables. You should use `->` and variables together to write efficient, elegant scripts.

```dust
json = download("https://api.sampleapis.com/futurama/characters");
from_json(json)
    -> select(input, "name");
    -> get(input, 4)
```

### Functions

Functions are first-class values in dust, so they are assigned to variables like any other value. The function body is wrapped in single parentheses. To call a function, it's just like calling a command: simply pass it an argument or use an empty set of parentheses to pass an empty value.

In the function bod, the **`input` variable** represents whatever value is passed to the function when called.

```dust
say_hi = 'output "hi"';
add_one = 'input + 1';

say_hi();
assert_eq(add_one(3), 4);
```

This function simply passes the input to the shell's standard output.

```dust
print = 'output(input)';
```

Because functions are stored in variables, we can use collections like maps to
organize them.

```dust
math.add = 'input.0 + input.1';
math.subtract = 'input.0 - input.1';

assert_eq(math.add(2, 2), 4);
assert_eq(math.subtract(100, 1), 99);
```

### Time

Dust can record, parse and convert time values. Dust can parse TOML datetime
values or can create time values using commands.

```dust
dob = from_toml("1979-05-27T07:32:00-08:00")

output "Date of birth = " + local(dob);
```

```dust
time = now();

output "Universal time is " + utc(time);
output "Local time is " + local(time);
```

[dnf]: https://dnf.readthedocs.io/en/latest/index.html
[evalexpr]: https://github.com/ISibboI/evalexpr
