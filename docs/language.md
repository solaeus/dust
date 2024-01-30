# Dust Language Reference

!!! This is a **work in progress** and has incomplete information. !!!

This is an in-depth description of the syntax and abstractions used by the Dust language. It is not
necessary to read or understand all of it before you start using Dust. Instead, refer to it when
you need help with the syntax or understanding how the code is run.

Each section of this document corresponds to a node in the concrete syntax tree. Creating this tree
is the first step in interpreting Dust code. Second, the syntax tree is traversed and an abstract
tree is generated. Each node in the syntax tree corresponds to a node in the abstract tree. Third,
the abstract tree is verified to ensure that it will not generate any values that violate the type
restrictions. Finally, the abstract tree is run, beginning at the [root](#root).

You may reference the [grammar file](tree-sitter-dust/grammar.js) and the [Tree Sitter docs]
(https://tree-sitter.github.io/) while reading this guide to understand how the language is parsed.

<!--toc:start-->
- [Dust Language Reference](#dust-language-reference)
  - [Root](#root)
  - [Values](#values)
    - [Boolean](#boolean)
    - [Integer](#integer)
    - [Float](#float)
    - [Range](#range)
    - [String](#string)
    - [List](#list)
    - [Map](#map)
    - [Function](#function)
    - [Option](#option)
    - [Structure](#structure)
  - [Types](#types)
    - [Basic Types](#basic-types)
    - [Number](#number)
    - [Any](#any)
    - [None](#none)
    - [List Type](#list-type)
    - [Map Type](#map-type)
    - [Iter](#iter)
    - [Function Type](#function-type)
    - [Option Type](#option-type)
    - [Custom Types](#custom-types)
  - [Statements](#statements)
    - [Assignment](#assignment)
    - [Blocks](#blocks)
      - [Synchronous Blocks](#synchronous-blocks)
      - [Asynchronous Blocks](#asynchronous-blocks)
    - [Break](#break)
    - [For Loop](#for-loop)
    - [While Loop](#while-loop)
    - [If/Else](#ifelse)
    - [Match](#match)
    - [Pipe](#pipe)
    - [Expression](#expression)
  - [Expressions](#expressions)
      - [Identifier](#identifier)
      - [Index](#index)
      - [Logic](#logic)
      - [Math](#math)
      - [Value](#value)
      - [New](#new)
      - [Command](#command)
  - [Built-In Values](#built-in-values)
  - [Comments](#comments)
<!--toc:end-->

## Root

The root node represents all of the source code. It is a sequence of [statements](#statements) that
are executed synchronously, in order. The output of the program is always the result of the final
statement or the first error encountered.

## Values

There are ten kinds of value in Dust. Some are very simple and are parsed directly from the source
code, some are collections and others are used in special ways, like functions and structures. All
values can be assinged to an [identifier](#identifiers).

Dust does not have a null type. Absent values are represented with the `none` value, which is a
kind of [option](#option). You may not create a variable without a value and no variable can ever
be in an 'undefined' state during execution.

### Boolean

Booleans are true or false. They are represented by the literal tokens `true` and `false`.

### Integer

Integers are whole numbers that may be positive, negative or zero. Internally, an integer is a
signed 64-bit value.

```dust
42
```

Integers always **overflow** when their maximum or minimum value is reached. Overflowing means that
if the value is too high or low for the 64-bit integer, it will wrap around. You can use the built-
in values `int:max` and `int:min` to get the highest and lowest possible values.

```dust
assert_equal(int:max + 1, int:min)
assert_equal(int:min - 1, int:max)
```

### Float

A float is a numeric value with a decimal. Floats are 64-bit and, like integers, will **overflow**
at their bounds.

```dust
42.0
```

### Range

A range represents a contiguous sequence of integers. Dust ranges are **inclusive** so both the high
and low bounds will be represented.

```dust
0..100
```

### String

A string is a **utf-8** sequence used to represent text. Strings can be wrapped in single or double quotes as well as backticks.

```dust
'42'
"42"
`42`
'forty-two'
```

### List

A list is **collection** of values stored as a sequence and accessible by [indexing](#index) their position with an integer. Lists indexes begin at zero for the first item.

```dust
[ 42 'forty-two' ]
[ 123, 'one', 'two', 'three' ]
```

Note that the commas are optional, including trailing commas.

```dust
[1 2 3 4 5]:2
# Output: 3
```

### Map

Maps are flexible collections with arbitrary **key-value pairs**, similar to JSON objects. A map is
created with a pair of curly braces and its entries are variables declared inside those braces. Map
contents can be accessed using a colon `:`. Commas may optionally be included after the key-value
pairs.

```dust
reminder = {
    message = "Buy milk"
    tags = ["groceries", "home"]
}

reminder:message
# Output: Buy milk
```

Internally a map is represented by a B-tree. The implicit advantage of using a B-tree instead of a
hash map is that a B-tree is sorted and therefore can be easily compared to another. Maps are also
used by the interpreter as the data structure for holding variables. You can even inspect the active
**execution context** by calling the built-in `context()` function.

The map stores each [identifier](#identifiers)'s key with a value and the value's type. For internal
use by the interpreter, a type can be set to a key without a value. This makes it possible to check
the types of values before they are computed.

### Function

A function encapsulates a section of the abstract tree so that it can be run seperately and with
different arguments. The function body is a [block](#block), so adding `async` will cause the body
to run like any other `async` block. Unlike some languages, there are no concepts like futures or
async functions in Dust.

Functions are **first-class values** in Dust, so they can be assigned to variables like any other
value.

```dust
# This simple function has no arguments and no return value.
say_hi = () <none> {
    output("hi") # The "output" function is a built-in that prints to stdout.
}

# This function has one argument and will return a value.
add_one = (number <num>) <num> {
    number + 1
}

say_hi()
assert_equal(add_one(3), 4)
```

Functions can also be **anonymous**. This is useful for using **callbacks** (i.e. functions that are
called by another function).

```dust
# Use a callback to retain only the numeric characters in a string.
str:retain(
	'a1b2c3'
	(char <str>) <bool> {
		is_some(int:parse(char))
	}
)
```

### Option

An option represents a value that may not be present. It has two variants: **some** and **none**. 

```dust
say_something = (message <option(str)>) <str> {
    either_or(message, "hiya")
}

say_something(some("goodbye"))
# goodbye

say_something(none)
# hiya
```

Dust includes built-in functions to work with option values: `is_none`, `is_some` and `either_or`.

### Structure

A structure is a **concrete type value**. It is a value, like any other, and can be [assigned]
(#assignment) to an [identifier](#identifier). It can then be instantiated as a [map](#map) that
will only allow the variables present in the structure. Default values may be provided for each
variable in the structure, which will be propagated to the map it creates. Values without defaults
must be given a value during instantiation.

```dust
struct User {
    name <str>
    email <str>
    id <int> = generate_id()
}

bob = new User {
    name = "Bob"
    email = "bob@example.com"
}

# The variable "bob" is a structured map.
```

A map created by using [new](#new) is called a **structured map**. In other languages it may be
called a "homomorphic mapped type". Dust will generate errors if you try to set any values on the
structured map that are not allowed by the structure.

## Types

Dust enforces strict type checking. To make the language easier to write, **type inference** is used
to allow variables to be declared without specifying the type. Instead, the interpreter will figure
it out and set the strictest type possible.

To make the type-setting syntax easier to distinguish from the rest of your code, a **type
specification** is wrapped in pointed brackets. So variable assignment using types looks like this:

```dust
my_float <float> = 666.0
```

### Basic Types

The simple types, and their notation are:

- boolean `bool`
- integer `int`
- float `float`
- string `str`

### Number

The `num` type may represent a value of type `int` or `float`.

### Any

The `any` type does not enforce type bounds.

### None

The `none` type indicates that no value should be found after executing the statement or block, with
one expection: the `none` variant of the `option` type.

### List Type

A list's contents can be specified to create type-safe lists. The `list(str)` type would only allow
string values. Writing `list` without the parentheses and content type is equivalent to writing
`list(any)`.

### Map Type

The `map` type is unstructured and can hold any key-value pair.

### Iter

The `iter` type refers to types that can be used with a [for loop](#for-loop). These include `list`,
`range`, `string` and `map`.

### Function Type

A function's type specification is more complex than other types. A function value must always have
its arguments and return type specified when the **function value** is created.

```dust
my_function = (number <int>, text <str>) <none> {
    output(number)
    output(text)
}
```

But what if we need to specify a **function type** without creating the function value? This is
necessary when using callbacks or defining structures that have functions set at instantiation.

```dust
use_adder = (adder <(int) -> int>, number <int>) -> <int> {
    adder(number)
}

use_adder(
    (i <int>) <int> { i + 2 }
    40
)

# Output: 42
```

```dust
struct Message {
    send_n_times <(str, int) -> none>
}

stdout_message = new Message {
    send_n_times = (content <str>, n <int>) <none> {
        for _ in 0..n {
            output(content)
        }
    }
}
```

### Option Type

The `option(type)` type is expected to be either `some(value)` or `none`. The type of the value
inside the `some` is always specified.

```dust
result <option(str)> = none

for file in fs:read_dir("./") {
    if file:size > 100 {
        result = some(file:path)
        break
    }
}

output(result)
```

```dust
get_line_break_index(text <str>) <some(int)> {
    str:find(text, '\n')
}
```

### Custom Types

Custom types such as **structures** are referenced by their variable identifier.

```dust
File = struct {
    path <str>
    size <int>
    type <str>
}

print_file_info(file <File>) <none> {
  	info = file:path
    		+ '\n'
    		+ file:size
    		+ '\n' 
    		+ file:type
		
  	output(info)
}
```

## Statements

TODO

### Assignment

TODO

### Blocks

TODO

#### Synchronous Blocks

TODO

#### Asynchronous Blocks

```dust
# An async block will run each statement in its own thread.
async {
    output(random_integer())
    output(random_float())
    output(random_boolean())
}
```

```dust
data = async {
    output("Reading a file...")
    read("examples/assets/faithful.csv")
}
```

### Break

TODO

### For Loop

TODO

```dust
list = [ 1, 2, 3 ]

for number in list {
    output(number + 1)
}
```

### While Loop

TODO

A **while** loop continues until a predicate is false.

```dust
i = 0
while i < 10 {
    output(i)
    i += 1
}
```

### If/Else

TODO

### Match

TODO

### Pipe

TODO

### Expression

TODO

## Expressions

TODO

#### Identifier

TODO

#### Index

TODO

#### Logic

TODO

#### Math

TODO

#### Value

TODO

#### New

TODO

#### Command

TODO

## Built-In Values

TODO

## Comments

TODO
