#!/bin/sh

hyperfine \
    --shell none \
    --prepare 'sync' \
    --warmup 1 \
    'target/release/dust bench/sustained_addition/sustained_addition.ds' \
    'luajit bench/sustained_addition/sustained_addition.lua' \
    'bun bench/sustained_addition/sustained_addition.js' \
    'lua bench/sustained_addition/sustained_addition.lua' \
    'python bench/sustained_addition/sustained_addition.py' \
    'ruby bench/sustained_addition/sustained_addition.rb' \
    'php bench/sustained_addition/sustained_addition.php' \
    'perl bench/sustained_addition/sustained_addition.pl' \
    'Rscript bench/sustained_addition/sustained_addition.R' \
    'java bench/sustained_addition/sustained_addition.java' \
    'julia bench/sustained_addition/sustained_addition.jl' \
    'pypy bench/sustained_addition/sustained_addition.py' \
    'node bench/sustained_addition/sustained_addition.js' \
    'deno bench/sustained_addition/sustained_addition.js' \
    'clojure bench/sustained_addition/sustained_addition.clj'
