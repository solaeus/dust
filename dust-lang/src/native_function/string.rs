use crate::{DustString, Instruction, instruction::CallNative, risky_vm::Thread, r#type::TypeKind};

pub fn to_string(instruction: Instruction, thread: &mut Thread) {
    let CallNative {
        destination,
        function: _,
        argument_list_index,
    } = CallNative::from(instruction);

    let address = thread
        .current_call
        .chunk
        .arguments
        .get(argument_list_index as usize)
        .unwrap()
        .values
        .first()
        .unwrap();

    let string = match address.r#type() {
        TypeKind::Integer => {
            let integer = thread
                .current_memory
                .integers
                .get(address.index as usize)
                .unwrap()
                .as_value();

            DustString::from(integer.to_string())
        }
        _ => unreachable!(),
    };

    *thread
        .current_memory
        .strings
        .get_mut(destination.index as usize)
        .unwrap()
        .as_value_mut() = DustString::from(string);
}
