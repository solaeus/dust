hyperfine \
    --shell none \
    --prepare 'sync' \
    --warmup 5 \
    '../target/release/dust assets/count_to_one_million.ds' \
    'node assets/count_to_one_million.js' \
    'deno assets/count_to_one_million.js' \
    'python assets/count_to_one_million.py' \
    'lua assets/count_to_one_million.lua'
