# Dust

High-level programming language with effortless concurrency, automatic memory management and type
safety.

## Concepts

Dust is heavily influenced by Rust, but aims to be more high-level and easier to use. Its approach
to safety is similar in that it will refuse to run programs with issues that can be caught by
static analysis. However, Dust is not compiled and while performance is a concern, it is less
important than safety and ease of use.

### Effortless Concurrency

Rust promises *fearless* concurrency, and Dust takes it a step further by making concurrency as
simple as possible. Dust is organized into **statements**, and any sequence of statements can be
run concurrently by simply adding the `async` keyword before the block of statements.

```dust
# This function will count from 0 to 9, sleeping for an increasing amount of
# time between each number.
count_slowly = fn (
	multiplier: int,
) {
	i = 0

	while i < 10 {
		sleep_time = i * multiplier;

		thread.sleep(sleep_time)
		thread.write_line(i as str)

		i += 1
	}
}

async {
	count_slowly(200) # Finishes last
	count_slowly(100) # Finishes seconds
	count_slowly(50)  # Finishes first
}
```

### Automatic Memory Management

Dust uses a garbage collector to automatically manage memory. During the analysis phase, Dust will
determine the number of references to each value. During execution, the intepreter will check the
context after each statement and remove values that will not be used again.

```dust
x = 0 # x is assigned but never used
      # x is removed from memory

y = 41 # y is assigned
y + 1  # y is kept alive for this statement
       # y is removed from memory
```

### Type Safety

Dust is statically typed and null-free, but the type of a value can usually be inferred from its
usage. Dust will refuse to run programs with type errors, but will usually not require type
annotations.

```dust
# These two statements are identical to Dust
x = 1
x: int = 1

# Numbers with decimals are floats
y = 10.0
y: float = 10.0

# Strings are enclosed in single or double quotes and are guaranteed to be valid UTF-8
z = "Hello, world!"
z: string = "Hello, world!"
```

Aside from the ubiqutous `bool`, `int`, `float`, and `string` types, Dust also has `list`, `map`,
`range`, structures, enums and functions.
