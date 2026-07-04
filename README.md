# Dust

Programming language focused on correctness, performance and ease of use.

```rust
enum Language {
    C,
    Dust,
    JavaScript,
    Lua,
    Python,
    Rust,
}

struct Coder {
    name: String,
    favorite_language: Language,
}

impl Coder {
    fn say_hi(self) {
        io::print_line(string::format(
            "Hi! I'm {} and I love {}!",
            self.name,
            self.favorite_language,
        ));
    }

    fn learn_dust(mut self) {
        io::print_line(string::format("{} is learning Dust...", self.name));

        self.favorite_language = Language::Dust;
    }
}

fn main() {
    let alice = Coder {
        name: "Alice",
        favorite_language: Language::Lua,
    };

    alice.say_hi();
    alice.learn_dust();
    alice.say_hi();
}
```

> [!IMPORTANT]
> 🧪 💡 ⚗️
>
> Dust is still experimental.

Development is active and, while many aspects of the implementation are stable, research is ongoing
into design optimizations and performance improvements.

## Overview

The goal of this project is, simply put, to deliver power and safety in a language that is easy
to learn. Source code is compiled to compact bytecode that runs in a portable and fast virtual
machine. Built on the latest research, lessons learned from other languages and a great deal of
experimentation, Dust offers a unique combination of features.

### Ease-of-use

Dust delivers the power of Rust's type system in a language with **no lifetimes and no borrow
checker**. Your mental energy should be spent on the problem you're solving, not struggling to
understand language mechanics.

### Correctness

Dust has a static, nominal type system. No gradual typing. No duck typing. If a program compiles, it
behaves as expected in every code path. Dust uses a from-scratch reimplementation of the Rust type
system that was largely influenced by parts of the `rustc` code, but built specifically for Dust.

### Performance

The Dust compiler is built on cache-friendly data structures at every level. The entire compiler as
well as individual components like the lexer and parser are benchmarked for performance to inform
development decisions. 

As a bytecode-compiled language, Dust uses a custom instruction format for its register-based
virtual machine. Register-based VMs are an emerging technology. While there are examples in
production languages like Lua, Dust may be the first to use static data types.

## Contributing

In order to innovate, it is necessary to have both a deep understanding of the algorithms at work
and a close familiarity with the code itself. That can only be gained by actually writing it. This
project has found success in using LLM tools to write and maintain tests based on human-written
examples. Beyond test generation and edit predictions, using LLM tools would hinder innovation. This
is currently a solo project but any future contributors would be expected to respect that reasoning
and make their edits personally. It is not a goal of the project to churn out lots of features right
away. Good languages are built on a solid foundation and carefully maintained by the humans who know
them best.
