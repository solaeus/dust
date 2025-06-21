#!/bin/sh
hyperfine \
    --shell none \
    --prepare 'sync' \
    --warmup 5 \
    'target/release/dust bench/addictive_addition/addictive_addition.ds' \
    'node bench/addictive_addition/addictive_addition.js' \
    'deno bench/addictive_addition/addictive_addition.js' \
    'bun bench/addictive_addition/addictive_addition.js' \
    'python bench/addictive_addition/addictive_addition.py' \
    'lua bench/addictive_addition/addictive_addition.lua' \
    'luajit bench/addictive_addition/addictive_addition.lua' \
    'ruby bench/addictive_addition/addictive_addition.rb' \
    'java bench/addictive_addition/addictive_addition.java'
