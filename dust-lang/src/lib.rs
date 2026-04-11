//! The Dust programming language library.
#![expect(incomplete_features)]
#![feature(
    current_thread_id,
    generic_const_exprs,
    inherent_associated_types,
    iter_array_chunks,
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
/// without exceeding the size of `Vec<T>`.
const fn optimal_small_vec_inline_capacity<T>() -> usize {
    use smallvec::SmallVec;

    macro_rules! try_capacities {
         ($($capacity:literal),*) => {
            $(
                let small_vec_size = size_of::<SmallVec<[T; $capacity]>>();
                let vec_size = size_of::<Vec<T>>();

                if small_vec_size <= vec_size {
                    return $capacity;
                }
            )*
        };
    }

    try_capacities!(16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5);

    4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimal_small_vec_inline_capacity() {
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
