#[derive(Clone, Debug)]
pub struct Object {
    pub value: ObjectValue,
    pub mark: bool,
}

impl Object {
    pub fn empty() -> Self {
        Object {
            value: ObjectValue::Empty,
            mark: false,
        }
    }

    pub fn string(string: String) -> Self {
        Object {
            value: ObjectValue::String(string),
            mark: false,
        }
    }

    pub fn boolean_list<T: Into<Vec<bool>>>(booleans: T) -> Self {
        Object {
            value: ObjectValue::BooleanList(booleans.into()),
            mark: false,
        }
    }

    pub fn byte_list<T: Into<Vec<u8>>>(bytes: T) -> Self {
        Object {
            value: ObjectValue::ByteList(bytes.into()),
            mark: false,
        }
    }

    pub fn character_list<T: Into<Vec<char>>>(characters: T) -> Self {
        Object {
            value: ObjectValue::CharacterList(characters.into()),
            mark: false,
        }
    }

    pub fn float_list<T: Into<Vec<f64>>>(floats: T) -> Self {
        Object {
            value: ObjectValue::FloatList(floats.into()),
            mark: false,
        }
    }

    pub fn integer_list<T: Into<Vec<i64>>>(integers: T) -> Self {
        Object {
            value: ObjectValue::IntegerList(integers.into()),
            mark: false,
        }
    }

    pub fn function_list<T: Into<Vec<usize>>>(functions: T) -> Self {
        Object {
            value: ObjectValue::FunctionList(functions.into()),
            mark: false,
        }
    }

    pub fn object_list<T: Into<Vec<*const Object>>>(objects: T) -> Self {
        Object {
            value: ObjectValue::ObjectList(objects.into()),
            mark: false,
        }
    }

    pub fn as_string(&self) -> Option<&String> {
        if let ObjectValue::String(ref string) = self.value {
            Some(string)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub enum ObjectValue {
    Empty,
    BooleanList(Vec<bool>),
    ByteList(Vec<u8>),
    CharacterList(Vec<char>),
    FloatList(Vec<f64>),
    IntegerList(Vec<i64>),
    FunctionList(Vec<usize>),
    ObjectList(Vec<*const Object>),
    String(String),
}
