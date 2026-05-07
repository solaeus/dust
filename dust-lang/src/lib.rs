//! The Dust programming language library.
#![expect(incomplete_features)]
#![feature(
    current_thread_id,
    generic_const_exprs,
    inherent_associated_types,
    iterator_try_collect,
    thread_id_value
)]
#![macro_use]

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

/// Determines an optimal inline capacity for `SmallVec<T>` based on the size of `T` and the target
/// platform's pointer size. Given a minimum capacity, it determines if an extra element can be
/// added without increasing the stack size. A hard minimum of 2 is enforced, passing 0 or 1 will be
/// treated as 2.
///
/// For example, on 64-bit platforms, `optimal_inline_capacity!(u32, 4)` returns 5 because
/// `SmallVec<[u32; 5]>` has the same stack size as `SmallVec<[u32; 4]>`. This is an unconditional
/// win over hard coding 4 as the capacity.
#[macro_export]
macro_rules! optimal_inline_capacity {
    ($type: ty, $minimum: expr) => {{
        use smallvec::SmallVec;
        use $crate::try_capacities;

        const MINIMUM: usize = if $minimum > 2 { $minimum } else { 2 };
        let vec_size = size_of::<Vec<$type>>();
        let minimum_small_vec_size = size_of::<SmallVec<[$type; MINIMUM]>>();

        let compared_to_vec = try_capacities!(
            (3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16),
            $type,
            vec_size
        );
        let compared_to_minimum = try_capacities!(
            (3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16),
            $type,
            minimum_small_vec_size
        );

        if compared_to_vec > compared_to_minimum {
            compared_to_vec
        } else {
            compared_to_minimum
        }
    }};
}

#[macro_export]
macro_rules! try_capacities {
    (($($capacity: literal),*), $type: ty, $target_size: expr) => {
        {
            let mut result = MINIMUM;

            $(
                let small_vec_size = size_of::<SmallVec<[$type; $capacity]>>();

                if small_vec_size <= $target_size {
                    result = $capacity;
                }
            )*

            result
        }
    };
}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(target_pointer_width = "64")]
    fn test_optimal_inline_capacity() {
        assert_eq!(optimal_inline_capacity!(u8, 0), 8);
        assert_eq!(optimal_inline_capacity!(u32, 4), 5);
        assert_eq!(optimal_inline_capacity!((u32, u16), 0), 2);
        assert_eq!(optimal_inline_capacity!((u32, u16), 3), 3);
        assert_eq!(optimal_inline_capacity!(u64, 0), 2);
        assert_eq!(optimal_inline_capacity!(u128, 0), 2);
    }
}
