//! The Dust programming language library.
#![expect(incomplete_features)]
#![feature(
    current_thread_id,
    generic_const_exprs,
    inherent_associated_types,
    iterator_try_collect,
    thread_id_value
)]

use smallvec::{Array, SmallVec};

pub mod compiler;
mod constants;
pub mod crate_config;
pub mod disassembler;
pub mod dust_type;
pub mod dust_value;
pub mod error;
pub mod instruction;
pub mod lexer;
mod native_function;
pub mod parser;
pub mod program;
pub mod prototype;
pub mod source;
pub mod syntax;
pub mod token;

#[cfg(feature = "mimalloc")]
mod allocator {
    use mimalloc::MiMalloc;

    #[global_allocator]
    static GLOBAL: MiMalloc = MiMalloc;
}

/// Determines an optimal inline capacity for `SmallVec<T>` based on the size of `T` and the target
/// platform's pointer size. Given a minimum capacity, it determines if extra elements can be added
/// without increasing the stack size. If there is no desired minimum, pass 0 as the `MINIMUM`
/// parameter. The returned value is always within `2..=(MINIMUM + 16)`.
///
/// For example, on 64-bit platforms, `optimize_inline_capacity::<u32, 4>()` returns 5 because
/// `SmallVec<[u32; 5]>` has the same stack size as `SmallVec<[u32; 4]>`.
const fn optimize_inline_capacity<T, const MINIMUM: usize>() -> usize
where
    [T; MINIMUM]: Array,
    [T; MINIMUM + 1]: Array,
    [T; MINIMUM + 2]: Array,
    [T; MINIMUM + 3]: Array,
    [T; MINIMUM + 4]: Array,
    [T; MINIMUM + 5]: Array,
    [T; MINIMUM + 6]: Array,
    [T; MINIMUM + 7]: Array,
    [T; MINIMUM + 8]: Array,
    [T; MINIMUM + 9]: Array,
    [T; MINIMUM + 10]: Array,
    [T; MINIMUM + 11]: Array,
    [T; MINIMUM + 12]: Array,
    [T; MINIMUM + 13]: Array,
    [T; MINIMUM + 14]: Array,
    [T; MINIMUM + 15]: Array,
    [T; MINIMUM + 16]: Array,
{
    let minimum_small_vec_size = size_of::<SmallVec<[T; MINIMUM]>>();
    let vec_size = size_of::<Vec<T>>();

    if minimum_small_vec_size > vec_size && MINIMUM <= 2 {
        return 2;
    }

    let target_size = if minimum_small_vec_size > vec_size {
        minimum_small_vec_size
    } else {
        vec_size
    };
    let checks = [
        size_of::<SmallVec<[T; MINIMUM + 1]>>() <= target_size,
        size_of::<SmallVec<[T; MINIMUM + 2]>>() <= target_size,
        size_of::<SmallVec<[T; MINIMUM + 3]>>() <= target_size,
        size_of::<SmallVec<[T; MINIMUM + 4]>>() <= target_size,
        size_of::<SmallVec<[T; MINIMUM + 5]>>() <= target_size,
        size_of::<SmallVec<[T; MINIMUM + 6]>>() <= target_size,
        size_of::<SmallVec<[T; MINIMUM + 7]>>() <= target_size,
        size_of::<SmallVec<[T; MINIMUM + 8]>>() <= target_size,
        size_of::<SmallVec<[T; MINIMUM + 9]>>() <= target_size,
        size_of::<SmallVec<[T; MINIMUM + 10]>>() <= target_size,
        size_of::<SmallVec<[T; MINIMUM + 11]>>() <= target_size,
        size_of::<SmallVec<[T; MINIMUM + 12]>>() <= target_size,
        size_of::<SmallVec<[T; MINIMUM + 13]>>() <= target_size,
        size_of::<SmallVec<[T; MINIMUM + 14]>>() <= target_size,
        size_of::<SmallVec<[T; MINIMUM + 15]>>() <= target_size,
        size_of::<SmallVec<[T; MINIMUM + 16]>>() <= target_size,
    ];

    let mut index = 0;

    while index < checks.len() && checks[index] {
        index += 1;
    }

    let capacity = MINIMUM + index;

    if capacity < 2 {
        2
    } else {
        capacity
    }
}

#[cfg(test)]
#[expect(clippy::disallowed_macros)]
mod tests {
    use super::*;

    #[test]
    fn optimal_inline_capacity_unit() {
        if cfg!(target_pointer_width = "64") {
            assert_eq!(optimize_inline_capacity::<(), 0>(), 16);
        } else {
            todo!()
        }
    }

    #[test]
    fn optimal_inline_capacity_u8() {
        if cfg!(target_pointer_width = "64") {
            assert_eq!(optimize_inline_capacity::<u8, 0>(), 8);
        } else {
            todo!()
        }
    }

    #[test]
    fn optimal_inline_capacity_u16() {
        if cfg!(target_pointer_width = "64") {
            assert_eq!(optimize_inline_capacity::<u16, 0>(), 4);
        } else {
            todo!()
        }
    }

    #[test]
    fn optimal_inline_capacity_u32() {
        if cfg!(target_pointer_width = "64") {
            assert_eq!(optimize_inline_capacity::<u32, 0>(), 2);
            assert_eq!(optimize_inline_capacity::<u32, 4>(), 5);
        } else {
            todo!()
        }
    }

    #[test]
    fn optimal_inline_capacity_u64() {
        assert_eq!(optimize_inline_capacity::<u64, 0>(), 2);
    }

    #[test]
    fn optimal_inline_capacity_u128() {
        assert_eq!(optimize_inline_capacity::<u128, 0>(), 2);
    }
}
