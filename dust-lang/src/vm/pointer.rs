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
    RegisterList(u16),

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
    ForeignRegisterList(u16, u16),
}

impl Debug for Pointer {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl Display for Pointer {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Pointer::ConstantBoolean(index) => write!(f, "P_C_BOOL({index})"),
            Pointer::ConstantByte(index) => write!(f, "P_C_BYTE({index})"),
            Pointer::ConstantCharacter(index) => write!(f, "P_C_CHAR({index})"),
            Pointer::ConstantFloat(index) => write!(f, "P_C_FLOAT({index})"),
            Pointer::ConstantInteger(index) => write!(f, "P_C_INT({index})"),
            Pointer::ConstantString(index) => write!(f, "P_C_STR({index})"),
            Pointer::RegisterBoolean(index) => write!(f, "P_R_BOOL({index})"),
            Pointer::RegisterByte(index) => write!(f, "P_R_BYTE({index})"),
            Pointer::RegisterCharacter(index) => write!(f, "P_R_CHAR({index})"),
            Pointer::RegisterFloat(index) => write!(f, "P_R_FLOAT({index})"),
            Pointer::RegisterInteger(index) => write!(f, "P_R_INT({index})"),
            Pointer::RegisterString(index) => write!(f, "P_R_STR({index})"),
            Pointer::RegisterList(index) => write!(f, "P_R_LIST({index})"),
            Pointer::ForeignConstantBoolean(index, _) => write!(f, "P_FC_BOOL({index})"),
            Pointer::ForeignConstantByte(index, _) => write!(f, "P_FC_BYTE({index})"),
            Pointer::ForeignConstantCharacter(index, _) => write!(f, "P_FC_CHAR({index})"),
            Pointer::ForeignConstantFloat(index, _) => write!(f, "P_FC_FLOAT({index})"),
            Pointer::ForeignConstantInteger(index, _) => write!(f, "P_FC_INT({index})"),
            Pointer::ForeignConstantString(index, _) => write!(f, "P_FC_STR({index})"),
            Pointer::ForeignRegisterBoolean(index, _) => write!(f, "P_FR_BOOL({index})"),
            Pointer::ForeignRegisterByte(index, _) => write!(f, "P_FR_BYTE({index})"),
            Pointer::ForeignRegisterCharacter(index, _) => write!(f, "P_FR_CHAR({index})"),
            Pointer::ForeignRegisterFloat(index, _) => write!(f, "P_FR_FLOAT({index})"),
            Pointer::ForeignRegisterInteger(index, _) => write!(f, "P_FR_INT({index})"),
            Pointer::ForeignRegisterString(index, _) => write!(f, "P_FR_STR({index})"),
            Pointer::ForeignRegisterList(index, _) => write!(f, "P_FR_LIST({index})"),
        }
    }
}
