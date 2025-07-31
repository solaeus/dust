use crate::List;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
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

    pub fn list(list: List) -> Self {
        Object {
            value: ObjectValue::List(list),
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

    pub fn as_list(&self) -> Option<&List> {
        if let ObjectValue::List(ref list) = self.value {
            Some(list)
        } else {
            None
        }
    }

    pub fn size(&self) -> usize {
        size_of::<Object>() + self.value.heap_size()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum ObjectValue {
    Empty,
    List(List),
    String(String),
}

impl ObjectValue {
    fn heap_size(&self) -> usize {
        match self {
            ObjectValue::Empty => 0,
            ObjectValue::List(list) => size_of::<List>() + list.heap_size(),
            ObjectValue::String(string) => size_of::<String>() + string.capacity(),
        }
    }
}
