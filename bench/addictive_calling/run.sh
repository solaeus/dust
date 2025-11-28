#!/bin/sh

hyperfine \
    --shell none \
    --prepare 'sync' \
    --warmup 5 \
    'target/release/dust bench/addictive_calling/addictive_calling.ds' \
    'luajit bench/addictive_calling/addictive_calling.lua' \
    'bun bench/addictive_calling/addictive_calling.js' \
    'lua bench/addictive_calling/addictive_calling.lua' \
    'python bench/addictive_calling/addictive_calling.py' \
    'ruby bench/addictive_calling/addictive_calling.rb' \
    'php bench/addictive_calling/addictive_calling.php' \
    'perl bench/addictive_calling/addictive_calling.pl' \
    'Rscript bench/addictive_calling/addictive_calling.R' \
    'java bench/addictive_calling/addictive_calling.java' \
    'julia bench/addictive_calling/addictive_calling.jl' \
    'pypy bench/addictive_calling/addictive_calling.py' \
    'node bench/addictive_calling/addictive_calling.js' \
    'deno bench/addictive_calling/addictive_calling.js' \
    'clojure bench/addictive_calling/addictive_calling.clj'
