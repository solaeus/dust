hyperfine \
    --shell none \
    --prepare 'sync' \
    --warmup 5 \
    '../../target/release/dust ../../examples/fibonacci.ds' \
    'node fibonacci.js' \
    'deno fibonacci.js' \
    'bun fibonacci.js' \
    'python fibonacci.py'
