use std::simd::{
    Mask, Simd,
    cmp::{SimdPartialEq, SimdPartialOrd},
    num::SimdUint,
};

const SIMD_LANE_COUNT: usize = {
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

const BITLIST_LIMIT: u64 = if SIMD_LANE_COUNT == 64 {
    u64::MAX
} else {
    (1 << SIMD_LANE_COUNT) - 1
};

pub fn validate_utf8_and_find_token_starts(input: &[u8]) -> (bool, Vec<usize>) {
    let mut token_start_indices = Vec::new();
    let mut window_start_index = 0;
    let mut is_invalid = false;
    let mut previous_was_whitespace = true as u64; // Initialized to true for start-of-input

    while window_start_index < input.len() && !is_invalid {
        let input_vector = load_window(input, window_start_index);
        let window_length = (input.len() - window_start_index).min(SIMD_LANE_COUNT);

        fill_significant_start_indices(
            input_vector,
            window_start_index,
            window_length,
            &mut token_start_indices,
            &mut previous_was_whitespace,
        );

        let non_ascii = input_vector & Simd::splat(0x80);

        // Fast path for all-ASCII windows
        if non_ascii.simd_eq(Simd::splat(0)).all() {
            let mut ascii_scan_index = window_start_index + SIMD_LANE_COUNT;

            while ascii_scan_index < input.len() {
                let candidate_window = load_window(input, ascii_scan_index);
                let window_length = (input.len() - ascii_scan_index).min(SIMD_LANE_COUNT);

                fill_significant_start_indices(
                    candidate_window,
                    ascii_scan_index,
                    window_length,
                    &mut token_start_indices,
                    &mut previous_was_whitespace,
                );

                let non_ascii = candidate_window & Simd::splat(0x80);

                if !non_ascii.simd_eq(Simd::splat(0)).all() {
                    break;
                }

                ascii_scan_index += SIMD_LANE_COUNT;
            }

            window_start_index = ascii_scan_index;

            continue;
        }

        let lookahead_vector =
            load_window(input, window_start_index.saturating_add(SIMD_LANE_COUNT));

        // 1) Bound check: bytes must be <= 0xF4
        let bytes_greater_than_f4_mask = input_vector
            .saturating_sub(Simd::splat(0xF4))
            .simd_ne(Simd::splat(0));
        let mut error_mask_for_window = bytes_greater_than_f4_mask;

        // 2) Neighbor constraints via next-byte alignment
        let next_byte_vector = shift_left_by_one_with_lookahead(input_vector, lookahead_vector);
        let invalid_ed_follow_mask =
            input_vector.simd_eq(Simd::splat(0xED)) & next_byte_vector.simd_gt(Simd::splat(0x9F));
        let invalid_f4_follow_mask =
            input_vector.simd_eq(Simd::splat(0xF4)) & next_byte_vector.simd_gt(Simd::splat(0x8F));
        let invalid_e0_follow_mask =
            input_vector.simd_eq(Simd::splat(0xE0)) & next_byte_vector.simd_lt(Simd::splat(0xA0));
        let invalid_f0_follow_mask =
            input_vector.simd_eq(Simd::splat(0xF0)) & next_byte_vector.simd_lt(Simd::splat(0x90));
        let forbidden_c0_c1_mask =
            input_vector.simd_eq(Simd::splat(0xC0)) | input_vector.simd_eq(Simd::splat(0xC1));

        error_mask_for_window |= invalid_ed_follow_mask
            | invalid_f4_follow_mask
            | invalid_e0_follow_mask
            | invalid_f0_follow_mask
            | forbidden_c0_c1_mask;

        // 3) Classify high nibble -> {0,1,2,3,4}
        let utf8_class_vector = classify_high_nibbles_to_utf8_classes(input_vector);
        let utf8_class_vector_next = classify_high_nibbles_to_utf8_classes(lookahead_vector);
        let previous_window =
            load_window(input, window_start_index.saturating_sub(SIMD_LANE_COUNT));
        let utf8_class_vector_prev = classify_high_nibbles_to_utf8_classes(previous_window);

        // 4) Sequence validity via shift/sub/add (look-behind using right shifts)
        let class_shift_by_one =
            shift_right_utf8_classes_by_one(utf8_class_vector, utf8_class_vector_prev);
        let class_shift_by_two =
            shift_right_utf8_classes_by_two(utf8_class_vector, utf8_class_vector_prev);
        let class_shift_by_three =
            shift_right_utf8_classes_by_three(utf8_class_vector, utf8_class_vector_prev);

        let accumulated_validation = utf8_class_vector
            + class_shift_by_one.saturating_sub(Simd::splat(1u8))
            + class_shift_by_two.saturating_sub(Simd::splat(2u8))
            + class_shift_by_three.saturating_sub(Simd::splat(3u8));

        let at_least_one_mask = accumulated_validation.simd_ge(Simd::splat(1u8));
        let is_lead_mask = utf8_class_vector.simd_ge(Simd::splat(2u8));

        // If lead: require accumulated_validation <= utf8_class_vector; else: always true.
        let within_original_class_if_lead_mask =
            (!is_lead_mask) | (accumulated_validation.simd_le(utf8_class_vector));

        let invalid_sequence_mask = (!at_least_one_mask) | (!within_original_class_if_lead_mask);

        error_mask_for_window |= invalid_sequence_mask;

        // Forward-looking check: lead bytes must be followed by enough continuation bytes.
        // We shift classes left (lookahead) and ensure the next k-1 bytes are continuation (class 0).
        let class_left_by_one =
            shift_left_utf8_classes_by_one(utf8_class_vector, utf8_class_vector_next);
        let class_left_by_two =
            shift_left_utf8_classes_by_two(utf8_class_vector, utf8_class_vector_next);
        let class_left_by_three =
            shift_left_utf8_classes_by_three(utf8_class_vector, utf8_class_vector_next);

        let is2 = utf8_class_vector.simd_eq(Simd::splat(2u8));
        let is3 = utf8_class_vector.simd_eq(Simd::splat(3u8));
        let is4 = utf8_class_vector.simd_eq(Simd::splat(4u8));

        let next1_is_cont = class_left_by_one.simd_eq(Simd::splat(0u8));
        let next2_is_cont = class_left_by_two.simd_eq(Simd::splat(0u8));
        let next3_is_cont = class_left_by_three.simd_eq(Simd::splat(0u8));

        let missing_after_2 = is2 & (!next1_is_cont);
        let missing_after_3 = is3 & (!(next1_is_cont & next2_is_cont));
        let missing_after_4 = is4 & (!(next1_is_cont & next2_is_cont & next3_is_cont));

        error_mask_for_window |= missing_after_2 | missing_after_3 | missing_after_4;

        // 5) EOF incomplete sequence check over last up-to-4 real lanes (backward-consistent)
        if window_start_index + SIMD_LANE_COUNT >= input.len() {
            let number_of_valid_lanes = (input.len() - window_start_index).min(SIMD_LANE_COUNT);
            let utf8_class_array = utf8_class_vector.to_array();

            // Count trailing continuation bytes (class == 0), up to 3
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

            if trailing_continuation_count > 0 {
                // Need a preceding lead whose class-1 equals c
                if scan_index == 0 {
                    is_invalid = true;
                } else {
                    let lead_class_value = utf8_class_array[scan_index - 1];
                    let is_lead = lead_class_value >= 2;
                    let needed_continuations = lead_class_value.saturating_sub(1) as usize;
                    if !(is_lead && needed_continuations == trailing_continuation_count) {
                        is_invalid = true;
                    }
                }
            }
        }

        is_invalid |= error_mask_for_window.any();
        window_start_index += SIMD_LANE_COUNT;
    }

    (!is_invalid, token_start_indices)
}

#[inline(always)]
fn load_window(input_bytes: &[u8], start_index: usize) -> Simd<u8, SIMD_LANE_COUNT> {
    let mut window_bytes = [0u8; SIMD_LANE_COUNT];

    if start_index < input_bytes.len() {
        let copy_count = (input_bytes.len() - start_index).min(SIMD_LANE_COUNT);
        let input_slice = &input_bytes[start_index..start_index + copy_count];

        window_bytes[..copy_count].copy_from_slice(input_slice);
    }

    Simd::from_array(window_bytes)
}

#[inline(always)]
fn shift_left_by_one_with_lookahead(
    current_window: Simd<u8, SIMD_LANE_COUNT>,
    lookahead_window: Simd<u8, SIMD_LANE_COUNT>,
) -> Simd<u8, SIMD_LANE_COUNT> {
    let mut shifted = current_window.rotate_elements_left::<1>();

    let lookahead_first_lane = lookahead_window.as_array()[0];
    let last_lane_mask = Mask::from_bitmask(1 << (SIMD_LANE_COUNT - 1));
    let injected_value = Simd::splat(lookahead_first_lane);

    shifted = last_lane_mask.select(injected_value, shifted);

    shifted
}

#[inline(always)]
fn classify_high_nibbles_to_utf8_classes(
    byte_vector: Simd<u8, SIMD_LANE_COUNT>,
) -> Simd<u8, SIMD_LANE_COUNT> {
    let high_nibbles = byte_vector >> Simd::splat(4u8);
    let mut utf8_class_vector = Simd::splat(1u8);

    let continuation_mask =
        high_nibbles.simd_ge(Simd::splat(0x8)) & high_nibbles.simd_le(Simd::splat(0xB));
    utf8_class_vector = continuation_mask.select(Simd::splat(0u8), utf8_class_vector);

    let lead_two_mask =
        high_nibbles.simd_ge(Simd::splat(0xC)) & high_nibbles.simd_le(Simd::splat(0xD));
    utf8_class_vector = lead_two_mask.select(Simd::splat(2u8), utf8_class_vector);

    let lead_three_mask = high_nibbles.simd_eq(Simd::splat(0xE));
    utf8_class_vector = lead_three_mask.select(Simd::splat(3u8), utf8_class_vector);

    let lead_four_mask = high_nibbles.simd_eq(Simd::splat(0xF));
    utf8_class_vector = lead_four_mask.select(Simd::splat(4u8), utf8_class_vector);

    utf8_class_vector
}

#[inline(always)]
fn shift_right_utf8_classes_by_one(
    class_vector: Simd<u8, SIMD_LANE_COUNT>,
    previous_class_vector: Simd<u8, SIMD_LANE_COUNT>,
) -> Simd<u8, SIMD_LANE_COUNT> {
    let mut shifted = class_vector.rotate_elements_right::<1>();

    let previous_last_lane = previous_class_vector.as_array()[SIMD_LANE_COUNT - 1];
    let first_lane_mask = Mask::from_bitmask(1);
    let injected_value = Simd::splat(previous_last_lane);

    shifted = first_lane_mask.select(injected_value, shifted);

    shifted
}

#[inline(always)]
fn shift_right_utf8_classes_by_two(
    class_vector: Simd<u8, SIMD_LANE_COUNT>,
    previous_class_vector: Simd<u8, SIMD_LANE_COUNT>,
) -> Simd<u8, SIMD_LANE_COUNT> {
    let mut shifted = class_vector.rotate_elements_right::<2>();

    let previous_class_array = previous_class_vector.as_array();
    let first_lane_mask = Mask::from_bitmask(1);
    let second_lane_mask = Mask::from_bitmask(1 << 1);
    let injected_first_value = Simd::splat(previous_class_array[SIMD_LANE_COUNT - 2]);
    let injected_second_value = Simd::splat(previous_class_array[SIMD_LANE_COUNT - 1]);

    shifted = first_lane_mask.select(injected_first_value, shifted);
    shifted = second_lane_mask.select(injected_second_value, shifted);

    shifted
}

#[inline(always)]
fn shift_left_utf8_classes_by_one(
    class_vector: Simd<u8, SIMD_LANE_COUNT>,
    next_class_vector: Simd<u8, SIMD_LANE_COUNT>,
) -> Simd<u8, SIMD_LANE_COUNT> {
    let mut shifted = class_vector.rotate_elements_left::<1>();

    let next_first_lane = next_class_vector.as_array()[0];
    let last_lane_mask = Mask::from_bitmask(1 << (SIMD_LANE_COUNT - 1));
    let injected_value = Simd::splat(next_first_lane);

    shifted = last_lane_mask.select(injected_value, shifted);

    shifted
}

#[inline(always)]
fn shift_left_utf8_classes_by_two(
    class_vector: Simd<u8, SIMD_LANE_COUNT>,
    next_class_vector: Simd<u8, SIMD_LANE_COUNT>,
) -> Simd<u8, SIMD_LANE_COUNT> {
    let mut shifted = class_vector.rotate_elements_left::<2>();

    let next_class_array = next_class_vector.as_array();
    let second_to_last_lane_mask = Mask::from_bitmask(1 << (SIMD_LANE_COUNT - 2));
    let last_lane_mask = Mask::from_bitmask(1 << (SIMD_LANE_COUNT - 1));
    let injected_second_to_last_value = Simd::splat(next_class_array[0]);
    let injected_last_value = Simd::splat(next_class_array[1]);

    shifted = second_to_last_lane_mask.select(injected_second_to_last_value, shifted);
    shifted = last_lane_mask.select(injected_last_value, shifted);

    shifted
}

#[inline(always)]
fn shift_left_utf8_classes_by_three(
    class_vector: Simd<u8, SIMD_LANE_COUNT>,
    next_class_vector: Simd<u8, SIMD_LANE_COUNT>,
) -> Simd<u8, SIMD_LANE_COUNT> {
    let mut shifted = class_vector.rotate_elements_left::<3>();

    let next_class_array = next_class_vector.as_array();
    let third_to_last_lane_mask = Mask::from_bitmask(1 << (SIMD_LANE_COUNT - 3));
    let second_to_last_lane_mask = Mask::from_bitmask(1 << (SIMD_LANE_COUNT - 2));
    let last_lane_mask = Mask::from_bitmask(1 << (SIMD_LANE_COUNT - 1));
    let injected_third_to_last_value = Simd::splat(next_class_array[0]);
    let injected_second_to_last_value = Simd::splat(next_class_array[1]);
    let injected_last_value = Simd::splat(next_class_array[2]);

    shifted = third_to_last_lane_mask.select(injected_third_to_last_value, shifted);
    shifted = second_to_last_lane_mask.select(injected_second_to_last_value, shifted);
    shifted = last_lane_mask.select(injected_last_value, shifted);

    shifted
}

#[inline(always)]
fn shift_right_utf8_classes_by_three(
    class_vector: Simd<u8, SIMD_LANE_COUNT>,
    previous_class_vector: Simd<u8, SIMD_LANE_COUNT>,
) -> Simd<u8, SIMD_LANE_COUNT> {
    let mut shifted = class_vector.rotate_elements_right::<3>();

    let previous_class_array = previous_class_vector.as_array();
    let first_lane_mask = Mask::from_bitmask(1);
    let second_lane_mask = Mask::from_bitmask(1 << 1);
    let third_lane_mask = Mask::from_bitmask(1 << 2);
    let injected_first_value = Simd::splat(previous_class_array[SIMD_LANE_COUNT - 3]);
    let injected_second_value = Simd::splat(previous_class_array[SIMD_LANE_COUNT - 2]);
    let injected_third_value = Simd::splat(previous_class_array[SIMD_LANE_COUNT - 1]);

    shifted = first_lane_mask.select(injected_first_value, shifted);
    shifted = second_lane_mask.select(injected_second_value, shifted);
    shifted = third_lane_mask.select(injected_third_value, shifted);

    shifted
}

#[inline(always)]
fn fill_significant_start_indices(
    input_vector: Simd<u8, SIMD_LANE_COUNT>,
    window_start_index: usize,
    window_length: usize,
    significant_start_indices: &mut Vec<usize>,
    previous_was_whitespace: &mut u64,
) {
    let tail_mask: u64 = if window_length == SIMD_LANE_COUNT {
        BITLIST_LIMIT
    } else {
        (1u64 << window_length) - 1
    };

    let space = input_vector.simd_eq(Simd::splat(b' ')).to_bitmask();
    let tab = input_vector.simd_eq(Simd::splat(b'\t')).to_bitmask();
    let newline = input_vector.simd_eq(Simd::splat(b'\n')).to_bitmask();
    let carriage_return = input_vector.simd_eq(Simd::splat(b'\r')).to_bitmask();

    let whitespace_bits = (space | tab | newline | carriage_return) & tail_mask;
    let non_whitespace_bits = !whitespace_bits;

    let prev_ws_bit = *previous_was_whitespace & 1;
    let shifted_ws = ((whitespace_bits << 1) | prev_ws_bit) & tail_mask;
    let mut start_bits = non_whitespace_bits & shifted_ws;

    while start_bits != 0 {
        let lane = start_bits.trailing_zeros() as usize;

        significant_start_indices.push(window_start_index + lane);

        start_bits &= start_bits - 1;
    }

    if window_length == SIMD_LANE_COUNT {
        *previous_was_whitespace = (whitespace_bits >> (window_length - 1)) & 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        assert!(validate_utf8_and_find_token_starts(&all_valid_utf8).0);
    }

    #[test]
    fn test_token_start_indices() {
        let input = b"Hello, \nWorld!\tThis is a test.\r\n";
        let (_, indices) = validate_utf8_and_find_token_starts(input);
        let expected_indices = vec![
            0,  // 'H'
            8,  // 'W'
            15, // 'T'
            20, // 'i'
            23, // 'a'
            25, // 't'
        ];

        assert_eq!(indices, expected_indices);
    }
}

// https://www.cl.cam.ac.uk/~mgk25/ucs/examples/UTF-8-test.txt
#[cfg(test)]
mod kuhn_tests {
    use super::validate_utf8_and_find_token_starts;

    fn assert_valid(bytes: &[u8]) {
        assert!(
            validate_utf8_and_find_token_starts(bytes).0,
            "Expected valid, got invalid: {:x?}",
            bytes
        );
    }

    fn assert_invalid(bytes: &[u8]) {
        assert!(
            !validate_utf8_and_find_token_starts(bytes).0,
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
