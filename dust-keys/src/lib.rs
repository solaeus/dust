#![feature(proc_macro_totokens)]

use proc_macro::{ToTokens, TokenStream};
use std::fmt::Write;

#[proc_macro]
pub fn create_operation_keys(input: TokenStream) -> proc_macro::TokenStream {
    let input = input.to_string();
    let lines = input.lines();
    let mut output = String::new();

    for (index, line) in lines.enumerate() {
        let mut words = line.splitn(3, ' ');
        let first = words.next().unwrap_or_default();
        let second = words.next().unwrap_or_default();
        let third = words.next().unwrap_or_default();

        let _ = writeln!(
            output,
            "\
            pub const {first}_{second}_{third}: u64 = {index};
            pub const {first}_{third}_{second}: u64 = {index};"
        );
    }

    output.into_token_stream()
}
