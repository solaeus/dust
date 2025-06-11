#![macro_use]

macro_rules! get_memory {
    ($memory:expr, $field:ident, $address:expr) => {{
        let index = $address.index as usize;

        assert!(
            index < $memory.$field.len(),
            "Memory access out of bounds in `get_memory`: {}",
            $address
        );

        &$memory.$field[index]
    }};
}

pub(crate) use get_memory;

macro_rules! get_register {
    ($memory:expr, $field:ident, $address:expr) => {{
        let index = $address.index as usize;

        assert!(
            index < $memory.registers.$field.len(),
            "Register access out of bounds in `get_register`: {}",
            $address
        );

        &$memory.registers.$field[index]
    }};
}

pub(crate) use get_register;

macro_rules! get_constant {
    ($chunk:expr, $field:ident, $address:expr) => {{
        let index = $address.index as usize;

        assert!(
            index < $chunk.$field.len(),
            "Constant access out of bounds in `get_constant`: {}",
            $address
        );

        &$chunk.$field[index]
    }};
}

pub(crate) use get_constant;

macro_rules! set_memory {
    ($memory:expr, $memory_field:ident, $index:expr, $value:expr) => {{
        let index = $index as usize;

        assert!(
            index < $memory.$memory_field.len(),
            "Memory access out of bounds in `set_memory`: {index}",
        );

        $memory.$memory_field[index] = $value;
    }};
}

pub(crate) use set_memory;

macro_rules! set_register {
    ($memory:expr, $memory_field:ident, $index:expr, $value:expr) => {{
        let index = $index as usize;

        assert!(
            index < $memory.registers.$memory_field.len(),
            "Register access out of bounds in `set_register`: {index}",
        );

        $memory.registers.$memory_field[index] = $value;
    }};
}

pub(crate) use set_register;

macro_rules! set {
    ($memory:expr, $memory_field:ident, $destination:expr, $value:expr) => {{
        if $destination.is_register {
            set_register!($memory, $memory_field, $destination.index, $value);
        } else {
            set_memory!($memory, $memory_field, $destination.index, $value);
        }
    }};
}

pub(crate) use set;

macro_rules! malformed_instruction {
    ($instruction: expr, $ip: expr) => {{
        panic!(
            "Malformed {} instruction at IP {}",
            $instruction.operation(),
            $ip
        );
    }};
}

pub(crate) use malformed_instruction;
