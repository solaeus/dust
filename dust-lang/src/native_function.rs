use serde::{Deserialize, Serialize};

const PANIC: u8 = 0b0000_0000;
const TO_STRING: u8 = 0b0000_0001;
const WRITE_LINE: u8 = 0b0000_0010;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum NativeFunction {
    Panic = PANIC as isize,
    ToString = TO_STRING as isize,
    WriteLine = WRITE_LINE as isize,
}

impl From<u8> for NativeFunction {
    fn from(byte: u8) -> Self {
        match byte {
            PANIC => NativeFunction::Panic,
            TO_STRING => NativeFunction::ToString,
            WRITE_LINE => NativeFunction::WriteLine,
            _ => {
                if cfg!(test) {
                    panic!("Invalid operation byte: {}", byte)
                } else {
                    NativeFunction::Panic
                }
            }
        }
    }
}

impl From<NativeFunction> for u8 {
    fn from(native_function: NativeFunction) -> Self {
        match native_function {
            NativeFunction::Panic => PANIC,
            NativeFunction::ToString => TO_STRING,
            NativeFunction::WriteLine => WRITE_LINE,
        }
    }
}
