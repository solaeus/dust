hyperfine \
    --shell none \
    --prepare 'sync' \
    --warmup 5 \
    '../../target/release/dust recursion.ds' \
    'node recursion.js' \
    'deno recursion.js' \
    'bun recursion.js'
