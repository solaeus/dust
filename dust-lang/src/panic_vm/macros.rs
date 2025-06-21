#![macro_use]
pub use cells::*;
pub use constants::*;
pub use heap::*;
pub use stack::*;

#[macro_use]
pub mod cells {
    macro_rules! copy_from_cell {
        ($index: expr, $memory: expr, $field: ident) => {{
            let index = $index as usize;

            assert!(
                index < $memory.cells.$field.len(),
                "Cell index out of bounds"
            );

            $memory.cells.$field[index]
        }};
    }
}

#[macro_use]
pub mod constants {
    macro_rules! copy_constant {
        ($index: expr, $constants:expr, $value_variant: ident) => {{
            let index = $index as usize;

            assert!(index < $constants.len(), "Constant index out of bounds");

            let value = &$constants[index];

            if let Value::$value_variant(value) = value {
                *value
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

#[macro_use]
pub mod heap {}

#[macro_use]
pub mod stack {
    macro_rules! copy_from_stack {
        ($index: expr, $memory: expr, $field: ident) => {{
            let index = $index as usize;

            assert!(
                index < $memory.stack.integers.len(),
                "Stack index out of bounds"
            );

            $memory.stack.$field[index]
        }};
    }

    macro_rules! clone_from_stack {
        ($index: expr, $memory: expr, $field: ident) => {{
            let index = $index as usize;

            assert!(
                index < $memory.stack.$field.len(),
                "Stack index out of bounds"
            );

            $memory.stack.$field[index].clone()
        }};
    }

    macro_rules! get_from_stack {
        ($index: expr, $memory: expr, $field: ident) => {{
            let index = $index as usize;

            assert!(
                index < $memory.stack.$field.len(),
                "Stack index out of bounds"
            );

            &$memory.stack.$field[index]
        }};
    }

    macro_rules! get_mut_from_stack {
        ($index: expr, $memory: expr, $field: ident) => {{
            let index = $index as usize;

            assert!(
                index < $memory.stack.$field.len(),
                "Stack index out of bounds"
            );

            &mut $memory.stack.$field[index]
        }};
    }

    macro_rules! get_mut_many_from_stack {
        ($indexes: expr, $memory: expr, $field: ident) => {{
            $memory
                .stack
                .$field
                .get_disjoint_mut($indexes)
                .unwrap_or_else(|error| {
                    panic!(
                        "{error} while getting multiple mutable references to stack field `{}` at indexes {:?}",
                        stringify!($field),
                        $indexes,
                    )
                })
        }};
    }

    macro_rules! set_to_stack {
        ($value: expr, $index: expr, $memory: expr, $field: ident) => {{
            let index = $index as usize;

            assert!(
                index < $memory.stack.$field.len(),
                "Stack index out of bounds"
            );

            $memory.stack.$field[index] = $value;
        }};
    }
}
