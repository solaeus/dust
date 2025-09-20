mod chunk_compiler;
mod error;

#[cfg(test)]
mod tests;

pub use chunk_compiler::ChunkCompiler;
pub use error::CompileError;
use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;
use smallvec::SmallVec;
use tracing::{Level, span};

pub use std::{cell::RefCell, rc::Rc};

use crate::{
    Chunk, ConstantTable, Lexer, Position, Resolver, Source, Span,
    dust_crate::Program,
    dust_error::DustError,
    parser::{ParseResult, Parser},
    resolver::{
        Declaration, DeclarationId, DeclarationKind, FunctionTypeNode, Scope, ScopeId, ScopeKind,
        TypeId,
    },
    source::{SourceFile, SourceFileId},
    syntax_tree::SyntaxTree,
};

pub fn compile_main(source_code: Vec<u8>) -> Result<Chunk, DustError> {
    let lexer = Lexer::new(&source_code);
    let parser = Parser::new(SourceFileId::MAIN, lexer);
    let ParseResult {
        syntax_trees,
        errors,
    } = parser.parse();
    let source = Source::Script(SourceFile {
        name: "dust_program".into(),
        source_code,
    });

    if !errors.is_empty() {
        return Err(DustError::parse(errors, source));
    }

    let mut context = CompileContext {
        source: source.clone(),
        resolver: Resolver::new(true),
        file_trees: syntax_trees,
        constants: ConstantTable::new(),
        prototypes: IndexMap::default(),
    };
    let chunk_compiler = ChunkCompiler::new(
        DeclarationId::MAIN,
        FunctionTypeNode::default(),
        SourceFileId::MAIN,
        &mut context,
    );
    let compile_result = chunk_compiler.compile_main();

    match compile_result {
        Ok(chunk) => Ok(chunk),
        Err(error) => Err(DustError::compile(error, source)),
    }
}

pub struct Compiler {
    context: CompileContext,
}

impl Compiler {
    pub fn new(source: Source, resolver: Resolver) -> Self {
        Self {
            context: CompileContext {
                source,
                file_trees: Vec::new(),
                constants: ConstantTable::new(),
                resolver,
                prototypes: IndexMap::default(),
            },
        }
    }

    pub fn compile(self) -> Result<Program, DustError> {
        let context = self.compile_inner()?;

        Ok(Program {
            name: context.source.into_program_name(),
            constants: context.constants,
            prototypes: context.prototypes,
        })
    }

    pub fn compile_with_extras(
        self,
    ) -> Result<(Program, Source, Vec<SyntaxTree>, Resolver), DustError> {
        let context = self.compile_inner()?;

        Ok((
            Program {
                name: context.source.program_name().to_string(),
                constants: context.constants,
                prototypes: context.prototypes,
            },
            context.source,
            context.file_trees,
            context.resolver,
        ))
    }

    fn compile_inner(mut self) -> Result<CompileContext, DustError> {
        let span = span!(Level::INFO, "compile");
        let _enter = span.enter();

        for (index, file) in self.context.source.get_files().iter().enumerate().skip(1) {
            let file_scope = Scope {
                kind: ScopeKind::Module,
                parent: ScopeId::MAIN,
                imports: SmallVec::new(),
                modules: SmallVec::new(),
            };
            let file_scope_id = self.context.resolver.add_scope(file_scope);
            let module_id = self.context.resolver.add_declaration(
                &file.name,
                Declaration {
                    kind: DeclarationKind::FileModule {
                        inner_scope_id: file_scope_id,
                        is_parsed: false,
                    },
                    scope_id: ScopeId::MAIN,
                    type_id: TypeId::NONE,
                    position: Position::new(SourceFileId(index as u32), Span::default()),
                    is_public: true,
                },
            );

            self.context
                .resolver
                .add_module_to_scope(ScopeId::MAIN, module_id);
        }

        let main_source_code = &self
            .context
            .source
            .get_file(SourceFileId::MAIN)
            .unwrap()
            .source_code;
        let lexer = Lexer::new(main_source_code);
        let parser = Parser::new(SourceFileId::MAIN, lexer);
        let ParseResult {
            syntax_trees,
            errors: module_errors,
        } = parser.parse();

        self.context.file_trees.extend(syntax_trees);

        if !module_errors.is_empty() {
            return Err(DustError::parse(module_errors, self.context.source));
        }

        self.context
            .prototypes
            .insert(DeclarationId::MAIN, Chunk::default()); // Insert a placeholder

        let chunk_compiler = ChunkCompiler::new(
            DeclarationId::MAIN,
            FunctionTypeNode::default(),
            SourceFileId::MAIN,
            &mut self.context,
        );

        let chunk = match chunk_compiler.compile_main() {
            Ok(chunk) => chunk,
            Err(error) => {
                return Err(DustError::compile(error, self.context.source));
            }
        };

        self.context.prototypes[0] = chunk;

        Ok(self.context)
    }
}

#[derive(Debug)]
pub struct CompileContext {
    pub source: Source,
    pub file_trees: Vec<SyntaxTree>,
    pub constants: ConstantTable,
    pub resolver: Resolver,
    pub prototypes: IndexMap<DeclarationId, Chunk, FxBuildHasher>,
}
