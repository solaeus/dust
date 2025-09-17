mod chunk_compiler;
mod error;

#[cfg(test)]
mod tests;

pub use chunk_compiler::ChunkCompiler;
pub use error::CompileError;
use indexmap::IndexMap;
use smallvec::SmallVec;

use std::sync::Arc;
pub use std::{cell::RefCell, rc::Rc};

use crate::{
    Chunk, Position, Resolver, Source, Span,
    dust_crate::Program,
    dust_error::DustError,
    parser::{ParseResult, Parser},
    resolver::{DeclarationId, DeclarationKind, Scope, ScopeId, ScopeKind, TypeId},
    source::SourceFile,
    syntax_tree::SyntaxTree,
};

pub fn compile_main(source_code: &str) -> Result<Chunk, DustError> {
    let mut resolver = Resolver::new(true);
    let source = Source::Script(SourceFile {
        name: Arc::new("dust_program".to_string()),
        source_code: Arc::new(source_code.to_string()),
    });
    let parser = Parser::new(&source, &mut resolver);
    let ParseResult {
        syntax_tree,
        errors,
    } = parser.parse(0, ScopeId::MAIN);

    let source_file = SourceFile {
        name: Arc::new("dust_program".to_string()),
        source_code: Arc::new(source_code.to_string()),
    };
    let source = Source::Script(source_file);

    if !errors.is_empty() {
        return Err(DustError::parse(errors, source));
    }

    let file_trees = vec![syntax_tree];
    let chunk_compiler = ChunkCompiler::new(
        &file_trees,
        Rc::new(resolver),
        &source,
        Rc::new(RefCell::new(IndexMap::default())),
    );
    let compile_result = chunk_compiler.compile();

    match compile_result {
        Ok(chunk) => Ok(chunk),
        Err(error) => Err(DustError::compile(error, source)),
    }
}

pub struct Compiler {
    source: Source,
    file_trees: Vec<SyntaxTree>,
}

impl Compiler {
    pub fn new(source: Source) -> Self {
        Self {
            source,
            file_trees: Vec::new(),
        }
    }

    pub fn source(&self) -> &Source {
        &self.source
    }

    pub fn compile(mut self, mut resolver: Resolver) -> Result<Program, DustError> {
        let program_name = self.source.program_name();

        for file_index in 1..self.source.len() {
            let SourceFile { name, .. } = self.source.get_file(file_index).unwrap();
            let file_scope = Scope {
                kind: ScopeKind::Module,
                parent: ScopeId::MAIN,
                imports: SmallVec::new(),
                modules: SmallVec::new(),
            };
            let file_scope_id = resolver.add_scope(file_scope);
            let module_id = resolver.add_declaration(
                DeclarationKind::FileModule {
                    inner_scope_id: file_scope_id,
                    is_parsed: false,
                },
                ScopeId::MAIN,
                TypeId::NONE,
                true,
                name,
                Position::new(file_index as u32, Span::default()),
            );

            resolver.add_module_to_scope(ScopeId::MAIN, module_id);
        }

        let SourceFile { name, .. } = match self.source.get_file(0) {
            Some(file) => file,
            None => {
                todo!("Error");
            }
        };

        resolver.add_declaration(
            DeclarationKind::FileModule {
                inner_scope_id: ScopeId::MAIN,
                is_parsed: true,
            },
            ScopeId::MAIN,
            TypeId::NONE,
            true,
            name,
            Position::new(0, Span::default()),
        );

        let parser = Parser::new(&self.source, &mut resolver);
        let ParseResult {
            syntax_tree,
            errors: module_errors,
        } = parser.parse(0, ScopeId::MAIN);

        self.file_trees.push(syntax_tree);

        if !module_errors.is_empty() {
            return Err(DustError::parse(module_errors, self.source));
        }

        let mut prototypes = IndexMap::default();

        prototypes.insert(DeclarationId::MAIN, Chunk::default());

        let prototypes = Rc::new(RefCell::new(prototypes));
        let chunk_compiler = ChunkCompiler::new(
            &self.file_trees,
            Rc::new(resolver),
            &self.source,
            prototypes.clone(),
        );

        prototypes.borrow_mut()[0] = match chunk_compiler.compile() {
            Ok(chunk) => chunk,
            Err(error) => {
                return Err(DustError::compile(error, self.source));
            }
        };

        let prototypes = Rc::into_inner(prototypes)
            .expect("Unneccessary borrow of 'prototypes'")
            .into_inner()
            .into_iter()
            .map(|(_, chunk)| chunk)
            .collect::<Vec<Chunk>>();

        Ok(Program {
            name: program_name,
            prototypes,
            cell_count: 0,
        })
    }
}
