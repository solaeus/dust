# Dust

High-level programming language with effortless concurrency, automatic memory management and type
safety.

Dust is a work in progress. Because it aims to deliver a high level of safety, extensive testing
is required. The language is still in the design phase, and the syntax is subject to change.

## Usage

The Dust command line tool can be used to run Dust programs. It is not yet available outside of
this repository.

```sh
cargo run --package dust-shell -- examples/hello_world.ds
```

```sh
cargo run --package dust-shell -- -c '"Hello my name is " + read_line() + "!"'
```

Dust is easily embedded in another program. You can run a dust program of any size or complexity
with a single function.

```rust
use dust_lang::{run, Value};

fn main() {
    let code = "
        let x = 'Dust'
        let y = ' is awesome!'

        write_line(x + y)

        42
    ";

    let result = run(code);

    assert_eq!(result, Ok(Some(Value::integer(42))));
}
```

## Concepts

### Effortless Concurrency

Dust makes concurrency as effortless as possible. Dust is organized into **statements**, and any
sequence of statements can be run concurrently by simply adding the `async` keyword before the block
of statements.

```rust
// Counts from 0 to 9, sleeping for an increasing amount of time between each.
let count_slowly = fn (multiplier: int) {
    i = 0

    while i < 10 {
        sleep(i * multiplier)
        write_line(i.to_string())

        i += 1
    }
}

async {
    count_slowly(200) // Finishes last
    count_slowly(100) // Finishes second
    count_slowly(50)  // Finishes first
}
```

### Automatic Memory Management

Dust uses a garbage collector to automatically manage memory.

```rust
let x = 0     // x is assigned but never used
              // x is removed from memory

let y = 41    // y is assigned
let z = y + 1 // y is kept alive for this statement
              // y is removed from memory

write_line(z) // z is kept alive for this statement
              // z is removed from memory
```

### Type Safety

Dust is statically typed and null-free, but the type of a value can usually be inferred from its
usage. Dust will refuse to run programs with type errors, but will usually not require type
annotations.

```rust
// These two statements are identical to Dust
let x = 1
let x: int = 1

// Numbers with decimals are floats
let y = 10.0
let y: float = 10.0

// Strings are enclosed in double quotes and are guaranteed to be valid UTF-8
let z = "Hello, world!"
let z: str = "Hello, world!"
```

Aside from the ubiqutous `bool`, `int`, `float`, and `str` types, Dust also has lists, maps,
ranges, structures, enums and functions.
