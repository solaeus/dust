use std::cmp::Ordering;

use crate::{
    instruction::OperandType,
    jit_vm::{
        Object, ObjectPool, ThreadStatus, object::ObjectValue, object_pool::ObjectIndex,
        thread_pool::ThreadContext,
    },
};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn allocate_list(
    list_type: i8,
    list_length: i64,
    thread_context: *mut ThreadContext,
) -> i64 {
    let list_length = list_length as usize;
    let thread_context = unsafe { &mut *thread_context };
    let object_pool = unsafe { &mut *thread_context.object_pool_pointer };
    let register_stack = unsafe { &mut *thread_context.register_vec_pointer };
    let register_tags = unsafe { &mut *thread_context.register_tag_vec_pointer };
    let register_window = &register_stack[0..thread_context.registers_used];
    let register_tags_window = &register_tags[0..thread_context.registers_used];
    let object = match OperandType(list_type as u8) {
        OperandType::LIST_BOOLEAN => Object::boolean_list(vec![false; list_length]),
        OperandType::LIST_BYTE => Object::byte_list(vec![0; list_length]),
        OperandType::LIST_CHARACTER => Object::character_list(vec![char::default(); list_length]),
        OperandType::LIST_FLOAT => Object::float_list(vec![0.0; list_length]),
        OperandType::LIST_INTEGER => Object::integer_list(vec![0; list_length]),
        OperandType::LIST_FUNCTION => Object::function_list(vec![0; list_length]),
        OperandType::LIST_STRING | OperandType::LIST_LIST => {
            Object::object_list_with_capacity(list_length)
        }
        _ => panic!(
            "Unsupported type for list allocation: {}",
            OperandType(list_type as u8)
        ),
    };
    let object_index = object_pool.allocate(object, register_window, register_tags_window);

    object_index.encode() as i64
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn insert_into_list(
    encoded_object_index: i64,
    index: i64,
    item: i64,
    thread_context: *mut ThreadContext,
) {
    let object_index = ObjectIndex::decode(encoded_object_index as u64);
    let thread_context = unsafe { &mut *thread_context };
    let object_pool = unsafe { &mut *thread_context.object_pool_pointer };

    let object = object_pool
        .get_mut(object_index)
        .expect("List object not found");
    let index = index as usize;

    match &mut object.value {
        ObjectValue::BooleanList(booleans) => {
            let boolean = item != 0;

            if index == booleans.len() {
                booleans.push(boolean);
            } else if index < booleans.len() {
                booleans[index] = boolean;
            } else {
                panic!("Index out of bounds for list insertion");
            }
        }
        ObjectValue::ByteList(bytes) => {
            let byte = item as u8;

            if index == bytes.len() {
                bytes.push(byte);
            } else if index < bytes.len() {
                bytes[index] = byte;
            } else {
                panic!("Index out of bounds for list insertion");
            }
        }
        ObjectValue::CharacterList(characters) => {
            let character = char::from_u32(item as u32).unwrap_or_default();

            if index == characters.len() {
                characters.push(character);
            } else if index < characters.len() {
                characters[index] = character;
            } else {
                panic!("Index out of bounds for list insertion");
            }
        }
        ObjectValue::FloatList(floats) => {
            let float = f64::from_bits(item as u64);

            if index == floats.len() {
                floats.push(float);
            } else if index < floats.len() {
                floats[index] = float;
            } else {
                panic!("Index out of bounds for list insertion");
            }
        }
        ObjectValue::IntegerList(integers) => {
            if index == integers.len() {
                integers.push(item);
            } else if index < integers.len() {
                integers[index] = item;
            } else {
                panic!("Index out of bounds for list insertion");
            }
        }
        ObjectValue::ObjectList(object_pointers) => {
            let encoded_object_index = item as u64;

            if index == object_pointers.len() {
                object_pointers.push(encoded_object_index);
            } else if index < object_pointers.len() {
                object_pointers[index] = encoded_object_index;
            } else {
                panic!("Index out of bounds for list insertion");
            }
        }
        ObjectValue::FunctionList(function_index) => {
            let function_pointer = item as usize;

            if index == function_index.len() {
                function_index.push(function_pointer);
            } else if index < function_index.len() {
                function_index[index] = function_pointer;
            } else {
                panic!("Index out of bounds for list insertion");
            }
        }
        _ => panic!("Object is not a list"),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_from_list(
    list_pointer: i64,
    index: i64,
    thread_context: *mut ThreadContext,
) -> i64 {
    let object = unsafe { &mut *(list_pointer as *mut Object) };
    let index = index as usize;

    match &object.value {
        ObjectValue::BooleanList(booleans) => {
            if index < booleans.len() {
                if booleans[index] { 1 } else { 0 }
            } else {
                let thread_context = unsafe { &mut *thread_context };
                thread_context.status = ThreadStatus::ErrorListIndexOutOfBounds;

                0
            }
        }
        ObjectValue::ByteList(bytes) => {
            if index < bytes.len() {
                bytes[index] as i64
            } else {
                let thread_context = unsafe { &mut *thread_context };
                thread_context.status = ThreadStatus::ErrorListIndexOutOfBounds;

                0
            }
        }
        ObjectValue::CharacterList(characters) => {
            if index < characters.len() {
                characters[index] as u32 as i64
            } else {
                let thread_context = unsafe { &mut *thread_context };
                thread_context.status = ThreadStatus::ErrorListIndexOutOfBounds;

                0
            }
        }
        ObjectValue::FloatList(floats) => {
            if index < floats.len() {
                floats[index].to_bits() as i64
            } else {
                let thread_context = unsafe { &mut *thread_context };
                thread_context.status = ThreadStatus::ErrorListIndexOutOfBounds;

                0
            }
        }
        ObjectValue::IntegerList(integers) => {
            if index < integers.len() {
                integers[index]
            } else {
                let thread_context = unsafe { &mut *thread_context };
                thread_context.status = ThreadStatus::ErrorListIndexOutOfBounds;

                0
            }
        }
        ObjectValue::ObjectList(object_pointers) => {
            if index < object_pointers.len() {
                object_pointers[index] as i64
            } else {
                let thread_context = unsafe { &mut *thread_context };
                thread_context.status = ThreadStatus::ErrorListIndexOutOfBounds;

                0
            }
        }
        ObjectValue::FunctionList(function_indices) => {
            if index < function_indices.len() {
                function_indices[index] as i64
            } else {
                let thread_context = unsafe { &mut *thread_context };
                thread_context.status = ThreadStatus::ErrorListIndexOutOfBounds;

                0
            }
        }
        _ => panic!("Object is not a list"),
    }
}

fn compare_lists(
    comparator: Ordering,
    left: &[u64],
    right: &[u64],
    object_pool: &ObjectPool,
) -> bool {
    if left.len() != right.len() {
        return false;
    }

    for (left_index_encoded, right_index_encoded) in left.iter().zip(right.iter()) {
        let left_index = ObjectIndex::decode(*left_index_encoded);
        let right_index = ObjectIndex::decode(*right_index_encoded);

        let left_object = object_pool.get(left_index).expect("List object not found");
        let right_object = object_pool.get(right_index).expect("List object not found");

        let comparison = match (&left_object.value, &right_object.value) {
            (ObjectValue::BooleanList(left), ObjectValue::BooleanList(right)) => left.cmp(right),
            (ObjectValue::ByteList(left), ObjectValue::ByteList(right)) => left.cmp(right),
            (ObjectValue::CharacterList(left), ObjectValue::CharacterList(right)) => {
                left.cmp(right)
            }
            (ObjectValue::FloatList(left), ObjectValue::FloatList(right)) => match comparator {
                Ordering::Less => return left < right,
                Ordering::Equal => return left == right,
                Ordering::Greater => return left > right,
            },
            (ObjectValue::IntegerList(left), ObjectValue::IntegerList(right)) => left.cmp(right),
            (ObjectValue::ObjectList(left), ObjectValue::ObjectList(right)) => {
                return compare_lists(comparator, left, right, object_pool);
            }
            (ObjectValue::FunctionList(left), ObjectValue::FunctionList(right)) => left.cmp(right),
            _ => return false,
        };

        if comparison != comparator {
            return false;
        }
    }

    true
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn compare_lists_equal(
    encoded_left_index: i64,
    enoded_right_index: i64,
    object_pool_pointer: i64,
) -> i8 {
    let left_index = ObjectIndex::decode(encoded_left_index as u64);
    let right_index = ObjectIndex::decode(enoded_right_index as u64);
    let object_pool = unsafe { &mut *(object_pool_pointer as *mut ObjectPool) };

    let left = object_pool.get(left_index).expect("List object not found");
    let right = object_pool.get(right_index).expect("List object not found");

    let lists_are_equal = match (&left.value, &right.value) {
        (ObjectValue::BooleanList(left), ObjectValue::BooleanList(right)) => left == right,
        (ObjectValue::ByteList(left), ObjectValue::ByteList(right)) => left == right,
        (ObjectValue::CharacterList(left), ObjectValue::CharacterList(right)) => left == right,
        (ObjectValue::FloatList(left), ObjectValue::FloatList(right)) => left == right,
        (ObjectValue::IntegerList(left), ObjectValue::IntegerList(right)) => left == right,
        (ObjectValue::FunctionList(left), ObjectValue::FunctionList(right)) => left == right,
        (ObjectValue::ObjectList(left), ObjectValue::ObjectList(right)) => {
            compare_lists(Ordering::Equal, left, right, object_pool)
        }
        _ => false,
    };

    lists_are_equal as i8
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn compare_lists_less_than(
    encoded_left_index: i64,
    enoded_right_index: i64,
    object_pool_pointer: i64,
) -> i8 {
    let left_index = ObjectIndex::decode(encoded_left_index as u64);
    let right_index = ObjectIndex::decode(enoded_right_index as u64);
    let object_pool = unsafe { &mut *(object_pool_pointer as *mut ObjectPool) };

    let left = object_pool.get(left_index).expect("List object not found");
    let right = object_pool.get(right_index).expect("List object not found");

    let result = match (&left.value, &right.value) {
        (ObjectValue::BooleanList(a), ObjectValue::BooleanList(b)) => a < b,
        (ObjectValue::ByteList(a), ObjectValue::ByteList(b)) => a < b,
        (ObjectValue::CharacterList(a), ObjectValue::CharacterList(b)) => a < b,
        (ObjectValue::FloatList(a), ObjectValue::FloatList(b)) => a < b,
        (ObjectValue::IntegerList(a), ObjectValue::IntegerList(b)) => a < b,
        (ObjectValue::ObjectList(a), ObjectValue::ObjectList(b)) => {
            compare_lists(Ordering::Less, a, b, object_pool)
        }
        (ObjectValue::FunctionList(a), ObjectValue::FunctionList(b)) => a < b,
        _ => false,
    };

    result as i8
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn compare_lists_less_than_equal(
    encoded_left_index: i64,
    enoded_right_index: i64,
    object_pool_pointer: i64,
) -> i8 {
    let left_index = ObjectIndex::decode(encoded_left_index as u64);
    let right_index = ObjectIndex::decode(enoded_right_index as u64);
    let object_pool = unsafe { &mut *(object_pool_pointer as *mut ObjectPool) };

    let left = object_pool.get(left_index).expect("List object not found");
    let right = object_pool.get(right_index).expect("List object not found");

    let result = match (&left.value, &right.value) {
        (ObjectValue::BooleanList(a), ObjectValue::BooleanList(b)) => a < b,
        (ObjectValue::ByteList(a), ObjectValue::ByteList(b)) => a < b,
        (ObjectValue::CharacterList(a), ObjectValue::CharacterList(b)) => a < b,
        (ObjectValue::FloatList(a), ObjectValue::FloatList(b)) => a < b,
        (ObjectValue::IntegerList(a), ObjectValue::IntegerList(b)) => a < b,
        (ObjectValue::ObjectList(a), ObjectValue::ObjectList(b)) => {
            compare_lists(Ordering::Less, a, b, object_pool)
        }
        (ObjectValue::FunctionList(a), ObjectValue::FunctionList(b)) => a < b,
        _ => false,
    };

    result as i8
}
