mod binder;
mod chunk_compiler;
mod error;

#[cfg(test)]
mod tests;

pub use chunk_compiler::ChunkCompiler;
pub use error::CompileError;
use smallvec::SmallVec;
use tracing::{Level, span};

pub use std::{cell::RefCell, rc::Rc};

use crate::{
    chunk::Chunk,
    compiler::binder::Binder,
    constant_table::ConstantTable,
    dust_crate::Program,
    dust_error::DustError,
    lexer::Lexer,
    parser::{ParseResult, Parser},
    resolver::{
        Declaration, DeclarationKind, ModuleKind, Resolver, Scope, ScopeId, ScopeKind, TypeId,
    },
    source::{Position, Source, SourceCode, SourceFile, SourceFileId, Span},
    syntax_tree::SyntaxTree,
    r#type::{FunctionType, Type},
};

pub fn compile_main(source_code: String) -> Result<Chunk, DustError> {
    let source = Source::new();
    let mut files = source.write_files();
    let file = SourceFile {
        name: "main".to_string(),
        source_code: SourceCode::String(source_code),
    };

    files.push(file);

    let file = files.first().unwrap();
    let lexer = Lexer::new(file.source_code.as_ref());
    let parser = Parser::new(SourceFileId(0), lexer);
    let ParseResult {
        syntax_tree,
        errors,
    } = parser.parse_main();

    drop(files);

    if !errors.is_empty() {
        return Err(DustError::parse(errors, source));
    }

    let mut resolver = Resolver::new();
    let file_trees = vec![syntax_tree];
    let binder = Binder::new(
        SourceFileId(0),
        source.clone(),
        &mut resolver,
        &file_trees[0],
        ScopeId::MAIN,
    );

    binder
        .bind_main()
        .map_err(|compile_error| DustError::compile(compile_error, source.clone()))?;

    let mut context = CompileContext {
        source: source.clone(),
        resolver,
        file_trees,
        constants: ConstantTable::new(),
        prototypes: Vec::new(),
    };
    let chunk_compiler = ChunkCompiler::new(
        None,
        SourceFileId(0),
        FunctionType::new([], [], Type::None),
        &mut context,
        ScopeId::MAIN,
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
                resolver: Resolver::new(),
                prototypes: Vec::new(),
            },
        }
    }

    pub fn resolver(&self) -> &Resolver {
        &self.context.resolver
    }

    pub fn compile(self, name: Option<String>) -> Result<Program, DustError> {
        let context = self.compile_inner()?;

        Ok(Program {
            name: name.unwrap_or_else(|| "anonymous".to_string()),
            source: context.source,
            constants: context.constants,
            prototypes: context.prototypes,
        })
    }

    pub fn compile_with_extras(
        self,
        name: Option<String>,
    ) -> Result<(Program, Vec<SyntaxTree>, Resolver), DustError> {
        let context = self.compile_inner()?;

        Ok((
            Program {
                name: name.unwrap_or_else(|| "anonymous".to_string()),
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
        let mut errors = Vec::new();

        let main_file = files.first().unwrap();
        let lexer = Lexer::new(main_file.source_code.as_ref());
        let parser = Parser::new(SourceFileId(0), lexer);
        let ParseResult {
            syntax_tree,
            errors: main_errors,
        } = parser.parse_main();

        self.context.file_trees.push(syntax_tree);
        errors.extend(main_errors);

        for (index, file) in files.iter().enumerate().skip(1) {
            let file_scope = Scope {
                kind: ScopeKind::Module,
                parent: ScopeId::MAIN,
                imports: SmallVec::new(),
                modules: SmallVec::new(),
            };
            let file_scope_id = self.context.resolver.add_scope(file_scope);
            let file_module_name = file.name.trim_end_matches(".ds");
            let module_id = self.context.resolver.add_declaration(
                file_module_name,
                Declaration {
                    kind: DeclarationKind::Module {
                        kind: ModuleKind::File,
                        inner_scope_id: file_scope_id,
                    },
                    scope_id: ScopeId::MAIN,
                    type_id: TypeId::NONE,
                    position: Position::new(SourceFileId(index as u32), Span::default()),
                    is_public: true,
                },
            );

            let file_id = SourceFileId(index as u32);
            let lexer = Lexer::new(file.source_code.as_ref());
            let parser = Parser::new(file_id, lexer);
            let ParseResult {
                syntax_tree,
                errors: file_errors,
            } = parser.parse_file_module();

            errors.extend(file_errors);

            if !errors.is_empty() {
                continue;
            }

            let binder = Binder::new(
                file_id,
                self.context.source.clone(),
                &mut self.context.resolver,
                &syntax_tree,
                file_scope_id,
            );

            binder
                .bind_file_module(file_module_name)
                .map_err(|compile_error| {
                    DustError::compile(compile_error, self.context.source.clone())
                })?;
            self.context.file_trees.push(syntax_tree);
            self.context
                .resolver
                .get_scope_mut(ScopeId::MAIN)
                .unwrap()
                .modules
                .push(module_id);
        }

        let main_binder = Binder::new(
            SourceFileId(0),
            self.context.source.clone(),
            &mut self.context.resolver,
            &self.context.file_trees[0],
            ScopeId::MAIN,
        );

        main_binder.bind_main().map_err(|compile_error| {
            DustError::compile(compile_error, self.context.source.clone())
        })?;

        if !errors.is_empty() {
            return Err(DustError::parse(errors, self.context.source));
        }

        self.context.prototypes.push(Chunk::default()); // Insert a placeholder

        let chunk_compiler = ChunkCompiler::new(
            None,
            SourceFileId(0),
            FunctionType::new([], [], Type::None),
            &mut self.context,
            ScopeId::MAIN,
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
    pub prototypes: Vec<Chunk>,
}
