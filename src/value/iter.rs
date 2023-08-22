use crate::Value;

pub struct Iter(Value);

impl IntoIterator for Value {
    type Item = Value;

    type IntoIter = Iter;

    fn into_iter(self) -> Self::IntoIter {
        Iter(self)
    }
}

impl Iterator for Iter {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
