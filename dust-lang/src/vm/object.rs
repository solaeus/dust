use crate::List;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Object {
    pub value: ObjectValue,
    pub mark: bool,
}

impl Object {
    pub fn empty() -> Self {
        Self {
            value: ObjectValue::Empty,
            mark: false,
        }
    }

    pub fn function(index: usize) -> Self {
        Self {
            value: ObjectValue::Function(index),
            mark: false,
        }
    }

    pub fn list(list: List) -> Self {
        Self {
            value: ObjectValue::List(list),
            mark: false,
        }
    }

    pub fn string(string: String) -> Self {
        Self {
            value: ObjectValue::String(string),
            mark: false,
        }
    }

    pub fn as_function(&self) -> Option<&usize> {
        if let ObjectValue::Function(index) = &self.value {
            Some(index)
        } else {
            None
        }
    }

    pub fn as_list(&self) -> Option<&List> {
        if let ObjectValue::List(list) = &self.value {
            Some(list)
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<&String> {
        if let ObjectValue::String(string) = &self.value {
            Some(string)
        } else {
            None
        }
    }

    pub fn into_function(self) -> Option<usize> {
        if let ObjectValue::Function(index) = self.value {
            Some(index)
        } else {
            None
        }
    }

    pub fn into_list(self) -> Option<List> {
        if let ObjectValue::List(list) = self.value {
            Some(list)
        } else {
            None
        }
    }

    pub fn into_string(self) -> Option<String> {
        if let ObjectValue::String(string) = self.value {
            Some(string)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum ObjectValue {
    Empty,
    Function(usize),
    List(List),
    String(String),
}

impl ObjectValue {
    fn size(&self) -> usize {
        let heap_size = match self {
            ObjectValue::Empty => 0,
            ObjectValue::Function(_) => 0,
            ObjectValue::List(list) => list.heap_size(),
            ObjectValue::String(string) => string.capacity(),
        };

        heap_size + size_of::<ObjectValue>()
    }
}
