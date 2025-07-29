use std::sync::Arc;

use crate::{Chunk, DustString, List};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Object {
    pub mark: bool,
    pub data: ObjectData,
}

impl Object {
    pub fn empty() -> Self {
        Self {
            data: ObjectData::Empty,
            mark: false,
        }
    }

    pub fn function(chunk: Arc<Chunk>) -> Self {
        Self {
            data: ObjectData::Function(chunk),
            mark: false,
        }
    }

    pub fn list(list: List) -> Self {
        Self {
            data: ObjectData::ValueList(list),
            mark: false,
        }
    }

    pub fn string(string: DustString) -> Self {
        Self {
            data: ObjectData::String(string),
            mark: false,
        }
    }

    pub fn as_function(&self) -> Option<&Arc<Chunk>> {
        if let ObjectData::Function(chunk) = &self.data {
            Some(chunk)
        } else {
            None
        }
    }

    pub fn as_list(&self) -> Option<&List> {
        if let ObjectData::ValueList(list) = &self.data {
            Some(list)
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<&DustString> {
        if let ObjectData::String(string) = &self.data {
            Some(string)
        } else {
            None
        }
    }

    pub fn into_function(self) -> Option<Arc<Chunk>> {
        if let ObjectData::Function(chunk) = self.data {
            Some(chunk)
        } else {
            None
        }
    }

    pub fn into_list(self) -> Option<List> {
        if let ObjectData::ValueList(list) = self.data {
            Some(list)
        } else {
            None
        }
    }

    pub fn into_string(self) -> Option<DustString> {
        if let ObjectData::String(string) = self.data {
            Some(string)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum ObjectData {
    Empty,
    Function(Arc<Chunk>),
    ValueList(List),
    String(DustString),
}
