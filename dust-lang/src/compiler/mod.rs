mod chunk_compiler;
mod error;

#[cfg(test)]
mod tests;

pub use chunk_compiler::ChunkCompiler;
pub use error::CompileError;

pub use std::{cell::RefCell, rc::Rc};

use crate::{
    Chunk, OperandType, Resolver,
    dust_crate::Program,
    dust_error::DustError,
    parser::{ParseResult, Parser},
    resolver::ScopeId,
};

pub fn compile_main(source: &'_ str) -> Result<Chunk, DustError<'_>> {
    let mut resolver = Resolver::new();
    let parser = Parser::new(&mut resolver);
    let ParseResult {
        syntax_tree,
        errors,
    } = parser.parse_once(source, ScopeId::MAIN);

    if !errors.is_empty() {
        return Err(DustError::parse(errors, source));
    }

    let chunk_compiler = ChunkCompiler::new(syntax_tree, &resolver);

    chunk_compiler
        .compile()
        .map_err(|error| DustError::compile(error, source))
}

pub struct Compiler {
    resolver: Resolver,
    _allow_native_functions: bool,
}

impl Compiler {
    pub fn new(allow_native_functions: bool) -> Self {
        Self {
            resolver: Resolver::new(),
            _allow_native_functions: allow_native_functions,
        }
    }

    pub fn compile<'src>(
        &mut self,
        sources: &[(&str, &'src str)],
    ) -> Result<Program, DustError<'src>> {
        let prototypes = Rc::new(RefCell::new(Vec::new()));

        let mut parser = Parser::new(&mut self.resolver);
        let ParseResult {
            syntax_tree,
            errors,
        } = parser.parse(sources[0].1, ScopeId::MAIN);

        if !errors.is_empty() {
            return Err(DustError::parse(errors, sources[0].1));
        }

        prototypes.borrow_mut().push(Chunk::default());

        let chunk_compiler = ChunkCompiler::new(syntax_tree, &self.resolver);
        let main_chunk = chunk_compiler
            .compile()
            .map_err(|error| DustError::compile(error, sources[0].1))?;

        prototypes.borrow_mut()[0] = main_chunk;

        let prototypes = Rc::into_inner(prototypes)
            .expect("Unneccessary borrow of 'prototypes'")
            .into_inner();

        Ok(Program {
            prototypes,
            cell_count: 0,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Constant {
    Boolean(bool),
    Byte(u8),
    Character(char),
    Float(f64),
    Integer(i64),
    String { pool_start: u32, pool_end: u32 },
}

impl Constant {
    fn operand_type(&self) -> OperandType {
        match self {
            Constant::Boolean(_) => OperandType::BOOLEAN,
            Constant::Byte(_) => OperandType::BYTE,
            Constant::Character(_) => OperandType::CHARACTER,
            Constant::Float(_) => OperandType::FLOAT,
            Constant::Integer(_) => OperandType::INTEGER,
            Constant::String { .. } => OperandType::STRING,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
struct Local {
    register: u16,
    is_mutable: bool,
}
