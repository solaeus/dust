mod validate_utf8;

use std::simd::{
    Mask, Simd,
    cmp::{SimdPartialEq, SimdPartialOrd},
};

use crate::token::Token;

const SIMD_LANES: usize = {
    if cfg!(target_arch = "x86_64") {
        if cfg!(target_feature = "avx512f") {
            64
        } else if cfg!(target_feature = "avx2") {
            32
        } else {
            16
        }
    } else {
        16
    }
};

const BITLIST_LIMIT: u64 = if SIMD_LANES == 64 {
    u64::MAX
} else {
    (1 << SIMD_LANES) - 1
};

pub fn tokenize(source: &[u8]) -> Vec<Token> {
    let mut lexer = Lexer::new(source);

    lexer.collect()
}

#[derive(Debug, Default)]
pub struct Lexer<'src> {
    source: &'src [u8],

    next_emission_index: usize,
    next_byte_index: usize,
    window_start: usize,

    alphanumeric_bitlist: u64,
    blackspace_bitlist: u64, // inverse of whitespace bitlist
    control_bitlist: u64,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src [u8]) -> Self {
        Lexer {
            source,
            next_emission_index: 0,
            next_byte_index: 0,
            window_start: 0,
            alphanumeric_bitlist: 0,
            blackspace_bitlist: 0,
            control_bitlist: 0,
        }
    }

    pub fn source(&self) -> &'src [u8] {
        self.source
    }

    #[inline(always)]
    fn load_window(&mut self) -> bool {
        if self.next_byte_index >= self.source.len() {
            return false;
        }
        let take = (self.source.len() - self.next_byte_index).min(SIMD_LANES);
        let window = {
            let mut window_bytes = [0u8; SIMD_LANES];
            let end = self.next_byte_index + take;
            let source_bytes = &self.source[self.next_byte_index..end];

            window_bytes[..take].copy_from_slice(source_bytes);

            Simd::<u8, SIMD_LANES>::from_array(window_bytes)
        };
        let tail_mask: u64 = if take < SIMD_LANES {
            ((1u64 << take) - 1) & BITLIST_LIMIT
        } else {
            BITLIST_LIMIT
        };
        self.alphanumeric_bitlist = alphanumeric_bitlist(window) & tail_mask;
        self.blackspace_bitlist = {
            let whitespace = whitespace_bitlist(window) & tail_mask;

            (!whitespace) & tail_mask
        };
        self.control_bitlist = control_bitlist(window) & tail_mask;
        self.window_start = self.next_byte_index;
        self.next_byte_index += take;
        true
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

#[inline(always)]
fn alphanumeric_bitlist(window: Simd<u8, SIMD_LANES>) -> u64 {
    let is_uppercase = window.simd_ge(Simd::splat(b'A')) & window.simd_le(Simd::splat(b'Z'));
    let is_lowercase = window.simd_ge(Simd::splat(b'a')) & window.simd_le(Simd::splat(b'z'));
    let is_digit = window.simd_ge(Simd::splat(b'0')) & window.simd_le(Simd::splat(b'9'));
    let is_underscore = window.simd_eq(Simd::splat(b'_'));

    (is_uppercase | is_lowercase | is_digit | is_underscore).to_bitmask()
}

#[inline(always)]
fn whitespace_bitlist(window: Simd<u8, SIMD_LANES>) -> u64 {
    let is_space = window.simd_eq(Simd::splat(b' '));
    let is_tab = window.simd_eq(Simd::splat(b'\t'));
    let is_newline = window.simd_eq(Simd::splat(b'\n')) | window.simd_eq(Simd::splat(b'\r'));

    (is_space | is_tab | is_newline).to_bitmask()
}

#[inline(always)]
fn control_bitlist(window: Simd<u8, SIMD_LANES>) -> u64 {
    let control_bytes = [
        b'(', b')', b'[', b']', b'{', b'}', b';', b',', b'.', b':', b'+', b'-', b'*', b'/', b'%',
        b'!', b'=', b'<', b'>', b'&', b'|',
    ];

    let mut mask = Mask::splat(false);

    for &byte in &control_bytes {
        mask |= window.simd_eq(Simd::splat(byte));
    }

    mask.to_bitmask()
}
