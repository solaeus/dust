mod simd_vectors;

use std::simd::{
    LaneCount, Simd, SupportedLaneCount,
    cmp::{SimdPartialEq, SimdPartialOrd},
    num::SimdUint,
};

use simd_vectors::*;

use crate::{
    Span,
    token::{Token, TokenKind},
};

pub fn validate_utf8_and_find_token_spans(input: &[u8]) -> (bool, Vec<Token>) {
    if cfg!(target_feature = "avx512bw") {
        validate_utf8_64_lanes(input)
    } else if cfg!(target_feature = "avx2") {
        validate_utf8_32_lanes(input)
    } else {
        validate_utf8_scalar(input)
    }
}

fn validate_utf8_inner<const LANE_COUNT: usize>(input: &[u8]) -> (bool, Vec<Token>)
where
    LaneCount<LANE_COUNT>: SupportedLaneCount,
{
    let mut tokens = Vec::new();
    let mut index = 0;
    let mut token_start: Option<usize> = None;

    loop {
        index = scan_ascii(input, index, &mut tokens, &mut token_start);

        if index >= input.len() {
            if let Some(current_word_start) = token_start.take() {
                let span = Span(current_word_start as u32, index as u32);
                let kind = classify_token_bytes(&input[current_word_start..index]);

                tokens.push(Token { kind, span });
            }

            return (true, tokens);
        }

        if token_start.is_none() {
            token_start = Some(index);
        }

        if scan_non_ascii_simd::<LANE_COUNT>(input, &mut index) {
            return (false, tokens);
        }
    }
}

fn validate_utf8_64_lanes(input: &[u8]) -> (bool, Vec<Token>) {
    validate_utf8_inner::<64>(input)
}

fn validate_utf8_32_lanes(input: &[u8]) -> (bool, Vec<Token>) {
    validate_utf8_inner::<32>(input)
}

fn validate_utf8_scalar(input: &[u8]) -> (bool, Vec<Token>) {
    let mut tokens = Vec::new();
    let mut index = 0;
    let mut current_word_start_index: Option<usize> = None;

    loop {
        index = scan_ascii(input, index, &mut tokens, &mut current_word_start_index);

        if index >= input.len() {
            if let Some(current_word_start) = current_word_start_index.take() {
                let span = Span(current_word_start as u32, index as u32);
                let kind = classify_token_bytes(&input[current_word_start..index]);

                tokens.push(Token { kind, span });
            }

            return (true, tokens);
        }

        if current_word_start_index.is_none() {
            current_word_start_index = Some(index);
        }

        if scan_non_ascii_scalar(input, &mut index) {
            return (false, tokens);
        }
    }
}

#[inline(always)]
fn scan_ascii(
    input: &[u8],
    mut index: usize,
    tokens: &mut Vec<Token>,
    current_word_start_index: &mut Option<usize>,
) -> usize {
    unsafe {
        let input_pointer = input.as_ptr();
        let input_length = input.len();
        let mut byte_index = index;

        while byte_index < input_length {
            let byte = *input_pointer.add(byte_index);

            if byte >= 128 {
                break;
            }

            if is_ascii_whitespace(byte) {
                if let Some(current_word_start) = current_word_start_index.take() {
                    let span = Span(current_word_start as u32, byte_index as u32);
                    let kind = classify_token_bytes(&input[current_word_start..byte_index]);
                    tokens.push(Token { kind, span });
                }

                byte_index += 1;

                continue;
            }

            // Treat '.' as part of a number if the current token started with a digit and the next byte is also a digit
            if byte == b'.'
                && let Some(current_word_start) = *current_word_start_index
            {
                let token_first_byte = *input_pointer.add(current_word_start);
                let next_is_digit = (byte_index + 1) < input_length
                    && (*input_pointer.add(byte_index + 1)).is_ascii_digit();

                if token_first_byte.is_ascii_digit() && next_is_digit {
                    byte_index += 1;

                    continue;
                }
            }

            if is_operator_or_punctuation(byte) {
                if let Some(current_word_start) = current_word_start_index.take() {
                    let span = Span(current_word_start as u32, byte_index as u32);
                    let kind = classify_token_bytes(&input[current_word_start..byte_index]);
                    tokens.push(Token { kind, span });
                }

                // Special case: -Infinity as a single token
                if byte == b'-' && byte_index + 9 <= input_length {
                    let slice = core::slice::from_raw_parts(input_pointer.add(byte_index), 9);
                    if slice == b"-Infinity" {
                        let span = Span(byte_index as u32, (byte_index + 9) as u32);
                        let kind = classify_token_bytes(&input[byte_index..byte_index + 9]);
                        tokens.push(Token { kind, span });
                        byte_index += 9;
                        continue;
                    }
                }

                // Greedy two-character operator match
                if byte_index + 1 < input_length {
                    let two_kind = classify_token_bytes(&input[byte_index..byte_index + 2]);
                    if two_kind != TokenKind::Unknown {
                        let span = Span(byte_index as u32, (byte_index + 2) as u32);
                        tokens.push(Token {
                            kind: two_kind,
                            span,
                        });
                        byte_index += 2;
                        continue;
                    }
                }

                // Fallback single-character operator
                let span = Span(byte_index as u32, (byte_index + 1) as u32);
                let kind = classify_token_bytes(&input[byte_index..byte_index + 1]);
                tokens.push(Token { kind, span });

                byte_index += 1;

                continue;
            }

            if current_word_start_index.is_none() {
                *current_word_start_index = Some(byte_index);
            }

            byte_index += 1;
        }

        index = byte_index;
    }

    index
}

#[cold]
fn scan_non_ascii_simd<const LANE_COUNT: usize>(input: &[u8], index: &mut usize) -> bool
where
    LaneCount<LANE_COUNT>: SupportedLaneCount,
{
    let mut window_start_index = *index;
    let mut is_invalid = false;
    let mut input_vector = ZERO;

    while window_start_index < input.len() && !is_invalid {
        load_window(input, window_start_index, &mut input_vector);

        let mut error_mask_for_window = input_vector.simd_gt(LEAD_MAX_UNICODE_PREFIX);
        let has_following_range_leads = input_vector.simd_eq(LEAD_SURROGATE_PREFIX).any()
            || input_vector.simd_eq(LEAD_MAX_UNICODE_PREFIX).any()
            || input_vector.simd_eq(LEAD_3BYTE_MIN_PREFIX).any()
            || input_vector.simd_eq(LEAD_4BYTE_MIN_PREFIX).any();
        let has_forbidden_overlong_leads = input_vector.simd_eq(FORBIDDEN_OVERLONG_C0).any()
            || input_vector.simd_eq(FORBIDDEN_OVERLONG_C1).any();

        if has_following_range_leads {
            let lookahead_first = *input.get(window_start_index + LANE_COUNT).unwrap_or(&0);
            let next_byte_vector = shift_left_by_one_with_lookahead(input_vector, lookahead_first);
            let invalid_ed_follow_mask = input_vector.simd_eq(LEAD_SURROGATE_PREFIX)
                & next_byte_vector.simd_gt(SECOND_MAX_AFTER_ED);
            let invalid_f4_follow_mask = input_vector.simd_eq(LEAD_MAX_UNICODE_PREFIX)
                & next_byte_vector.simd_gt(SECOND_MAX_AFTER_F4);
            let invalid_e0_follow_mask = input_vector.simd_eq(LEAD_3BYTE_MIN_PREFIX)
                & next_byte_vector.simd_lt(SECOND_MIN_AFTER_E0);
            let invalid_f0_follow_mask = input_vector.simd_eq(LEAD_4BYTE_MIN_PREFIX)
                & next_byte_vector.simd_lt(SECOND_MIN_AFTER_F0);
            let forbidden_c0_c1_mask = input_vector.simd_eq(FORBIDDEN_OVERLONG_C0)
                | input_vector.simd_eq(FORBIDDEN_OVERLONG_C1);

            error_mask_for_window |= invalid_ed_follow_mask
                | invalid_f4_follow_mask
                | invalid_e0_follow_mask
                | invalid_f0_follow_mask
                | forbidden_c0_c1_mask;
        }

        error_mask_for_window |= has_forbidden_overlong_leads;

        if error_mask_for_window.any() {
            is_invalid = true;
            break;
        }

        let utf8_class_vector = classify_high_nibbles_to_utf8_classes(input_vector);
        let mut utf8_class_vector_prev = ZERO;

        {
            let lanes = utf8_class_vector_prev.as_mut_array();
            if window_start_index >= 3 {
                lanes[LANE_COUNT - 3] = classify_high_nibble_scalar(input[window_start_index - 3]);
            }
            if window_start_index >= 2 {
                lanes[LANE_COUNT - 2] = classify_high_nibble_scalar(input[window_start_index - 2]);
            }
            if window_start_index >= 1 {
                lanes[LANE_COUNT - 1] = classify_high_nibble_scalar(input[window_start_index - 1]);
            }
        }

        let class_shift_by_one =
            shift_right_utf8_classes_by_one(utf8_class_vector, utf8_class_vector_prev);
        let class_shift_by_two =
            shift_right_utf8_classes_by_two(utf8_class_vector, utf8_class_vector_prev);
        let class_shift_by_three =
            shift_right_utf8_classes_by_three(utf8_class_vector, utf8_class_vector_prev);

        let accumulated_validation = utf8_class_vector
            + class_shift_by_one.saturating_sub(ONE)
            + class_shift_by_two.saturating_sub(TWO)
            + class_shift_by_three.saturating_sub(THREE);

        let at_least_one_mask = accumulated_validation.simd_ge(ONE);
        let is_lead_mask = utf8_class_vector.simd_ge(TWO);

        let within_original_class_if_lead_mask =
            (!is_lead_mask) | (accumulated_validation.simd_le(utf8_class_vector));

        let invalid_sequence_mask = (!at_least_one_mask) | (!within_original_class_if_lead_mask);

        error_mask_for_window |= invalid_sequence_mask;

        let has_lead_bytes = utf8_class_vector.simd_ge(TWO).any();
        if has_lead_bytes {
            let mut utf8_class_vector_next = ONE;

            {
                let lanes = utf8_class_vector_next.as_mut_array();
                let next_base_index = window_start_index + LANE_COUNT;

                if next_base_index < input.len() {
                    lanes[0] = classify_high_nibble_scalar(input[next_base_index]);

                    if next_base_index + 1 < input.len() {
                        lanes[1] = classify_high_nibble_scalar(input[next_base_index + 1]);

                        if next_base_index + 2 < input.len() {
                            lanes[2] = classify_high_nibble_scalar(input[next_base_index + 2]);
                        }
                    }
                }
            }

            let class_left_by_one =
                shift_left_utf8_classes_by_one(utf8_class_vector, utf8_class_vector_next);
            let class_left_by_two =
                shift_left_utf8_classes_by_two(utf8_class_vector, utf8_class_vector_next);
            let class_left_by_three =
                shift_left_utf8_classes_by_three(utf8_class_vector, utf8_class_vector_next);

            let is_sequence_len_2 = utf8_class_vector.simd_eq(TWO);
            let is_sequence_len_3 = utf8_class_vector.simd_eq(THREE);
            let is_sequence_len_4 = utf8_class_vector.simd_eq(FOUR);

            let next1_is_continuation = class_left_by_one.simd_eq(ZERO);
            let next2_is_continuation = class_left_by_two.simd_eq(ZERO);
            let next3_is_continuation = class_left_by_three.simd_eq(ZERO);

            let missing_after_2 = is_sequence_len_2 & (!next1_is_continuation);
            let missing_after_3 =
                is_sequence_len_3 & (!(next1_is_continuation & next2_is_continuation));
            let missing_after_4 = is_sequence_len_4
                & (!(next1_is_continuation & next2_is_continuation & next3_is_continuation));

            error_mask_for_window |= missing_after_2 | missing_after_3 | missing_after_4;
        }

        if window_start_index + LANE_COUNT >= input.len() {
            let number_of_valid_lanes = (input.len() - window_start_index).min(LANE_COUNT);
            let utf8_class_array = utf8_class_vector.to_array();
            let mut trailing_continuation_count: usize = 0;
            let mut scan_index = number_of_valid_lanes;

            while trailing_continuation_count < 3 && scan_index > 0 {
                let class_value = utf8_class_array[scan_index - 1];

                if class_value == 0 {
                    trailing_continuation_count += 1;
                    scan_index -= 1;
                } else {
                    break;
                }
            }
        }

        is_invalid |= error_mask_for_window.any();
        window_start_index += LANE_COUNT;

        if window_start_index >= input.len() {
            *index = input.len();

            return is_invalid;
        }

        // Stop at the first ASCII byte
        unsafe {
            let p = input.as_ptr().add(window_start_index);
            let end = input.as_ptr().add(input.len());
            if p < end && *p < 128 {
                break;
            }
        }
    }

    *index = window_start_index;
    is_invalid
}

#[cold]
fn scan_non_ascii_scalar(input: &[u8], index: &mut usize) -> bool {
    while *index < input.len() {
        let first = input[*index];

        if first < 128 {
            break;
        }

        let width = utf8_char_width(first);

        if *index + width > input.len() {
            return true;
        }

        match width {
            2 => {
                let second = input[*index + 1];

                if second as i8 >= -64 {
                    return true;
                }
            }
            3 => {
                let second = input[*index + 1];

                match (first, second) {
                    (0xE0, 0xA0..=0xBF)
                    | (0xE1..=0xEC, 0x80..=0xBF)
                    | (0xED, 0x80..=0x9F)
                    | (0xEE..=0xEF, 0x80..=0xBF) => {}
                    _ => return true,
                }

                let third = input[*index + 2];

                if third as i8 >= -64 {
                    return true;
                }
            }
            4 => {
                let second = input[*index + 1];

                match (first, second) {
                    (0xF0, 0x90..=0xBF) | (0xF1..=0xF3, 0x80..=0xBF) | (0xF4, 0x80..=0x8F) => {}
                    _ => return true,
                }

                let third = input[*index + 2];

                if third as i8 >= -64 {
                    return true;
                }

                let fourth = input[*index + 3];

                if fourth as i8 >= -64 {
                    return true;
                }
            }
            _ => return true,
        }

        *index += width;
    }

    false
}

#[inline(always)]
fn is_ascii_whitespace(byte: u8) -> bool {
    matches!(byte, b' ' | b'\n' | b'\r' | b'\t')
}

#[inline(always)]
fn is_operator_or_punctuation(byte: u8) -> bool {
    matches!(
        byte,
        b'!' | b'"'
            | b'#'
            | b'$'
            | b'%'
            | b'&'
            | b'\''
            | b'('
            | b')'
            | b'*'
            | b'+'
            | b','
            | b'-'
            | b'.'
            | b'/'
            | b':'
            | b';'
            | b'<'
            | b'='
            | b'>'
            | b'?'
            | b'@'
            | b'['
            | b'\\'
            | b']'
            | b'^'
            | b'_'
            | b'`'
            | b'{'
            | b'|'
            | b'}'
            | b'~'
    )
}

fn classify_token_bytes(word: &[u8]) -> TokenKind {
    match word {
        // Keywords
        b"any" => TokenKind::Any,
        b"async" => TokenKind::Async,
        b"bool" => TokenKind::Bool,
        b"break" => TokenKind::Break,
        b"byte" => TokenKind::Byte,
        b"cell" => TokenKind::Cell,
        b"char" => TokenKind::Char,
        b"const" => TokenKind::Const,
        b"else" => TokenKind::Else,
        b"float" => TokenKind::Float,
        b"fn" => TokenKind::Fn,
        b"if" => TokenKind::If,
        b"int" => TokenKind::Int,
        b"let" => TokenKind::Let,
        b"list" => TokenKind::List,
        b"loop" => TokenKind::Loop,
        b"map" => TokenKind::Map,
        b"mod" => TokenKind::Mod,
        b"mut" => TokenKind::Mut,
        b"pub" => TokenKind::Pub,
        b"return" => TokenKind::Return,
        b"str" => TokenKind::Str,
        b"struct" => TokenKind::Struct,
        b"use" => TokenKind::Use,
        b"while" => TokenKind::While,

        // Operators and punctuation
        b"->" => TokenKind::ArrowThin,
        b"*" => TokenKind::Asterisk,
        b"*=" => TokenKind::AsteriskEqual,
        b"!=" => TokenKind::BangEqual,
        b"!" => TokenKind::Bang,
        b":" => TokenKind::Colon,
        b"," => TokenKind::Comma,
        b"." => TokenKind::Dot,
        b"&&" => TokenKind::DoubleAmpersand,
        b"::" => TokenKind::DoubleColon,
        b".." => TokenKind::DoubleDot,
        b"==" => TokenKind::DoubleEqual,
        b"||" => TokenKind::DoublePipe,
        b"=" => TokenKind::Equal,
        b">" => TokenKind::Greater,
        b">=" => TokenKind::GreaterEqual,
        b"{" => TokenKind::LeftCurlyBrace,
        b"[" => TokenKind::LeftSquareBracket,
        b"(" => TokenKind::LeftParenthesis,
        b"<" => TokenKind::Less,
        b"<=" => TokenKind::LessEqual,
        b"-" => TokenKind::Minus,
        b"-=" => TokenKind::MinusEqual,
        b"%" => TokenKind::Percent,
        b"%=" => TokenKind::PercentEqual,
        b"+" => TokenKind::Plus,
        b"+=" => TokenKind::PlusEqual,
        b"}" => TokenKind::RightCurlyBrace,
        b"]" => TokenKind::RightSquareBracket,
        b")" => TokenKind::RightParenthesis,
        b";" => TokenKind::Semicolon,
        b"/" => TokenKind::Slash,
        b"/=" => TokenKind::SlashEqual,

        // Literals
        b"true" => TokenKind::TrueValue,
        b"false" => TokenKind::FalseValue,
        b"Infinity" => TokenKind::FloatValue,
        b"-Infinity" => TokenKind::FloatValue,
        word if !word.is_empty() && (word[0].is_ascii_alphabetic() || word[0] == b'_') => {
            if word.iter().all(|b| b.is_ascii_alphanumeric() || *b == b'_') {
                TokenKind::Identifier
            } else {
                TokenKind::Unknown
            }
        }
        word if word.len() > 2
            && word[0] == b'0'
            && word[1] == b'x'
            && word[2..].iter().all(|b| b.is_ascii_hexdigit()) =>
        {
            TokenKind::ByteValue
        }
        word if !word.is_empty() && word[0].is_ascii_digit() => {
            let mut iterator = word.iter().peekable();
            let mut has_decimal = false;
            let mut has_exponent = false;

            while let Some(&byte) = iterator.peek() {
                match byte {
                    b'0'..=b'9' => {
                        iterator.next();
                    }
                    b'.' if !has_decimal => {
                        has_decimal = true;
                        iterator.next();
                    }
                    b'e' | b'E' => {
                        if !has_exponent && has_decimal {
                            has_exponent = true;
                            has_decimal = true;
                            iterator.next();
                        } else {
                            break;
                        }
                    }
                    _ => break,
                }
            }

            if iterator.next().is_none() {
                if has_decimal {
                    TokenKind::FloatValue
                } else {
                    TokenKind::IntegerValue
                }
            } else {
                TokenKind::Unknown
            }
        }
        word if word.len() >= 2 && word[0] == b'"' && word[word.len() - 1] == b'"' => {
            TokenKind::StringValue
        }
        word if word.len() == 3 && word[0] == b'\'' && word[2] == b'\'' => {
            TokenKind::CharacterValue
        }
        _ => TokenKind::Unknown,
    }
}

#[inline(always)]
fn load_window<const LANE_COUNT: usize>(
    input_bytes: &[u8],
    start_index: usize,
    window: &mut Simd<u8, LANE_COUNT>,
) where
    LaneCount<LANE_COUNT>: SupportedLaneCount,
{
    if start_index + LANE_COUNT <= input_bytes.len() {
        let src = &input_bytes[start_index..start_index + LANE_COUNT];

        window.as_mut_array().copy_from_slice(src);

        return;
    }

    *window = ZERO;

    if start_index < input_bytes.len() {
        let copy_count = (input_bytes.len() - start_index).min(LANE_COUNT);
        let input_slice = &input_bytes[start_index..start_index + copy_count];

        window.as_mut_array()[..copy_count].copy_from_slice(input_slice);
    }
}

#[inline(always)]
fn shift_left_by_one_with_lookahead<const LANE_COUNT: usize>(
    current_window: Simd<u8, LANE_COUNT>,
    lookahead_first: u8,
) -> Simd<u8, LANE_COUNT>
where
    LaneCount<LANE_COUNT>: SupportedLaneCount,
{
    let mut shifted = current_window.rotate_elements_left::<1>();

    shifted.as_mut_array()[LANE_COUNT - 1] = lookahead_first;

    shifted
}

#[inline(always)]
fn classify_high_nibbles_to_utf8_classes<const LANE_COUNT: usize>(
    byte_vector: Simd<u8, LANE_COUNT>,
) -> Simd<u8, LANE_COUNT>
where
    LaneCount<LANE_COUNT>: SupportedLaneCount,
{
    let high_nibbles = byte_vector >> SHIFT_NIBBLE;
    let mut utf8_class_vector = ONE;

    let continuation_mask =
        high_nibbles.simd_ge(HIGH_NIBBLE_CONT_MIN) & high_nibbles.simd_le(HIGH_NIBBLE_CONT_MAX);
    utf8_class_vector = continuation_mask.select(ZERO, utf8_class_vector);

    let lead_two_mask = high_nibbles.simd_ge(HIGH_NIBBLE_TWO_BYTE_MIN)
        & high_nibbles.simd_le(HIGH_NIBBLE_TWO_BYTE_MAX);
    utf8_class_vector = lead_two_mask.select(TWO, utf8_class_vector);

    let lead_three_mask = high_nibbles.simd_eq(HIGH_NIBBLE_THREE_BYTE);
    utf8_class_vector = lead_three_mask.select(THREE, utf8_class_vector);

    let lead_four_mask = high_nibbles.simd_eq(HIGH_NIBBLE_FOUR_BYTE);
    utf8_class_vector = lead_four_mask.select(FOUR, utf8_class_vector);

    utf8_class_vector
}

#[inline(always)]
fn classify_high_nibble_scalar<const LANE_COUNT: usize>(byte: u8) -> u8
where
    LaneCount<LANE_COUNT>: SupportedLaneCount,
{
    let high_nibble = byte >> 4;

    if (0x08..=0x0B).contains(&high_nibble) {
        0
    } else if (0x0C..=0x0D).contains(&high_nibble) {
        2
    } else if high_nibble == 0x0E {
        3
    } else if high_nibble == 0x0F {
        4
    } else {
        1
    }
}

#[inline(always)]
fn shift_right_utf8_classes_by_one<const LANE_COUNT: usize>(
    class_vector: Simd<u8, LANE_COUNT>,
    previous_class_vector: Simd<u8, LANE_COUNT>,
) -> Simd<u8, LANE_COUNT>
where
    LaneCount<LANE_COUNT>: SupportedLaneCount,
{
    let mut shifted = class_vector.rotate_elements_right::<1>();
    let previous_last_lane = previous_class_vector.as_array()[LANE_COUNT - 1];
    shifted.as_mut_array()[0] = previous_last_lane;

    shifted
}

#[inline(always)]
fn shift_right_utf8_classes_by_two<const LANE_COUNT: usize>(
    class_vector: Simd<u8, LANE_COUNT>,
    previous_class_vector: Simd<u8, LANE_COUNT>,
) -> Simd<u8, LANE_COUNT>
where
    LaneCount<LANE_COUNT>: SupportedLaneCount,
{
    let mut shifted = class_vector.rotate_elements_right::<2>();
    let previous_class_array = previous_class_vector.as_array();

    {
        let lanes = shifted.as_mut_array();
        lanes[0] = previous_class_array[LANE_COUNT - 2];
        lanes[1] = previous_class_array[LANE_COUNT - 1];
    }

    shifted
}

#[inline(always)]
fn shift_left_utf8_classes_by_one<const LANE_COUNT: usize>(
    class_vector: Simd<u8, LANE_COUNT>,
    next_class_vector: Simd<u8, LANE_COUNT>,
) -> Simd<u8, LANE_COUNT>
where
    LaneCount<LANE_COUNT>: SupportedLaneCount,
{
    let mut shifted = class_vector.rotate_elements_left::<1>();
    let next_first_lane = next_class_vector.as_array()[0];
    shifted.as_mut_array()[LANE_COUNT - 1] = next_first_lane;

    shifted
}

#[inline(always)]
fn shift_left_utf8_classes_by_two<const LANE_COUNT: usize>(
    class_vector: Simd<u8, LANE_COUNT>,
    next_class_vector: Simd<u8, LANE_COUNT>,
) -> Simd<u8, LANE_COUNT>
where
    LaneCount<LANE_COUNT>: SupportedLaneCount,
{
    let mut shifted = class_vector.rotate_elements_left::<2>();
    let next_class_array = next_class_vector.as_array();

    {
        let lanes = shifted.as_mut_array();
        lanes[LANE_COUNT - 2] = next_class_array[0];
        lanes[LANE_COUNT - 1] = next_class_array[1];
    }

    shifted
}

#[inline(always)]
fn shift_left_utf8_classes_by_three<const LANE_COUNT: usize>(
    class_vector: Simd<u8, LANE_COUNT>,
    next_class_vector: Simd<u8, LANE_COUNT>,
) -> Simd<u8, LANE_COUNT>
where
    LaneCount<LANE_COUNT>: SupportedLaneCount,
{
    let mut shifted = class_vector.rotate_elements_left::<3>();
    let next_class_array = next_class_vector.as_array();

    {
        let lanes = shifted.as_mut_array();
        lanes[LANE_COUNT - 3] = next_class_array[0];
        lanes[LANE_COUNT - 2] = next_class_array[1];
        lanes[LANE_COUNT - 1] = next_class_array[2];
    }

    shifted
}

#[inline(always)]
fn shift_right_utf8_classes_by_three<const LANE_COUNT: usize>(
    class_vector: Simd<u8, LANE_COUNT>,
    previous_class_vector: Simd<u8, LANE_COUNT>,
) -> Simd<u8, LANE_COUNT>
where
    LaneCount<LANE_COUNT>: SupportedLaneCount,
{
    let mut shifted = class_vector.rotate_elements_right::<3>();
    let previous_class_array = previous_class_vector.as_array();

    {
        let lanes = shifted.as_mut_array();
        lanes[0] = previous_class_array[LANE_COUNT - 3];
        lanes[1] = previous_class_array[LANE_COUNT - 2];
        lanes[2] = previous_class_array[LANE_COUNT - 1];
    }

    shifted
}

// https://tools.ietf.org/html/rfc3629
const UTF8_CHAR_WIDTH: &[u8; 256] = &[
    // 1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 1
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 2
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 3
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 4
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 5
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 6
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 7
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 8
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 9
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // A
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // B
    0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // C
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // D
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, // E
    4, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // F
];

/// Given a first byte, determines how many bytes are in this UTF-8 character.
#[inline]
const fn utf8_char_width(b: u8) -> usize {
    UTF8_CHAR_WIDTH[b as usize] as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_spans() {
        let input = b"let a:int=42;\nlet b:int=666;";
        let (_, tokens) = validate_utf8_and_find_token_spans(input);
        let expected_tokens = vec![
            Token {
                kind: TokenKind::Let,
                span: Span(0, 3),
            },
            Token {
                kind: TokenKind::Identifier,
                span: Span(4, 5),
            },
            Token {
                kind: TokenKind::Colon,
                span: Span(5, 6),
            },
            Token {
                kind: TokenKind::Int,
                span: Span(6, 9),
            },
            Token {
                kind: TokenKind::Equal,
                span: Span(9, 10),
            },
            Token {
                kind: TokenKind::IntegerValue,
                span: Span(10, 12),
            },
            Token {
                kind: TokenKind::Semicolon,
                span: Span(12, 13),
            },
            Token {
                kind: TokenKind::Let,
                span: Span(14, 17),
            },
            Token {
                kind: TokenKind::Identifier,
                span: Span(18, 19),
            },
            Token {
                kind: TokenKind::Colon,
                span: Span(19, 20),
            },
            Token {
                kind: TokenKind::Int,
                span: Span(20, 23),
            },
            Token {
                kind: TokenKind::Equal,
                span: Span(23, 24),
            },
            Token {
                kind: TokenKind::IntegerValue,
                span: Span(24, 27),
            },
            Token {
                kind: TokenKind::Semicolon,
                span: Span(27, 28),
            },
            Token {
                kind: TokenKind::Eof,
                span: Span(28, 28),
            },
        ];

        assert_eq!(tokens, expected_tokens);
    }
    #[test]
    fn all_ascii() {
        let all_ascii: Vec<u8> = (0..=0x7F).collect();

        assert!(validate_utf8_and_find_token_spans(&all_ascii).0);
    }

    #[test]
    fn all_valid_utf8_bytes() {
        let utf8_range = 0..=0x10FFFF;
        let surrogate_range = 0xD800..=0xDFFF;
        let mut bytes = [0u8; 4];
        let mut all_valid_utf8 = Vec::new();

        for codepoint in utf8_range {
            if surrogate_range.contains(&codepoint) {
                continue;
            }

            let character = char::from_u32(codepoint).unwrap();

            character.encode_utf8(&mut bytes);
            all_valid_utf8.extend_from_slice(&bytes[..character.len_utf8()]);
        }

        assert!(validate_utf8_and_find_token_spans(&all_valid_utf8).0);
    }

    #[test]
    fn eof_missing_after_2_lead_in_last_lane() {
        const LANE_COUNT: usize = 32;

        let mut buf = vec![b'a'; 2 * LANE_COUNT + 1];
        let lead_idx = 2 * LANE_COUNT; // last lane of last window
        buf[lead_idx] = 0xC2; // needs 1 continuation beyond EOF -> invalid

        assert!(!validate_utf8_32_lanes(&buf).0);
    }

    #[test]
    fn eof_missing_after_3_lead_in_second_last_lane() {
        const LANE_COUNT: usize = 32;

        let mut buf = vec![b'a'; 2 * LANE_COUNT + 2];
        let lead_idx = 2 * LANE_COUNT - 1; // second-to-last lane of last full window
        buf[lead_idx] = 0xE0;
        buf[lead_idx + 1] = 0xA0; // continuation inside last window

        assert!(!validate_utf8_32_lanes(&buf).0);
    }

    #[test]
    fn eof_missing_after_4_lead_in_third_last_lane() {
        const LANE_COUNT: usize = 32;

        let mut buf = vec![b'a'; 2 * LANE_COUNT + 3];
        let lead_idx = 2 * LANE_COUNT - 2; // third-to-last lane
        buf[lead_idx] = 0xF0;
        buf[lead_idx + 1] = 0x90;
        buf[lead_idx + 2] = 0x80;

        assert!(!validate_utf8_32_lanes(&buf).0);
    }
}

// https://www.cl.cam.ac.uk/~mgk25/ucs/examples/UTF-8-test.txt
#[cfg(test)]
mod kuhn_tests {
    use super::*;

    fn assert_valid(bytes: &[u8]) {
        assert!(
            validate_utf8_and_find_token_spans(bytes).0,
            "Expected valid, got invalid: {:x?}",
            bytes
        );
    }

    fn assert_invalid(bytes: &[u8]) {
        assert!(
            !validate_utf8_and_find_token_spans(bytes).0,
            "Expected invalid, got valid: {:x?}",
            bytes
        );
    }

    // Boundary: first valid sequences for 1–4 byte forms, reject 5–6 byte forms
    #[test]
    fn valid_minimal_sequences_per_length() {
        assert_valid(&[0x00]); // U+0000
        assert_valid(&[0xC2, 0x80]); // U+0080
        assert_valid(&[0xE0, 0xA0, 0x80]); // U+0800
        assert_valid(&[0xF0, 0x90, 0x80, 0x80]); // U+10000

        // 5- and 6-byte forms are illegal under RFC 3629
        assert_invalid(&[0xF8, 0x88, 0x80, 0x80, 0x80]);
        assert_invalid(&[0xFC, 0x84, 0x80, 0x80, 0x80, 0x80]);
    }

    // Boundary: last valid sequences per form that are within Unicode range; reject beyond
    #[test]
    fn valid_maximal_sequences_within_range_and_reject_beyond() {
        assert_valid(&[0x7F]); // U+007F
        assert_valid(&[0xDF, 0xBF]); // U+07FF
        assert_valid(&[0xEF, 0xBF, 0xBF]); // U+FFFF (valid UTF-8 encoding)

        // 4-byte sequences above U+10FFFF are invalid; these are legacy >21-bit forms
        assert_invalid(&[0xF7, 0xBF, 0xBF, 0xBF]); // U+1FFFFF (legacy)
        assert_invalid(&[0xFB, 0xBF, 0xBF, 0xBF, 0xBF]); // 5-byte legacy
        assert_invalid(&[0xFD, 0xBF, 0xBF, 0xBF, 0xBF, 0xBF]); // 6-byte legacy
    }

    // Boundary edges and Unicode max
    #[test]
    fn valid_edge_boundaries_and_invalid_above_unicode_max() {
        assert_valid(&[0xED, 0x9F, 0xBF]); // U+0D7FF
        assert_valid(&[0xEE, 0x80, 0x80]); // U+0E000
        assert_valid(&[0xEF, 0xBF, 0xBD]); // U+FFFD
        assert_valid(&[0xF4, 0x8F, 0xBF, 0xBF]); // U+10FFFF
        assert_invalid(&[0xF4, 0x90, 0x80, 0x80]); // U+110000 (too large)
    }

    // Continuations that appear without a lead
    #[test]
    fn invalid_unexpected_continuation_bytes() {
        assert_invalid(&[0x80]);
        assert_invalid(&[0xBF]);

        for n in 2..=7 {
            let v = vec![0x80; n];
            assert_invalid(&v);
        }

        let mut all_continuations = Vec::with_capacity(64);
        for b in 0x80..=0xBF {
            all_continuations.push(b);
        }
        assert_invalid(&all_continuations);
    }

    // Start bytes not followed by required continuation bytes
    #[test]
    fn invalid_lonely_start_bytes() {
        for b in 0xC0..=0xDF {
            assert_invalid(&[b, 0x20]); // 2-byte start + space
        }
        for b in 0xE0..=0xEF {
            assert_invalid(&[b, 0x20]); // 3-byte start + space
        }
        for b in 0xF0..=0xF7 {
            assert_invalid(&[b, 0x20]); // 4-byte start + space
        }
        for b in 0xF8..=0xFB {
            assert_invalid(&[b, 0x20]); // 5-byte start (legacy) + space
        }
        for b in 0xFC..=0xFD {
            assert_invalid(&[b, 0x20]); // 6-byte start (legacy) + space
        }
    }

    // Lead provided but last continuation missing (incomplete sequences)
    #[test]
    fn invalid_incomplete_sequences_last_continuation_missing() {
        // Minimal representatives
        assert_invalid(&[0xC2]); // need one continuation
        assert_invalid(&[0xE0, 0xA0]); // need one more continuation
        assert_invalid(&[0xF0, 0x90, 0x80]); // need one more continuation

        // Maximal representatives missing last
        assert_invalid(&[0xDF]); // 2-byte max, missing continuation
        assert_invalid(&[0xEF, 0xBF]); // 3-byte max, missing continuation
        assert_invalid(&[0xF7, 0xBF, 0xBF]); // legacy 4-byte form, missing continuation
        assert_invalid(&[0xFB, 0xBF, 0xBF, 0xBF]); // legacy 5-byte, missing last
        assert_invalid(&[0xFD, 0xBF, 0xBF, 0xBF, 0xBF]); // legacy 6-byte, missing last
    }

    // Concatenation of several incomplete sequences remains invalid
    #[test]
    fn invalid_concatenated_incomplete_sequences() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&[0xC2]); // 2-byte incomplete
        bytes.extend_from_slice(&[0xE0, 0xA0]); // 3-byte incomplete
        bytes.extend_from_slice(&[0xF0, 0x90, 0x80]); // 4-byte incomplete
        bytes.extend_from_slice(&[0xDF]); // 2-byte max missing one
        bytes.extend_from_slice(&[0xEF, 0xBF]); // 3-byte max missing one
        bytes.extend_from_slice(&[0xF7, 0xBF, 0xBF]); // legacy 4-byte missing one
        assert_invalid(&bytes);
    }

    // Bytes forbidden in UTF-8
    #[test]
    fn invalid_impossible_bytes_fe_ff() {
        assert_invalid(&[0xFE]);
        assert_invalid(&[0xFF]);
        assert_invalid(&[0xFE, 0xFE, 0xFF, 0xFF]);
    }

    // Overlong encodings of ASCII slash '/'
    #[test]
    fn invalid_overlong_ascii_variants() {
        assert_invalid(&[0xC0, 0xAF]);
        assert_invalid(&[0xE0, 0x80, 0xAF]);
        assert_invalid(&[0xF0, 0x80, 0x80, 0xAF]);
        assert_invalid(&[0xF8, 0x80, 0x80, 0x80, 0xAF]);
        assert_invalid(&[0xFC, 0x80, 0x80, 0x80, 0x80, 0xAF]);
    }

    // Maximum overlong examples
    #[test]
    fn invalid_maximum_overlong_sequences() {
        assert_invalid(&[0xC1, 0xBF]); // overlong for U+007F
        assert_invalid(&[0xE0, 0x9F, 0xBF]); // overlong for U+07FF
        assert_invalid(&[0xF0, 0x8F, 0xBF, 0xBF]); // overlong for U+FFFF
        assert_invalid(&[0xF8, 0x87, 0xBF, 0xBF, 0xBF]); // legacy 5-byte
        assert_invalid(&[0xFC, 0x83, 0xBF, 0xBF, 0xBF, 0xBF]); // legacy 6-byte
    }

    // Overlong encodings of NUL
    #[test]
    fn invalid_overlong_nul_variants() {
        assert_invalid(&[0xC0, 0x80]);
        assert_invalid(&[0xE0, 0x80, 0x80]);
        assert_invalid(&[0xF0, 0x80, 0x80, 0x80]);
        assert_invalid(&[0xF8, 0x80, 0x80, 0x80, 0x80]);
        assert_invalid(&[0xFC, 0x80, 0x80, 0x80, 0x80, 0x80]);
    }

    // UTF-16 surrogate halves are not valid Unicode scalar values
    #[test]
    fn invalid_single_utf16_surrogates() {
        assert_invalid(&[0xED, 0xA0, 0x80]); // U+D800
        assert_invalid(&[0xED, 0xAD, 0xBF]); // U+DB7F
        assert_invalid(&[0xED, 0xAE, 0x80]); // U+DB80
        assert_invalid(&[0xED, 0xAF, 0xBF]); // U+DBFF
        assert_invalid(&[0xED, 0xB0, 0x80]); // U+DC00
        assert_invalid(&[0xED, 0xBE, 0x80]); // U+DF80
        assert_invalid(&[0xED, 0xBF, 0xBF]); // U+DFFF
    }

    // Pairs of surrogate halves remain invalid in UTF-8
    #[test]
    fn invalid_paired_utf16_surrogates() {
        assert_invalid(&[0xED, 0xA0, 0x80, 0xED, 0xB0, 0x80]); // D800 DC00
        assert_invalid(&[0xED, 0xA0, 0x80, 0xED, 0xBF, 0xBF]); // D800 DFFF
        assert_invalid(&[0xED, 0xAD, 0xBF, 0xED, 0xB0, 0x80]); // DB7F DC00
        assert_invalid(&[0xED, 0xAD, 0xBF, 0xED, 0xBF, 0xBF]); // DB7F DFFF
        assert_invalid(&[0xED, 0xAE, 0x80, 0xED, 0xB0, 0x80]); // DB80 DC00
        assert_invalid(&[0xED, 0xAE, 0x80, 0xED, 0xBF, 0xBF]); // DB80 DFFF
        assert_invalid(&[0xED, 0xAF, 0xBF, 0xED, 0xB0, 0x80]); // DBFF DC00
        assert_invalid(&[0xED, 0xAF, 0xBF, 0xED, 0xBF, 0xBF]); // DBFF DFFF
    }

    // Noncharacters: structurally valid UTF-8 (we accept them as valid)
    #[test]
    fn valid_noncharacter_code_points() {
        // U+FFFE, U+FFFF
        assert_valid(&[0xEF, 0xBF, 0xBE]);
        assert_valid(&[0xEF, 0xBF, 0xBF]);

        // U+FDD0, U+FDEF
        assert_valid(&[0xEF, 0xB7, 0x90]); // U+FDD0
        assert_valid(&[0xEF, 0xB7, 0xAF]); // U+FDEF

        // Plane 1 examples: U+1FFFE, U+1FFFF
        assert_valid(&[0xF0, 0x9F, 0xBF, 0xBE]); // U+1FFFE
        assert_valid(&[0xF0, 0x9F, 0xBF, 0xBF]); // U+1FFFF
    }
}
