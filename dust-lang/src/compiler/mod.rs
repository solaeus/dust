mod binder;
mod code_generator;
mod context;
mod error;

#[cfg(test)]
mod tests;

pub use context::{
    CompileContext, Declaration, DeclarationKind, FunctionTypeNode, ModuleKind, Scope, ScopeId,
    ScopeKind, TypeId, TypeNode,
};

pub use code_generator::CodeGenerator;
pub use error::CompileError;
use smallvec::SmallVec;
use tracing::{Level, span};

use crate::{
    compiler::binder::Binder,
    dust_crate::Program,
    dust_error::DustError,
    lexer::Lexer,
    parser::{ParseResult, Parser},
    prototype::Prototype,
    source::{Position, Source, SourceCode, SourceFile, SourceFileId, Span},
    syntax::Syntax,
};

pub const DEFAULT_PROGRAM_NAME: &str = "Dust Program";

pub fn compile_main(source_code: String) -> Result<Prototype, DustError> {
    let mut source = Source::new();
    source.add_file(SourceFile {
        name: "main".to_string(),
        source_code: SourceCode::String(source_code),
    });

    let compiler = Compiler::new(source);
    let mut program = compiler.compile(None)?;

    Ok(program.prototypes.remove(0))
}

pub fn compile(source_code: String) -> Result<Vec<Prototype>, DustError> {
    let mut source = Source::new();
    source.add_file(SourceFile {
        name: "main".to_string(),
        source_code: SourceCode::String(source_code),
    });

    let compiler = Compiler::new(source);
    let program = compiler.compile(None)?;

    Ok(program.prototypes)
}

pub struct Compiler {
    context: CompileContext,
    source: Source,
    syntax: Syntax,
}

impl Compiler {
    pub fn new(source: Source) -> Self {
        Self {
            syntax: Syntax::with_capacity(source.file_count()),
            source,
            context: CompileContext::new(),
        }
    }

    pub fn context(&self) -> &CompileContext {
        &self.context
    }

    pub fn compile(self, name: Option<String>) -> Result<Program, DustError> {
        let (context, _, _) = self.compile_inner()?;

        Ok(Program {
            name: name.unwrap_or_else(|| DEFAULT_PROGRAM_NAME.to_string()),
            constants: context.constants,
            prototypes: context.prototypes,
        })
    }

    pub fn compile_with_context(
        self,
        name: Option<String>,
    ) -> Result<(Program, Source, Syntax), DustError> {
        let (context, source, syntax) = self.compile_inner()?;

        Ok((
            Program {
                name: name.unwrap_or_else(|| DEFAULT_PROGRAM_NAME.to_string()),
                constants: context.constants,
                prototypes: context.prototypes,
            },
            source,
            syntax,
        ))
    }

    fn compile_inner(mut self) -> Result<(CompileContext, Source, Syntax), DustError> {
        let span = span!(Level::INFO, "compile");
        let _enter = span.enter();

        let source = self.source.clone();
        let files = source.read_files();
        let mut errors = Vec::new();

        let main_file = files.first().unwrap();
        let lexer = Lexer::new(main_file.source_code.as_ref());
        let parser = Parser::new(SourceFileId(0), lexer);
        let ParseResult {
            syntax_tree,
            errors: main_errors,
        } = parser.parse_main();

        self.syntax.add_tree(syntax_tree);
        errors.extend(main_errors);

        for (index, file) in files.iter().enumerate().skip(1) {
            let file_scope = Scope {
                kind: ScopeKind::Module,
                parent: ScopeId::MAIN,
                imports: SmallVec::new(),
                modules: SmallVec::new(),
            };
            let file_scope_id = self.context.add_scope(file_scope);
            let file_module_name = file.name.trim_end_matches(".ds");
            let module_id = self.context.add_declaration(
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
                self.source.clone(),
                &mut self.context,
                &syntax_tree,
                file_scope_id,
            );

            binder
                .bind_file_module(file_module_name)
                .map_err(|compile_error| DustError::compile(compile_error, self.source.clone()))?;
            self.syntax.add_tree(syntax_tree);
            self.context
                .get_scope_mut(ScopeId::MAIN)
                .unwrap()
                .modules
                .push(module_id);
        }

        let main_syntax = self.syntax.get_tree(SourceFileId::MAIN).unwrap();
        let main_binder = Binder::new(
            SourceFileId(0),
            self.source.clone(),
            &mut self.context,
            main_syntax,
            ScopeId::MAIN,
        );

        main_binder
            .bind_main()
            .map_err(|compile_error| DustError::compile(compile_error, self.source.clone()))?;

        if !errors.is_empty() {
            return Err(DustError::parse(errors, self.source));
        }

        self.context.prototypes.push(Prototype::default()); // Insert a placeholder

        let prototype_compiler = CodeGenerator::new(
            None,
            0,
            SourceFileId::MAIN,
            FunctionTypeNode {
                type_parameters: (0, 0),
                value_parameters: (0, 0),
                return_type: TypeId::NONE,
            },
            self.source.clone(),
            &self.syntax,
            &mut self.context,
            ScopeId::MAIN,
        );

        let prototype = match prototype_compiler.compile_main() {
            Ok(prototype) => prototype,
            Err(error) => {
                return Err(DustError::compile(error, self.source));
            }
        };

        self.context.prototypes[0] = prototype;

        Ok((self.context, self.source, self.syntax))
    }
}
