//! The Dust programming language library.
#![expect(incomplete_features)]
#![feature(
    current_thread_id,
    generic_const_exprs,
    inherent_associated_types,
    iterator_try_collect,
    thread_id_value
)]

pub mod compiler;
mod constants;
pub mod disassembler;
pub mod dust_type;
pub mod dust_value;
pub mod error;
mod instruction;
pub mod lexer;
mod native_function;
pub mod parser;
mod program;
pub mod project;
mod prototype;
pub mod source;
pub mod syntax;
mod token;
pub mod vm;

#[cfg(feature = "mimalloc")]
mod allocator {
    use mimalloc::MiMalloc;

    #[global_allocator]
    static GLOBAL: MiMalloc = MiMalloc;
}

/// Determines the optimal inline capacity for `SmallVec<T>` based on the size of `T` and the target
/// platform's pointer size.
///
/// Use this only for small types. The returned capacity is never less than 4, to ensure that the
/// overhead of using `SmallVec` is justified. Otherwise, the capacity is as large as possible
/// without exceeding the size of `Vec<T>` unless the size of `SmallVec<[T; 4]>` is already larger
/// than `Vec<T>`, in which case it uses as much capacity as possible without exceeding that size.
///
/// For example, on a 64-bit platform, `u8` returns 8 and `u32` returns 4. Note that this function
/// cannot account for the size of a type that owns the SmallVec. Due to padding, it may be possible
/// to use an even larger capacity without increasing the overall size of the owning type. That
/// level of optimization would have to be done case-by-case, would not account for the platform's
/// pointer size and would be fragile to refactoring.
const fn optimal_small_vec_inline_capacity<T>() -> usize {
    use smallvec::SmallVec;

    macro_rules! try_capacities {
        (($($capacity:literal),*), $target_size: expr) => {{
                $(
                    let small_vec_size = size_of::<SmallVec<[T; $capacity]>>();

                    if small_vec_size <= $target_size {
                        return $capacity;
                    }
                )*

                4
            }};
        }

    try_capacities!(
        (16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4),
        size_of::<Vec<T>>()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optimal_small_vec_inline_capacities() {
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(optimal_small_vec_inline_capacity::<u8>(), 8);
            assert_eq!(optimal_small_vec_inline_capacity::<u32>(), 4);
        }

        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(optimal_small_vec_inline_capacity::<u8>(), 16);
            assert_eq!(optimal_small_vec_inline_capacity::<u32>(), 8);
        }
    }
}
