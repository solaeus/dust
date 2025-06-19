#![macro_use]

macro_rules! get_boolean {
    ($address: expr, $memory: expr, $chunk: expr, $cells: expr) => {{
        match $address.memory {
            MemoryKind::CONSTANT => $address.index != 0,
            MemoryKind::CELL => get_from_cell!($address.index, $cells, Boolean),
            MemoryKind::HEAP => get_from_heap!($address.index, $memory, booleans),
            MemoryKind::STACK => get_from_stack!($address.index, $memory, booleans),
            _ => unreachable!(),
        }
    }};
}

macro_rules! take_or_decode_boolean {
    ($address: expr, $memory: expr, $chunk: expr, $cells: expr) => {{
        match $address.memory {
            MemoryKind::CONSTANT => $address.index != 0,
            MemoryKind::CELL => take_from_cell!($address.index, $cells, Boolean),
            MemoryKind::HEAP => take_from_heap!($address.index, $memory, booleans),
            MemoryKind::STACK => take_from_stack!($address.index, $memory, booleans),
            _ => unreachable!(),
        }
    }};
}

macro_rules! get_byte {
    ($address: expr, $memory: expr, $chunk: expr, $cells: expr) => {{
        match $address.memory {
            MemoryKind::CONSTANT => $address.index as u8,
            MemoryKind::CELL => get_from_cell!($address.index, $cells, Byte),
            MemoryKind::HEAP => get_from_heap!($address.index, $memory, bytes),
            MemoryKind::STACK => get_from_stack!($address.index, $memory, bytes),
            _ => unreachable!(),
        }
    }};
}

macro_rules! take_or_decode_byte {
    ($address: expr, $memory: expr, $chunk: expr, $cells: expr) => {{
        match $address.memory {
            MemoryKind::CONSTANT => $address.index as u8,
            MemoryKind::CELL => take_from_cell!($address.index, $cells, Byte),
            MemoryKind::HEAP => take_from_heap!($address.index, $memory, bytes),
            MemoryKind::STACK => take_from_stack!($address.index, $memory, bytes),
            _ => unreachable!(),
        }
    }};
}

macro_rules! get_character {
    ($address: expr, $memory: expr, $chunk: expr, $cells: expr) => {{
        match $address.memory {
            MemoryKind::CONSTANT => get_constant!($address.index, $chunk).as_character_or_panic(),
            MemoryKind::CELL => get_from_cell!($address.index, $cells, Character),
            MemoryKind::HEAP => get_from_heap!($address.index, $memory, characters),
            MemoryKind::STACK => get_from_stack!($address.index, $memory, characters),
            _ => unreachable!(),
        }
    }};
}

macro_rules! take_or_get_character {
    ($address: expr, $memory: expr, $chunk: expr, $cells: expr) => {{
        match $address.memory {
            MemoryKind::CONSTANT => get_constant!($address.index, $chunk).as_character_or_panic(),
            MemoryKind::CELL => take_from_cell!($address.index, $cells, Character),
            MemoryKind::HEAP => take_from_heap!($address.index, $memory, characters),
            MemoryKind::STACK => take_from_stack!($address.index, $memory, characters),
            _ => unreachable!(),
        }
    }};
}

macro_rules! get_float {
    ($address: expr, $memory: expr, $chunk: expr, $cells: expr) => {{
        match $address.memory {
            MemoryKind::CONSTANT => get_constant!($address.index, $chunk).as_float_or_panic(),
            MemoryKind::CELL => get_from_cell!($address.index, $cells, Float),
            MemoryKind::HEAP => get_from_heap!($address.index, $memory, floats),
            MemoryKind::STACK => get_from_stack!($address.index, $memory, floats),
            _ => unreachable!(),
        }
    }};
}

macro_rules! take_or_get_float {
    ($address: expr, $memory: expr, $chunk: expr, $cells: expr) => {{
        match $address.memory {
            MemoryKind::CONSTANT => get_constant!($address.index, $chunk).as_float_or_panic(),
            MemoryKind::CELL => take_from_cell!($address.index, $cells, Float),
            MemoryKind::HEAP => take_from_heap!($address.index, $memory, floats),
            MemoryKind::STACK => take_from_stack!($address.index, $memory, floats),
            _ => unreachable!(),
        }
    }};
}

macro_rules! get_integer {
    ($address: expr, $memory: expr, $chunk: expr, $cells: expr) => {{
        match $address.memory {
            MemoryKind::CONSTANT => get_constant!($address.index, $chunk).as_integer_or_panic(),
            MemoryKind::CELL => get_from_cell!($address.index, $cells, Integer),
            MemoryKind::HEAP => get_from_heap!($address.index, $memory, integers),
            MemoryKind::STACK => get_from_stack!($address.index, $memory, integers),
            _ => unreachable!(),
        }
    }};
}

macro_rules! take_or_get_integer {
    ($address: expr, $memory: expr, $chunk: expr, $cells: expr) => {{
        match $address.memory {
            MemoryKind::CONSTANT => get_constant!($address.index, $chunk).as_integer_or_panic(),
            MemoryKind::CELL => take_from_cell!($address.index, $cells, Integer),
            MemoryKind::HEAP => take_from_heap!($address.index, $memory, integers),
            MemoryKind::STACK => take_from_stack!($address.index, $memory, integers),
            _ => unreachable!(),
        }
    }};
}

macro_rules! get_string {
    ($address: expr, $memory: expr, $chunk: expr, $cells: expr) => {{
        use crate::instruction::MemoryKind;

        match $address.memory {
            MemoryKind::CONSTANT => get_constant!($address.index, $chunk)
                .as_string_or_panic()
                .clone(),
            MemoryKind::CELL => get_from_cell!($address.index, $cells, String),
            MemoryKind::HEAP => get_from_heap!($address.index, $memory, strings),
            _ => unreachable!(),
        }
    }};
}

macro_rules! take_or_get_string {
    ($address: expr, $memory: expr, $chunk: expr, $cells: expr) => {{
        match $address.memory {
            MemoryKind::CONSTANT => get_constant!($address.index, $chunk)
                .as_string_or_panic()
                .clone(),
            MemoryKind::CELL => take_from_cell!($address.index, $cells, String),
            MemoryKind::HEAP => take_from_heap!($address.index, $memory, strings),
            _ => unreachable!(),
        }
    }};
}

macro_rules! get_list {
    ($address: expr, $memory: expr, $chunk: expr, $cells: expr) => {{
        match $address.memory {
            MemoryKind::CELL => get_from_cell!($address.index, $cells, List),
            MemoryKind::HEAP => get_from_heap!($address.index, $memory, lists),
            _ => unreachable!(),
        }
    }};
}

macro_rules! take_or_get_list {
    ($address: expr, $memory: expr, $chunk: expr, $cells: expr) => {{
        match $address.memory {
            MemoryKind::CELL => take_from_cell!($address.index, $cells, List),
            MemoryKind::HEAP => take_from_heap!($address.index, $memory, lists),
            _ => unreachable!(),
        }
    }};
}

macro_rules! get_function {
    ($address: expr, $memory: expr, $chunk: expr, $cells: expr) => {{
        match $address.memory {
            MemoryKind::CONSTANT => get_constant!($address.index, $chunk)
                .as_function_or_panic()
                .clone(),
            MemoryKind::CELL => get_from_cell!($address.index, $cells, Function),
            MemoryKind::HEAP => get_from_heap!($address.index, $memory, functions),
            MemoryKind::STACK => Arc::clone($chunk),
            _ => unreachable!(),
        }
    }};
}

macro_rules! take_or_get_function {
    ($address: expr, $memory: expr, $chunk: expr, $cells: expr) => {{
        match $address.memory {
            MemoryKind::CONSTANT => get_constant!($address.index, $chunk)
                .as_function_or_panic()
                .clone(),
            MemoryKind::CELL => take_from_cell!($address.index, $cells, Function),
            MemoryKind::HEAP => take_from_heap!($address.index, $memory, functions),
            MemoryKind::STACK => Arc::clone($chunk),
            _ => unreachable!(),
        }
    }};
}

macro_rules! set_boolean {
    ($address: expr, $memory: expr, $cells: expr, $boolean: expr) => {{
        match $address.memory {
            MemoryKind::CELL => set_cell!($address.index, $cells, Boolean, $boolean),
            MemoryKind::HEAP => set_to_heap!($address.index, $memory, booleans, $boolean),
            MemoryKind::STACK => set_to_stack!($address.index, $memory, booleans, $boolean),
            _ => unreachable!(),
        }
    }};
}

macro_rules! set_byte {
    ($address: expr, $memory: expr, $cells: expr, $byte: expr) => {{
        match $address.memory {
            MemoryKind::CELL => set_cell!($address.index, $cells, Byte, $byte),
            MemoryKind::HEAP => set_to_heap!($address.index, $memory, bytes, $byte),
            MemoryKind::STACK => set_to_stack!($address.index, $memory, bytes, $byte),
            _ => unreachable!(),
        }
    }};
}

macro_rules! set_character {
    ($address: expr, $memory: expr, $cells: expr, $character: expr) => {{
        match $address.memory {
            MemoryKind::CELL => set_cell!($address.index, $cells, Character, $character),
            MemoryKind::HEAP => set_to_heap!($address.index, $memory, characters, $character),
            MemoryKind::STACK => set_to_stack!($address.index, $memory, characters, $character),
            _ => unreachable!(),
        }
    }};
}

macro_rules! set_float {
    ($address: expr, $memory: expr, $cells: expr, $float: expr) => {{
        match $address.memory {
            MemoryKind::CELL => set_cell!($address.index, $cells, Float, $float),
            MemoryKind::HEAP => set_to_heap!($address.index, $memory, floats, $float),
            MemoryKind::STACK => set_to_stack!($address.index, $memory, floats, $float),
            _ => unreachable!(),
        }
    }};
}

macro_rules! set_integer {
    ($address: expr, $memory: expr, $cells: expr, $integer: expr) => {{
        match $address.memory {
            MemoryKind::CELL => set_cell!($address.index, $cells, Integer, $integer),
            MemoryKind::HEAP => set_to_heap!($address.index, $memory, integers, $integer),
            MemoryKind::STACK => set_to_stack!($address.index, $memory, integers, $integer),
            _ => unreachable!(),
        }
    }};
}

macro_rules! set_string {
    ($address: expr, $memory: expr, $cells: expr, $string: expr) => {{
        match $address.memory {
            MemoryKind::CELL => set_cell!($address.index, $cells, String, $string),
            MemoryKind::HEAP => set_to_heap!($address.index, $memory, strings, $string),
            _ => unreachable!(),
        }
    }};
}

macro_rules! set_list {
    ($address: expr, $memory: expr, $cells: expr, $list: expr) => {{
        match $address.memory {
            MemoryKind::CELL => set_cell!($address.index, $cells, List, $list),
            MemoryKind::HEAP => set_to_heap!($address.index, $memory, lists, $list),
            _ => unreachable!(),
        }
    }};
}

macro_rules! set_function {
    ($address: expr, $memory: expr, $cells: expr, $function: expr) => {{
        match $address.memory {
            MemoryKind::CELL => set_cell!($address.index, $cells, Function, $function),
            MemoryKind::HEAP => set_to_heap!($address.index, $memory, functions, $function),
            _ => unreachable!(),
        }
    }};
}

macro_rules! set_cell {
    ($index: expr, $cells: expr, $cell_value_variant: ident, $value: expr) => {{
        use crate::panic_vm::CellValue;

        let index = $index as usize;
        let cells = read_lock_cells!($cells);

        assert!(index < cells.len(), "Cell index out of bounds");

        *cells[index].lock.write().expect("Failed to lock cell") =
            CellValue::$cell_value_variant($value);
    }};
}

macro_rules! set_to_heap {
    ($index: expr, $memory: expr, $field: ident, $value: expr) => {{
        let index = $index as usize;

        assert!(
            index < $memory.heap.$field.len(),
            "Heap index out of bounds"
        );

        $memory.heap.$field[index] = HeapSlot::Open($value);
    }};
}

macro_rules! set_to_stack {
    ($index: expr, $memory: expr, $field: ident, $value: expr) => {{
        let index = $index as usize;

        assert!(
            index < $memory.stack.$field.len(),
            "Stack index out of bounds"
        );

        $memory.stack.$field[index] = $value;
    }};
}

macro_rules! read_lock_cells {
    ($cells: expr) => {{ $cells.read().expect("Failed to read cells") }};
}

macro_rules! get_from_cell {
    ($index: expr, $cells: expr, $cell_value_variant: ident) => {{
        use crate::panic_vm::CellValue;

        let index = $index as usize;
        let cells = read_lock_cells!($cells);

        assert!(index < cells.len(), "Cell index out of bounds");

        let cell_value = cells[index].lock.read().expect("Failed to read cell");

        if let CellValue::$cell_value_variant(value) = &*cell_value {
            value.clone()
        } else {
            panic!("Expected a boolean cell at index {}", index);
        }
    }};
}

macro_rules! get_constant {
    ($index: expr, $chunk: expr) => {{
        let index = $index as usize;
        let constants = $chunk.constants();

        assert!(index < constants.len(), "Constant index out of bounds");

        constants[index].clone()
    }};
}

macro_rules! get_from_heap {
    ($index: expr, $memory: expr, $field: ident) => {{
        use crate::panic_vm::HeapSlot;

        if let HeapSlot::Open(value) = get_heap_slot!($index, $memory, $field) {
            value.clone()
        } else {
            panic!("Closed heap slot at index {}", $index);
        }
    }};
}

macro_rules! take_heap_slot {
    ($index:expr, $memory:expr, $field:ident) => {{
        use std::mem::take;

        let index = $index as usize;

        assert!(
            index < $memory.heap.$field.len(),
            "Heap index out of bounds"
        );

        take(&mut $memory.heap.$field[index])
    }};
}

macro_rules! take_from_heap {
    ($index:expr, $memory:expr, $field:ident) => {{
        if let HeapSlot::Open(value) = take_heap_slot!($index, $memory, $field) {
            value
        } else {
            panic!("Attempted to take a closed heap slot at index {}", $index);
        }
    }};
}

macro_rules! get_heap_slot {
    ($index: expr, $memory: expr, $field: ident) => {{
        let index = $index as usize;

        assert!(
            index < $memory.heap.$field.len(),
            "Heap index out of bounds"
        );

        &$memory.heap.$field[index]
    }};
}

macro_rules! close_heap_slot {
    ($index: expr, $memory: expr, $field: ident) => {{
        let index = $index as usize;

        assert!(
            index < $memory.heap.$field.len(),
            "Heap index out of bounds"
        );

        $memory.heap.$field[index] = HeapSlot::Closed;
    }};
}

macro_rules! get_from_stack {
    ($index: expr, $memory: expr, $field: ident) => {{
        let index = $index as usize;

        assert!(
            index < $memory.stack.$field.len(),
            "Stack index out of bounds"
        );

        $memory.stack.$field[index].clone()
    }};
}

macro_rules! take_from_stack {
    ($index: expr, $memory: expr, $field: ident) => {{
        use std::mem::take;

        let index = $index as usize;

        assert!(
            index < $memory.stack.$field.len(),
            "Stack index out of bounds"
        );

        take(&mut $memory.stack.$field[index])
    }};
}

macro_rules! take_from_cell {
    ($index: expr, $memory: expr, $field: ident) => {{
        use std::mem::take;

        let index = $index as usize;
        let cells = read_lock_cells!($memory);

        assert!(index < cells.len(), "Cell index out of bounds");

        let mut cell_value = cells[index].lock.write().expect("Failed to write cell");

        if let CellValue::$field(value) = take(&mut *cell_value) {
            value
        } else {
            panic!("Expected a {} cell at index {}", stringify!($field), index);
        }
    }};
}

pub(crate) use get_boolean;
pub(crate) use get_byte;
pub(crate) use get_character;
pub(crate) use get_constant;
pub(crate) use get_float;
pub(crate) use get_from_cell;
pub(crate) use get_from_heap;
pub(crate) use get_from_stack;
pub(crate) use get_function;
pub(crate) use get_heap_slot;
pub(crate) use get_integer;
pub(crate) use get_list;
pub(crate) use get_string;
pub(crate) use read_lock_cells;
pub(crate) use set_boolean;
pub(crate) use set_byte;
pub(crate) use set_cell;
pub(crate) use set_character;
pub(crate) use set_float;
pub(crate) use set_function;
pub(crate) use set_integer;
pub(crate) use set_list;
pub(crate) use set_string;
pub(crate) use set_to_heap;
pub(crate) use set_to_stack;
