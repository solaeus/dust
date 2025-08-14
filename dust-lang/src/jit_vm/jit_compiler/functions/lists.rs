use crate::{List, Object, OperandType, jit_vm::ObjectPool};

/// # Safety
/// This function dereferences a raw pointer to an `ObjectPool`.
pub unsafe extern "C" fn allocate_list(
    list_type: i8,
    length: i64,
    object_pool: *mut ObjectPool,
) -> i64 {
    let object_pool = unsafe { &mut *object_pool };
    let list = match OperandType(list_type as u8) {
        OperandType::LIST_BOOLEAN => List::boolean(Vec::with_capacity(length as usize)),
        OperandType::LIST_BYTE => List::byte(Vec::with_capacity(length as usize)),
        OperandType::LIST_CHARACTER => List::character(Vec::with_capacity(length as usize)),
        OperandType::LIST_FLOAT => List::float(Vec::with_capacity(length as usize)),
        OperandType::LIST_INTEGER => List::integer(Vec::with_capacity(length as usize)),
        OperandType::LIST_STRING => List::string(Vec::with_capacity(length as usize)),
        OperandType::LIST_LIST => List::list(Vec::with_capacity(length as usize)),
        _ => panic!(
            "Unsupported type for list allocation: {}",
            OperandType(list_type as u8)
        ),
    };
    let list_object = Object::list(list);
    let object_pointer = object_pool.allocate(list_object);

    object_pointer as i64
}

pub unsafe extern "C" fn insert_into_list(list_pointer: i64, index: i64, item: i64) {
    let object = unsafe { &mut *(list_pointer as *mut Object) };
    let index = index as usize;

    if let Some(list) = object.as_mut_list() {
        match list {
            List::Boolean(boolean_list) => {
                let boolean = item != 0;

                if index == boolean_list.len() {
                    boolean_list.push(boolean);
                } else if index < boolean_list.len() {
                    boolean_list[index] = boolean;
                } else {
                    panic!("Index out of bounds for list insertion");
                }
            }
            List::Byte(byte_list) => {
                let byte = item as u8;

                if index == byte_list.len() {
                    byte_list.push(byte);
                } else if index < byte_list.len() {
                    byte_list[index] = byte;
                } else {
                    panic!("Index out of bounds for list insertion");
                }
            }
            List::Character(character_list) => {
                let character = char::from_u32(item as u32).unwrap_or_default();

                if index == character_list.len() {
                    character_list.push(character);
                } else if index < character_list.len() {
                    character_list[index] = character;
                } else {
                    panic!("Index out of bounds for list insertion");
                }
            }
            List::Float(float_list) => {
                let float = f64::from_bits(item as u64);

                if index == float_list.len() {
                    float_list.push(float);
                } else if index < float_list.len() {
                    float_list[index] = float;
                } else {
                    panic!("Index out of bounds for list insertion");
                }
            }
            List::Integer(integer_list) => {
                if index == integer_list.len() {
                    integer_list.push(item);
                } else if index < integer_list.len() {
                    integer_list[index] = item;
                } else {
                    panic!("Index out of bounds for list insertion");
                }
            }
            List::String(string_list) => todo!(),
            List::List(list_list) => todo!(),
            List::Function(function_list) => todo!(),
        }
    } else {
        panic!("Object is not a list");
    }
}
