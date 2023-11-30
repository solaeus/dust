#!/usr/bin/fish
# Build the project in debug mode.

cd tree-sitter-dust/
tree-sitter generate --debug-build --no-bindings

cd ..
cargo build
