#![macro_use]

macro_rules! get_boolean {
    ($memory:expr, $address:expr) => {{
        let index = $address.index as usize;

        match $address.kind {
            AddressKind::BOOLEAN_CELL => {
                let read_cells = $memory
                    .cells
                    .read()
                    .expect("Failed to acquire read lock on cells");

                assert!(
                    index < read_cells.len(),
                    "Cell access out of bounds in `get_boolean`: {}",
                    $address
                );

                read_cells[index]
                    .value
                    .read()
                    .expect("Expected boolean cell")
                    .expect_boolean()
            }
            AddressKind::BOOLEAN_REGISTER => {
                assert!(
                    index < $memory.registers.booleans.len(),
                    "Register access out of bounds in `get_boolean`: {}",
                    $address
                );

                &$memory.registers.booleans[index]
            }
            AddressKind::BOOLEAN_MEMORY => {
                assert!(
                    index < $memory.booleans.len(),
                    "Memory access out of bounds in `get_boolean`: {}",
                    $address
                );

                &$memory.booleans[index]
            }
            _ => unexpected_address!($address),
        }
    }};
}

pub(crate) use get_boolean;

macro_rules! get_byte {
    ($memory:expr, $address:expr) => {{
        let index = $address.index as usize;

        match $address.kind {
            AddressKind::BYTE_CELL => {
                let read_cells = $memory
                    .cells
                    .read()
                    .expect("Failed to acquire read lock on cells");

                assert!(
                    index < read_cells.len(),
                    "Cell access out of bounds in `get_byte`: {}",
                    $address
                );

                read_cells[index].read_byte().expect("Expected byte cell")
            }
            AddressKind::BYTE_REGISTER => {
                assert!(
                    index < $memory.registers.bytes.len(),
                    "Register access out of bounds in `get_byte`: {}",
                    $address
                );

                $memory.registers.bytes[index]
            }
            AddressKind::BYTE_MEMORY => {
                assert!(
                    index < $memory.bytes.len(),
                    "Memory access out of bounds in `get_byte`: {}",
                    $address
                );

                $memory.bytes[index]
            }
            _ => unexpected_address!($address),
        }
    }};
}

pub(crate) use get_byte;

macro_rules! get_character {
    ($memory:expr, $chunk: expr, $address:expr) => {{
        let index = $address.index as usize;

        match $address.kind {
            AddressKind::CHARACTER_CELL => {
                let read_cells = $memory
                    .cells
                    .read()
                    .expect("Failed to acquire read lock on cells");

                assert!(
                    index < read_cells.len(),
                    "Cell access out of bounds in `get_character`: {}",
                    $address
                );

                &read_cells[index]
                    .read_character()
                    .expect("Expected character cell")
            }
            AddressKind::CHARACTER_CONSTANT => {
                assert!(
                    index < $chunk.character_constants.len(),
                    "Constant access out of bounds in `get_character`: {}",
                    $address
                );

                &$chunk.character_constants[index]
            }
            AddressKind::CHARACTER_REGISTER => {
                assert!(
                    index < $memory.registers.characters.len(),
                    "Register access out of bounds in `get_character`: {}",
                    $address
                );

                &$memory.registers.characters[index]
            }
            AddressKind::CHARACTER_MEMORY => {
                assert!(
                    index < $memory.characters.len(),
                    "Memory access out of bounds in `get_character`: {}",
                    $address
                );

                &$memory.characters[index]
            }
            _ => unexpected_address!($address),
        }
    }};
}

pub(crate) use get_character;

macro_rules! get_float {
    ($memory:expr, $chunk: expr, $address:expr) => {{
        let index = $address.index as usize;

        match $address.kind {
            AddressKind::FLOAT_CELL => {
                let read_cells = $memory
                    .cells
                    .read()
                    .expect("Failed to acquire read lock on cells");

                assert!(
                    index < read_cells.len(),
                    "Cell access out of bounds in `get_float`: {}",
                    $address
                );

                read_cells[index].read_float().expect("Expected float cell")
            }
            AddressKind::FLOAT_CONSTANT => {
                assert!(
                    index < $chunk.float_constants.len(),
                    "Constant access out of bounds in `get_float`: {}",
                    $address
                );

                $chunk.float_constants[index]
            }
            AddressKind::FLOAT_REGISTER => {
                assert!(
                    index < $memory.registers.floats.len(),
                    "Register access out of bounds in `get_float`: {}",
                    $address
                );

                $memory.registers.floats[index]
            }
            AddressKind::FLOAT_MEMORY => {
                assert!(
                    index < $memory.floats.len(),
                    "Memory access out of bounds in `get_float`: {}",
                    $address
                );

                $memory.floats[index]
            }
            _ => unexpected_address!($address),
        }
    }};
}

pub(crate) use get_float;

macro_rules! get_integer {
    ($memory:expr, $chunk: expr, $address:expr) => {{
        let index = $address.index as usize;

        match $address.kind {
            AddressKind::INTEGER_CELL => {
                let read_cells = $memory
                    .cells
                    .read()
                    .expect("Failed to acquire read lock on cells");

                assert!(
                    index < read_cells.len(),
                    "Cell access out of bounds in `get_integer`: {}",
                    $address
                );

                read_cells[index]
                    .read_integer()
                    .expect("Expected integer cell")
            }
            AddressKind::INTEGER_CONSTANT => {
                assert!(
                    index < $chunk.integer_constants.len(),
                    "Constant access out of bounds in `get_integer`: {}",
                    $address
                );

                $chunk.integer_constants[index]
            }
            AddressKind::INTEGER_REGISTER => {
                assert!(
                    index < $memory.registers.integers.len(),
                    "Register access out of bounds in `get_integer`: {}",
                    $address
                );

                $memory.registers.integers[index]
            }
            AddressKind::INTEGER_MEMORY => {
                assert!(
                    index < $memory.integers.len(),
                    "Memory access out of bounds in `get_integer`: {}",
                    $address
                );

                $memory.integers[index]
            }
            _ => unexpected_address!($address),
        }
    }};
}

pub(crate) use get_integer;

macro_rules! get_string {
    ($memory:expr, $chunk: expr, $address:expr) => {{
        let index = $address.index as usize;

        match $address.kind {
            AddressKind::STRING_CELL => {
                let read_cells = $memory
                    .cells
                    .read()
                    .expect("Failed to acquire read lock on cells");

                assert!(
                    index < read_cells.len(),
                    "Cell access out of bounds in `get_string`: {}",
                    $address
                );

                &read_cells[index]
                    .read_string()
                    .expect("Expected string cell")
            }
            AddressKind::STRING_CONSTANT => {
                assert!(
                    index < $chunk.string_constants.len(),
                    "Constant access out of bounds in `get_string`: {}",
                    $address
                );

                &$chunk.string_constants[index]
            }
            AddressKind::STRING_REGISTER => {
                assert!(
                    index < $memory.registers.strings.len(),
                    "Register access out of bounds in `get_string`: {}",
                    $address
                );

                &$memory.registers.strings[index]
            }
            AddressKind::STRING_MEMORY => {
                assert!(
                    index < $memory.strings.len(),
                    "Memory access out of bounds in `get_string`: {}",
                    $address
                );

                &$memory.strings[index]
            }
            _ => unexpected_address!($address),
        }
    }};
}

pub(crate) use get_string;

macro_rules! get_list {
    ($memory:expr, $chunk: expr, $address:expr) => {{
        let index = $address.index as usize;

        match $address.kind {
            AddressKind::LIST_CELL => {
                let read_cells = $memory
                    .cells
                    .read()
                    .expect("Failed to acquire read lock on cells");

                assert!(
                    index < read_cells.len(),
                    "Cell access out of bounds in `get_list`: {}",
                    $address
                );

                &read_cells[index].read_list().expect("Expected list cell")
            }
            AddressKind::LIST_REGISTER => {
                assert!(
                    index < $memory.registers.lists.len(),
                    "Register access out of bounds in `get_list`: {}",
                    $address
                );

                &$memory.registers.lists[index]
            }
            AddressKind::LIST_MEMORY => {
                assert!(
                    index < $memory.lists.len(),
                    "Memory access out of bounds in `get_list`: {}",
                    $address
                );

                &$memory.lists[index]
            }
            _ => unexpected_address!($address),
        }
    }};
}

pub(crate) use get_list;

macro_rules! get_function {
    ($memory:expr, $chunk: expr, $address:expr) => {{
        let index = $address.index as usize;

        match $address.kind {
            AddressKind::FUNCTION_CELL => {
                let read_cells = $memory
                    .cells
                    .read()
                    .expect("Failed to acquire read lock on cells");

                assert!(
                    index < read_cells.len(),
                    "Cell access out of bounds in `get_function`: {}",
                    $address
                );

                read_cells[index]
                    .read_function()
                    .expect("Expected function cell")
            }
            AddressKind::FUNCTION_PROTOTYPE => {
                assert!(
                    index < $chunk.prototypes.len(),
                    "Constant access out of bounds in `get_function`: {}",
                    $address
                );

                $chunk.prototypes[index]
            }
            AddressKind::FUNCTION_REGISTER => {
                assert!(
                    index < $memory.registers.functions.len(),
                    "Register access out of bounds in `get_function`: {}",
                    $address
                );

                $memory.registers.functions[index]
            }
            AddressKind::FUNCTION_MEMORY => {
                assert!(
                    index < $memory.functions.len(),
                    "Memory access out of bounds in `get_function`: {}",
                    $address
                );

                $memory.functions[index]
            }
            _ => unexpected_address!($address),
        }
    }};
}

pub(crate) use get_function;

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

macro_rules! get_boolean_from_cells {
    ($memory:expr, $index:expr) => {{
        let index = $index as usize;
        let read_cells = $memory
            .cells
            .read()
            .expect("Failed to acquire read lock on cells");

        assert!(
            index < read_cells.len(),
            "Cell access out of bounds in `get_cell`: {index}",
        );

        read_cells[index]
            .read_boolean()
            .expect("Expected boolean cell")
    }};
}

pub(crate) use get_boolean_from_cells;

macro_rules! get_byte_from_cells {
    ($memory:expr, $index:expr) => {{
        let index = $index as usize;
        let read_cells = $memory
            .cells
            .read()
            .expect("Failed to acquire read lock on cells");

        assert!(
            index < read_cells.len(),
            "Cell access out of bounds in `get_cell`: {index}",
        );

        read_cells[index].read_byte().expect("Expected byte cell")
    }};
}

pub(crate) use get_byte_from_cells;

macro_rules! get_character_from_cells {
    ($memory:expr, $index:expr) => {{
        let index = $index as usize;
        let read_cells = $memory
            .cells
            .read()
            .expect("Failed to acquire read lock on cells");

        assert!(
            index < read_cells.len(),
            "Cell access out of bounds in `get_cell`: {index}",
        );

        read_cells[index]
            .read_character()
            .expect("Expected character cell")
    }};
}

pub(crate) use get_character_from_cells;

macro_rules! get_float_from_cells {
    ($memory:expr, $index:expr) => {{
        let index = $index as usize;
        let read_cells = $memory
            .cells
            .read()
            .expect("Failed to acquire read lock on cells");

        assert!(
            index < read_cells.len(),
            "Cell access out of bounds in `get_cell`: {index}",
        );

        read_cells[index].read_float().expect("Expected float cell")
    }};
}

pub(crate) use get_float_from_cells;

macro_rules! get_integer_from_cells {
    ($memory:expr, $index:expr) => {{
        let index = $index as usize;
        let read_cells = $memory
            .cells
            .read()
            .expect("Failed to acquire read lock on cells");

        assert!(
            index < read_cells.len(),
            "Cell access out of bounds in `get_cell`: {index}",
        );

        read_cells[index]
            .read_integer()
            .expect("Expected integer cell")
    }};
}

pub(crate) use get_integer_from_cells;

macro_rules! get_string_from_cells {
    ($memory:expr, $index:expr) => {{
        let index = $index as usize;
        let read_cells = $memory
            .cells
            .read()
            .expect("Failed to acquire read lock on cells");

        assert!(
            index < read_cells.len(),
            "Cell access out of bounds in `get_cell`: {index}",
        );

        read_cells[index]
            .read_string()
            .expect("Expected string cell")
    }};
}

pub(crate) use get_string_from_cells;

macro_rules! get_list_from_cells {
    ($memory:expr, $index:expr) => {{
        let index = $index as usize;
        let read_cells = $memory
            .cells
            .read()
            .expect("Failed to acquire read lock on cells");

        assert!(
            index < read_cells.len(),
            "Cell access out of bounds in `get_cell`: {index}",
        );

        read_cells[index].read_list().expect("Expected list cell")
    }};
}

pub(crate) use get_list_from_cells;

macro_rules! get_function_from_cells {
    ($memory:expr, $index:expr) => {{
        let index = $index as usize;
        let read_cells = $memory
            .cells
            .read()
            .expect("Failed to acquire read lock on cells");

        assert!(
            index < read_cells.len(),
            "Cell access out of bounds in `get_cell`: {index}",
        );

        read_cells[index]
            .read_function()
            .expect("Expected function cell")
    }};
}

pub(crate) use get_function_from_cells;

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

macro_rules! unexpected_address {
    ($address: expr) => {
        panic!("Unexpected address kind in instruction: {}", $address)
    };
}

pub(crate) use unexpected_address;
