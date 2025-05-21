use std::io::{Write, stdout};

use crate::{
    Address, Arguments, Destination, DustString, Instruction,
    instruction::{AddressKind, CallNative},
    panic_vm::{CallFrame, Memory, RegisterTable, Thread},
    r#type::TypeKind,
};

pub fn read_line(
    instruction: Instruction,
    thread: &mut Thread,
    registers: RegisterTable,
) -> RegisterTable {
    todo!()

    // let CallNative { destination, .. } = CallNative::from(instruction);
    // let mut buffer = String::new();
    // let stdin = std::io::stdin();

    // stdin.read_line(&mut buffer).unwrap();

    // *thread
    //     .current_memory
    //     .strings
    //     .get_mut(destination.index as usize)
    //     .unwrap()
    //     .as_value_mut() = DustString::from(buffer.trim_end_matches('\n'));

    // registers
}

pub fn write_line(_: Destination, arguments: &Arguments, _: &mut CallFrame, memory: &mut Memory) {
    let mut stdout = stdout();

    for address in &arguments.values {
        match address.kind {
            AddressKind::STRING_REGISTER => {
                let string = &memory.registers.strings[address.index as usize];

                stdout.write_all(string.as_bytes()).unwrap();
            }
            AddressKind::STRING_MEMORY => {
                let string = memory.strings[address.index as usize].as_value();

                stdout.write_all(string.as_bytes()).unwrap();
            }
            _ => unreachable!(),
        }
    }

    let _ = stdout.write_all(b"\n");
}
