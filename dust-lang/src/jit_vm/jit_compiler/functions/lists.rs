use crate::{List, Object, OperandType, jit_vm::ObjectPool};

/// # Safety
/// This function dereferences a raw pointer to an `ObjectPool`.
pub unsafe extern "C" fn allocate_list(
    list_type: i8,
    length: i64,
    object_pool: *mut ObjectPool,
) -> i64 {
    match OperandType(list_type as u8) {
        OperandType::LIST_INTEGER => {
            let object_pool = unsafe { &mut *object_pool };
            let integer_list = List::integer(Vec::with_capacity(length as usize));
            let list_object = Object::list(integer_list);
            let object_pointer = object_pool.allocate(list_object);

            object_pointer as i64
        }
        _ => panic!("Unsupported type for list allocation"),
    }
}

pub unsafe extern "C" fn insert_into_list(list_pointer: i64, index: i64, item: i64) {
    let object = unsafe { &mut *(list_pointer as *mut Object) };
    let index = index as usize;

    if let Some(list) = object.as_mut_list() {
        if let List::Integer(integer_list) = list {
            if index == integer_list.len() {
                integer_list.push(item);
            } else if index < integer_list.len() {
                integer_list.insert(index, item);
            } else {
                panic!("Index out of bounds for list insertion");
            }
        } else {
            panic!("Unsupported list type for insertion");
        }
    } else {
        panic!("Object is not a list");
    }
}
