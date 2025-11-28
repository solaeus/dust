#!/bin/sh

hyperfine \
    --shell none \
    --prepare 'sync' \
    --warmup 5 \
    'target/release/dust bench/fibonacci/fibonacci.ds' \
    'luajit bench/fibonacci/fibonacci.lua' \
    'bun bench/fibonacci/fibonacci.js' \
    'lua bench/fibonacci/fibonacci.lua' \
    'python bench/fibonacci/fibonacci.py' \
    'ruby bench/fibonacci/fibonacci.rb' \
    'php bench/fibonacci/fibonacci.php' \
    'perl bench/fibonacci/fibonacci.pl' \
    'Rscript bench/fibonacci/fibonacci.R' \
    'java bench/fibonacci/fibonacci.java' \
    'julia bench/fibonacci/fibonacci.jl' \
    'pypy bench/fibonacci/fibonacci.py' \
    'node bench/fibonacci/fibonacci.js' \
    'deno bench/fibonacci/fibonacci.js' \
    'clojure bench/fibonacci/fibonacci.clj'
