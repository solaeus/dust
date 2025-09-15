mod error;
mod parse_rule;

#[cfg(test)]
mod tests;

pub use error::ParseError;

use std::{
    mem::{replace, take},
    sync::Arc,
};

use lexical_core::{ParseFloatOptions, format::RUST_LITERAL, parse_with_options};
use smallvec::SmallVec;
use tracing::{Level, debug, error, info, span, warn};

use crate::{
    Lexer, Position, Resolver, Source, Span, Token, Type,
    dust_error::DustError,
    parser::parse_rule::{ParseRule, Precedence},
    resolver::{DeclarationId, DeclarationKind, Scope, ScopeId, ScopeKind, TypeId, TypeNode},
    source::SourceFile,
    syntax_tree::{SyntaxId, SyntaxKind, SyntaxNode, SyntaxTree},
};

pub fn parse_main(source: &'_ str) -> (SyntaxTree, Option<DustError>) {
    let mut resolver = Resolver::new(true);
    let parser = Parser::new(0, &mut resolver);
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse_once(source, ScopeId::MAIN);
    let dust_error = if errors.is_empty() {
        None
    } else {
        let name = "dust_program".to_string();
        let source = source.to_string();

        Some(DustError::parse(
            errors,
            Source::Script(Arc::new(SourceFile { name, source })),
        ))
    };

    (syntax_tree, dust_error)
}

pub struct ParseResult {
    pub syntax_tree: SyntaxTree,
    pub errors: Vec<ParseError>,
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    resolver: &'a mut Resolver,

    syntax_tree: SyntaxTree,

    current_token: Token,
    current_span: Span,
    previous_token: Token,
    previous_span: Span,

    current_scope_id: ScopeId,
    scope_count: u32,

    errors: Vec<ParseError>,
}

impl<'a> Parser<'a> {
    pub fn new(file_index: u32, resolver: &'a mut Resolver) -> Self {
        Self {
            lexer: Lexer::new(),
            resolver,
            syntax_tree: SyntaxTree::new(file_index),
            current_token: Token::Eof,
            current_span: Span::default(),
            previous_token: Token::Eof,
            previous_span: Span::default(),
            current_scope_id: ScopeId(0),
            errors: Vec::new(),
            scope_count: 0,
        }
    }

    /// Parses a source string as a complete file, returning the syntax tree and any parse errors.
    /// The parser is consumed and cannot be reused.
    pub fn parse_once(mut self, source: &'a str, scope: ScopeId) -> ParseResult {
        let (token, span) = self.lexer.initialize(source, 0);

        self.current_token = token;
        self.current_span = span;
        self.current_scope_id = scope;

        if scope == ScopeId::MAIN {
            self.parse_main_function_item();
        } else {
            let _ = self
                .parse_module_item()
                .map_err(|error| self.recover(error));
        }

        ParseResult {
            syntax_tree: self.syntax_tree,
            errors: self.errors,
        }
    }

    /// Parses a source string as a complete file, returning the syntax tree and any parse errors.
    /// Afterwards, the parser is reset and can be reused.
    pub fn parse(&mut self, source: &'a str, scope: ScopeId) -> ParseResult {
        let (token, span) = self.lexer.initialize(source, 0);

        self.current_token = token;
        self.current_span = span;
        self.current_scope_id = scope;

        if scope == ScopeId::MAIN {
            self.parse_main_function_item();
        } else {
            let _ = self
                .parse_module_item()
                .map_err(|error| self.recover(error));
        }

        let file_index = self.syntax_tree.file_index;
        let syntax_tree = replace(&mut self.syntax_tree, SyntaxTree::new(file_index));
        let errors = take(&mut self.errors);

        ParseResult {
            syntax_tree,
            errors,
        }
    }

    /// Parses a source string, allowing it to be a subtree of a larger syntax tree. Afterwards, the
    /// parser is reset and can be reused.
    pub fn parse_subtree(&mut self, source: &'a str, offset: usize, scope: ScopeId) -> ParseResult {
        let (token, span) = self.lexer.initialize(source, offset);

        self.current_token = token;
        self.current_span = span;
        self.current_scope_id = scope;

        let placeholder_node = SyntaxNode {
            kind: SyntaxKind::MainFunctionItem,
            span: Span::default(),
            children: (0, 0),
            payload: TypeId::NONE.0,
        };

        self.syntax_tree.push_node(placeholder_node);

        while self.current_token != Token::Eof {
            let _ = self
                .pratt(Precedence::None)
                .map_err(|error| self.recover(error));
        }

        self.syntax_tree.nodes.swap_remove(0);

        let file_index = self.syntax_tree.file_index;
        let syntax_tree = replace(&mut self.syntax_tree, SyntaxTree::new(file_index));
        let errors = take(&mut self.errors);

        ParseResult {
            syntax_tree,
            errors,
        }
    }

    fn current_position(&self) -> Position {
        Position::new(self.syntax_tree.file_index, self.current_span)
    }

    fn new_child_buffer() -> SmallVec<[SyntaxId; 4]> {
        SmallVec::<[SyntaxId; 4]>::new()
    }

    fn new_child_scope(&mut self, kind: ScopeKind) -> ScopeId {
        let current_scope = self
            .resolver
            .get_scope(self.current_scope_id)
            .expect("Scope must exist");
        let scope_id = self.resolver.add_scope(Scope {
            kind,
            parent: self.current_scope_id,
            imports: SmallVec::new(),
            modules: SmallVec::new(),
            depth: current_scope.depth + 1,
            index: self.scope_count,
        });
        self.scope_count += 1;

        scope_id
    }

    fn pratt(&mut self, precedence: Precedence) -> Result<(), ParseError> {
        if self.current_token.is_whitespace() {
            self.advance()?;
        }

        let prefix_rule = ParseRule::from(self.current_token);

        if let Some(prefix_parser) = prefix_rule.prefix {
            debug!("{} at {} is prefix", self.current_token, self.current_span,);

            prefix_parser(self)?;
        }

        let mut infix_rule = ParseRule::from(self.current_token);

        while precedence <= infix_rule.precedence
            && let Some(infix_parser) = infix_rule.infix
            && self.previous_token != Token::Semicolon
        {
            debug!(
                "{} at {} as infix {}",
                self.current_token, self.current_span, infix_rule.precedence,
            );

            infix_parser(self)?;

            infix_rule = ParseRule::from(self.current_token);
        }

        Ok(())
    }

    fn advance(&mut self) -> Result<(), ParseError> {
        let (next_token, next_position) = self.lexer.next_token();

        if next_token.is_whitespace() {
            return self.advance();
        }

        self.previous_token = replace(&mut self.current_token, next_token);
        self.previous_span = replace(&mut self.current_span, next_position);

        Ok(())
    }

    fn recover(&mut self, error: ParseError) {
        error!("{error:?}");

        self.errors.push(error);

        if self.previous_token == Token::Semicolon {
            warn!("Error recovery is continuing after a semicolon");

            return;
        }

        while !matches!(
            self.current_token,
            Token::Semicolon | Token::RightCurlyBrace | Token::Eof
        ) {
            let _ = self.advance().map_err(|error| self.recover(error));
        }

        warn!(
            "Error recovery has skipped to {} at {}",
            self.current_token, self.current_span
        );

        if matches!(
            self.current_token,
            Token::Semicolon | Token::RightCurlyBrace
        ) {
            let _ = self.advance().map_err(|error| self.recover(error));
        }
    }

    fn allow(&mut self, allowed: Token) -> Result<bool, ParseError> {
        let allowed = self.current_token == allowed;

        if allowed {
            self.advance()?;
        }

        Ok(allowed)
    }

    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        if self.current_token != expected {
            return Err(ParseError::ExpectedToken {
                expected,
                actual: self.current_token,
                position: self.current_position(),
            });
        }

        self.advance()?;

        Ok(())
    }

    fn current_source(&self) -> &'a str {
        &self.lexer.source()[self.current_span.as_usize_range()]
    }

    fn parse_item(&mut self) -> Result<(), ParseError> {
        self.pratt(Precedence::None)?;

        let last_node_id = self.syntax_tree.last_node_id();

        if let Some(node) = self.syntax_tree.get_node(last_node_id) {
            if node.kind.is_item() {
                return Ok(());
            }

            if node.kind.is_statement() {
                let item_statement_node = SyntaxNode {
                    kind: SyntaxKind::ItemStatement,
                    span: node.span,
                    children: (last_node_id.0, 0),
                    payload: TypeId::NONE.0,
                };

                self.syntax_tree.push_node(item_statement_node);

                return Ok(());
            }

            Err(ParseError::ExpectedItem {
                actual: node.kind,
                position: Position::new(self.syntax_tree.file_index, node.span),
            })
        } else {
            Err(ParseError::UnexpectedToken {
                actual: self.previous_token,
                position: Position::new(self.syntax_tree.file_index, self.previous_span),
            })
        }
    }

    pub fn parse_pub_item(&mut self) -> Result<(), ParseError> {
        info!("Parsing pub item");

        self.advance()?;

        match self.current_token {
            Token::Use => self.parse_use_item()?,
            Token::Mod => self.parse_module_item()?,
            Token::Fn => self.parse_function_statement_or_expression()?,
            _ => {
                return Err(ParseError::ExpectedMultipleTokens {
                    expected: &[Token::Use, Token::Mod, Token::Fn],
                    actual: self.current_token,
                    position: self.current_position(),
                });
            }
        }

        Ok(())
    }

    fn _parse_statement(&mut self) -> Result<(), ParseError> {
        self.pratt(Precedence::None)?;

        if let Some(node) = self.syntax_tree.last_node()
            && !node.kind.is_statement()
        {
            Err(ParseError::ExpectedStatement {
                actual: node.kind,
                position: Position::new(self.syntax_tree.file_index, node.span),
            })
        } else {
            Ok(())
        }
    }

    fn parse_expression(&mut self) -> Result<(), ParseError> {
        self.pratt(Precedence::None)?;

        if let Some(node) = self.syntax_tree.last_node()
            && !node.kind.is_expression()
        {
            Err(ParseError::ExpectedExpression {
                actual: node.kind,
                position: Position::new(self.syntax_tree.file_index, node.span),
            })
        } else {
            Ok(())
        }
    }

    fn parse_sub_expression(&mut self, precedence: Precedence) -> Result<(), ParseError> {
        self.pratt(precedence.increment())?;

        if let Some(node) = self.syntax_tree.last_node()
            && !node.kind.is_expression()
        {
            Err(ParseError::ExpectedExpression {
                actual: node.kind,
                position: Position::new(self.syntax_tree.file_index, node.span),
            })
        } else {
            Ok(())
        }
    }

    fn parse_unexpected(&mut self) -> Result<(), ParseError> {
        Err(ParseError::UnexpectedToken {
            actual: self.current_token,
            position: self.current_position(),
        })
    }

    pub fn parse_main_function_item(&mut self) {
        let span = span!(Level::INFO, "main");
        let _enter = span.enter();
        let placeholder_node = SyntaxNode {
            kind: SyntaxKind::MainFunctionItem,
            span: Span::default(),
            children: (0, 0),
            payload: TypeId::NONE.0,
        };
        self.current_scope_id = ScopeId::MAIN;

        self.syntax_tree.push_node(placeholder_node);

        let mut children = Self::new_child_buffer();

        while self.current_token != Token::Eof {
            if let Err(error) = self.pratt(Precedence::None) {
                self.recover(error);
            } else {
                let child_id = self.syntax_tree.last_node_id();

                if child_id == SyntaxId(0) {
                    break;
                }

                children.push(child_id);
            }
        }

        let last_node_type = if let Some(last_node) = self.syntax_tree.last_node()
            && last_node.kind.is_expression()
        {
            last_node.payload
        } else {
            TypeId::NONE.0
        };

        let first_child = self.syntax_tree.children.len() as u32;
        let child_count = children.len() as u32;

        self.syntax_tree.nodes[0] = SyntaxNode {
            kind: SyntaxKind::MainFunctionItem,
            span: Span(0, self.current_span.1),
            children: (first_child, child_count),
            payload: last_node_type,
        };

        self.syntax_tree.children.extend(children);
    }

    pub fn parse_module_item(&mut self) -> Result<(), ParseError> {
        info!("Parsing module item");

        let start = self.current_span.0;
        let starting_scope_id = self.current_scope_id;
        let is_public = self.previous_token == Token::Pub;

        // Allows for nested modules and whole file modules
        let (end_token, declaration_id) = if self.current_token == Token::Mod {
            self.advance()?;

            let identifier_text = self.current_source();
            self.current_scope_id = self.new_child_scope(ScopeKind::Module);
            let declaration_id = self.resolver.add_declaration(
                DeclarationKind::Module {
                    inner_scope_id: self.current_scope_id,
                },
                starting_scope_id,
                TypeId::NONE,
                is_public,
                identifier_text,
                Position::new(self.syntax_tree.file_index, self.current_span),
            );

            self.expect(Token::Identifier)?;
            self.expect(Token::LeftCurlyBrace)?;

            (Token::RightCurlyBrace, declaration_id)
        } else {
            (Token::Eof, DeclarationId(0))
        };

        let mut children = Self::new_child_buffer();

        while !self.allow(end_token)? {
            self.parse_item()?;

            children.push(self.syntax_tree.last_node_id());
        }

        let end = self.previous_span.1;
        self.current_scope_id = starting_scope_id;

        let first_child = self.syntax_tree.children.len() as u32;
        let child_count = children.len() as u32;
        let node = SyntaxNode {
            kind: SyntaxKind::ModuleItem,
            span: Span(start, end),
            children: (first_child, child_count),
            payload: declaration_id.0,
        };

        self.syntax_tree.push_node(node);
        self.syntax_tree.children.extend(children);
        self.resolver
            .add_module_to_scope(starting_scope_id, declaration_id);

        Ok(())
    }

    fn parse_use_item(&mut self) -> Result<(), ParseError> {
        info!("Parsing use statement");

        let start = self.current_span.0;

        self.advance()?;

        let mut identifier_span = self.current_span;
        let mut identifier_text = self.current_source();

        self.expect(Token::Identifier)?;

        let mut declaration_id = self
            .resolver
            .find_declaration_in_scope(identifier_text, self.current_scope_id)
            .ok_or_else(|| ParseError::UndeclaredModule {
                identifier: identifier_text.to_string(),
                position: Position::new(self.syntax_tree.file_index, identifier_span),
            })?;
        let mut declaration = *self
            .resolver
            .get_declaration(declaration_id)
            .ok_or(ParseError::MissingDeclaration { id: declaration_id })?;

        while self.allow(Token::DoubleColon)? {
            identifier_span = self.current_span;
            identifier_text = self.current_source();

            self.expect(Token::Identifier)?;

            let DeclarationKind::Module { inner_scope_id } = declaration.kind else {
                return Err(ParseError::ExpectedModule {
                    identifier: identifier_text.to_string(),
                    position: Position::new(self.syntax_tree.file_index, identifier_span),
                });
            };
            let Some(next_declaration_id) = self
                .resolver
                .find_declaration_in_scope(identifier_text, inner_scope_id)
            else {
                return Err(ParseError::UndeclaredModule {
                    identifier: identifier_text.to_string(),
                    position: Position::new(self.syntax_tree.file_index, identifier_span),
                });
            };

            declaration_id = next_declaration_id;
            declaration = *self
                .resolver
                .get_declaration(declaration_id)
                .ok_or(ParseError::MissingDeclaration { id: declaration_id })?;

            if !declaration.is_public {
                return Err(ParseError::PrivateImport {
                    identifier: identifier_text.to_string(),
                    position: Position::new(self.syntax_tree.file_index, identifier_span),
                });
            }
        }

        self.allow(Token::Semicolon)?;

        let end = self.previous_span.1;
        let node = SyntaxNode {
            kind: SyntaxKind::UseItem,
            span: Span(start, end),
            children: (0, 0),
            payload: declaration_id.0,
        };

        self.syntax_tree.push_node(node);
        self.resolver
            .add_import_to_scope(self.current_scope_id, declaration_id);

        Ok(())
    }

    fn parse_function_statement_or_expression(&mut self) -> Result<(), ParseError> {
        let start = self.current_span.0;
        let is_public = self.previous_token == Token::Pub;

        self.advance()?;

        match self.current_token {
            Token::Identifier => {
                info!("Parsing function statement");

                let identifier_position =
                    Position::new(self.syntax_tree.file_index, self.current_span);
                let identifier_text = self.current_source();

                if let Some(existing_declaration) = self
                    .resolver
                    .find_declaration_in_scope(identifier_text, self.current_scope_id)
                {
                    let existing_declaration = self
                        .resolver
                        .get_declaration(existing_declaration)
                        .ok_or(ParseError::MissingDeclaration {
                            id: existing_declaration,
                        })?;

                    return Err(ParseError::DeclarationConflict {
                        identifier: identifier_text.to_string(),
                        first_declaration: existing_declaration.identifier_position,
                        second_declaration: identifier_position,
                    });
                }

                self.advance()?;
                self.parse_function_expression()?;

                let type_id = self.syntax_tree.last_node().unwrap().payload;
                let end = self.previous_span.1;
                let declaration_id = self.resolver.add_declaration(
                    DeclarationKind::Function,
                    self.current_scope_id,
                    TypeId(type_id),
                    is_public,
                    identifier_text,
                    identifier_position,
                );
                let function_expression_id = self.syntax_tree.last_node_id();
                let node = SyntaxNode {
                    kind: SyntaxKind::FunctionStatement,
                    span: Span(start, end),
                    children: (declaration_id.0, function_expression_id.0),
                    payload: TypeId::NONE.0,
                };

                self.syntax_tree.push_node(node);

                if is_public {
                    self.resolver
                        .add_module_to_scope(self.current_scope_id, declaration_id);
                }

                Ok(())
            }
            Token::LeftParenthesis => {
                self.parse_function_expression()?;

                Ok(())
            }
            _ => Err(ParseError::ExpectedMultipleTokens {
                expected: &[Token::Identifier, Token::LeftParenthesis],
                actual: self.current_token,
                position: self.current_position(),
            }),
        }
    }

    fn parse_function_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing function expression");

        let starting_scope_id = self.current_scope_id;
        self.current_scope_id = self.new_child_scope(ScopeKind::Function);

        let start = self.current_span.0;
        let (function_signature_id, function_type_id) = self.parse_function_signature()?;

        self.parse_block_expression()?;

        let block_id = self.syntax_tree.last_node_id();
        let end = self.previous_span.1;
        let node = SyntaxNode {
            kind: SyntaxKind::FunctionExpression,
            span: Span(start, end),
            children: (function_signature_id.0, block_id.0),
            payload: function_type_id.0,
        };

        self.syntax_tree.push_node(node);

        self.current_scope_id = starting_scope_id;

        Ok(())
    }

    fn parse_function_signature(&mut self) -> Result<(SyntaxId, TypeId), ParseError> {
        info!("Parsing function signature");

        let start = self.current_span.0;
        let (value_parameter_list_node_id, type_children) =
            self.parse_function_value_parameters()?;
        let return_type = if self.allow(Token::ArrowThin)? {
            self.parse_type()?
        } else {
            TypeId::NONE
        };
        let end = self.previous_span.1;
        let function_type = TypeNode::Function {
            type_parameters: (0, 0),
            value_parameters: type_children,
            return_type,
        };
        let function_type_id = self.resolver.add_type(function_type);
        let node = SyntaxNode {
            kind: SyntaxKind::FunctionSignature,
            span: Span(start, end),
            children: (value_parameter_list_node_id.0, 0),
            payload: 0,
        };
        let node_id = self.syntax_tree.push_node(node);

        Ok((node_id, function_type_id))
    }

    fn parse_function_value_parameters(&mut self) -> Result<(SyntaxId, (u32, u32)), ParseError> {
        info!("Parsing function value parameters");

        let start = self.current_span.0;

        self.expect(Token::LeftParenthesis)?;

        let mut syntax_children = Self::new_child_buffer();
        let mut type_children = SmallVec::<[TypeId; 4]>::new();

        while !self.allow(Token::RightParenthesis)? {
            info!("Parsing function value parameter");

            let parameter_start = self.current_span.0;
            let identifier_position = Position::new(self.syntax_tree.file_index, self.current_span);
            let identifier_text = if self.current_token == Token::Identifier {
                let text = self.current_source();

                self.advance()?;

                text
            } else {
                return Err(ParseError::ExpectedToken {
                    expected: Token::Identifier,
                    actual: self.current_token,
                    position: self.current_position(),
                });
            };
            let parameter_name_node = SyntaxNode {
                kind: SyntaxKind::FunctionValueParameterName,
                span: identifier_position.span,
                children: (0, 0),
                payload: 0,
            };
            let parameter_name_node_id = self.syntax_tree.push_node(parameter_name_node);

            self.expect(Token::Colon)?;

            let type_position = self.current_span;
            let type_id = self.parse_type()?;

            type_children.push(type_id);

            let type_node = SyntaxNode {
                kind: SyntaxKind::FunctionValueParameterType,
                span: type_position,
                children: (0, 0),
                payload: 0,
            };
            let type_node_id = self.syntax_tree.push_node(type_node);
            let parameter_end = self.previous_span.1;

            if let Some(existing_declaration_id) = self
                .resolver
                .find_declaration_in_scope(identifier_text, self.current_scope_id)
            {
                let existing_declaration = self
                    .resolver
                    .get_declaration(existing_declaration_id)
                    .ok_or(ParseError::MissingDeclaration {
                    id: existing_declaration_id,
                })?;

                return Err(ParseError::DeclarationConflict {
                    identifier: identifier_text.to_string(),
                    first_declaration: existing_declaration.identifier_position,
                    second_declaration: identifier_position,
                });
            }

            let declaration_id = self.resolver.add_declaration(
                DeclarationKind::Local { shadowed: None },
                self.current_scope_id,
                type_id,
                false,
                identifier_text,
                identifier_position,
            );
            let node = SyntaxNode {
                kind: SyntaxKind::FunctionValueParameter,
                span: Span(parameter_start, parameter_end),
                children: (parameter_name_node_id.0, type_node_id.0),
                payload: declaration_id.0,
            };
            let node_id = self.syntax_tree.push_node(node);

            syntax_children.push(node_id);

            self.allow(Token::Comma)?;
        }

        let first_child = self.syntax_tree.children.len() as u32;
        let child_count = syntax_children.len() as u32;
        let end = self.previous_span.1;
        let node = SyntaxNode {
            kind: SyntaxKind::FunctionValueParameters,
            span: Span(start, end),
            children: (first_child, child_count),
            payload: 0,
        };

        let node_id = self.syntax_tree.push_node(node);

        self.syntax_tree.children.extend(syntax_children);

        let type_children = self.resolver.push_type_members(&type_children);

        Ok((node_id, type_children))
    }

    fn parse_type(&mut self) -> Result<TypeId, ParseError> {
        info!("Parsing type");

        let start = self.current_span.0;

        let (node_kind, r#type) = match self.current_token {
            Token::Bool => (SyntaxKind::BooleanType, TypeId::BOOLEAN),
            Token::Byte => (SyntaxKind::ByteType, TypeId::BYTE),
            Token::Char => (SyntaxKind::CharacterType, TypeId::CHARACTER),
            Token::Float => (SyntaxKind::FloatType, TypeId::FLOAT),
            Token::Int => (SyntaxKind::IntegerType, TypeId::INTEGER),
            Token::Str => (SyntaxKind::StringType, TypeId::STRING),
            Token::Identifier => {
                let identifier_text = self.current_source();
                let declaration_id = self
                    .resolver
                    .find_declaration_in_scope(identifier_text, self.current_scope_id)
                    .ok_or(ParseError::UndeclaredType {
                        identifier: identifier_text.to_string(),
                        position: self.current_position(),
                    })?;
                let declaration = self
                    .resolver
                    .get_declaration(declaration_id)
                    .ok_or(ParseError::MissingDeclaration { id: declaration_id })?;

                (SyntaxKind::TypePath, declaration.type_id)
            }
            _ => {
                return Err(ParseError::ExpectedMultipleTokens {
                    expected: &[
                        Token::Bool,
                        Token::Byte,
                        Token::Char,
                        Token::Float,
                        Token::Int,
                        Token::Str,
                        Token::Identifier,
                    ],
                    actual: self.current_token,
                    position: self.current_position(),
                });
            }
        };

        self.advance()?;

        let end = self.previous_span.1;
        let node = SyntaxNode {
            kind: node_kind,
            span: Span(start, end),
            children: (0, 0),
            payload: 0,
        };

        self.syntax_tree.push_node(node);

        Ok(r#type)
    }

    fn parse_let_statement(&mut self) -> Result<(), ParseError> {
        info!("Parsing let statement");

        let start = self.current_span.0;

        self.advance()?;

        let is_mutable = self.allow(Token::Mut)?;
        let identifier_position = Position::new(self.syntax_tree.file_index, self.current_span);
        let identifier_text = self.current_source();

        self.expect(Token::Identifier)?;

        let shadowed = self
            .resolver
            .find_declaration_in_scope(identifier_text, self.current_scope_id);
        let (syntax_kind, declaration_kind) = if is_mutable {
            (
                SyntaxKind::LetMutStatement,
                DeclarationKind::LocalMutable { shadowed },
            )
        } else {
            (
                SyntaxKind::LetStatement,
                DeclarationKind::Local { shadowed },
            )
        };
        let (explicit_type, type_node_id) = if self.allow(Token::Colon)? {
            (Some(self.parse_type()?), self.syntax_tree.last_node_id())
        } else {
            (None, SyntaxId::NONE)
        };

        self.expect(Token::Equal)?;
        self.pratt(Precedence::None)?;

        let end = self.previous_span.1;
        let expression_id = self.syntax_tree.last_node_id();
        let expression_node = self
            .syntax_tree
            .get_node(expression_id)
            .ok_or(ParseError::MissingNode { id: expression_id })?;
        let expression_type = expression_node.payload;

        if expression_node.kind != SyntaxKind::ExpressionStatement {
            return Err(ParseError::ExpectedToken {
                actual: self.current_token,
                expected: Token::Semicolon,
                position: self.current_position(),
            });
        }

        if let Some(explicit_type) = explicit_type
            && explicit_type.0 != expression_type
        {
            todo!("Error");
        }

        let declaration_id = self.resolver.add_declaration(
            declaration_kind,
            self.current_scope_id,
            TypeId(expression_type),
            false,
            identifier_text,
            identifier_position,
        );
        let node = SyntaxNode {
            kind: syntax_kind,
            span: Span(start, end),
            children: (type_node_id.0, expression_id.0),
            payload: declaration_id.0,
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_reassign_statement(&mut self) -> Result<(), ParseError> {
        info!("Parsing reassign statement");

        let operator = self.current_token;
        let operator_precedence = ParseRule::from(operator).precedence;

        let path_node_id = self.syntax_tree.last_node_id();
        let path_node = *self
            .syntax_tree
            .get_node(path_node_id)
            .ok_or(ParseError::MissingNode { id: path_node_id })?;
        let declaration_id = DeclarationId(path_node.children.0);
        let start = path_node.span.0;

        if path_node.kind != SyntaxKind::PathExpression {
            return Err(ParseError::InvalidAssignmentTarget {
                found: path_node.kind,
                position: Position::new(self.syntax_tree.file_index, path_node.span),
            });
        }

        let declaration = *self
            .resolver
            .get_declaration(declaration_id)
            .ok_or(ParseError::MissingDeclaration { id: declaration_id })?;

        if !matches!(declaration.kind, DeclarationKind::LocalMutable { .. }) {
            return Err(ParseError::AssignmentToImmutable {
                found: declaration.kind,
                position: Position::new(self.syntax_tree.file_index, path_node.span),
            });
        }

        self.expect(Token::Equal)?;
        self.parse_sub_expression(operator_precedence)?;

        let expression_id = self.syntax_tree.last_node_id();
        let expression_node = self
            .syntax_tree
            .get_node(expression_id)
            .ok_or(ParseError::MissingNode { id: expression_id })?;
        let expression_type = expression_node.payload;

        if expression_node.kind != SyntaxKind::ExpressionStatement {
            self.expect(Token::Semicolon)?;
        }

        if declaration.type_id.0 != expression_type {
            todo!("Error");
        }

        let end = self.previous_span.1;
        let node = SyntaxNode {
            kind: SyntaxKind::ReassignStatement,
            span: Span(start, end),
            children: (path_node_id.0, expression_id.0),
            payload: declaration_id.0,
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_boolean_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing boolean expression");

        let boolean_payload = match self.current_token {
            Token::TrueValue => true as u32,
            Token::FalseValue => false as u32,
            _ => {
                return Err(ParseError::ExpectedMultipleTokens {
                    expected: &[Token::TrueValue, Token::FalseValue],
                    actual: self.current_token,
                    position: self.current_position(),
                });
            }
        };
        let node = SyntaxNode {
            kind: SyntaxKind::BooleanExpression,
            span: self.current_span,
            children: (boolean_payload, 0),
            payload: TypeId::BOOLEAN.0,
        };

        self.advance()?;
        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_byte_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing byte expression");

        let position = self.current_span;
        let byte_text_utf8 = &self.current_source().as_bytes()[2..]; // Skip the "0x" prefix

        self.advance()?;

        let byte_payload = u8::from_ascii_radix(byte_text_utf8, 16).unwrap_or_default() as u32;
        let node = SyntaxNode {
            kind: SyntaxKind::ByteExpression,
            span: position,
            children: (byte_payload, 0),
            payload: TypeId::BYTE.0,
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_character_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing character expression");

        let start = self.current_span.0;
        let character_text = self.current_source();

        self.advance()?;

        let end = self.previous_span.1;
        let character_payload = character_text.chars().nth(1).unwrap_or_default() as u32;
        let node = SyntaxNode {
            kind: SyntaxKind::CharacterExpression,
            span: Span(start, end),
            children: (character_payload, 0),
            payload: TypeId::CHARACTER.0,
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_float_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing float expression");

        let start = self.current_span.0;
        let float_text = self.current_source();

        self.advance()?;

        let end = self.previous_span.1;
        let float = parse_with_options::<f64, RUST_LITERAL>(
            float_text.as_bytes(),
            &ParseFloatOptions::default(),
        )
        .unwrap_or_default();
        let payload = SyntaxNode::encode_float(float);
        let node = SyntaxNode {
            kind: SyntaxKind::FloatExpression,
            span: Span(start, end),
            children: payload,
            payload: TypeId::FLOAT.0,
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_integer_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing integer expression");

        let start = self.current_span.0;
        let integer_text = self.current_source();

        self.advance()?;

        let end = self.previous_span.1;
        let integer = Self::parse_integer(integer_text);
        let (left_payload, right_payload) = SyntaxNode::encode_integer(integer);
        let node = SyntaxNode {
            kind: SyntaxKind::IntegerExpression,
            span: Span(start, end),
            children: (left_payload, right_payload),
            payload: TypeId::INTEGER.0,
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_integer(text: &str) -> i64 {
        let mut integer = 0_i64;
        let mut chars = text.chars().peekable();

        let is_positive = if chars.peek() == Some(&'-') {
            chars.next();

            false
        } else {
            true
        };

        let mut digit_place = 0;

        for character in chars.rev() {
            let Some(digit) = character.to_digit(10) else {
                continue;
            };

            let place_value = 10_i64.pow(digit_place);
            let digit_value = digit as i64 * place_value;
            digit_place += 1;

            integer = integer.saturating_add(digit_value);
        }

        if is_positive { integer } else { -integer }
    }

    fn parse_string_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing string expression");

        let node = SyntaxNode {
            kind: SyntaxKind::StringExpression,
            span: self.current_span,
            children: (0, 0),
            payload: TypeId::STRING.0,
        };

        self.advance()?;
        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_unary_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing unary expression");

        let operator = self.current_token;
        let node_kind = match operator {
            Token::Minus => SyntaxKind::NegationExpression,
            Token::Bang => SyntaxKind::NotExpression,
            _ => {
                return Err(ParseError::ExpectedMultipleTokens {
                    expected: &[Token::Minus, Token::Bang],
                    actual: operator,
                    position: self.current_position(),
                });
            }
        };
        let operator_precedence = ParseRule::from(operator).precedence;
        let start = self.current_span.0;

        self.advance()?;
        self.parse_sub_expression(operator_precedence)?;

        let operand = self.syntax_tree.last_node_id();
        let operand_node = self
            .syntax_tree
            .get_node(operand)
            .ok_or(ParseError::MissingNode { id: operand })?;
        let end = self.previous_span.1;
        let r#type = match operator {
            Token::Minus => match TypeId(operand_node.payload) {
                TypeId::BYTE => TypeId::BYTE,
                TypeId::INTEGER => TypeId::INTEGER,
                TypeId::FLOAT => TypeId::FLOAT,
                _ => {
                    let operand_type = self
                        .resolver
                        .resolve_type(TypeId(operand_node.payload))
                        .unwrap_or(Type::None);

                    return Err(ParseError::NegationTypeMismatch {
                        operand_type,
                        operand_position: Position::new(
                            self.syntax_tree.file_index,
                            operand_node.span,
                        ),
                        position: Position::new(self.syntax_tree.file_index, Span(start, end)),
                    });
                }
            },
            Token::Bang => match TypeId(operand_node.payload) {
                TypeId::BOOLEAN => TypeId::BOOLEAN,
                _ => {
                    let operand_type = self
                        .resolver
                        .resolve_type(TypeId(operand_node.payload))
                        .unwrap_or(Type::None);

                    return Err(ParseError::NotTypeMismatch {
                        operand_type,
                        operand_position: Position::new(
                            self.syntax_tree.file_index,
                            operand_node.span,
                        ),
                        position: Position::new(self.syntax_tree.file_index, Span(start, end)),
                    });
                }
            },
            _ => unreachable!(),
        };

        let node = SyntaxNode {
            kind: node_kind,
            span: Span(start, end),
            children: (operand.0, 0),
            payload: r#type.0,
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_binary_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing math binary expression");

        let left = self.syntax_tree.last_node_id();
        let left_node = *self
            .syntax_tree
            .get_node(left)
            .ok_or(ParseError::MissingNode { id: left })?;
        let start = left_node.span.0;
        let operator = self.current_token;
        let node_kind = match operator {
            Token::Plus => SyntaxKind::AdditionExpression,
            Token::PlusEqual => SyntaxKind::AdditionAssignmentExpression,
            Token::Minus => SyntaxKind::SubtractionExpression,
            Token::MinusEqual => SyntaxKind::SubtractionAssignmentExpression,
            Token::Asterisk => SyntaxKind::MultiplicationExpression,
            Token::AsteriskEqual => SyntaxKind::MultiplicationAssignmentExpression,
            Token::Slash => SyntaxKind::DivisionExpression,
            Token::SlashEqual => SyntaxKind::DivisionAssignmentExpression,
            Token::Percent => SyntaxKind::ModuloExpression,
            Token::PercentEqual => SyntaxKind::ModuloAssignmentExpression,
            Token::Greater => SyntaxKind::GreaterThanExpression,
            Token::GreaterEqual => SyntaxKind::GreaterThanOrEqualExpression,
            Token::Less => SyntaxKind::LessThanExpression,
            Token::LessEqual => SyntaxKind::LessThanOrEqualExpression,
            Token::DoubleEqual => SyntaxKind::EqualExpression,
            Token::BangEqual => SyntaxKind::NotEqualExpression,
            Token::DoubleAmpersand => SyntaxKind::AndExpression,
            Token::DoublePipe => SyntaxKind::OrExpression,
            _ => {
                return Err(ParseError::ExpectedMultipleTokens {
                    expected: &[
                        Token::Plus,
                        Token::Minus,
                        Token::Asterisk,
                        Token::Slash,
                        Token::Percent,
                    ],
                    actual: operator,
                    position: self.current_position(),
                });
            }
        };

        if matches!(
            operator,
            Token::PlusEqual
                | Token::MinusEqual
                | Token::AsteriskEqual
                | Token::SlashEqual
                | Token::PercentEqual
        ) {
            let declaration = if left_node.kind == SyntaxKind::PathExpression {
                self.resolver
                    .get_declaration(DeclarationId(left_node.children.0))
                    .ok_or(ParseError::MissingDeclaration {
                        id: DeclarationId(left_node.children.0),
                    })?
            } else {
                return Err(ParseError::InvalidAssignmentTarget {
                    found: left_node.kind,
                    position: Position::new(self.syntax_tree.file_index, left_node.span),
                });
            };

            if !matches!(declaration.kind, DeclarationKind::LocalMutable { .. }) {
                return Err(ParseError::AssignmentToImmutable {
                    found: declaration.kind,
                    position: Position::new(self.syntax_tree.file_index, left_node.span),
                });
            }
        }

        let operator_precedence = ParseRule::from(operator).precedence;

        self.advance()?;
        self.parse_sub_expression(operator_precedence)?;

        let right = self.syntax_tree.last_node_id();
        let right_node = self
            .syntax_tree
            .get_node(right)
            .ok_or(ParseError::MissingNode { id: right })?;
        let end = self.previous_span.1;
        let r#type = match operator {
            Token::Plus => match (TypeId(left_node.payload), TypeId(right_node.payload)) {
                (TypeId::CHARACTER, TypeId::CHARACTER)
                | (TypeId::CHARACTER, TypeId::STRING)
                | (TypeId::STRING, TypeId::CHARACTER) => TypeId::STRING,
                _ => {
                    let left_type = self
                        .resolver
                        .resolve_type(TypeId(left_node.payload))
                        .unwrap_or(Type::None);
                    let right_type = self
                        .resolver
                        .resolve_type(TypeId(right_node.payload))
                        .unwrap_or(Type::None);

                    if left_type != right_type {
                        return Err(ParseError::AdditionTypeMismatch {
                            left_type,
                            left_position: Position::new(
                                self.syntax_tree.file_index,
                                left_node.span,
                            ),
                            right_type,
                            right_position: Position::new(
                                self.syntax_tree.file_index,
                                right_node.span,
                            ),
                            position: Position::new(self.syntax_tree.file_index, Span(start, end)),
                        });
                    }

                    TypeId(left_node.payload)
                }
            },
            Token::Greater
            | Token::GreaterEqual
            | Token::Less
            | Token::LessEqual
            | Token::DoubleEqual
            | Token::BangEqual => TypeId::BOOLEAN,
            Token::PlusEqual
            | Token::MinusEqual
            | Token::AsteriskEqual
            | Token::SlashEqual
            | Token::PercentEqual => TypeId::NONE,
            _ => {
                let left_type = self
                    .resolver
                    .resolve_type(TypeId(left_node.payload))
                    .unwrap_or(Type::None);
                let right_type = self
                    .resolver
                    .resolve_type(TypeId(right_node.payload))
                    .unwrap_or(Type::None);

                if left_type != right_type {
                    return Err(ParseError::BinaryOperandTypeMismatch {
                        operator,
                        left_type,
                        left_position: Position::new(self.syntax_tree.file_index, left_node.span),
                        right_type,
                        right_position: Position::new(self.syntax_tree.file_index, right_node.span),
                        position: Position::new(self.syntax_tree.file_index, Span(start, end)),
                    });
                }

                TypeId(left_node.payload)
            }
        };

        let node = SyntaxNode {
            kind: node_kind,
            span: Span(start, end),
            children: (left.0, right.0),
            payload: r#type.0,
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_call_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing call expression");

        self.advance()?;

        let function_node_id = self.syntax_tree.last_node_id();
        let function_node =
            *self
                .syntax_tree
                .get_node(function_node_id)
                .ok_or(ParseError::MissingNode {
                    id: function_node_id,
                })?;
        let function_node_type = self.resolver.get_type_node(TypeId(function_node.payload));
        let start = function_node.span.0;

        if !matches!(function_node_type, Some(TypeNode::Function { .. })) {
            return Err(ParseError::ExpectedFunction {
                found: function_node.kind,
                position: Position::new(self.syntax_tree.file_index, function_node.span),
            });
        };

        if function_node.kind == SyntaxKind::PathExpression {
            let identifier = &self.lexer.source()[function_node.span.as_usize_range()];

            if self
                .resolver
                .find_declaration_in_scope(identifier, self.current_scope_id)
                .is_none()
            {
                if let Some(declarations) = self.resolver.find_declarations(identifier) {
                    return Err(ParseError::OutOfScopeVariable {
                        position: self.current_position(),
                        declaration_positions: declarations
                            .iter()
                            .map(|delcaration| delcaration.identifier_position)
                            .collect(),
                    });
                }

                return Err(ParseError::UndeclaredVariable {
                    identifier: identifier.to_string(),
                    position: self.current_position(),
                });
            }
        }

        let mut children = Self::new_child_buffer();

        info!("Parsing call arguments");

        while !self.allow(Token::RightParenthesis)? {
            info!("Parsing call argument");

            self.parse_expression()?;

            let argument_id = self.syntax_tree.last_node_id();

            children.push(argument_id);

            self.allow(Token::Comma)?;
        }

        let call_value_arguments_node = SyntaxNode {
            kind: SyntaxKind::CallValueArguments,
            span: Span(function_node.span.1, self.previous_span.1),
            children: (
                self.syntax_tree.children.len() as u32,
                children.len() as u32,
            ),
            payload: 0,
        };
        let function_type_node = self
            .resolver
            .get_type_node(TypeId(function_node.payload))
            .ok_or(ParseError::MissingType {
                id: TypeId(function_node.payload),
            })?;
        let TypeNode::Function { return_type, .. } = function_type_node else {
            return Err(ParseError::ExpectedFunction {
                found: function_node.kind,
                position: Position::new(self.syntax_tree.file_index, function_node.span),
            });
        };

        let call_value_arguments_id = self.syntax_tree.push_node(call_value_arguments_node);
        let end = self.previous_span.1;
        let node = SyntaxNode {
            kind: SyntaxKind::CallExpression,
            span: Span(start, end),
            children: (function_node_id.0, call_value_arguments_id.0),
            payload: return_type.0,
        };

        self.syntax_tree.push_node(node);
        self.syntax_tree.children.extend(children);

        Ok(())
    }

    fn parse_grouped_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing grouped expression");

        let start = self.current_span.0;

        self.advance()?;
        self.parse_expression()?;
        self.expect(Token::RightParenthesis)?;

        let end = self.previous_span.1;
        let expression_id = self.syntax_tree.last_node_id();
        let r#type = self
            .syntax_tree
            .get_node(expression_id)
            .map(|node| node.payload)
            .ok_or(ParseError::MissingNode { id: expression_id })?;
        let node = SyntaxNode {
            kind: SyntaxKind::GroupedExpression,
            span: Span(start, end),
            children: (expression_id.0, 0),
            payload: r#type,
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_block_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing block expression");

        let start = self.current_span.0;
        let starting_scope_id = self.current_scope_id;
        self.current_scope_id = self.new_child_scope(ScopeKind::Block);

        let mut children = Self::new_child_buffer();

        self.advance()?;

        while !self.allow(Token::RightCurlyBrace)? {
            if let Err(error) = self.pratt(Precedence::None) {
                self.recover(error);
            } else {
                let child_id = self.syntax_tree.last_node_id();

                if child_id == SyntaxId(0) {
                    break;
                }

                children.push(child_id);
            }
        }

        let first_child = self.syntax_tree.children.len() as u32;
        let child_count = children.len() as u32;
        self.current_scope_id = starting_scope_id;

        if let Some(last_node) = self.syntax_tree.last_node()
            && last_node.kind.is_expression()
            && last_node.payload != TypeId::NONE.0
        {
            let block_node = SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                span: Span(start, self.previous_span.1),
                children: (first_child, child_count),
                payload: starting_scope_id.0,
            };

            self.syntax_tree.push_node(block_node);
            self.syntax_tree.children.extend(children);
        } else {
            let block_node = SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                span: Span(start, self.previous_span.1),
                children: (first_child, child_count),
                payload: starting_scope_id.0,
            };
            let block_node_id = self.syntax_tree.push_node(block_node);

            self.syntax_tree.children.extend(children);

            let expression_statement_node = SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                span: block_node.span,
                children: (block_node_id.0, 0),
                payload: TypeId::NONE.0,
            };

            self.syntax_tree.push_node(expression_statement_node);
        }

        Ok(())
    }

    fn parse_if(&mut self) -> Result<(), ParseError> {
        todo!()
    }

    fn parse_while_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing while expression");

        let start = self.current_span.0;

        self.advance()?;
        self.parse_expression()?;

        let condition_id = self.syntax_tree.last_node_id();
        let condition_node = self
            .syntax_tree
            .get_node(condition_id)
            .ok_or(ParseError::MissingNode { id: condition_id })?;
        let condition_type = TypeId(condition_node.payload);

        if condition_type != TypeId::BOOLEAN {
            let condition_type = self
                .resolver
                .resolve_type(condition_type)
                .unwrap_or(Type::None);

            return Err(ParseError::ExpectedBooleanCondition {
                condition_type,
                condition_position: Position::new(self.syntax_tree.file_index, condition_node.span),
            });
        }

        self.parse_block_expression()?;

        let body_id = self.syntax_tree.last_node_id();
        let end = self.previous_span.1;
        let while_node = SyntaxNode {
            kind: SyntaxKind::WhileExpression,
            span: Span(start, end),
            children: (condition_id.0, body_id.0),
            payload: TypeId::NONE.0,
        };
        let while_node_id = self.syntax_tree.push_node(while_node);
        let expression_statement_node = SyntaxNode {
            kind: SyntaxKind::ExpressionStatement,
            span: while_node.span,
            children: (while_node_id.0, 0),
            payload: TypeId::NONE.0,
        };

        self.syntax_tree.push_node(expression_statement_node);

        Ok(())
    }

    fn parse_break_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing break statement");

        let start = self.current_span.0;

        self.advance()?;
        self.allow(Token::Semicolon)?;

        let end = self.previous_span.1;
        let node = SyntaxNode {
            kind: SyntaxKind::BreakExpression,
            span: Span(start, end),
            children: (0, 0),
            payload: TypeId::NONE.0,
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_array(&mut self) -> Result<(), ParseError> {
        todo!()
    }

    fn parse_return(&mut self) -> Result<(), ParseError> {
        todo!()
    }

    fn parse_path_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing path expression");

        let identifier_span = self.current_span;
        let identifier_text = self.current_source();

        self.advance()?;

        let Some(declaration_id) = self
            .resolver
            .find_declaration_in_scope(identifier_text, self.current_scope_id)
        else {
            if let Some(declarations) = self.resolver.find_declarations(identifier_text) {
                return Err(ParseError::OutOfScopeVariable {
                    position: Position::new(self.syntax_tree.file_index, identifier_span),
                    declaration_positions: declarations
                        .iter()
                        .map(|delcaration| delcaration.identifier_position)
                        .collect(),
                });
            }

            return Err(ParseError::UndeclaredVariable {
                identifier: identifier_text.to_string(),
                position: Position::new(self.syntax_tree.file_index, identifier_span),
            });
        };
        let declaration = self
            .resolver
            .get_declaration(declaration_id)
            .ok_or(ParseError::MissingDeclaration { id: declaration_id })?;
        let node = SyntaxNode {
            kind: SyntaxKind::PathExpression,
            span: identifier_span,
            children: (declaration_id.0, 0),
            payload: declaration.type_id.0,
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_list(&mut self) -> Result<(), ParseError> {
        todo!()
    }

    fn parse_semicolon(&mut self) -> Result<(), ParseError> {
        let start = self.current_span.0;

        self.advance()?;

        let end = self.previous_span.1;
        let Some(last_node) = self.syntax_tree.last_node() else {
            return Err(ParseError::UnexpectedToken {
                actual: self.current_token,
                position: self.current_position(),
            });
        };
        let is_optional = last_node.kind.has_block();

        let node = if is_optional {
            info!("Parsing semicolon statement");

            SyntaxNode {
                kind: SyntaxKind::SemicolonStatement,
                span: Span(start, end),
                children: (is_optional as u32, 0),
                payload: TypeId::NONE.0,
            }
        } else {
            info!("Parsing expression statement");

            let span = Span(last_node.span.0, end);
            let expression_id = self.syntax_tree.last_node_id();

            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                span,
                children: (expression_id.0, 0),
                payload: last_node.payload,
            }
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_str(&mut self) -> Result<(), ParseError> {
        todo!()
    }
}
