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
    let parser = Parser::new(0, &mut resolver);
    let ParseResult {
        syntax_tree,
        errors,
    } = parser.parse_once(source_code, ScopeId::MAIN);
    let syntax_tree = Arc::new(syntax_tree);

    if !errors.is_empty() {
        let name = "dust_program".to_string();
        let source = source_code.to_string();

        return Err(DustError::parse(
            errors,
            Source::Script(Arc::new(SourceFile { name, source })),
        ));
    }

    let syntax_trees = Arc::new(vec![(syntax_tree, Arc::new(resolver))]);
    let source_file = SourceFile {
        name: "dust_program".to_string(),
        source: source_code.to_string(),
    };
    let source = Source::Script(Arc::new(source_file));

    let chunk_compiler = ChunkCompiler::new(
        syntax_trees,
        source.clone(),
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
    file_trees: Vec<(Arc<SyntaxTree>, Arc<Resolver>)>,
}

impl Compiler {
    pub fn new(source: Source) -> Self {
        Self {
            source,
            file_trees: Vec::new(),
        }
    }

    pub fn compile(mut self) -> Result<Program, DustError> {
        let mut errors = Vec::new();

        let program_name = match &self.source {
            Source::Script(source_file) => {
                let SourceFile { name, source } = source_file.as_ref();
                let mut resolver = Resolver::new(true);
                let ParseResult {
                    syntax_tree,
                    errors: script_errors,
                } = {
                    let parser = Parser::new(0, &mut resolver);

                    parser.parse_once(source, ScopeId::MAIN)
                };

                self.file_trees
                    .push((Arc::new(syntax_tree), Arc::new(resolver)));
                errors.extend(script_errors);

                name.clone()
            }
            Source::Files(files) => {
                for (file_index, source_file) in files.iter().enumerate() {
                    let SourceFile { name, source } = source_file.as_ref();
                    let file_index = file_index as u32;
                    let scope = Scope {
                        kind: ScopeKind::Module,
                        parent: ScopeId::MAIN,
                        imports: SmallVec::new(),
                        exports: SmallVec::new(),
                        depth: 0,
                        index: 0,
                    };
                    let mut resolver = Resolver::new(true);
                    let scope_id = resolver.add_scope(scope);

                    resolver.add_declaration(
                        DeclarationKind::Module {
                            inner_scope_id: scope_id,
                        },
                        ScopeId::MAIN,
                        TypeId::NONE,
                        name,
                        Position::new(file_index, Span::default()),
                    );

                    let ParseResult {
                        syntax_tree,
                        errors: module_errors,
                    } = {
                        let parser = Parser::new(file_index, &mut resolver);

                        parser.parse_once(source, scope_id)
                    };

                    self.file_trees
                        .push((Arc::new(syntax_tree), Arc::new(resolver)));

                    errors.extend(module_errors);
                }

                let first_file = files.first().expect("No files provided");

                first_file.name.clone()
            }
        };

        if !errors.is_empty() {
            return Err(DustError::parse(errors, self.source));
        }

        let mut prototypes = IndexMap::default();

        prototypes.insert(DeclarationId::MAIN, Chunk::default());

        let prototypes = Rc::new(RefCell::new(prototypes));
        let chunk_compiler = ChunkCompiler::new(
            Arc::new(self.file_trees),
            self.source.clone(),
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
