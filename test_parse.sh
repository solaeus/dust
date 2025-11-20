#!/bin/bash
cat > /tmp/test_parser.rs << 'INNER'
#[test]
fn debug_list_equal() {
    use dust_lang::{parser::parse_main, tests::list_cases};
    let source = list_cases::LIST_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);
    println!("Error: {:?}", error);
    println!("{:#?}", syntax_tree.sorted_nodes());
    panic!("stop here");
}
INNER
echo 'mod test_parser;' >> dust-lang/src/lib.rs
cargo test debug_list_equal --lib -- --nocapture 2>&1 | grep -A 300 "Error:"
git checkout dust-lang/src/lib.rs
rm /tmp/test_parser.rs
