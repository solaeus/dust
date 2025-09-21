use std::simd::{
    Simd,
    cmp::{SimdPartialEq, SimdPartialOrd},
    num::SimdUint,
};

const LANE_COUNT: usize = {
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

pub fn validate_utf8_simd(input_bytes: &[u8]) -> bool {
    let input_length = input_bytes.len();
    let mut window_start_index = 0usize;
    let mut is_invalid = false;

    while window_start_index < input_length && !is_invalid {
        let input_vector = load_window(input_bytes, window_start_index);
        let lookahead_vector_1 = load_window(input_bytes, window_start_index.saturating_add(1));

        // 1) Bound check: bytes must be <= 0xF4
        let bytes_greater_than_f4_mask = input_vector
            .saturating_sub(Simd::splat(0xF4))
            .simd_ne(Simd::splat(0));
        let mut error_mask_for_window = bytes_greater_than_f4_mask;

        // 2) Neighbor constraints via next-byte alignment
        let next_byte_vector = shift_left_by_one_with_lookahead(input_vector, lookahead_vector_1);
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
        let utf8_class_vector_next = classify_high_nibbles_to_utf8_classes(lookahead_vector_1);

        // 4) Sequence validity via shift/sub/add
        let class_shift_by_one =
            shift_right_utf8_classes_by_one(utf8_class_vector, utf8_class_vector_next);
        let class_shift_by_two =
            shift_right_utf8_classes_by_two(utf8_class_vector, utf8_class_vector_next);

        let accumulated_validation = utf8_class_vector
            + class_shift_by_one.saturating_sub(Simd::splat(1u8))
            + class_shift_by_two.saturating_sub(Simd::splat(2u8));

        let at_least_one_mask = accumulated_validation.simd_ge(Simd::splat(1u8));
        let is_lead_mask = utf8_class_vector.simd_ge(Simd::splat(2u8));

        // If lead: require accumulated_validation <= utf8_class_vector; else: always true.
        // Equivalent mask logic: (!is_lead) | (accumulated_validation <= utf8_class_vector)
        let within_original_class_if_lead_mask =
            (!is_lead_mask) | (accumulated_validation.simd_le(utf8_class_vector));

        let invalid_sequence_mask = (!at_least_one_mask) | (!within_original_class_if_lead_mask);

        error_mask_for_window |= invalid_sequence_mask;

        // 5) EOF incomplete sequence check over last up-to-4 real lanes
        if window_start_index + LANE_COUNT >= input_length {
            let number_of_valid_lanes = (input_length - window_start_index).min(LANE_COUNT);
            let mut required_continuations: u8 = 0;
            let utf8_class_array = utf8_class_vector.to_array();
            let suffix_start_index = number_of_valid_lanes.saturating_sub(4);
            let mut suffix_error_flag: u8 = 0;
            let mut scan_index = number_of_valid_lanes;
            let mut iteration_count = 0;
            while iteration_count < 4 {
                if scan_index == suffix_start_index {
                    break;
                }
                scan_index -= 1;
                let class_value = utf8_class_array[scan_index];
                let is_lead = (class_value >= 2) as u8;
                let needed_from_lead = is_lead * (class_value.saturating_sub(1));
                let can_consume_one = (required_continuations > 0) as u8;
                required_continuations = required_continuations
                    .saturating_sub(can_consume_one)
                    .saturating_add(needed_from_lead);
                let is_ascii = (class_value == 1) as u8;
                suffix_error_flag |= is_ascii & ((required_continuations != 0) as u8);
                iteration_count += 1;
            }
            suffix_error_flag |= (required_continuations != 0) as u8;
            if suffix_error_flag != 0 {
                is_invalid = true;
            }
        }

        is_invalid |= error_mask_for_window.any();
        window_start_index += LANE_COUNT;
    }

    !is_invalid
}

#[inline(always)]
fn load_window(input_bytes: &[u8], start_index: usize) -> Simd<u8, LANE_COUNT> {
    let mut window_bytes = [0u8; LANE_COUNT];

    if start_index < input_bytes.len() {
        let copy_count = (input_bytes.len() - start_index).min(LANE_COUNT);
        let input_slice = &input_bytes[start_index..start_index + copy_count];

        window_bytes[..copy_count].copy_from_slice(input_slice);
    }

    Simd::from_array(window_bytes)
}

#[inline(always)]
fn shift_left_by_one_with_lookahead(
    current_window: Simd<u8, LANE_COUNT>,
    lookahead_window: Simd<u8, LANE_COUNT>,
) -> Simd<u8, LANE_COUNT> {
    let current_array = current_window.to_array();
    let lookahead_array = lookahead_window.to_array();
    let mut shifted_array = [0u8; LANE_COUNT];
    let mut lane_index = 0;

    while lane_index + 1 < LANE_COUNT {
        shifted_array[lane_index] = current_array[lane_index + 1];
        lane_index += 1;
    }

    shifted_array[LANE_COUNT - 1] = lookahead_array[0];

    Simd::from_array(shifted_array)
}

#[inline(always)]
fn classify_high_nibbles_to_utf8_classes(
    byte_vector: Simd<u8, LANE_COUNT>,
) -> Simd<u8, LANE_COUNT> {
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
    class_vector: Simd<u8, LANE_COUNT>,
    next_class_vector: Simd<u8, LANE_COUNT>,
) -> Simd<u8, LANE_COUNT> {
    let class_array = class_vector.to_array();
    let next_class_array = next_class_vector.to_array();
    let mut shifted_array = [0u8; LANE_COUNT];
    let mut lane_index = LANE_COUNT;

    while lane_index > 0 {
        lane_index -= 1;

        if lane_index == 0 {
            shifted_array[lane_index] = next_class_array[LANE_COUNT - 1];
        } else {
            shifted_array[lane_index] = class_array[lane_index - 1];
        }
    }

    Simd::from_array(shifted_array)
}

#[inline(always)]
fn shift_right_utf8_classes_by_two(
    class_vector: Simd<u8, LANE_COUNT>,
    next_class_vector: Simd<u8, LANE_COUNT>,
) -> Simd<u8, LANE_COUNT> {
    let class_array = class_vector.to_array();
    let next_class_array = next_class_vector.to_array();
    let mut shifted_array = [0u8; LANE_COUNT];
    let mut lane_index = LANE_COUNT;

    while lane_index > 0 {
        lane_index -= 1;

        if lane_index == 0 {
            shifted_array[lane_index] = next_class_array[LANE_COUNT - 2];
        } else if lane_index == 1 {
            shifted_array[lane_index] = next_class_array[LANE_COUNT - 1];
        } else {
            shifted_array[lane_index] = class_array[lane_index - 2];
        }
    }

    Simd::from_array(shifted_array)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_utf8() {
        let every_ascii = "\
            ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`\
            abcdefghijklmnopqrstuvwxyz{|}~\
            !\"#$%&'()*+,-./0123456789:;<=>?@\
            ";
        let thumbs_up_medium_skin_tone = "👍🏽";
        let valid_strings = [
            "",
            "Hello, world!",
            "こんにちは",
            "😊",
            "𠜎", // U+2000E
            every_ascii,
            thumbs_up_medium_skin_tone,
        ];

        for input in &valid_strings {
            assert!(
                validate_utf8_simd(input.as_bytes()),
                "Failed on input: {input}"
            );
        }
    }

    #[test]
    fn invalid_utf8() {
        let invalid_bytes = [
            vec![0x80],                   // Lone continuation byte
            vec![0xC0],                   // Overlong encoding start
            vec![0xC1],                   // Overlong encoding start
            vec![0xE0, 0x80],             // Incomplete sequence
            vec![0xE0, 0x9F],             // Invalid continuation byte for E0
            vec![0xED, 0xA0, 0x80],       // Surrogate half start
            vec![0xF4, 0x90, 0x80, 0x80], // Code point > U+10FFFF
            vec![0xF5, 0x80, 0x80, 0x80], // Invalid leading byte
            vec![0xFF],                   // Invalid byte
            vec![0xFE],                   // Invalid byte
        ];

        for bytes in &invalid_bytes {
            assert!(!validate_utf8_simd(bytes), "Failed on input: {:?}", bytes);
        }
    }

    #[test]
    fn mixed_valid_invalid() {
        let mixed = b"Valid ASCII \xF4\x8F\xBF\xBF Invalid \xF4\x90\x80\x80";

        assert!(!validate_utf8_simd(mixed));
    }

    #[test]
    fn edge_cases() {
        let edge_cases = [
            (true, vec![]),                       // Empty input
            (true, vec![0x00]),                   // Single null byte
            (true, vec![0x7F]),                   // Single ASCII DEL byte
            (true, vec![0xC2, 0xA2]),             // Valid 2-byte sequence (¢)
            (true, vec![0xE2, 0x82, 0xAC]),       // Valid 3-byte sequence (€)
            (true, vec![0xF0, 0x9F, 0x92, 0xA9]), // Valid 4-byte sequence (💩)
            (false, vec![0xF0, 0x9F, 0x92]),      // Incomplete 4-byte sequence
        ];

        for (expected, bytes) in edge_cases {
            assert_eq!(
                validate_utf8_simd(&bytes),
                expected,
                "Failed on input: {:?}",
                bytes
            );
        }
    }
}
