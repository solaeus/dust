#![macro_use]
pub use cells::*;
pub use constants::*;

#[macro_use]
pub mod cells {}

#[macro_use]
pub mod constants {
    macro_rules! copy_constant {
        ($index: expr, $constants:expr, $value_variant: ident) => {{
            assert!($index < $constants.len(), "Constant index out of bounds");

            let value = &$constants[$index];

            if let Value::$value_variant(value) = value {
                *value
            } else {
                panic!(
                    "Expected a {} constant at index {}, found {:?}",
                    stringify!($value_variant),
                    $index,
                    value
                );
            }
        }};
    }

    macro_rules! clone_constant {
        ($index: expr, $constants:expr, $value_variant: ident) => {{
            let index = $index as usize;

            assert!(index < $constants.len(), "Constant index out of bounds");

            let value = &$constants[index];

            if let Value::$value_variant(value) = value {
                value.clone()
            } else {
                panic!(
                    "Expected a {} constant at index {}, found {:?}",
                    stringify!($value_variant),
                    index,
                    value
                );
            }
        }};
    }

    macro_rules! get_constant {
        ($index: expr, $constants:expr, $value_variant: ident) => {{
            let index = $index as usize;

            assert!(index < $constants.len(), "Constant index out of bounds");

            let value = &$constants[index];

            if let Value::$value_variant(value) = value {
                value
            } else {
                panic!(
                    "Expected a {} constant at index {}, found {:?}",
                    stringify!($value_variant),
                    index,
                    value
                );
            }
        }};
    }
}
