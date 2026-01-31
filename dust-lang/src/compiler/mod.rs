mod context;
mod declaration_binder;
mod emitter;
mod error;
mod type_binder;
mod type_graph;

#[cfg(test)]
mod tests;

pub use context::{
    CompileContext, Declaration, DeclarationKind, ModuleKind, Scope, ScopeId, ScopeKind,
};
pub use emitter::Emitter;
pub use error::CompileError;
use smallvec::SmallVec;
pub use type_graph::{TypeGraph, TypeId, TypeNode};

use tracing::{Level, span};

use crate::{
    compiler::{
        context::DeclarationId, declaration_binder::DeclarationBinder, type_binder::TypeBinder,
    },
    dust_crate::Program,
    dust_error::DustError,
    lexer::Lexer,
    parser::{ParseResult, Parser},
    prototype::Prototype,
    source::{Source, SourceCode, SourceFile, SourceFileId},
    syntax::{Syntax, SyntaxId, SyntaxKind, SyntaxReader},
};

pub const DEFAULT_PROGRAM_NAME: &str = "Dust Program";

pub fn compile_main_prototype(source_code: String) -> Result<Prototype, DustError> {
    let mut source = Source::new();
    source.add_file(SourceFile {
        name: "main".to_string(),
        source_code: SourceCode::String(source_code),
    });

    let compiler = Compiler::new(source);
    let mut program = compiler.compile(None)?;

    Ok(program.prototypes.remove(0))
}

pub fn compile_prototypes(source_code: String) -> Result<Vec<Prototype>, DustError> {
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
        self.compile_with_extras(name)
            .map(|(program, _, _)| program)
    }

    pub fn compile_with_extras(
        self,
        name: Option<String>,
    ) -> Result<(Program, Source, Syntax), DustError> {
        let (
            CompileContext {
                mut constants,
                prototypes,
                ..
            },
            source,
            syntax,
        ) = self.compile_inner()?;
        let name_index =
            constants.add_string(name.as_deref().unwrap_or(DEFAULT_PROGRAM_NAME).as_bytes());

        Ok((
            Program {
                name_index,
                constants,
                prototypes,
            },
            source,
            syntax,
        ))
    }

    fn compile_inner(mut self) -> Result<(CompileContext, Source, Syntax), DustError> {
        let span = span!(Level::INFO, "compile");
        let _enter = span.enter();

        // Parsing phase
        {
            let span = span!(Level::INFO, "parse");
            let _enter = span.enter();

            let mut parse_errors = Vec::new();

            for (index, file) in self.source.files().iter().enumerate() {
                let file_id = SourceFileId(index as u32);
                let lexer = Lexer::new(file.source_code.as_ref());
                let parser = Parser::new(file_id, lexer);
                let ParseResult {
                    syntax_tree,
                    errors,
                } = if file_id == SourceFileId::MAIN {
                    parser.parse_main()
                } else {
                    parser.parse_file_module()
                };

                self.syntax.add_tree(syntax_tree);
                parse_errors.extend(errors);
            }

            if !parse_errors.is_empty() {
                return Err(DustError::parse(parse_errors, self.source));
            }
        }

        // Declaration binding phase
        {
            let span = span!(Level::INFO, "declare");
            let _enter = span.enter();

            let main_declaration_binder = DeclarationBinder::new(
                None,
                SourceFileId::MAIN,
                &self.source,
                &self.syntax,
                &mut self.context,
                ScopeId::PROJECT,
            );

            match main_declaration_binder.bind_main() {
                Ok(()) => (),
                Err(error) => return Err(DustError::compile(error, self.source)),
            }
        }

        // Type binding phase
        let main_function_type = {
            let span = span!(Level::INFO, "resolve");
            let _enter = span.enter();

            let main_type_binder =
                TypeBinder::new(None, SourceFileId::MAIN, &mut self.context, &self.syntax);

            match main_type_binder.resolve_main() {
                Ok(main_type) => main_type,
                Err(error) => return Err(DustError::compile(error, self.source)),
            }
        };

        // Emission phase
        {
            let span = span!(Level::INFO, "emit");
            let _enter = span.enter();

            self.context.prototypes.push(Prototype::default()); // Placeholder for main prototype

            let main_emitter = Emitter::new(
                None,
                0,
                SourceFileId::MAIN,
                main_function_type,
                &self.source,
                &self.syntax,
                &mut self.context,
                ScopeId::PROJECT,
            );

            self.context.prototypes[0] = match main_emitter.emit_main() {
                Ok(prototype) => prototype,
                Err(error) => return Err(DustError::compile(error, self.source)),
            };
        }

        Ok((self.context, self.source, self.syntax))
    }
}

fn get_type_id(node: SyntaxReader, context: &mut CompileContext) -> Result<TypeId, CompileError> {
    match node.kind() {
        SyntaxKind::BooleanType => Ok(TypeId::BOOLEAN),
        SyntaxKind::ByteType => Ok(TypeId::BYTE),
        SyntaxKind::CharacterType => Ok(TypeId::CHARACTER),
        SyntaxKind::FloatType => Ok(TypeId::FLOAT),
        SyntaxKind::IntegerType => Ok(TypeId::INTEGER),
        SyntaxKind::StringType => Ok(TypeId::STRING),
        SyntaxKind::ListType => {
            let element_type_node = node.left_child().ok_or(CompileError::MissingChild {
                parent_kind: node.kind(),
                child_index: 0,
            })?;

            let element_type_id = get_type_id(element_type_node, context)?;
            let lise_type_id = context.types.add_type(TypeNode::List {
                element_type: element_type_id,
            });

            Ok(lise_type_id)
        }
        SyntaxKind::FunctionType => {
            let function_type_node = {
                let type_node_value_parameters = if node.has_left_child() {
                    let function_value_parameters_node =
                        node.left_child().ok_or(CompileError::MissingChild {
                            parent_kind: node.kind(),
                            child_index: 0,
                        })?;

                    let value_parameters = function_value_parameters_node
                        .multiple_children()
                        .ok_or(CompileError::MissingChildren {
                            parent_kind: function_value_parameters_node.kind(),
                            start_index: function_value_parameters_node.inner().children.0,
                            count: function_value_parameters_node.inner().children.1,
                        })?;

                    let mut value_parameter_type_ids = SmallVec::<[TypeId; 4]>::new();

                    for value_parameter in value_parameters {
                        let type_id = if value_parameter.id == SyntaxId::NONE {
                            TypeId::NONE
                        } else {
                            get_type_id(value_parameter, context)?
                        };

                        value_parameter_type_ids.push(type_id);
                    }

                    context.types.add_type_members(&value_parameter_type_ids)
                } else {
                    (0, 0)
                };

                let return_type_id = if node.has_right_child() {
                    let function_return_type_node =
                        node.right_child().ok_or(CompileError::MissingChild {
                            parent_kind: node.kind(),
                            child_index: 1,
                        })?;

                    get_type_id(function_return_type_node, context)?
                } else {
                    TypeId::NONE
                };

                TypeNode::Function {
                    type_parameters: (0, 0),
                    value_parameters: type_node_value_parameters,
                    return_type_id,
                }
            };
            let function_type_id = context.types.add_type(function_type_node);

            Ok(function_type_id)
        }
        _ => {
            todo!()
        }
    }
}
