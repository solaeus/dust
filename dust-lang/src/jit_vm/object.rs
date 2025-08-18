use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug)]
#[repr(C)]
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

    pub fn object_list<T: Into<Vec<*mut Object>>>(objects: T) -> Self {
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

    pub fn as_mut_string(&mut self) -> Option<&mut String> {
        if let ObjectValue::String(ref mut string) = self.value {
            Some(string)
        } else {
            None
        }
    }

    pub fn size(&self) -> usize {
        match &self.value {
            ObjectValue::Empty => 0,
            ObjectValue::BooleanList(booleans) => booleans.capacity() * size_of::<bool>(),
            ObjectValue::ByteList(bytes) => bytes.capacity() * size_of::<u8>(),
            ObjectValue::CharacterList(characters) => characters.capacity() * size_of::<char>(),
            ObjectValue::FloatList(floats) => floats.capacity() * size_of::<f64>(),
            ObjectValue::IntegerList(integers) => integers.capacity() * size_of::<i64>(),
            ObjectValue::FunctionList(functions) => functions.capacity() * size_of::<usize>(),
            ObjectValue::ObjectList(objects) => objects.capacity() * size_of::<*const Object>(),
            ObjectValue::String(string) => string.capacity() * size_of::<u8>(),
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ObjectValue {
    Empty,
    String(String),
    BooleanList(Vec<bool>),
    ByteList(Vec<u8>),
    CharacterList(Vec<char>),
    FloatList(Vec<f64>),
    IntegerList(Vec<i64>),
    FunctionList(Vec<usize>),
    ObjectList(Vec<*mut Object>),
}

impl Display for ObjectValue {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match &self {
            ObjectValue::Empty => write!(f, "(empty)"),
            ObjectValue::String(string) => write!(f, "{string}"),
            ObjectValue::BooleanList(booleans) => write!(f, "{booleans:?}"),
            ObjectValue::ByteList(bytes) => write!(f, "{bytes:?}"),
            ObjectValue::CharacterList(characters) => write!(f, "{characters:?}"),
            ObjectValue::FloatList(floats) => write!(f, "{floats:?}"),
            ObjectValue::IntegerList(integers) => write!(f, "{integers:?}"),
            ObjectValue::FunctionList(functions) => write!(f, "{functions:?}"),
            ObjectValue::ObjectList(objects) => {
                write!(f, "[")?;

                for (index, object_pointer) in objects.iter().enumerate() {
                    let object_string = unsafe { &**object_pointer }.to_string();

                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{object_string}")?;
                }

                write!(f, "]")
            }
        }
    }
}

pub enum StringRope {
    Leaf {
        bytes: Vec<u8>,
    },
    SmallLeaf {
        bytes: [u8; 32],
    },
    Concatenated {
        length: usize,
        left: *mut Object,
        right: *mut Object,
    },
    Slice {
        length: usize,
        base: *mut Object,
        start: usize,
        end: usize,
    },
}
