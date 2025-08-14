#!/bin/sh

hyperfine \
    --shell none \
    --prepare 'sync' \
    --warmup 5 \
    'lua bench/addictive_addition/addictive_addition.lua' \
    'python bench/addictive_addition/addictive_addition.py' \
    'ruby bench/addictive_addition/addictive_addition.rb' \
    'php bench/addictive_addition/addictive_addition.php' \
    'perl bench/addictive_addition/addictive_addition.pl' \
    'Rscript bench/addictive_addition/addictive_addition.R' \
    'target/release/dust bench/addictive_addition/addictive_addition.ds' \
    'luajit bench/addictive_addition/addictive_addition.lua' \
    'java bench/addictive_addition/addictive_addition.java' \
    'julia bench/addictive_addition/addictive_addition.jl' \
    'pypy bench/addictive_addition/addictive_addition.py' \
    'node bench/addictive_addition/addictive_addition.js' \
    'bun bench/addictive_addition/addictive_addition.js' \
    'deno bench/addictive_addition/addictive_addition.js' \
    'clojure bench/addictive_addition/addictive_addition.clj' \
    'nu bench/addictive_addition/addictive_addition.nu' \
    'rhai-run bench/addictive_addition/addictive_addition.rhai'
