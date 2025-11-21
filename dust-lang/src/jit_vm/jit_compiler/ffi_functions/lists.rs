use crate::{
    instruction::OperandType,
    jit_vm::{Object, ThreadStatus, object::ObjectValue, thread::ThreadContext},
};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn allocate_list(
    list_type: i8,
    list_length: usize,
    thread_context: *mut ThreadContext,
) -> i64 {
    let thread_context = unsafe { &mut *thread_context };
    let object_pool = unsafe { &mut *thread_context.object_pool_pointer };
    let register_stack = unsafe { &mut *thread_context.register_stack_vec_pointer };
    let register_tags = unsafe { &mut *thread_context.register_tags_vec_pointer };
    let register_stack_used_length = unsafe { *thread_context.register_stack_used_length_pointer };
    let register_window = &register_stack[0..register_stack_used_length];
    let register_tags_window = &register_tags[0..register_stack_used_length];
    let object = match OperandType(list_type as u8) {
        OperandType::LIST_BOOLEAN => Object::boolean_list(Vec::with_capacity(list_length)),
        OperandType::LIST_BYTE => Object::byte_list(Vec::with_capacity(list_length)),
        OperandType::LIST_CHARACTER => Object::character_list(Vec::with_capacity(list_length)),
        OperandType::LIST_FLOAT => Object::float_list(Vec::with_capacity(list_length)),
        OperandType::LIST_INTEGER => Object::integer_list(Vec::with_capacity(list_length)),
        OperandType::LIST_FUNCTION => Object::function_list(Vec::with_capacity(list_length)),
        OperandType::LIST_STRING | OperandType::LIST_LIST => {
            Object::object_list(Vec::with_capacity(list_length))
        }
        _ => panic!(
            "Unsupported type for list allocation: {}",
            OperandType(list_type as u8)
        ),
    };
    let object_pointer = object_pool.allocate(object, register_window, register_tags_window);

    object_pointer as i64
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn insert_into_list(list_pointer: i64, index: i64, item: i64) {
    let object = unsafe { &mut *(list_pointer as *mut Object) };
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
            let object_pointer = item as *mut Object;

            if index == object_pointers.len() {
                object_pointers.push(object_pointer);
            } else if index < object_pointers.len() {
                object_pointers[index] = object_pointer;
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn compare_lists_equal(
    left_list_pointer: i64,
    right_list_pointer: i64,
) -> i8 {
    let left = unsafe { &*(left_list_pointer as *mut Object) };
    let right = unsafe { &*(right_list_pointer as *mut Object) };

    let result = match (&left.value, &right.value) {
        (ObjectValue::BooleanList(a), ObjectValue::BooleanList(b)) => a == b,
        (ObjectValue::ByteList(a), ObjectValue::ByteList(b)) => a == b,
        (ObjectValue::CharacterList(a), ObjectValue::CharacterList(b)) => a == b,
        (ObjectValue::FloatList(a), ObjectValue::FloatList(b)) => a == b,
        (ObjectValue::IntegerList(a), ObjectValue::IntegerList(b)) => a == b,
        (ObjectValue::ObjectList(a), ObjectValue::ObjectList(b)) => {
            if a.len() != b.len() {
                return false as i8;
            }

            for (left, right) in a.iter().zip(b.iter()) {
                let left_obj = unsafe { &**left };
                let right_obj = unsafe { &**right };

                if left_obj.value != right_obj.value {
                    return false as i8;
                }
            }

            true
        }
        (ObjectValue::FunctionList(a), ObjectValue::FunctionList(b)) => a == b,
        _ => false,
    };

    result as i8
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn compare_lists_less_than(
    left_list_pointer: i64,
    right_list_pointer: i64,
) -> i8 {
    let left = unsafe { &*(left_list_pointer as *mut Object) };
    let right = unsafe { &*(right_list_pointer as *mut Object) };

    let result = match (&left.value, &right.value) {
        (ObjectValue::BooleanList(a), ObjectValue::BooleanList(b)) => a < b,
        (ObjectValue::ByteList(a), ObjectValue::ByteList(b)) => a < b,
        (ObjectValue::CharacterList(a), ObjectValue::CharacterList(b)) => a < b,
        (ObjectValue::FloatList(a), ObjectValue::FloatList(b)) => a < b,
        (ObjectValue::IntegerList(a), ObjectValue::IntegerList(b)) => a < b,
        (ObjectValue::ObjectList(a), ObjectValue::ObjectList(b)) => {
            if a.len() != b.len() {
                return (a.len() < b.len()) as i8;
            }

            for (left, right) in a.iter().zip(b.iter()) {
                let left_obj = unsafe { &**left };
                let right_obj = unsafe { &**right };

                if left_obj.value != right_obj.value {
                    return (left_obj.value < right_obj.value) as i8;
                }
            }

            false
        }
        (ObjectValue::FunctionList(a), ObjectValue::FunctionList(b)) => a < b,
        _ => false,
    };

    result as i8
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn compare_lists_less_than_equal(
    left_list_pointer: i64,
    right_list_pointer: i64,
) -> i8 {
    let left = unsafe { &*(left_list_pointer as *mut Object) };
    let right = unsafe { &*(right_list_pointer as *mut Object) };

    let result = match (&left.value, &right.value) {
        (ObjectValue::BooleanList(a), ObjectValue::BooleanList(b)) => a <= b,
        (ObjectValue::ByteList(a), ObjectValue::ByteList(b)) => a <= b,
        (ObjectValue::CharacterList(a), ObjectValue::CharacterList(b)) => a <= b,
        (ObjectValue::FloatList(a), ObjectValue::FloatList(b)) => a <= b,
        (ObjectValue::IntegerList(a), ObjectValue::IntegerList(b)) => a <= b,
        (ObjectValue::ObjectList(a), ObjectValue::ObjectList(b)) => {
            if a.len() != b.len() {
                return (a.len() < b.len()) as i8;
            }

            for (left, right) in a.iter().zip(b.iter()) {
                let left_obj = unsafe { &**left };
                let right_obj = unsafe { &**right };

                if left_obj.value != right_obj.value {
                    return (left_obj.value < right_obj.value) as i8;
                }
            }

            true
        }
        (ObjectValue::FunctionList(a), ObjectValue::FunctionList(b)) => a <= b,
        _ => false,
    };

    result as i8
}
