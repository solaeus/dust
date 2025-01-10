pub struct TypeCode(pub u8);

impl TypeCode {
    const INTEGER: u8 = 0;
    const FLOAT: u8 = 1;
    const STRING: u8 = 2;
    const BOOLEAN: u8 = 3;
    const CHARACTER: u8 = 4;
    const BYTE: u8 = 5;
}
