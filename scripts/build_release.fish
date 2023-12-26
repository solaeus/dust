#!/bin/fish
# Build the project in release mode.

cd tree-sitter-dust/
tree-sitter generate --no-bindings

cd ..
cargo build --release
