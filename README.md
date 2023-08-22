# Dust

Dust is a data-oriented programming language and interactive shell. The command-line interface can also be run as a singular command or it can run a file. Dust is minimal, easy to read and easy to learn by example. Your code will always do exactly what it looks like it's going to do.

A basic dust program:

```dust
output "Hello world!"
```

Dust can do two things at the same time:

```dust
async (
    'output "will this one finish first?"',
    'output "or will this one?"'
)
```

Display CSV data as a line plot in a GUI window:

```dust
read("examples/assets/faithful.csv")
    -> from_csv(input)
    -> get_rows(input)
    -> transform(input, 'input:get(1)')
    -> plot(input)
```

## Features

### Data Visualization

Aside from downloading and processing data, dust is able to display it. Unfortunately the command line is a text interface that struggles to render data in the form of charts and graphs. That's why dust is able to instantly spin up GUI windows with beautifully rendered data. Plots, graphs and charts are available from directly within dust. No external tools are needed.

[bar_graph_demo.webm](https://github.com/solaeus/whale/assets/112188538/deba6e3c-35d4-47e9-9db9-2045ff2e7c9c)

### Powerful Tooling

Built-in tools called **macros** reduce complex tasks to plain, simple code.

```whale
download "https://api.sampleapis.com/futurama/cast"
```

### Pipelines

Like a pipe in bash, zsh or fish, the yield operator `->` evaluates the expression on the left and passes it as input to the expression on the right. That input is always assigned to the **`input` variable** for that context. These expressions may simply contain a value or they can call a macro or function that returns a value.

```whale
download "https://api.sampleapis.com/futurama/cast" -> output(input)
```
 
### Format conversion

Effortlessly convert between whale and formats like JSON, CSV and TOML.

```whale
download "https://api.sampleapis.com/futurama/cast"
  -> from_json(input)
  -> to_csv(input)
```

### Structured data

Unlike a traditional command line shell, whale can represent data with more than just strings. Lists, maps and tables are everywhere in whale. When you pull in external data, it is easy to deserialize it into whale.

```whale
download "https://api.sampleapis.com/futurama/cast"
  -> input:from_json()
  -> input:get(0)
  -> input.name
```

### Disk Management

Whale scripts are clear and easy-to-maintain. You can manage disks with sets of key-value pairs instead of remembering positional arguments.

```
new_disk.name = "My Files"
new_disk.filesystem = "btrfs";
new_disk.path = "/dev/sdb";
new_disk.label = "gpt";
new_disk.range = (0, 8000);

new_disk:partition();
```

### First-Class Functions

Assign custom functions to variables. Functions can return any value or none at all. Use functions to build structured data or automate tasks.

```whale
User = '
  this.name = input.0;
  this.age = input.1;
  this
';

user_0 = User("bob", "44");
user_1 = User("mary", "77");
```

This "fetch" function will download JSON data and parse it.

```
fetch = '
  raw_data = download(input);
  from_json(raw_data)
';
```

## Usage

Dust is an experimental project under active development. At this stage, features come and go and the API is always changing. It should not be considered for serious use yet.

## Installation

You must have the default rust toolchain installed and up-to-date. Clone the repository and run `cargo run` to start the interactive shell. To see other command line options, use `cargo run -- --help`.

## The Whale Programming Language

Dust is a hard fork of [evalexpr]; a simple expression language. Dust's core language features maintain this simplicity. But it can manage large, complex sets of data and perform complicated tasks through macros. It should not take long for a new user to learn the language, especially with the assistance of the shell.

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

```whale
x = 1;
y = "hello, it is " + now().time;
z = "42.42";

list = (3, 2, x);
big_list = (x, y, z, list)
map.x = "foobar";
function = 'output "I'm a function"';
```

### Macros

**Macros** are dust's built-in tools. Some of them can reconfigure your whole system while others are do very little. They may accept different inputs, or none at all. Macros in the `random` group can be run without input, but the `random_integer` macro can optionally take two numbers as in inclusive range.

```whale
die_roll = random_integer(1, 6);
d20_roll = random_integer(1, 20);
coin_flip = random_boolean();
```

Other macros can be found by pressing TAB in the interactive shell.

```whale
message = "I hate dust.";
replace(message, "hate", "love");
```

### Lists

Lists are sequential collections. They can be built by grouping values with parentheses and separating them with commas. Values can be indexed by their position to access their contents. Lists are used to represent rows in tables and most macros take a list as an argument.

```whale
list = (true, 42, "Ok");

assert_eq(get(list, 0), true);
```

### Maps

Maps are flexible collections with arbitrary key-value pairs, similar to JSON objects. Under the hood, all of dust's runtime variables are stored in a map, so, as with variables, the key is always a string.

```whale
reminder.message = "Buy milk";
reminder.tags = ("groceries", "home");

json = to_json(reminder);
append_to_file(json, "info.txt");
```

### Tables

Tables are strict collections, each row must have a value for each column. Empty cells must be explicitly set to an empty value. Querying a table is similar to SQL.

```whale
animals.all = create_table (
    ("name", "species", "age"),
    ("rover", "cat", 14),
    ("spot", "snake", 9),
    ("bob", "giraffe", 2),
);
```

The macros `create_table` and `insert` make sure that all of the memory used to hold the rows is allocated at once, so it is good practice to group your rows together instead of using a call for each row.

```whale
animals.all:insert(
    ("eliza", "ostrich", 4),
    ("pat", "white rhino", 7),
    ("jim", "walrus", 9)
);

assert_eq(animals:length(), 6);

animals.by_name = animals:sort_by("name");
```

### The Yield Operator

Like a pipe in bash, zsh or fish, the yield operator evaluates the expression on the left and passes it as input to the expression on the right. That input is always assigned to the **`input` variable** for that context. These expressions may simply contain a value or they can call a macro or function that returns a value.

```whale
"Hello dust!" -> output(input)
```

This can be useful when working on the command line but to make a script easier to read or to avoid fetching the same resource multiple times, we can also declare variables. You should use `->` and variables together to write short, elegant scripts.

```whale
json = download("https://api.sampleapis.com/futurama/characters");
from_json(json)
    -> select(input, "name");
    -> get(input, 4)
```

### Functions

Functions are first-class values in dust, so they are assigned to variables like any other value. The function body is wrapped in single parentheses. To call a function, it's just like calling a macro: simply pass it an argument or use an empty set of parentheses to pass an empty value.

In the function bod, the **`input` variable** represents whatever value is passed to the function when called.

```whale
say_hi = 'output "hi"';
add_one = 'input + 1';

assert_eq(add_one(3), 4);
say_hi();
```

This function simply passes the input to the shell's standard output.

```whale
print = 'output(input)';
```

Because functions are stored in variables, we can use collections like maps to
organize them.

```whale
math.add = 'input.0 + input.1';
math.subtract = 'input.0 - input.1';

assert_eq(math.add(2, 2), 4);
assert_eq(math.subtract(100, 1), 99);
```

### Time

Whale can record, parse and convert time values. Whale can parse TOML datetime
values or can create time values using macros.

```whale
dob = from_toml("1979-05-27T07:32:00-08:00")

output "Date of birth = " + local(dob);
```

```whale
time = now();

output "Universal time is " + time:utc();
output "Local time is " + time:local();
```

[dnf]: https://dnf.readthedocs.io/en/latest/index.html
[evalexpr]: https://github.com/ISibboI/evalexpr
