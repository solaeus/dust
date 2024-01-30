# The Dust Programming Language

!!! This is a **work in progress** and has incomplete information. !!!

<!--toc:start-->
- [The Dust Programming Language](#the-dust-programming-language)
  - [Values](#values)
    - [Boolean](#boolean)
    - [Integer](#integer)
    - [Float](#float)
    - [String](#string)
    - [List](#list)
    - [Maps](#maps)
    - [Function](#function)
    - [Option](#option)
    - [Structure](#structure)
  - [Types](#types)
    - [Basic Types](#basic-types)
    - [Number](#number)
    - [Any](#any)
    - [None](#none)
    - [List and List Contents](#list-and-list-contents)
    - [Unstructured Map](#unstructured-map)
    - [Function](#function)
    - [Option](#option)
    - [Structures](#structures)
  - [Identifiers](#identifiers)
  - [Assignment](#assignment)
  - [Loops](#loops)
    - [While](#while)
    - [For/Async For](#forasync-for)
  - [Blocks](#blocks)
    - [Synchronous Blocks](#synchronous-blocks)
    - [Asynchronous Blocks](#asynchronous-blocks)
<!--toc:end-->

Dust is a general purpose, interpreted and strictly typed language with first-class functions. This guide is an in-depth description of the abstractions and concepts that are used to implement the language.

Dust aims to be panic-free. That means that the interpreter will only fail to run a program due to an intended error, such as a type error or a syntax error.

## Values

There are ten kinds of value in Dust. Some are very simple and are parsed directly from the source code, some are collections and others are used in special ways, like functions and structures. All values can be assinged to an [identifier][].

Dust does not have a null type. Absent values are represented with the `none` value, which is a kind of [option](#option). You may not create a variable without a value and no variable can ever be in an 'undefined' state during execution.

### Boolean

Booleans are true or false. They are represented by the literal tokens `true` and `false`.

### Integer

Integers are whole numbers that may be positive, negative or zero. Internally, each integer is a signed 64-bit value. Integers always **overflow** when their maximum or minimum value is reached. Overflowing means that if the value is too high or low for the 64-bit integer, it will wrap around. So `maximum_value + 1` yields the minimum value and `minimum_value - 1` yields the maximum value.

```dust
42
```

### Float

A float is a numeric value with a decimal. Floats are 64-bit and, like integers, will **overflow** at their bounds.

```dust
42.0
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

A list is **collection** of values stored as a sequence and accessible by indexing their position with an integer. Lists indexes begin at zero for the first item.

```dust
[ 42 'forty-two' ]
[ 123, 'one', 'two', 'three' ]
```

Note that the commas are optional, including trailing commas.

```dust
[1 2 3 4 5]:2
# Output: 3
```

### Maps

Maps are flexible collections with arbitrary **key-value pairs**, similar to JSON objects. A map is created with a pair of curly braces and its entries are variables declared inside those braces. Map contents can be accessed using a colon `:`. Commas may optionally be included after the key-value pairs.

```dust
reminder = {
    message = "Buy milk"
    tags = ["groceries", "home"]
}

reminder:message
# Output: Buy milk
```

Internally a map is represented by a b-tree. The implicit advantage of using a b-tree instead of a hash map is that a b-tree is sorted and therefore can be easily compared to another. Maps are also used by the interpreter as the data structure for a **[context][]**.

The map stores an [identifier][]'s key, the value it represents and the value's type. For internal use by the interpreter, a type can be set to a key without a value. This makes it possible to check the types of values before they are computed.

### Function

Functions are first-class values in Dust, so they are assigned to variables like any other value.

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

You don't need commas when listing arguments and you don't need to add whitespace inside the function body but doing so may make your code easier to read.

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

A structure is an **concrete type value**. It is a value, like any other, and can be [assigned](#assignment) to an [identifier](). It can also be instantiated as a [map]() that will only allow the variables present in the structure. Default values may be provided for each variable in the structure, which will be propagated to the map it creates. Values without defaults must be given a value during instantiation.

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

A map created by using [new]() is called a **structured map**. In other languages it may be called a "homomorphic mapped type". Dust will generate errors if you try to set any values on the structured map that are not allowed by the structure.

## Types

Dust enforces strict type checking. To make the language easier to write, **type inference** is used to allow variables to be declared without specifying the type. Instead, the interpreter will figure it out and set the strictest type possible.

To make the type-setting syntax easier to distinguish from the rest of your code, a **type specification** is wrapped in pointed brackets. So variable assignment using types looks like this:

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

The `number` type may represent a value of type `int` or `float`.

### Any

The `any` type does not enforce type bounds.

### None

The `none` type indicates that no value should be found after executing the statement or block, with one expection: the `none` variant of the `option` type.

### List and List Contents

### Unstructured Map

### Function

A function's type specification is more complex than other types. A function value must always have its arguments and return type specified when the **function value** is created.

```dust
my_function = (number <int>, text <str>) <none> {
    output(number)
    output(text)
}
```

But what if we need to specify a **function type** without creating the function value? This is necessary when using callbacks or defining structures that have functions set at instantiation.

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

### Option and None

### Structures

## Identifiers

## Assignment

## Loops

### While

A **while** loop continues until a predicate is false.

```dust
i = 0
while i < 10 {
    output(i)
    i += 1
}
```

### For/Async For

A **for** loop operates on a list without mutating it or the items inside. It does not return a value.

```dust
list = [ 1, 2, 3 ]

for number in list {
    output(number + 1)
}
```

## Blocks

### Synchronous Blocks

### Asynchronous Blocks

Dust features effortless concurrency anywhere in your code. Any block of code can be made to run its contents asynchronously. Dust's concurrency is written in safe Rust and uses a thread pool whose size depends on the number of cores available.

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
