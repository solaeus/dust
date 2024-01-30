# The Dust Programming Language

<!--toc:start-->
- [The Dust Programming Language](#the-dust-programming-language)
  - [Value](#value)
    - [Boolean](#boolean)
    - [Integer](#integer)
    - [Float](#float)
    - [String](#string)
    - [List](#list)
    - [Maps](#maps)
    - [Function](#function)
    - [Option](#option)
  - [Types](#types)
  - [Loops](#loops)
    - [While](#while)
    - [For/Async For](#forasync-for)
  - [Concurrency](#concurrency)
<!--toc:end-->

Dust is a general purpose, interpreted and strictly typed language with first-class functions. This guide is an in-depth description of the abstractions and concepts that are used to implement the language.

Dust aims to be panic-free. That means that the interpreter will only fail to run a program due to and intended error, such as a type error or a syntax error.

## Values

There are ten kinds of value in Dust. Some are very simple and are parsed directly from the source code, some are collections and others are used in special ways, like functions and structures. All values can be assinged to an [identifier][].

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

Dust enforces strict type checking, but you don't usually need to write the type, dust can figure it out on its own. The **number** and **any** types are special types that allow you to relax the type bounds.

```dust
string <str> = "foobar"
integer <int> = 42
float <float> = 42.42

numbers <[number]> = [integer float]

stuff <[any]> = [string integer float]
```

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

## Concurrency

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
