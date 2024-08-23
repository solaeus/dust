use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

use serde::{Deserialize, Serialize};

use crate::{Identifier, Struct, StructType, TypeConflict, Value};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Constructor {
    pub struct_type: StructType,
}

impl Constructor {
    pub fn construct_unit(&self) -> Result<Value, ConstructError> {
        if let StructType::Unit { name } = &self.struct_type {
            Ok(Value::r#struct(Struct::Unit { name: name.clone() }))
        } else {
            Err(ConstructError::ExpectedUnit)
        }
    }

    pub fn construct_tuple(&self, fields: Vec<Value>) -> Result<Value, ConstructError> {
        if let StructType::Tuple {
            name: expected_name,
            fields: expected_fields,
        } = &self.struct_type
        {
            if fields.len() != expected_fields.len() {
                return Err(ConstructError::FieldCountMismatch);
            }

            for (i, value) in fields.iter().enumerate() {
                let expected_type = expected_fields.get(i).unwrap();
                let actual_type = value.r#type();

                expected_type.check(&actual_type)?;
            }

            Ok(Value::r#struct(Struct::Tuple {
                name: expected_name.clone(),
                fields,
            }))
        } else {
            Err(ConstructError::ExpectedTuple)
        }
    }

    pub fn construct_fields(
        &self,
        fields: HashMap<Identifier, Value>,
    ) -> Result<Value, ConstructError> {
        if let StructType::Fields {
            name: expected_name,
            fields: expected_fields,
        } = &self.struct_type
        {
            if fields.len() != expected_fields.len() {
                return Err(ConstructError::FieldCountMismatch);
            }

            for (field_name, field_value) in fields.iter() {
                let expected_type = expected_fields.get(field_name).unwrap();
                let actual_type = field_value.r#type();

                expected_type.check(&actual_type)?;
            }

            Ok(Value::r#struct(Struct::Fields {
                name: expected_name.clone(),
                fields,
            }))
        } else {
            Err(ConstructError::ExpectedFields)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ConstructError {
    FieldCountMismatch,
    ExpectedUnit,
    ExpectedTuple,
    ExpectedFields,
    TypeConflict(TypeConflict),
}

impl From<TypeConflict> for ConstructError {
    fn from(conflict: TypeConflict) -> Self {
        Self::TypeConflict(conflict)
    }
}

impl Display for ConstructError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ConstructError::FieldCountMismatch => write!(f, "Field count mismatch"),
            ConstructError::ExpectedUnit => write!(f, "Expected unit struct"),
            ConstructError::ExpectedTuple => write!(f, "Expected tuple struct"),
            ConstructError::ExpectedFields => write!(f, "Expected fields struct"),
            ConstructError::TypeConflict(TypeConflict { expected, actual }) => {
                write!(f, "Type conflict: expected {}, got {}", expected, actual)
            }
        }
    }
}
