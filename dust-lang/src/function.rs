use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{Chunk, Type};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Function {
    chunk: Chunk,
}

impl Function {
    pub fn new(chunk: Chunk) -> Self {
        Self { chunk }
    }

    pub fn chunk(&self) -> &Chunk {
        &self.chunk
    }

    pub fn r#type(&self) -> Type {
        Type::Function(self.chunk.r#type().clone())
    }

    pub fn as_borrowed(&self) -> FunctionBorrowed {
        FunctionBorrowed::new(&self.chunk)
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.chunk.r#type())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct FunctionBorrowed<'a> {
    chunk: &'a Chunk,
}

impl<'a> FunctionBorrowed<'a> {
    pub fn new(chunk: &'a Chunk) -> Self {
        Self { chunk }
    }

    pub fn chunk(&self) -> &Chunk {
        self.chunk
    }

    pub fn r#type(&self) -> Type {
        Type::Function(self.chunk.r#type().clone())
    }

    pub fn to_owned(&self) -> Function {
        Function::new(self.chunk.clone())
    }
}

impl<'a> Display for FunctionBorrowed<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.chunk.r#type())
    }
}
