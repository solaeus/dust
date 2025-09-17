mod chunk_compiler;
mod error;

#[cfg(test)]
mod tests;

pub use chunk_compiler::ChunkCompiler;
pub use error::CompileError;
use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;
use smallvec::SmallVec;

pub use std::{cell::RefCell, rc::Rc};
use std::{collections::HashMap, sync::Arc};

use crate::{
    Chunk, ConstantTable, Position, Resolver, Source, Span,
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
        name: Arc::new("main".to_string()),
        source_code: Arc::new(source_code.to_string()),
    });
    let parser = Parser::new(&source, &mut resolver);
    let ParseResult {
        syntax_trees,
        errors,
    } = parser.parse(0, DeclarationId::MAIN, ScopeId::MAIN);

    if !errors.is_empty() {
        return Err(DustError::parse(errors, source));
    }

    let source_file = source.get_file(0).unwrap().clone();
    let mut prototypes = IndexMap::default();
    let mut constants = ConstantTable::new();
    let chunk_compiler = ChunkCompiler::new(
        &syntax_trees,
        syntax_trees.get(&DeclarationId::MAIN).unwrap(),
        &resolver,
        &mut constants,
        &source,
        source_file,
        &mut prototypes,
    );
    let compile_result = chunk_compiler.compile();

    match compile_result {
        Ok(chunk) => Ok(chunk),
        Err(error) => Err(DustError::compile(error, source)),
    }
}

pub struct Compiler {
    source: Source,
    file_trees: HashMap<DeclarationId, SyntaxTree, FxBuildHasher>,
}

impl Compiler {
    pub fn new(source: Source) -> Self {
        Self {
            source,
            file_trees: HashMap::default(),
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

        let source_file = match self.source.get_file(0) {
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
            &source_file.name,
            Position::new(0, Span::default()),
        );

        let parser = Parser::new(&self.source, &mut resolver);
        let ParseResult {
            syntax_trees,
            errors: module_errors,
        } = parser.parse(0, DeclarationId::MAIN, ScopeId::MAIN);

        self.file_trees.extend(syntax_trees);

        if !module_errors.is_empty() {
            return Err(DustError::parse(module_errors, self.source));
        }

        let mut prototypes = IndexMap::default();

        prototypes.insert(DeclarationId::MAIN, Chunk::default());

        let mut constants = ConstantTable::new();
        let chunk_compiler = ChunkCompiler::new(
            &self.file_trees,
            self.file_trees.get(&DeclarationId::MAIN).unwrap(),
            &resolver,
            &mut constants,
            &self.source,
            source_file.clone(),
            &mut prototypes,
        );

        let chunk = match chunk_compiler.compile() {
            Ok(chunk) => chunk,
            Err(error) => {
                return Err(DustError::compile(error, self.source));
            }
        };

        prototypes.insert(DeclarationId::MAIN, chunk);

        let prototypes = prototypes
            .into_iter()
            .map(|(_, chunk)| chunk)
            .collect::<Vec<Chunk>>();

        Ok(Program {
            name: program_name,
            constants,
            prototypes,
        })
    }
}
