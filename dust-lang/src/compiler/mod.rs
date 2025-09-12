mod chunk_compiler;
mod error;

#[cfg(test)]
mod tests;

pub use chunk_compiler::ChunkCompiler;
pub use error::CompileError;

use std::sync::Arc;
pub use std::{cell::RefCell, rc::Rc};

use crate::{
    Chunk, Position, Resolver, Source, Span,
    dust_crate::Program,
    dust_error::DustError,
    parser::{ParseResult, Parser},
    resolver::{DeclarationKind, Scope, ScopeId, ScopeKind, TypeId},
    source::SourceFile,
    syntax_tree::SyntaxTree,
};

pub fn compile_main(source: &str) -> Result<Chunk, DustError> {
    let mut resolver = Resolver::new(true);
    let parser = Parser::new(0, &mut resolver);
    let ParseResult {
        syntax_tree,
        errors,
    } = parser.parse_once(source, ScopeId::MAIN);

    if !errors.is_empty() {
        let name = Arc::new("dust_program".to_string());
        let source = Arc::new(source.to_string());

        return Err(DustError::parse(errors, Source::Script { name, content: source }));
    }

    let chunk_compiler = ChunkCompiler::new(
        Arc::new(syntax_tree),
        source,
        &resolver,
        Rc::new(RefCell::new(Vec::new())),
    );

    chunk_compiler.compile().map_err(|error| {
        let name = Arc::new("dust_program".to_string());
        let source = Arc::new(source.to_string());

        DustError::compile(error, Source::Script { name, content: source })
    })
}

pub struct Compiler {
    source: Source,
    file_trees: Vec<Arc<SyntaxTree>>,
    resolver: Resolver,
}

impl Compiler {
    pub fn new(source: Source) -> Self {
        Self {
            source,
            file_trees: Vec::new(),
            resolver: Resolver::new(true),
        }
    }

    pub fn compile(mut self) -> Result<Program, DustError> {
        let mut errors = Vec::new();

        let (program_name, main_source) = match &self.source {
            Source::Script { name, content: source } => {
                let ParseResult {
                    syntax_tree,
                    errors: script_errors,
                } = {
                    let parser = Parser::new(0, &mut self.resolver);

                    parser.parse_once(source, ScopeId::MAIN)
                };

                self.file_trees.push(Arc::new(syntax_tree));
                errors.extend(script_errors);

                (name.clone(), source.clone())
            }
            Source::Files(files) => {
                for (file_index, SourceFile { name, source }) in files.iter().enumerate() {
                    let file_index = file_index as u32;
                    let scope = Scope {
                        kind: ScopeKind::Module,
                        parent: ScopeId::MAIN,
                        imports: (0, 0),
                        depth: 0,
                        index: 0,
                    };
                    let scope_id = self.resolver.add_scope(scope);

                    self.resolver.add_declaration(
                        DeclarationKind::Module,
                        scope_id,
                        TypeId::NONE,
                        name,
                        Position::new(file_index, Span::default()),
                    );

                    let ParseResult {
                        syntax_tree,
                        errors: module_errors,
                    } = {
                        let parser = Parser::new(file_index, &mut self.resolver);

                        parser.parse_once(source, scope_id)
                    };

                    self.file_trees.push(Arc::new(syntax_tree));

                    errors.extend(module_errors);
                }

                let first_file = &files[0];

                (first_file.name.clone(), first_file.source.clone())
            }
        };

        let main_syntax_tree = self.file_trees[0].clone();
        let prototypes = Rc::new(RefCell::new(vec![Chunk::default()]));

        let chunk_compiler = ChunkCompiler::new(
            main_syntax_tree,
            &main_source,
            &self.resolver,
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
            .into_inner();

        Ok(Program {
            name: program_name.to_string(),
            prototypes,
            cell_count: 0,
        })
    }
}
