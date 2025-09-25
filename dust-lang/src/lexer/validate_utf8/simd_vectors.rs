use std::simd::{LaneCount, Simd, SupportedLaneCount};

pub const ZERO<const LANE_COUNT: usize>: Simd<u8, LANE_COUNT> = Simd::splat(0)
    where LaneCount<LANE_COUNT>: SupportedLaneCount;
pub const ONE<const LANE_COUNT: usize>: Simd<u8, LANE_COUNT> = Simd::splat(1)
    where LaneCount<LANE_COUNT>: SupportedLaneCount;
pub const TWO<const LANE_COUNT: usize>: Simd<u8, LANE_COUNT> = Simd::splat(2)
    where LaneCount<LANE_COUNT>: SupportedLaneCount;
pub const THREE<const LANE_COUNT: usize>: Simd<u8, LANE_COUNT> = Simd::splat(3)
    where LaneCount<LANE_COUNT>: SupportedLaneCount;
pub const FOUR<const LANE_COUNT: usize>: Simd<u8, LANE_COUNT> = Simd::splat(4)
    where LaneCount<LANE_COUNT>: SupportedLaneCount;

pub const LEAD_SURROGATE_PREFIX<const LANE_COUNT: usize>: Simd<u8, LANE_COUNT> = Simd::splat(0xED)
    where LaneCount<LANE_COUNT>: SupportedLaneCount;
pub const LEAD_MAX_UNICODE_PREFIX<const LANE_COUNT: usize>: Simd<u8, LANE_COUNT> = Simd::splat(0xF4)
    where LaneCount<LANE_COUNT>: SupportedLaneCount;
pub const LEAD_3BYTE_MIN_PREFIX<const LANE_COUNT: usize>: Simd<u8, LANE_COUNT> = Simd::splat(0xE0)
    where LaneCount<LANE_COUNT>: SupportedLaneCount;
pub const LEAD_4BYTE_MIN_PREFIX<const LANE_COUNT: usize>: Simd<u8, LANE_COUNT> = Simd::splat(0xF0)
    where LaneCount<LANE_COUNT>: SupportedLaneCount;

pub const FORBIDDEN_OVERLONG_C0<const LANE_COUNT: usize>: Simd<u8, LANE_COUNT> = Simd::splat(0xC0)
    where LaneCount<LANE_COUNT>: SupportedLaneCount;
pub const FORBIDDEN_OVERLONG_C1<const LANE_COUNT: usize>: Simd<u8, LANE_COUNT> = Simd::splat(0xC1)
    where LaneCount<LANE_COUNT>: SupportedLaneCount;

pub const SECOND_MAX_AFTER_ED<const LANE_COUNT: usize>: Simd<u8, LANE_COUNT> = Simd::splat(0x9F)
    where LaneCount<LANE_COUNT>: SupportedLaneCount;
pub const SECOND_MAX_AFTER_F4<const LANE_COUNT: usize>: Simd<u8, LANE_COUNT> = Simd::splat(0x8F)
    where LaneCount<LANE_COUNT>: SupportedLaneCount;
pub const SECOND_MIN_AFTER_E0<const LANE_COUNT: usize>: Simd<u8, LANE_COUNT> = Simd::splat(0xA0)
    where LaneCount<LANE_COUNT>: SupportedLaneCount;
pub const SECOND_MIN_AFTER_F0<const LANE_COUNT: usize>: Simd<u8, LANE_COUNT> = Simd::splat(0x90)
    where LaneCount<LANE_COUNT>: SupportedLaneCount;

pub const HIGH_NIBBLE_CONT_MIN<const LANE_COUNT: usize>: Simd<u8, LANE_COUNT> = Simd::splat(0x8)
    where LaneCount<LANE_COUNT>: SupportedLaneCount;
pub const HIGH_NIBBLE_CONT_MAX<const LANE_COUNT: usize>: Simd<u8, LANE_COUNT> = Simd::splat(0xB)
    where LaneCount<LANE_COUNT>: SupportedLaneCount;
pub const HIGH_NIBBLE_TWO_BYTE_MIN<const LANE_COUNT: usize>: Simd<u8, LANE_COUNT> = Simd::splat(0xC)
    where LaneCount<LANE_COUNT>: SupportedLaneCount;
pub const HIGH_NIBBLE_TWO_BYTE_MAX<const LANE_COUNT: usize>: Simd<u8, LANE_COUNT> = Simd::splat(0xD)
    where LaneCount<LANE_COUNT>: SupportedLaneCount;
pub const HIGH_NIBBLE_THREE_BYTE<const LANE_COUNT: usize>: Simd<u8, LANE_COUNT> = Simd::splat(0xE)
    where LaneCount<LANE_COUNT>: SupportedLaneCount;
pub const HIGH_NIBBLE_FOUR_BYTE<const LANE_COUNT: usize>: Simd<u8, LANE_COUNT> = Simd::splat(0xF)
    where LaneCount<LANE_COUNT>: SupportedLaneCount;

pub const SHIFT_NIBBLE<const LANE_COUNT: usize>: Simd<u8, LANE_COUNT> = Simd::splat(4)
    where LaneCount<LANE_COUNT>: SupportedLaneCount;
