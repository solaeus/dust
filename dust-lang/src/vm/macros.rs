#![macro_use]

macro_rules! read_register {
    ($index: expr, $registers: expr) => {{
        use tracing::trace;

        trace!("Reading register at index {}", $index);

        if $index < $registers.len() {
            $registers[$index]
        } else {
            return Err(RuntimeError::RegisterIndexOutOfBounds);
        }
    }};
}

macro_rules! read_register_mut {
    ($index: expr, $registers: expr) => {{
        use tracing::trace;

        trace!("Reading register mutably at index {}", $index);

        if $index < $registers.len() {
            &mut $registers[$index]
        } else {
            return Err(RuntimeError::RegisterIndexOutOfBounds);
        }
    }};
}

macro_rules! read_constant {
    ($index: expr, $constants: expr) => {{
        use tracing::trace;

        trace!("Reading constant at index {}", $index);

        if $index < $constants.len() {
            &$constants[$index]
        } else {
            return Err(RuntimeError::InvalidConstantIndex);
        }
    }};
}

macro_rules! read_object {
    ($index: expr, $object_pool: expr) => {{
        use tracing::trace;

        trace!("Reading object at index {}", $index);

        if $index < $object_pool.len() {
            &$object_pool[$index]
        } else {
            return Err(RuntimeError::ObjectIndexOutOfBounds);
        }
    }};
}
