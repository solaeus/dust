mod chunk_compiler;
mod error;

#[cfg(test)]
mod tests;

pub use chunk_compiler::ChunkCompiler;
pub use error::CompileError;

pub use std::{cell::RefCell, rc::Rc};

use crate::{
    Chunk, Resolver, Span,
    dust_crate::Program,
    dust_error::DustError,
    parser::{ParseResult, Parser},
    resolver::{DeclarationKind, Scope, ScopeId, ScopeKind, TypeId},
    syntax_tree::SyntaxTree,
};

pub fn compile_main(source: &'_ str) -> Result<Chunk, DustError<'_>> {
    let mut resolver = Resolver::new(true);
    let parser = Parser::new(&mut resolver);
    let ParseResult {
        syntax_tree,
        errors,
    } = parser.parse_once(source, ScopeId::MAIN);

    if !errors.is_empty() {
        return Err(DustError::parse(errors, source));
    }

    let chunk_compiler = ChunkCompiler::new(
        syntax_tree,
        source,
        &resolver,
        Rc::new(RefCell::new(Vec::new())),
    );

    chunk_compiler
        .compile()
        .map_err(|error| DustError::compile(error, source))
}

pub struct Sources<'src> {
    pub main: &'src str,
    pub modules: Vec<(&'src str, &'src str)>,
}

pub struct Compiler<'src> {
    sources: Sources<'src>,
    module_trees: Vec<SyntaxTree>,
    resolver: Resolver,
}

impl<'src> Compiler<'src> {
    pub fn new(sources: Sources<'src>) -> Self {
        Self {
            sources,
            module_trees: Vec::new(),
            resolver: Resolver::new(true),
        }
    }

    pub fn compile(&mut self) -> Result<Program, DustError<'src>> {
        let ParseResult {
            syntax_tree,
            mut errors,
        } = {
            let parser = Parser::new(&mut self.resolver);

            parser.parse_once(self.sources.main, ScopeId::MAIN)
        };

        for (module_name, module_source) in &self.sources.modules {
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
                module_name,
                Span::default(),
            );

            let ParseResult {
                syntax_tree,
                errors: module_errors,
            } = {
                let parser = Parser::new(&mut self.resolver);

                parser.parse_once(module_source, scope_id)
            };

            self.module_trees.push(syntax_tree);
            errors.extend(module_errors);
        }

        if !errors.is_empty() {
            return Err(DustError::parse(errors, self.sources.main));
        }

        let prototypes = Rc::new(RefCell::new(vec![Chunk::default()]));

        let chunk_compiler = ChunkCompiler::new(
            syntax_tree,
            self.sources.main,
            &self.resolver,
            prototypes.clone(),
        );
        let main_chunk = chunk_compiler
            .compile()
            .map_err(|error| DustError::compile(error, self.sources.main))?;

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
