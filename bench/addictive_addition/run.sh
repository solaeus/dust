
hyperfine \
    --shell none \
    --prepare 'sync' \
    --warmup 5 \
    '../../target/release/dust addictive_addition.ds' \
    'node addictive_addition.js' \
    'deno addictive_addition.js' \
    'bun addictive_addition.js' \
    'python addictive_addition.py' \
    'lua addictive_addition.lua'