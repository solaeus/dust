use std::array;

use crate::platform::{INFO_LEVELS, MAX_PACKED_VALUE};

pub const PAGE_SIZE: usize = 1024 * 8;
pub const LOG_MAX_PACKED_VALUE: usize = MAX_PACKED_VALUE.ilog2() as usize;

pub struct PageAllocator {
    info_tree: [Vec<PageInfo>; INFO_LEVELS],
}

impl PageAllocator {
    pub fn new() -> Self {
        Self {
            info_tree: array::from_fn(|_| Vec::new()),
        }
    }

    pub fn find(&mut self, page_count: usize) -> Option<(usize, usize)> {
        todo!()
    }

    pub fn grow(&mut self, base: usize, size: usize) {
        todo!()
    }
}

/// Packed summary type that contains three numbers, start, max and end, into a single 8-byte value.
/// Each of these represent the number of ones in a bitmap. They have a maximum value of 2^21 - 1,
/// or all three may be 2^21. The latter case is represented by setting just the 64th bit.
#[derive(Clone, Copy)]
struct PageInfo(u64);

impl PageInfo {
    fn new(start: u32, max: u32, end: u32) -> Self {
        if max == MAX_PACKED_VALUE {
            return Self(1 << 63);
        }

        let mask = (MAX_PACKED_VALUE - 1) as u64;
        let packed = (start as u64 & mask)
            | ((max as u64 & mask) << LOG_MAX_PACKED_VALUE)
            | ((end as u64 & mask) << (2 * LOG_MAX_PACKED_VALUE));

        Self(packed)
    }

    fn start(self) -> u32 {
        if (self.0 & (1u64 << 63)) != 0 {
            return MAX_PACKED_VALUE;
        }

        (self.0 & (MAX_PACKED_VALUE as u64 - 1)) as u32
    }

    fn max(self) -> u32 {
        if (self.0 & (1u64 << 63)) != 0 {
            return MAX_PACKED_VALUE;
        }

        ((self.0 >> LOG_MAX_PACKED_VALUE) & (MAX_PACKED_VALUE as u64 - 1)) as u32
    }

    fn end(self) -> u32 {
        if (self.0 & (1u64 << 63)) != 0 {
            return MAX_PACKED_VALUE;
        }

        ((self.0 >> (2 * LOG_MAX_PACKED_VALUE)) & (MAX_PACKED_VALUE as u64 - 1)) as u32
    }

    fn unpack(self) -> (u32, u32, u32) {
        if (self.0 & (1u64 << 63)) != 0 {
            return (MAX_PACKED_VALUE, MAX_PACKED_VALUE, MAX_PACKED_VALUE);
        }

        let mask = (MAX_PACKED_VALUE as u64) - 1;

        let start = (self.0 & mask) as u32;
        let max = ((self.0 >> LOG_MAX_PACKED_VALUE) & mask) as u32;
        let end = ((self.0 >> (2 * LOG_MAX_PACKED_VALUE)) & mask) as u32;

        (start, max, end)
    }

    fn merge(summaries: &[PageInfo], log_max_pages: usize) -> PageInfo {
        assert!(summaries.len() >= 1);

        let (mut start, mut max, mut end) = summaries[0].unpack();

        for index in 1..summaries.len() {
            let (this_start, this_max, this_end) = summaries[index].unpack();

            if start == ((index as u32) << log_max_pages) {
                start = start.wrapping_add(this_start);
            }

            max = max.max(end.wrapping_sub(this_start).max(this_max));

            if this_end == (1 << log_max_pages) {
                end = end.wrapping_add(1 << log_max_pages);
            } else {
                end = this_end;
            }
        }

        PageInfo::new(start, max, end)
    }
}

pub struct OffsetAddress {}
