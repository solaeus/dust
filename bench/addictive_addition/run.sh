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
    'java -cp bench/addictive_addition AddictiveAddition' \
    'julia bench/addictive_addition/addictive_addition.jl' \
    'pypy bench/addictive_addition/addictive_addition.py' \
    'node bench/addictive_addition/addictive_addition.js' \
    'bun bench/addictive_addition/addictive_addition.js' \
    'deno bench/addictive_addition/addictive_addition.js' \
    'clojure bench/addictive_addition/addictive_addition.clj'
