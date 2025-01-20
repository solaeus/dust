use std::fmt::{self, Debug, Display, Formatter};

#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub enum Pointer {
    ConstantBoolean(u16),
    ConstantByte(u16),
    ConstantCharacter(u16),
    ConstantFloat(u16),
    ConstantInteger(u16),
    ConstantString(u16),

    RegisterBoolean(u16),
    RegisterByte(u16),
    RegisterCharacter(u16),
    RegisterFloat(u16),
    RegisterInteger(u16),
    RegisterString(u16),

    ForeignConstantBoolean(u16, u16),
    ForeignConstantByte(u16, u16),
    ForeignConstantCharacter(u16, u16),
    ForeignConstantFloat(u16, u16),
    ForeignConstantInteger(u16, u16),
    ForeignConstantString(u16, u16),

    ForeignRegisterBoolean(u16, u16),
    ForeignRegisterByte(u16, u16),
    ForeignRegisterCharacter(u16, u16),
    ForeignRegisterFloat(u16, u16),
    ForeignRegisterInteger(u16, u16),
    ForeignRegisterString(u16, u16),
}

impl Debug for Pointer {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl Display for Pointer {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Pointer::ConstantBoolean(index) => write!(f, "pc_bool({index})"),
            Pointer::ConstantByte(index) => write!(f, "pc_byte({index})"),
            Pointer::ConstantCharacter(index) => write!(f, "pc_char({index})"),
            Pointer::ConstantFloat(index) => write!(f, "pc_float({index})"),
            Pointer::ConstantInteger(index) => write!(f, "pc_int({index})"),
            Pointer::ConstantString(index) => write!(f, "pc_str({index})"),
            Pointer::RegisterBoolean(index) => write!(f, "pr_bool({index})"),
            Pointer::RegisterByte(index) => write!(f, "pr_byte({index})"),
            Pointer::RegisterCharacter(index) => write!(f, "pr_char({index})"),
            Pointer::RegisterFloat(index) => write!(f, "przfloat({index})"),
            Pointer::RegisterInteger(index) => write!(f, "pr_int({index})"),
            Pointer::RegisterString(index) => write!(f, "pr_str({index})"),
            Pointer::ForeignConstantBoolean(index, _) => write!(f, "pfc_bool({index})"),
            Pointer::ForeignConstantByte(index, _) => write!(f, "pfc_byte({index})"),
            Pointer::ForeignConstantCharacter(index, _) => write!(f, "pfc_char({index})"),
            Pointer::ForeignConstantFloat(index, _) => write!(f, "pfc_float({index})"),
            Pointer::ForeignConstantInteger(index, _) => write!(f, "pfc_int({index})"),
            Pointer::ForeignConstantString(index, _) => write!(f, "pfc_str({index})"),
            Pointer::ForeignRegisterBoolean(index, _) => write!(f, "pfr_bool({index})"),
            Pointer::ForeignRegisterByte(index, _) => write!(f, "pfr_byte({index})"),
            Pointer::ForeignRegisterCharacter(index, _) => write!(f, "pfr_char({index})"),
            Pointer::ForeignRegisterFloat(index, _) => write!(f, "pfr_float({index})"),
            Pointer::ForeignRegisterInteger(index, _) => write!(f, "pfr_int({index})"),
            Pointer::ForeignRegisterString(index, _) => write!(f, "pfr_str({index})"),
        }
    }
}
