use crate::List;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Object {
    pub value: ObjectValue,
    pub mark: bool,
}

impl Object {
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
