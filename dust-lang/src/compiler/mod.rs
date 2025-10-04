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
    chunk::Chunk,
    constant_table::ConstantTable,
    dust_crate::Program,
    dust_error::DustError,
    lexer::Lexer,
    parser::{ParseResult, Parser},
    resolver::{
        Declaration, DeclarationId, DeclarationKind, Resolver, Scope, ScopeId, ScopeKind, TypeId,
    },
    source::{Position, Source, SourceCode, SourceFile, SourceFileId, Span},
    syntax_tree::SyntaxTree,
    r#type::{FunctionType, Type},
};

pub fn compile_main(source_code: String) -> Result<Chunk, DustError> {
    let source = Source::new();
    let files = source.write_files();
    let file = SourceFile {
        name: "main".to_string(),
        source_code: SourceCode::String(source_code),
    };

    let file_id = source.add_file(file);
    let file = files.get(file_id.0 as usize).unwrap();

    let lexer = Lexer::new(file.source_code.as_ref());
    let parser = Parser::new(file_id, lexer);
    let ParseResult {
        syntax_tree,
        errors,
    } = parser.parse();

    drop(files);

    if !errors.is_empty() {
        return Err(DustError::parse(errors, source));
    }

    let mut context = CompileContext {
        source: source.clone(),
        resolver: Resolver::new(true),
        file_trees: vec![syntax_tree],
        constants: ConstantTable::new(),
        prototypes: IndexMap::default(),
    };
    let chunk_compiler = ChunkCompiler::new(
        DeclarationId::MAIN,
        file_id,
        FunctionType::new([], [], Type::None),
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
    pub fn new(source: Source) -> Self {
        Self {
            context: CompileContext {
                source,
                file_trees: Vec::new(),
                constants: ConstantTable::new(),
                resolver: Resolver::new(true),
                prototypes: IndexMap::default(),
            },
        }
    }

    pub fn resolver(&self) -> &Resolver {
        &self.context.resolver
    }

    pub fn compile(self) -> Result<(Program, Resolver), DustError> {
        let context = self.compile_inner()?;

        Ok((
            Program {
                source: context.source,
                constants: context.constants,
                prototypes: context.prototypes,
            },
            context.resolver,
        ))
    }

    pub fn compile_with_extras(self) -> Result<(Program, Vec<SyntaxTree>, Resolver), DustError> {
        let context = self.compile_inner()?;

        Ok((
            Program {
                source: context.source,
                constants: context.constants,
                prototypes: context.prototypes,
            },
            context.file_trees,
            context.resolver,
        ))
    }

    fn compile_inner(mut self) -> Result<CompileContext, DustError> {
        let span = span!(Level::INFO, "compile");
        let _enter = span.enter();

        let source = self.context.source.clone();
        let files = source.read_files();

        for (index, file) in files.iter().enumerate().skip(1) {
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

        let main_source_code = files.first().unwrap().source_code.as_ref();
        let lexer = Lexer::new(main_source_code);
        let parser = Parser::new(SourceFileId(0), lexer);
        let ParseResult {
            syntax_tree,
            errors: module_errors,
        } = parser.parse();

        self.context.file_trees.push(syntax_tree);

        if !module_errors.is_empty() {
            return Err(DustError::parse(module_errors, self.context.source));
        }

        self.context
            .prototypes
            .insert(DeclarationId::MAIN, Chunk::default()); // Insert a placeholder

        let chunk_compiler = ChunkCompiler::new(
            DeclarationId::MAIN,
            SourceFileId(0),
            FunctionType::new([], [], Type::None),
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
