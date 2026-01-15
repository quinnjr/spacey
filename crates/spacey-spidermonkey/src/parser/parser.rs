//! The main parser implementation.

use crate::Error;
use crate::ast::*;
use crate::lexer::{Scanner, Token, TokenKind};

/// A recursive descent parser for JavaScript.
pub struct Parser<'a> {
    scanner: Scanner<'a>,
    current: Token,
    previous: Token,
}

impl<'a> Parser<'a> {
    /// Creates a new parser for the given source code.
    pub fn new(source: &'a str) -> Self {
        let mut scanner = Scanner::new(source);
        let current = scanner.next_token();
        Self {
            scanner,
            current,
            previous: Token::new(TokenKind::Eof, crate::lexer::Span::new(0, 0)),
        }
    }

    /// Parses the source code into a Program AST node.
    pub fn parse_program(&mut self) -> Result<Program, Error> {
        let mut body = Vec::new();
        let mut source_type = SourceType::Script;

        while !self.is_at_end() {
            let item = self.parse_module_item()?;
            if matches!(
                item,
                ModuleItem::ImportDeclaration(_) | ModuleItem::ExportDeclaration(_)
            ) {
                source_type = SourceType::Module;
            }
            body.push(item);
        }

        Ok(Program { body, source_type })
    }

    /// Parses a module item (statement, import, or export).
    fn parse_module_item(&mut self) -> Result<ModuleItem, Error> {
        match &self.current.kind {
            TokenKind::Import => {
                self.advance();
                Ok(ModuleItem::ImportDeclaration(
                    self.parse_import_declaration()?,
                ))
            }
            TokenKind::Export => {
                self.advance();
                Ok(ModuleItem::ExportDeclaration(
                    self.parse_export_declaration()?,
                ))
            }
            _ => Ok(ModuleItem::Statement(self.parse_statement()?)),
        }
    }

    fn parse_import_declaration(&mut self) -> Result<ImportDeclaration, Error> {
        let mut specifiers = Vec::new();

        // Check for import * as foo from 'module'
        if self.check(&TokenKind::Star) {
            self.advance();
            if !self.check_contextual("as") {
                return Err(Error::SyntaxError("Expected 'as' after *".into()));
            }
            self.advance();
            let local = self.expect_identifier()?;
            specifiers.push(ImportSpecifier::Namespace(local));
        }
        // Check for import { foo, bar } from 'module'
        else if self.check(&TokenKind::LeftBrace) {
            self.advance();
            while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
                let imported = self.expect_identifier()?;
                let local = if self.check_contextual("as") {
                    self.advance();
                    self.expect_identifier()?
                } else {
                    imported.clone()
                };
                specifiers.push(ImportSpecifier::Named { imported, local });
                if !self.check(&TokenKind::RightBrace) {
                    self.expect(&TokenKind::Comma)?;
                }
            }
            self.expect(&TokenKind::RightBrace)?;
        }
        // Check for import foo from 'module'
        else if let TokenKind::Identifier(_) = &self.current.kind {
            let local = self.expect_identifier()?;
            specifiers.push(ImportSpecifier::Default(local));

            // Check for import foo, { bar } from 'module'
            if self.check(&TokenKind::Comma) {
                self.advance();
                if self.check(&TokenKind::LeftBrace) {
                    self.advance();
                    while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
                        let imported = self.expect_identifier()?;
                        let local = if self.check_contextual("as") {
                            self.advance();
                            self.expect_identifier()?
                        } else {
                            imported.clone()
                        };
                        specifiers.push(ImportSpecifier::Named { imported, local });
                        if !self.check(&TokenKind::RightBrace) {
                            self.expect(&TokenKind::Comma)?;
                        }
                    }
                    self.expect(&TokenKind::RightBrace)?;
                }
            }
        }

        // Expect 'from'
        if !self.check_contextual("from") {
            return Err(Error::SyntaxError("Expected 'from'".into()));
        }
        self.advance();

        // Parse the source string
        let source = if let TokenKind::String(s) = &self.current.kind {
            let s = s.clone();
            self.advance();
            s
        } else {
            return Err(Error::SyntaxError("Expected module specifier".into()));
        };

        self.consume_semicolon()?;

        Ok(ImportDeclaration { specifiers, source })
    }

    fn parse_export_declaration(&mut self) -> Result<ExportDeclaration, Error> {
        // export default
        if self.check(&TokenKind::Default) {
            self.advance();
            let expr = self.parse_assignment_expression()?;
            self.consume_semicolon()?;
            return Ok(ExportDeclaration::Default(Box::new(expr)));
        }

        // export * from 'module'
        if self.check(&TokenKind::Star) {
            self.advance();
            let exported = if self.check_contextual("as") {
                self.advance();
                Some(self.expect_identifier()?)
            } else {
                None
            };
            if !self.check_contextual("from") {
                return Err(Error::SyntaxError("Expected 'from'".into()));
            }
            self.advance();
            let source = if let TokenKind::String(s) = &self.current.kind {
                let s = s.clone();
                self.advance();
                s
            } else {
                return Err(Error::SyntaxError("Expected module specifier".into()));
            };
            self.consume_semicolon()?;
            return Ok(ExportDeclaration::All { source, exported });
        }

        // export { foo, bar }
        if self.check(&TokenKind::LeftBrace) {
            self.advance();
            let mut specifiers = Vec::new();
            while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
                let local = self.expect_identifier()?;
                let exported = if self.check_contextual("as") {
                    self.advance();
                    self.expect_identifier()?
                } else {
                    local.clone()
                };
                specifiers.push(ExportSpecifier { local, exported });
                if !self.check(&TokenKind::RightBrace) {
                    self.expect(&TokenKind::Comma)?;
                }
            }
            self.expect(&TokenKind::RightBrace)?;

            let source = if self.check_contextual("from") {
                self.advance();
                if let TokenKind::String(s) = &self.current.kind {
                    let s = s.clone();
                    self.advance();
                    Some(s)
                } else {
                    return Err(Error::SyntaxError("Expected module specifier".into()));
                }
            } else {
                None
            };

            self.consume_semicolon()?;
            return Ok(ExportDeclaration::Named { specifiers, source });
        }

        // export declaration (var, let, const, function, class)
        let decl = self.parse_statement()?;
        Ok(ExportDeclaration::Declaration(Box::new(decl)))
    }

    /// Parses a single statement.
    pub fn parse_statement(&mut self) -> Result<Statement, Error> {
        // Check for labeled statement (identifier followed by colon)
        if let TokenKind::Identifier(name) = &self.current.kind {
            let label_name = name.clone();
            // Peek ahead to see if this is a label
            let saved_pos = self.scanner.current_pos;
            let saved_current = self.current.clone();
            self.advance();
            if self.check(&TokenKind::Colon) {
                self.advance(); // consume ':'
                let body = Box::new(self.parse_statement()?);
                return Ok(Statement::Labeled(LabeledStatement {
                    label: Identifier { name: label_name },
                    body,
                }));
            } else {
                // Not a label, restore state
                self.scanner.current_pos = saved_pos;
                self.current = saved_current;
            }
        }

        match &self.current.kind {
            TokenKind::Var | TokenKind::Let | TokenKind::Const => self.parse_variable_declaration(),
            TokenKind::Function => self.parse_function_declaration(),
            TokenKind::Class => self.parse_class_declaration(),
            TokenKind::If => self.parse_if_statement(),
            TokenKind::While => self.parse_while_statement(),
            TokenKind::Do => self.parse_do_while_statement(),
            TokenKind::For => self.parse_for_statement(),
            TokenKind::Switch => self.parse_switch_statement(),
            TokenKind::Return => self.parse_return_statement(),
            TokenKind::Break => self.parse_break_statement(),
            TokenKind::Continue => self.parse_continue_statement(),
            TokenKind::Throw => self.parse_throw_statement(),
            TokenKind::Try => self.parse_try_statement(),
            TokenKind::With => self.parse_with_statement(),
            TokenKind::Debugger => {
                self.advance();
                self.consume_semicolon()?;
                Ok(Statement::Debugger)
            }
            TokenKind::LeftBrace => self.parse_block_statement(),
            TokenKind::Semicolon => {
                self.advance();
                Ok(Statement::Empty)
            }
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_variable_declaration(&mut self) -> Result<Statement, Error> {
        let kind = match &self.current.kind {
            TokenKind::Var => VariableKind::Var,
            TokenKind::Let => VariableKind::Let,
            TokenKind::Const => VariableKind::Const,
            _ => return Err(Error::SyntaxError("Expected variable keyword".into())),
        };
        self.advance();

        let declarations = self.parse_variable_declarators()?;
        self.consume_semicolon()?;

        Ok(Statement::VariableDeclaration(VariableDeclaration {
            kind,
            declarations,
        }))
    }

    fn parse_variable_declarators(&mut self) -> Result<Vec<VariableDeclarator>, Error> {
        let mut declarations = Vec::new();

        loop {
            let id = self.parse_binding_pattern()?;
            let init = if self.check(&TokenKind::Equal) {
                self.advance();
                Some(self.parse_assignment_expression()?)
            } else {
                None
            };

            declarations.push(VariableDeclarator { id, init });

            if !self.check(&TokenKind::Comma) {
                break;
            }
            self.advance();
        }

        Ok(declarations)
    }

    fn parse_binding_pattern(&mut self) -> Result<BindingPattern, Error> {
        match &self.current.kind {
            TokenKind::Identifier(name) => {
                let id = Identifier { name: name.clone() };
                self.advance();
                Ok(BindingPattern::Identifier(id))
            }
            TokenKind::LeftBracket => self.parse_array_binding_pattern(),
            TokenKind::LeftBrace => self.parse_object_binding_pattern(),
            _ => Err(Error::SyntaxError(format!(
                "Expected binding pattern, found {:?}",
                self.current.kind
            ))),
        }
    }

    fn parse_array_binding_pattern(&mut self) -> Result<BindingPattern, Error> {
        self.advance(); // consume '['
        let mut elements = Vec::new();
        let mut rest = None;

        while !self.check(&TokenKind::RightBracket) && !self.is_at_end() {
            if self.check(&TokenKind::Comma) {
                // Hole in array pattern
                elements.push(None);
                self.advance();
                continue;
            }

            if self.check(&TokenKind::Ellipsis) {
                self.advance();
                let pattern = self.parse_binding_pattern()?;
                rest = Some(Box::new(pattern));
                break; // Rest must be last
            }

            let pattern = self.parse_binding_pattern()?;
            let default = if self.check(&TokenKind::Equal) {
                self.advance();
                Some(self.parse_assignment_expression()?)
            } else {
                None
            };

            elements.push(Some(ArrayBindingElement { pattern, default }));

            if !self.check(&TokenKind::RightBracket) {
                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
        }

        self.expect(&TokenKind::RightBracket)?;

        Ok(BindingPattern::Array(ArrayBindingPattern {
            elements,
            rest,
        }))
    }

    fn parse_object_binding_pattern(&mut self) -> Result<BindingPattern, Error> {
        self.advance(); // consume '{'
        let mut properties = Vec::new();
        let mut rest = None;

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            if self.check(&TokenKind::Ellipsis) {
                self.advance();
                let id = self.expect_identifier()?;
                rest = Some(id);
                break; // Rest must be last
            }

            // Parse property key
            let key = match &self.current.kind {
                TokenKind::Identifier(name) => {
                    PropertyKey::Identifier(Identifier { name: name.clone() })
                }
                TokenKind::String(s) => PropertyKey::Literal(Literal::String(s.clone())),
                TokenKind::Number(n) => PropertyKey::Literal(Literal::Number(*n)),
                TokenKind::LeftBracket => {
                    self.advance();
                    let expr = self.parse_assignment_expression()?;
                    self.expect(&TokenKind::RightBracket)?;
                    PropertyKey::Computed(Box::new(expr))
                }
                _ => {
                    return Err(Error::SyntaxError("Expected property name".into()));
                }
            };

            // Consume key if identifier/literal
            if !matches!(key, PropertyKey::Computed(_)) {
                self.advance();
            }

            // Check for shorthand vs renamed binding
            let (value, shorthand) = if self.check(&TokenKind::Colon) {
                self.advance();
                (self.parse_binding_pattern()?, false)
            } else {
                // Shorthand: {a} is same as {a: a}
                if let PropertyKey::Identifier(id) = &key {
                    (BindingPattern::Identifier(id.clone()), true)
                } else {
                    return Err(Error::SyntaxError(
                        "Shorthand property must be an identifier".into(),
                    ));
                }
            };

            let default = if self.check(&TokenKind::Equal) {
                self.advance();
                Some(self.parse_assignment_expression()?)
            } else {
                None
            };

            properties.push(BindingProperty {
                key,
                value,
                shorthand,
                default,
            });

            if !self.check(&TokenKind::RightBrace) {
                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
        }

        self.expect(&TokenKind::RightBrace)?;

        Ok(BindingPattern::Object(ObjectBindingPattern {
            properties,
            rest,
        }))
    }

    fn parse_variable_declaration_no_semi(&mut self) -> Result<VariableDeclaration, Error> {
        let kind = match &self.current.kind {
            TokenKind::Var => VariableKind::Var,
            TokenKind::Let => VariableKind::Let,
            TokenKind::Const => VariableKind::Const,
            _ => return Err(Error::SyntaxError("Expected variable keyword".into())),
        };
        self.advance();

        let declarations = self.parse_variable_declarators()?;

        Ok(VariableDeclaration { kind, declarations })
    }

    fn parse_function_declaration(&mut self) -> Result<Statement, Error> {
        self.advance(); // consume 'function'

        let id = self.expect_identifier()?;
        self.expect(&TokenKind::LeftParen)?;

        let params = self.parse_parameters()?;

        self.expect(&TokenKind::RightParen)?;
        self.expect(&TokenKind::LeftBrace)?;

        let body = self.parse_function_body()?;

        self.expect(&TokenKind::RightBrace)?;

        Ok(Statement::FunctionDeclaration(FunctionDeclaration {
            id,
            params,
            body,
            is_async: false,
            is_generator: false,
        }))
    }

    fn parse_class_declaration(&mut self) -> Result<Statement, Error> {
        self.advance(); // consume 'class'

        let id = self.expect_identifier()?;

        // Parse optional extends clause
        let super_class = if self.check(&TokenKind::Extends) {
            self.advance();
            Some(Box::new(self.parse_unary()?))
        } else {
            None
        };

        let body = self.parse_class_body()?;

        Ok(Statement::ClassDeclaration(ClassDeclaration {
            id,
            super_class,
            body,
        }))
    }

    fn parse_class_expression(&mut self) -> Result<Expression, Error> {
        self.advance(); // consume 'class'

        // Optional name
        let id = if let TokenKind::Identifier(name) = &self.current.kind {
            let id = Identifier { name: name.clone() };
            self.advance();
            Some(id)
        } else {
            None
        };

        // Parse optional extends clause
        let super_class = if self.check(&TokenKind::Extends) {
            self.advance();
            Some(Box::new(self.parse_unary()?))
        } else {
            None
        };

        let body = self.parse_class_body()?;

        Ok(Expression::Class(ClassExpression {
            id,
            super_class,
            body,
        }))
    }

    fn parse_class_body(&mut self) -> Result<ClassBody, Error> {
        self.expect(&TokenKind::LeftBrace)?;
        let mut body = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            // Check for static block: static { ... }
            if self.check(&TokenKind::Static) {
                let saved_pos = self.scanner.current_pos;
                let saved_current = self.current.clone();
                self.advance();

                if self.check(&TokenKind::LeftBrace) {
                    // Static block
                    self.advance();
                    let mut statements = Vec::new();
                    while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
                        statements.push(self.parse_statement()?);
                    }
                    self.expect(&TokenKind::RightBrace)?;
                    body.push(ClassElement::StaticBlock(StaticBlock { body: statements }));
                    continue;
                } else {
                    // Not a static block, restore and parse as static member
                    self.scanner.current_pos = saved_pos;
                    self.current = saved_current;
                }
            }

            body.push(self.parse_class_element()?);
        }

        self.expect(&TokenKind::RightBrace)?;

        Ok(ClassBody { body })
    }

    fn parse_class_element(&mut self) -> Result<ClassElement, Error> {
        let is_static = if self.check(&TokenKind::Static) {
            self.advance();
            true
        } else {
            false
        };

        // Check for getter/setter
        let mut method_kind = MethodKind::Method;
        if let TokenKind::Identifier(name) = &self.current.kind
            && (name == "get" || name == "set")
        {
            let kind_name = name.clone();
            let saved_pos = self.scanner.current_pos;
            let saved_current = self.current.clone();
            self.advance();

            // Check if this is actually a method named 'get'/'set' or a getter/setter
            if !self.check(&TokenKind::LeftParen)
                && !self.check(&TokenKind::Equal)
                && !self.check(&TokenKind::Semicolon)
            {
                method_kind = if kind_name == "get" {
                    MethodKind::Get
                } else {
                    MethodKind::Set
                };
            } else {
                // It's a method named 'get' or 'set'
                self.scanner.current_pos = saved_pos;
                self.current = saved_current;
            }
        }

        // Parse key
        let (key, computed) = self.parse_class_element_key()?;

        // Check if this is a method or a field
        if self.check(&TokenKind::LeftParen) {
            // Method
            let kind = if let PropertyKey::Identifier(id) = &key {
                if id.name == "constructor" && !is_static {
                    MethodKind::Constructor
                } else {
                    method_kind
                }
            } else {
                method_kind
            };

            self.expect(&TokenKind::LeftParen)?;
            let params = self.parse_parameters()?;
            self.expect(&TokenKind::RightParen)?;
            self.expect(&TokenKind::LeftBrace)?;
            let body = self.parse_function_body()?;
            self.expect(&TokenKind::RightBrace)?;

            Ok(ClassElement::MethodDefinition(MethodDefinition {
                key,
                value: FunctionExpression {
                    id: None,
                    params,
                    body,
                    is_async: false,
                    is_generator: false,
                },
                kind,
                is_static,
                computed,
            }))
        } else {
            // Field
            let value = if self.check(&TokenKind::Equal) {
                self.advance();
                Some(self.parse_assignment_expression()?)
            } else {
                None
            };

            // Consume optional semicolon
            if self.check(&TokenKind::Semicolon) {
                self.advance();
            }

            Ok(ClassElement::PropertyDefinition(PropertyDefinition {
                key,
                value,
                is_static,
                computed,
            }))
        }
    }

    fn parse_class_element_key(&mut self) -> Result<(PropertyKey, bool), Error> {
        // Handle private identifiers
        if let TokenKind::PrivateIdentifier(name) = &self.current.kind {
            let key = PropertyKey::Identifier(Identifier {
                name: format!("#{}", name),
            });
            self.advance();
            return Ok((key, false));
        }

        // Computed property
        if self.check(&TokenKind::LeftBracket) {
            self.advance();
            let expr = self.parse_assignment_expression()?;
            self.expect(&TokenKind::RightBracket)?;
            return Ok((PropertyKey::Computed(Box::new(expr)), true));
        }

        // Identifier key
        if let TokenKind::Identifier(name) = &self.current.kind {
            let key = PropertyKey::Identifier(Identifier { name: name.clone() });
            self.advance();
            return Ok((key, false));
        }

        // String key
        if let TokenKind::String(s) = &self.current.kind {
            let key = PropertyKey::Literal(Literal::String(s.clone()));
            self.advance();
            return Ok((key, false));
        }

        // Number key
        if let TokenKind::Number(n) = &self.current.kind {
            let key = PropertyKey::Literal(Literal::Number(*n));
            self.advance();
            return Ok((key, false));
        }

        Err(Error::SyntaxError(format!(
            "Expected class element key, found {:?}",
            self.current.kind
        )))
    }

    fn parse_parameters(&mut self) -> Result<Vec<Parameter>, Error> {
        let mut params = Vec::new();

        if !self.check(&TokenKind::RightParen) {
            loop {
                // Check for rest parameter
                if self.check(&TokenKind::Ellipsis) {
                    self.advance();
                    let id = self.expect_identifier()?;
                    params.push(Parameter {
                        pattern: BindingPattern::Rest(Box::new(BindingPattern::Identifier(id))),
                        default: None,
                    });
                    break; // Rest parameter must be last
                }

                let id = self.expect_identifier()?;
                let default = if self.check(&TokenKind::Equal) {
                    self.advance();
                    Some(self.parse_assignment_expression()?)
                } else {
                    None
                };
                params.push(Parameter {
                    pattern: BindingPattern::Identifier(id),
                    default,
                });
                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
        }

        Ok(params)
    }

    fn parse_function_body(&mut self) -> Result<Vec<Statement>, Error> {
        let mut body = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            body.push(self.parse_statement()?);
        }

        Ok(body)
    }

    fn parse_if_statement(&mut self) -> Result<Statement, Error> {
        self.advance(); // consume 'if'
        self.expect(&TokenKind::LeftParen)?;
        let test = self.parse_expression()?;
        self.expect(&TokenKind::RightParen)?;
        let consequent = Box::new(self.parse_statement()?);
        let alternate = if self.check(&TokenKind::Else) {
            self.advance();
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };

        Ok(Statement::If(IfStatement {
            test,
            consequent,
            alternate,
        }))
    }

    fn parse_while_statement(&mut self) -> Result<Statement, Error> {
        self.advance(); // consume 'while'
        self.expect(&TokenKind::LeftParen)?;
        let test = self.parse_expression()?;
        self.expect(&TokenKind::RightParen)?;
        let body = Box::new(self.parse_statement()?);

        Ok(Statement::While(WhileStatement { test, body }))
    }

    fn parse_do_while_statement(&mut self) -> Result<Statement, Error> {
        self.advance(); // consume 'do'
        let body = Box::new(self.parse_statement()?);
        self.expect(&TokenKind::While)?;
        self.expect(&TokenKind::LeftParen)?;
        let test = self.parse_expression()?;
        self.expect(&TokenKind::RightParen)?;
        self.consume_semicolon()?;

        Ok(Statement::DoWhile(DoWhileStatement { test, body }))
    }

    fn parse_for_statement(&mut self) -> Result<Statement, Error> {
        self.advance(); // consume 'for'
        self.expect(&TokenKind::LeftParen)?;

        // Parse init - could be var declaration, expression, or empty
        let init;

        if self.check(&TokenKind::Semicolon) {
            init = None;
        } else if matches!(
            self.current.kind,
            TokenKind::Var | TokenKind::Let | TokenKind::Const
        ) {
            let decl = self.parse_variable_declaration_no_semi()?;
            // Check if this is a for-in or for-of
            if self.check(&TokenKind::In) {
                self.advance(); // consume 'in'
                let right = self.parse_expression()?;
                self.expect(&TokenKind::RightParen)?;
                let body = Box::new(self.parse_statement()?);
                return Ok(Statement::ForIn(ForInStatement {
                    left: ForInLeft::Declaration(Box::new(decl)),
                    right,
                    body,
                }));
            } else if self.check_contextual("of") {
                self.advance(); // consume 'of'
                let right = self.parse_assignment_expression()?;
                self.expect(&TokenKind::RightParen)?;
                let body = Box::new(self.parse_statement()?);
                return Ok(Statement::ForOf(ForOfStatement {
                    left: ForInLeft::Declaration(Box::new(decl)),
                    right,
                    body,
                    is_await: false,
                }));
            }
            init = Some(ForInit::Declaration(Box::new(decl)));
        } else {
            let expr = self.parse_expression()?;
            // Check if this is a for-in or for-of
            if self.check(&TokenKind::In) {
                self.advance(); // consume 'in'
                let right = self.parse_expression()?;
                self.expect(&TokenKind::RightParen)?;
                let body = Box::new(self.parse_statement()?);
                return Ok(Statement::ForIn(ForInStatement {
                    left: ForInLeft::Expression(expr),
                    right,
                    body,
                }));
            } else if self.check_contextual("of") {
                self.advance(); // consume 'of'
                let right = self.parse_assignment_expression()?;
                self.expect(&TokenKind::RightParen)?;
                let body = Box::new(self.parse_statement()?);
                return Ok(Statement::ForOf(ForOfStatement {
                    left: ForInLeft::Expression(expr),
                    right,
                    body,
                    is_await: false,
                }));
            }
            init = Some(ForInit::Expression(expr));
        }

        self.expect(&TokenKind::Semicolon)?;

        // Parse test
        let test = if self.check(&TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.expect(&TokenKind::Semicolon)?;

        // Parse update
        let update = if self.check(&TokenKind::RightParen) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.expect(&TokenKind::RightParen)?;

        let body = Box::new(self.parse_statement()?);

        Ok(Statement::For(ForStatement {
            init,
            test,
            update,
            body,
        }))
    }

    fn parse_switch_statement(&mut self) -> Result<Statement, Error> {
        self.advance(); // consume 'switch'
        self.expect(&TokenKind::LeftParen)?;
        let discriminant = self.parse_expression()?;
        self.expect(&TokenKind::RightParen)?;
        self.expect(&TokenKind::LeftBrace)?;

        let mut cases = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            let test = if self.check(&TokenKind::Case) {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(&TokenKind::Colon)?;
                Some(expr)
            } else if self.check(&TokenKind::Default) {
                self.advance();
                self.expect(&TokenKind::Colon)?;
                None
            } else {
                return Err(Error::SyntaxError("Expected 'case' or 'default'".into()));
            };

            let mut consequent = Vec::new();
            while !self.check(&TokenKind::Case)
                && !self.check(&TokenKind::Default)
                && !self.check(&TokenKind::RightBrace)
                && !self.is_at_end()
            {
                consequent.push(self.parse_statement()?);
            }

            cases.push(SwitchCase { test, consequent });
        }

        self.expect(&TokenKind::RightBrace)?;

        Ok(Statement::Switch(SwitchStatement {
            discriminant,
            cases,
        }))
    }

    fn parse_return_statement(&mut self) -> Result<Statement, Error> {
        self.advance(); // consume 'return'

        // Return value is optional - check for line terminator or semicolon
        let argument = if self.check(&TokenKind::Semicolon)
            || self.check(&TokenKind::RightBrace)
            || self.is_at_end()
        {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.consume_semicolon()?;

        Ok(Statement::Return(ReturnStatement { argument }))
    }

    fn parse_break_statement(&mut self) -> Result<Statement, Error> {
        self.advance(); // consume 'break'

        let label = if let TokenKind::Identifier(name) = &self.current.kind {
            let id = Identifier { name: name.clone() };
            self.advance();
            Some(id)
        } else {
            None
        };
        self.consume_semicolon()?;

        Ok(Statement::Break(BreakStatement { label }))
    }

    fn parse_continue_statement(&mut self) -> Result<Statement, Error> {
        self.advance(); // consume 'continue'

        let label = if let TokenKind::Identifier(name) = &self.current.kind {
            let id = Identifier { name: name.clone() };
            self.advance();
            Some(id)
        } else {
            None
        };
        self.consume_semicolon()?;

        Ok(Statement::Continue(ContinueStatement { label }))
    }

    fn parse_throw_statement(&mut self) -> Result<Statement, Error> {
        self.advance(); // consume 'throw'
        let argument = self.parse_expression()?;
        self.consume_semicolon()?;

        Ok(Statement::Throw(ThrowStatement { argument }))
    }

    fn parse_try_statement(&mut self) -> Result<Statement, Error> {
        self.advance(); // consume 'try'

        self.expect(&TokenKind::LeftBrace)?;
        let block = self.parse_block_body()?;
        self.expect(&TokenKind::RightBrace)?;

        let handler = if self.check(&TokenKind::Catch) {
            self.advance();
            let param = if self.check(&TokenKind::LeftParen) {
                self.advance();
                let id = self.expect_identifier()?;
                self.expect(&TokenKind::RightParen)?;
                Some(id)
            } else {
                None
            };
            self.expect(&TokenKind::LeftBrace)?;
            let body = self.parse_block_body()?;
            self.expect(&TokenKind::RightBrace)?;
            Some(CatchClause {
                param,
                body: BlockStatement { body },
            })
        } else {
            None
        };

        let finalizer = if self.check(&TokenKind::Finally) {
            self.advance();
            self.expect(&TokenKind::LeftBrace)?;
            let body = self.parse_block_body()?;
            self.expect(&TokenKind::RightBrace)?;
            Some(BlockStatement { body })
        } else {
            None
        };

        if handler.is_none() && finalizer.is_none() {
            return Err(Error::SyntaxError(
                "Missing catch or finally after try".into(),
            ));
        }

        Ok(Statement::Try(TryStatement {
            block: BlockStatement { body: block },
            handler,
            finalizer,
        }))
    }

    fn parse_with_statement(&mut self) -> Result<Statement, Error> {
        self.advance(); // consume 'with'
        self.expect(&TokenKind::LeftParen)?;
        let object = self.parse_expression()?;
        self.expect(&TokenKind::RightParen)?;
        let body = Box::new(self.parse_statement()?);

        Ok(Statement::With(WithStatement { object, body }))
    }

    fn parse_block_statement(&mut self) -> Result<Statement, Error> {
        self.advance(); // consume '{'
        let body = self.parse_block_body()?;
        self.expect(&TokenKind::RightBrace)?;

        Ok(Statement::Block(BlockStatement { body }))
    }

    fn parse_block_body(&mut self) -> Result<Vec<Statement>, Error> {
        let mut body = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            body.push(self.parse_statement()?);
        }

        Ok(body)
    }

    fn parse_expression_statement(&mut self) -> Result<Statement, Error> {
        let expression = self.parse_expression()?;
        self.consume_semicolon()?;
        Ok(Statement::Expression(ExpressionStatement { expression }))
    }

    // ==================== Expression Parsing ====================

    /// Parses an expression (including comma operator).
    pub fn parse_expression(&mut self) -> Result<Expression, Error> {
        self.parse_sequence_expression()
    }

    fn parse_sequence_expression(&mut self) -> Result<Expression, Error> {
        let mut expr = self.parse_assignment_expression()?;

        if self.check(&TokenKind::Comma) {
            let mut expressions = vec![expr];
            while self.check(&TokenKind::Comma) {
                self.advance();
                expressions.push(self.parse_assignment_expression()?);
            }
            expr = Expression::Sequence(SequenceExpression { expressions });
        }

        Ok(expr)
    }

    fn parse_assignment_expression(&mut self) -> Result<Expression, Error> {
        // Check for arrow function: identifier => ... or (params) => ...
        if let Some(arrow) = self.try_parse_arrow_function()? {
            return Ok(arrow);
        }

        let expr = self.parse_conditional()?;

        let operator = match &self.current.kind {
            TokenKind::Equal => Some(AssignmentOperator::Assign),
            TokenKind::PlusEqual => Some(AssignmentOperator::AddAssign),
            TokenKind::MinusEqual => Some(AssignmentOperator::SubtractAssign),
            TokenKind::StarEqual => Some(AssignmentOperator::MultiplyAssign),
            TokenKind::SlashEqual => Some(AssignmentOperator::DivideAssign),
            TokenKind::PercentEqual => Some(AssignmentOperator::ModuloAssign),
            TokenKind::StarStarEqual => Some(AssignmentOperator::ExponentAssign),
            TokenKind::LeftShiftEqual => Some(AssignmentOperator::LeftShiftAssign),
            TokenKind::RightShiftEqual => Some(AssignmentOperator::RightShiftAssign),
            TokenKind::UnsignedRightShiftEqual => {
                Some(AssignmentOperator::UnsignedRightShiftAssign)
            }
            TokenKind::AmpersandEqual => Some(AssignmentOperator::BitwiseAndAssign),
            TokenKind::PipeEqual => Some(AssignmentOperator::BitwiseOrAssign),
            TokenKind::CaretEqual => Some(AssignmentOperator::BitwiseXorAssign),
            TokenKind::AmpersandAmpersandEqual => Some(AssignmentOperator::LogicalAndAssign),
            TokenKind::PipePipeEqual => Some(AssignmentOperator::LogicalOrAssign),
            TokenKind::QuestionQuestionEqual => Some(AssignmentOperator::NullishCoalescingAssign),
            _ => None,
        };

        if let Some(op) = operator {
            self.advance();
            let value = self.parse_assignment_expression()?;
            return Ok(Expression::Assignment(AssignmentExpression {
                operator: op,
                left: Box::new(expr),
                right: Box::new(value),
            }));
        }

        Ok(expr)
    }

    /// Try to parse an arrow function. Returns None if this isn't an arrow function.
    fn try_parse_arrow_function(&mut self) -> Result<Option<Expression>, Error> {
        // Case 1: identifier => ...
        if let TokenKind::Identifier(name) = &self.current.kind {
            let param_name = name.clone();
            let saved_pos = self.scanner.current_pos;
            let saved_current = self.current.clone();
            self.advance();

            if self.check(&TokenKind::Arrow) {
                self.advance(); // consume '=>'
                let params = vec![Parameter {
                    pattern: BindingPattern::Identifier(Identifier { name: param_name }),
                    default: None,
                }];
                let body = self.parse_arrow_body()?;
                return Ok(Some(Expression::Arrow(ArrowFunctionExpression {
                    params,
                    body,
                    is_async: false,
                })));
            } else {
                // Not an arrow function, restore state
                self.scanner.current_pos = saved_pos;
                self.current = saved_current;
            }
        }

        // Case 2: () => ... or (params) => ...
        if self.check(&TokenKind::LeftParen) {
            let saved_pos = self.scanner.current_pos;
            let saved_current = self.current.clone();

            // Try to parse as arrow function params
            if let Ok(params) = self.try_parse_arrow_params()
                && self.check(&TokenKind::Arrow)
            {
                self.advance(); // consume '=>'
                let body = self.parse_arrow_body()?;
                return Ok(Some(Expression::Arrow(ArrowFunctionExpression {
                    params,
                    body,
                    is_async: false,
                })));
            }

            // Not an arrow function, restore state
            self.scanner.current_pos = saved_pos;
            self.current = saved_current;
        }

        Ok(None)
    }

    fn try_parse_arrow_params(&mut self) -> Result<Vec<Parameter>, Error> {
        self.advance(); // consume '('
        let mut params = Vec::new();

        if !self.check(&TokenKind::RightParen) {
            loop {
                // Check for rest parameter
                if self.check(&TokenKind::Ellipsis) {
                    self.advance();
                    if let TokenKind::Identifier(name) = &self.current.kind {
                        params.push(Parameter {
                            pattern: BindingPattern::Rest(Box::new(BindingPattern::Identifier(
                                Identifier { name: name.clone() },
                            ))),
                            default: None,
                        });
                        self.advance();
                        break; // Rest must be last
                    } else {
                        return Err(Error::SyntaxError(
                            "Expected parameter name after ...".into(),
                        ));
                    }
                }

                if let TokenKind::Identifier(name) = &self.current.kind {
                    let id = Identifier { name: name.clone() };
                    self.advance();
                    let default = if self.check(&TokenKind::Equal) {
                        self.advance();
                        Some(self.parse_assignment_expression()?)
                    } else {
                        None
                    };
                    params.push(Parameter {
                        pattern: BindingPattern::Identifier(id),
                        default,
                    });
                } else {
                    return Err(Error::SyntaxError("Expected parameter name".into()));
                }

                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
        }

        self.expect(&TokenKind::RightParen)?;
        Ok(params)
    }

    fn parse_arrow_body(&mut self) -> Result<ArrowBody, Error> {
        if self.check(&TokenKind::LeftBrace) {
            // Block body
            self.advance();
            let mut body = Vec::new();
            while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
                body.push(self.parse_statement()?);
            }
            self.expect(&TokenKind::RightBrace)?;
            Ok(ArrowBody::Block(body))
        } else {
            // Expression body
            let expr = self.parse_assignment_expression()?;
            Ok(ArrowBody::Expression(Box::new(expr)))
        }
    }

    fn parse_conditional(&mut self) -> Result<Expression, Error> {
        let expr = self.parse_logical_or()?;

        if self.check(&TokenKind::Question) {
            self.advance();
            let consequent = self.parse_assignment_expression()?;
            self.expect(&TokenKind::Colon)?;
            let alternate = self.parse_assignment_expression()?;
            return Ok(Expression::Conditional(ConditionalExpression {
                test: Box::new(expr),
                consequent: Box::new(consequent),
                alternate: Box::new(alternate),
            }));
        }

        Ok(expr)
    }

    fn parse_logical_or(&mut self) -> Result<Expression, Error> {
        let mut left = self.parse_logical_and()?;

        while self.check(&TokenKind::PipePipe) {
            self.advance();
            let right = self.parse_logical_and()?;
            left = Expression::Binary(BinaryExpression {
                operator: BinaryOperator::LogicalOr,
                left: Box::new(left),
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_logical_and(&mut self) -> Result<Expression, Error> {
        let mut left = self.parse_bitwise_or()?;

        while self.check(&TokenKind::AmpersandAmpersand) {
            self.advance();
            let right = self.parse_bitwise_or()?;
            left = Expression::Binary(BinaryExpression {
                operator: BinaryOperator::LogicalAnd,
                left: Box::new(left),
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_bitwise_or(&mut self) -> Result<Expression, Error> {
        let mut left = self.parse_bitwise_xor()?;

        while self.check(&TokenKind::Pipe) {
            self.advance();
            let right = self.parse_bitwise_xor()?;
            left = Expression::Binary(BinaryExpression {
                operator: BinaryOperator::BitwiseOr,
                left: Box::new(left),
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_bitwise_xor(&mut self) -> Result<Expression, Error> {
        let mut left = self.parse_bitwise_and()?;

        while self.check(&TokenKind::Caret) {
            self.advance();
            let right = self.parse_bitwise_and()?;
            left = Expression::Binary(BinaryExpression {
                operator: BinaryOperator::BitwiseXor,
                left: Box::new(left),
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_bitwise_and(&mut self) -> Result<Expression, Error> {
        let mut left = self.parse_equality()?;

        while self.check(&TokenKind::Ampersand) {
            self.advance();
            let right = self.parse_equality()?;
            left = Expression::Binary(BinaryExpression {
                operator: BinaryOperator::BitwiseAnd,
                left: Box::new(left),
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expression, Error> {
        let mut left = self.parse_relational()?;

        loop {
            let operator = match &self.current.kind {
                TokenKind::EqualEqual => BinaryOperator::Equal,
                TokenKind::NotEqual => BinaryOperator::NotEqual,
                TokenKind::StrictEqual => BinaryOperator::StrictEqual,
                TokenKind::StrictNotEqual => BinaryOperator::StrictNotEqual,
                _ => break,
            };
            self.advance();
            let right = self.parse_relational()?;
            left = Expression::Binary(BinaryExpression {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_relational(&mut self) -> Result<Expression, Error> {
        let mut left = self.parse_shift()?;

        loop {
            let operator = match &self.current.kind {
                TokenKind::LessThan => BinaryOperator::LessThan,
                TokenKind::LessThanEqual => BinaryOperator::LessThanEqual,
                TokenKind::GreaterThan => BinaryOperator::GreaterThan,
                TokenKind::GreaterThanEqual => BinaryOperator::GreaterThanEqual,
                TokenKind::In => BinaryOperator::In,
                TokenKind::Instanceof => BinaryOperator::InstanceOf,
                _ => break,
            };
            self.advance();
            let right = self.parse_shift()?;
            left = Expression::Binary(BinaryExpression {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_shift(&mut self) -> Result<Expression, Error> {
        let mut left = self.parse_additive()?;

        loop {
            let operator = match &self.current.kind {
                TokenKind::LeftShift => BinaryOperator::LeftShift,
                TokenKind::RightShift => BinaryOperator::RightShift,
                TokenKind::UnsignedRightShift => BinaryOperator::UnsignedRightShift,
                _ => break,
            };
            self.advance();
            let right = self.parse_additive()?;
            left = Expression::Binary(BinaryExpression {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expression, Error> {
        let mut left = self.parse_multiplicative()?;

        loop {
            let operator = match &self.current.kind {
                TokenKind::Plus => BinaryOperator::Add,
                TokenKind::Minus => BinaryOperator::Subtract,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expression::Binary(BinaryExpression {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expression, Error> {
        let mut left = self.parse_unary()?;

        loop {
            let operator = match &self.current.kind {
                TokenKind::Star => BinaryOperator::Multiply,
                TokenKind::Slash => BinaryOperator::Divide,
                TokenKind::Percent => BinaryOperator::Modulo,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expression::Binary(BinaryExpression {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expression, Error> {
        // Prefix update operators
        if self.check(&TokenKind::PlusPlus) {
            self.advance();
            let argument = self.parse_unary()?;
            return Ok(Expression::Update(UpdateExpression {
                operator: UpdateOperator::Increment,
                argument: Box::new(argument),
                prefix: true,
            }));
        }

        if self.check(&TokenKind::MinusMinus) {
            self.advance();
            let argument = self.parse_unary()?;
            return Ok(Expression::Update(UpdateExpression {
                operator: UpdateOperator::Decrement,
                argument: Box::new(argument),
                prefix: true,
            }));
        }

        // Unary operators
        let operator = match &self.current.kind {
            TokenKind::Bang => Some(UnaryOperator::LogicalNot),
            TokenKind::Tilde => Some(UnaryOperator::BitwiseNot),
            TokenKind::Minus => Some(UnaryOperator::Minus),
            TokenKind::Plus => Some(UnaryOperator::Plus),
            TokenKind::Typeof => Some(UnaryOperator::Typeof),
            TokenKind::Void => Some(UnaryOperator::Void),
            TokenKind::Delete => Some(UnaryOperator::Delete),
            _ => None,
        };

        if let Some(op) = operator {
            self.advance();
            let argument = self.parse_unary()?;
            return Ok(Expression::Unary(UnaryExpression {
                operator: op,
                argument: Box::new(argument),
            }));
        }

        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expression, Error> {
        let mut expr = self.parse_call_or_member()?;

        // Postfix update operators
        if self.check(&TokenKind::PlusPlus) {
            self.advance();
            expr = Expression::Update(UpdateExpression {
                operator: UpdateOperator::Increment,
                argument: Box::new(expr),
                prefix: false,
            });
        } else if self.check(&TokenKind::MinusMinus) {
            self.advance();
            expr = Expression::Update(UpdateExpression {
                operator: UpdateOperator::Decrement,
                argument: Box::new(expr),
                prefix: false,
            });
        }

        Ok(expr)
    }

    fn parse_call_or_member(&mut self) -> Result<Expression, Error> {
        let mut expr = if self.check(&TokenKind::New) {
            self.parse_new_expression()?
        } else {
            self.parse_primary()?
        };

        loop {
            if self.check(&TokenKind::LeftParen) {
                self.advance();
                let arguments = self.parse_arguments()?;
                self.expect(&TokenKind::RightParen)?;
                expr = Expression::Call(CallExpression {
                    callee: Box::new(expr),
                    arguments,
                    optional: false,
                });
            } else if self.check(&TokenKind::QuestionDot) {
                // Optional chaining
                self.advance();
                if self.check(&TokenKind::LeftParen) {
                    // Optional call: obj?.()
                    self.advance();
                    let arguments = self.parse_arguments()?;
                    self.expect(&TokenKind::RightParen)?;
                    expr = Expression::Call(CallExpression {
                        callee: Box::new(expr),
                        arguments,
                        optional: true,
                    });
                } else if self.check(&TokenKind::LeftBracket) {
                    // Optional computed: obj?.[prop]
                    self.advance();
                    let property = self.parse_expression()?;
                    self.expect(&TokenKind::RightBracket)?;
                    expr = Expression::Member(MemberExpression {
                        object: Box::new(expr),
                        property: MemberProperty::Expression(Box::new(property)),
                        computed: true,
                        optional: true,
                    });
                } else {
                    // Optional property: obj?.prop
                    let property = self.expect_identifier()?;
                    expr = Expression::Member(MemberExpression {
                        object: Box::new(expr),
                        property: MemberProperty::Identifier(property),
                        computed: false,
                        optional: true,
                    });
                }
            } else if self.check(&TokenKind::Dot) {
                self.advance();
                let property = self.expect_identifier()?;
                expr = Expression::Member(MemberExpression {
                    object: Box::new(expr),
                    property: MemberProperty::Identifier(property),
                    computed: false,
                    optional: false,
                });
            } else if self.check(&TokenKind::LeftBracket) {
                self.advance();
                let property = self.parse_expression()?;
                self.expect(&TokenKind::RightBracket)?;
                expr = Expression::Member(MemberExpression {
                    object: Box::new(expr),
                    property: MemberProperty::Expression(Box::new(property)),
                    computed: true,
                    optional: false,
                });
            } else if let TokenKind::NoSubstitutionTemplate(value) = &self.current.kind {
                // Tagged template with no substitutions
                let value = value.clone();
                self.advance();
                expr = Expression::TaggedTemplate(TaggedTemplateExpression {
                    tag: Box::new(expr),
                    quasi: TemplateLiteral {
                        quasis: vec![TemplateElement {
                            cooked: value.clone(),
                            raw: value,
                            tail: true,
                        }],
                        expressions: vec![],
                    },
                });
            } else if let TokenKind::TemplateHead(value) = &self.current.kind {
                // Tagged template with substitutions
                let value = value.clone();
                let quasi = self.parse_template_literal_inner(value)?;
                expr = Expression::TaggedTemplate(TaggedTemplateExpression {
                    tag: Box::new(expr),
                    quasi,
                });
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_new_expression(&mut self) -> Result<Expression, Error> {
        self.advance(); // consume 'new'

        let callee = if self.check(&TokenKind::New) {
            // Nested new: new new Foo()
            self.parse_new_expression()?
        } else {
            self.parse_primary()?
        };

        // Parse optional member access after the callee
        let mut callee = callee;
        loop {
            if self.check(&TokenKind::Dot) {
                self.advance();
                let property = self.expect_identifier()?;
                callee = Expression::Member(MemberExpression {
                    object: Box::new(callee),
                    property: MemberProperty::Identifier(property),
                    computed: false,
                    optional: false,
                });
            } else if self.check(&TokenKind::LeftBracket) {
                self.advance();
                let property = self.parse_expression()?;
                self.expect(&TokenKind::RightBracket)?;
                callee = Expression::Member(MemberExpression {
                    object: Box::new(callee),
                    property: MemberProperty::Expression(Box::new(property)),
                    computed: true,
                    optional: false,
                });
            } else {
                break;
            }
        }

        // Parse optional arguments
        let arguments = if self.check(&TokenKind::LeftParen) {
            self.advance();
            let args = self.parse_arguments()?;
            self.expect(&TokenKind::RightParen)?;
            args
        } else {
            Vec::new()
        };

        Ok(Expression::New(NewExpression {
            callee: Box::new(callee),
            arguments,
        }))
    }

    fn parse_arguments(&mut self) -> Result<Vec<Argument>, Error> {
        let mut args = Vec::new();

        if !self.check(&TokenKind::RightParen) {
            loop {
                if self.check(&TokenKind::Ellipsis) {
                    self.advance();
                    args.push(Argument::Spread(self.parse_assignment_expression()?));
                } else {
                    args.push(Argument::Expression(self.parse_assignment_expression()?));
                }
                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
        }

        Ok(args)
    }

    fn parse_primary(&mut self) -> Result<Expression, Error> {
        match &self.current.kind {
            TokenKind::Number(n) => {
                let value = *n;
                self.advance();
                Ok(Expression::Literal(Literal::Number(value)))
            }
            TokenKind::String(s) => {
                let value = s.clone();
                self.advance();
                Ok(Expression::Literal(Literal::String(value)))
            }
            TokenKind::True => {
                self.advance();
                Ok(Expression::Literal(Literal::Boolean(true)))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expression::Literal(Literal::Boolean(false)))
            }
            TokenKind::Null => {
                self.advance();
                Ok(Expression::Literal(Literal::Null))
            }
            TokenKind::Identifier(name) => {
                let id = Identifier { name: name.clone() };
                self.advance();
                Ok(Expression::Identifier(id))
            }
            TokenKind::This => {
                self.advance();
                Ok(Expression::This)
            }
            TokenKind::Super => {
                self.advance();
                Ok(Expression::Super)
            }
            TokenKind::Function => self.parse_function_expression(),
            TokenKind::Class => self.parse_class_expression(),
            TokenKind::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(&TokenKind::RightParen)?;
                Ok(expr)
            }
            TokenKind::LeftBracket => self.parse_array_literal(),
            TokenKind::LeftBrace => self.parse_object_literal(),
            TokenKind::RegExp { pattern, flags } => {
                let pattern = pattern.clone();
                let flags = flags.clone();
                self.advance();
                Ok(Expression::Literal(Literal::RegExp { pattern, flags }))
            }
            TokenKind::NoSubstitutionTemplate(value) => {
                let value = value.clone();
                self.advance();
                Ok(Expression::TemplateLiteral(TemplateLiteral {
                    quasis: vec![TemplateElement {
                        cooked: value.clone(),
                        raw: value,
                        tail: true,
                    }],
                    expressions: vec![],
                }))
            }
            TokenKind::TemplateHead(value) => self.parse_template_literal(value.clone()),
            _ => Err(Error::SyntaxError(format!(
                "Unexpected token: {:?}",
                self.current.kind
            ))),
        }
    }

    fn parse_template_literal(&mut self, head_value: String) -> Result<Expression, Error> {
        let template = self.parse_template_literal_inner(head_value)?;
        Ok(Expression::TemplateLiteral(template))
    }

    fn parse_template_literal_inner(
        &mut self,
        head_value: String,
    ) -> Result<TemplateLiteral, Error> {
        self.advance(); // consume the TemplateHead token

        let mut quasis = vec![TemplateElement {
            cooked: head_value.clone(),
            raw: head_value,
            tail: false,
        }];
        let mut expressions = Vec::new();

        loop {
            // Parse the expression inside ${}
            let expr = self.parse_expression()?;
            expressions.push(expr);

            // The parser expects a RightBrace here, but the scanner consumed it
            // We need to get the template continuation
            // First, we should be at a position where we can scan the template continuation

            // Expect the closing brace of the template expression
            self.expect(&TokenKind::RightBrace)?;

            // Scan the template continuation
            let continuation = self.scanner.scan_template_continuation();
            self.current = continuation;

            match &self.current.kind {
                TokenKind::TemplateMiddle(value) => {
                    quasis.push(TemplateElement {
                        cooked: value.clone(),
                        raw: value.clone(),
                        tail: false,
                    });
                    self.advance();
                }
                TokenKind::TemplateTail(value) => {
                    quasis.push(TemplateElement {
                        cooked: value.clone(),
                        raw: value.clone(),
                        tail: true,
                    });
                    self.advance();
                    break;
                }
                _ => {
                    return Err(Error::SyntaxError("Expected template continuation".into()));
                }
            }
        }

        Ok(TemplateLiteral {
            quasis,
            expressions,
        })
    }

    fn parse_function_expression(&mut self) -> Result<Expression, Error> {
        self.advance(); // consume 'function'

        // Optional name
        let id = if let TokenKind::Identifier(name) = &self.current.kind {
            let id = Identifier { name: name.clone() };
            self.advance();
            Some(id)
        } else {
            None
        };

        self.expect(&TokenKind::LeftParen)?;
        let params = self.parse_parameters()?;
        self.expect(&TokenKind::RightParen)?;
        self.expect(&TokenKind::LeftBrace)?;
        let body = self.parse_function_body()?;
        self.expect(&TokenKind::RightBrace)?;

        Ok(Expression::Function(FunctionExpression {
            id,
            params,
            body,
            is_async: false,
            is_generator: false,
        }))
    }

    fn parse_array_literal(&mut self) -> Result<Expression, Error> {
        self.advance(); // consume '['
        let mut elements = Vec::new();

        while !self.check(&TokenKind::RightBracket) && !self.is_at_end() {
            if self.check(&TokenKind::Comma) {
                elements.push(None); // Hole in array
            } else if self.check(&TokenKind::Ellipsis) {
                self.advance();
                elements.push(Some(ArrayElement::Spread(
                    self.parse_assignment_expression()?,
                )));
            } else {
                elements.push(Some(ArrayElement::Expression(
                    self.parse_assignment_expression()?,
                )));
            }

            if !self.check(&TokenKind::RightBracket) {
                self.expect(&TokenKind::Comma)?;
            }
        }

        self.expect(&TokenKind::RightBracket)?;

        Ok(Expression::Array(ArrayExpression { elements }))
    }

    fn parse_object_literal(&mut self) -> Result<Expression, Error> {
        self.advance(); // consume '{'
        let mut properties = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            if self.check(&TokenKind::Ellipsis) {
                self.advance();
                properties.push(ObjectProperty::Spread(self.parse_assignment_expression()?));
            } else {
                let property = self.parse_property()?;
                properties.push(ObjectProperty::Property(property));
            }

            if !self.check(&TokenKind::RightBrace) {
                self.expect(&TokenKind::Comma)?;
            }
        }

        self.expect(&TokenKind::RightBrace)?;

        Ok(Expression::Object(ObjectExpression { properties }))
    }

    fn parse_property(&mut self) -> Result<Property, Error> {
        // Property key can be identifier, string, or number
        let key = match &self.current.kind {
            TokenKind::Identifier(name) => {
                let key = PropertyKey::Identifier(Identifier { name: name.clone() });
                self.advance();
                key
            }
            TokenKind::String(s) => {
                let key = PropertyKey::Literal(Literal::String(s.clone()));
                self.advance();
                key
            }
            TokenKind::Number(n) => {
                let key = PropertyKey::Literal(Literal::Number(*n));
                self.advance();
                key
            }
            TokenKind::LeftBracket => {
                // Computed property
                self.advance();
                let expr = self.parse_assignment_expression()?;
                self.expect(&TokenKind::RightBracket)?;
                PropertyKey::Computed(Box::new(expr))
            }
            _ => return Err(Error::SyntaxError("Expected property name".into())),
        };

        self.expect(&TokenKind::Colon)?;
        let value = self.parse_assignment_expression()?;

        Ok(Property {
            key,
            value,
            shorthand: false,
        })
    }

    // ==================== Helper Methods ====================

    fn advance(&mut self) {
        self.previous = std::mem::replace(&mut self.current, self.scanner.next_token());
    }

    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.current.kind) == std::mem::discriminant(kind)
    }

    fn check_contextual(&self, name: &str) -> bool {
        if let TokenKind::Identifier(id) = &self.current.kind {
            id == name
        } else {
            false
        }
    }

    fn expect(&mut self, kind: &TokenKind) -> Result<(), Error> {
        if self.check(kind) {
            self.advance();
            Ok(())
        } else {
            Err(Error::SyntaxError(format!(
                "Expected {:?}, found {:?}",
                kind, self.current.kind
            )))
        }
    }

    fn expect_identifier(&mut self) -> Result<Identifier, Error> {
        if let TokenKind::Identifier(name) = &self.current.kind {
            let id = Identifier { name: name.clone() };
            self.advance();
            Ok(id)
        } else {
            Err(Error::SyntaxError(format!(
                "Expected identifier, found {:?}",
                self.current.kind
            )))
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.current.kind, TokenKind::Eof)
    }

    fn consume_semicolon(&mut self) -> Result<(), Error> {
        if self.check(&TokenKind::Semicolon) {
            self.advance();
            Ok(())
        } else if self.check(&TokenKind::RightBrace) || self.is_at_end() {
            // Automatic semicolon insertion
            Ok(())
        } else {
            // For now, accept missing semicolons (ASI)
            // A proper implementation would check for newlines
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to extract statement from ModuleItem
    fn get_stmt(item: &ModuleItem) -> &Statement {
        match item {
            ModuleItem::Statement(s) => s,
            _ => panic!("Expected statement"),
        }
    }

    #[test]
    fn test_parse_variable_declaration() {
        let mut parser = Parser::new("let x = 42;");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
    }

    #[test]
    fn test_parse_function_declaration() {
        let mut parser = Parser::new("function add(a, b) { return a + b; }");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
    }

    #[test]
    fn test_parse_binary_expression() {
        let mut parser = Parser::new("1 + 2 * 3;");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
    }

    #[test]
    fn test_parse_conditional_expression() {
        let mut parser = Parser::new("x ? y : z;");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
    }

    #[test]
    fn test_parse_do_while() {
        let mut parser = Parser::new("do { x++; } while (x < 10);");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
        assert!(matches!(get_stmt(&program.body[0]), Statement::DoWhile(_)));
    }

    #[test]
    fn test_parse_switch() {
        let mut parser = Parser::new("switch (x) { case 1: break; default: return; }");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
        assert!(matches!(get_stmt(&program.body[0]), Statement::Switch(_)));
    }

    #[test]
    fn test_parse_try_catch() {
        let mut parser = Parser::new("try { foo(); } catch (e) { bar(); }");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
        assert!(matches!(get_stmt(&program.body[0]), Statement::Try(_)));
    }

    #[test]
    fn test_parse_for_in() {
        let mut parser = Parser::new("for (var x in obj) { foo(); }");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
        assert!(matches!(get_stmt(&program.body[0]), Statement::ForIn(_)));
    }

    #[test]
    fn test_parse_function_expression() {
        let mut parser = Parser::new("var f = function(x) { return x; };");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
    }

    #[test]
    fn test_parse_new_expression() {
        let mut parser = Parser::new("new Foo(1, 2);");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
    }

    #[test]
    fn test_parse_update_expressions() {
        let mut parser = Parser::new("x++; ++y; z--;");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 3);
    }

    #[test]
    fn test_parse_labeled_statement() {
        let mut parser = Parser::new("outer: for (;;) { break outer; }");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
        assert!(matches!(get_stmt(&program.body[0]), Statement::Labeled(_)));
    }

    #[test]
    fn test_parse_compound_assignment() {
        let mut parser = Parser::new("x += 1; y *= 2;");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 2);
    }

    #[test]
    fn test_parse_bitwise_operators() {
        let mut parser = Parser::new("a & b | c ^ d;");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
    }

    #[test]
    fn test_parse_in_operator() {
        let mut parser = Parser::new("'x' in obj;");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
    }

    #[test]
    fn test_parse_instanceof() {
        let mut parser = Parser::new("x instanceof Array;");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
    }

    #[test]
    fn test_parse_arrow_function_single_param() {
        let mut parser = Parser::new("const fn = x => x + 1;");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
        if let Statement::VariableDeclaration(decl) = get_stmt(&program.body[0]) {
            assert_eq!(decl.kind, VariableKind::Const);
            assert!(decl.declarations[0].init.is_some());
            if let Some(Expression::Arrow(arrow)) = &decl.declarations[0].init {
                assert_eq!(arrow.params.len(), 1);
                if let BindingPattern::Identifier(id) = &arrow.params[0].pattern {
                    assert_eq!(id.name, "x");
                } else {
                    panic!("Expected identifier pattern");
                }
                assert!(!arrow.is_async);
            } else {
                panic!("Expected arrow function expression");
            }
        } else {
            panic!("Expected variable declaration");
        }
    }

    #[test]
    fn test_parse_arrow_function_multiple_params() {
        let mut parser = Parser::new("const add = (a, b) => a + b;");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
        if let Statement::VariableDeclaration(decl) = get_stmt(&program.body[0]) {
            if let Some(Expression::Arrow(arrow)) = &decl.declarations[0].init {
                assert_eq!(arrow.params.len(), 2);
                if let BindingPattern::Identifier(id) = &arrow.params[0].pattern {
                    assert_eq!(id.name, "a");
                } else {
                    panic!("Expected identifier pattern");
                }
                if let BindingPattern::Identifier(id) = &arrow.params[1].pattern {
                    assert_eq!(id.name, "b");
                } else {
                    panic!("Expected identifier pattern");
                }
            } else {
                panic!("Expected arrow function expression");
            }
        } else {
            panic!("Expected variable declaration");
        }
    }

    #[test]
    fn test_parse_arrow_function_no_params() {
        let mut parser = Parser::new("const getVal = () => 42;");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
        if let Statement::VariableDeclaration(decl) = get_stmt(&program.body[0]) {
            if let Some(Expression::Arrow(arrow)) = &decl.declarations[0].init {
                assert_eq!(arrow.params.len(), 0);
            } else {
                panic!("Expected arrow function expression");
            }
        } else {
            panic!("Expected variable declaration");
        }
    }

    #[test]
    fn test_parse_arrow_function_block_body() {
        let mut parser = Parser::new("const fn = (x) => { return x * 2; };");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
        if let Statement::VariableDeclaration(decl) = get_stmt(&program.body[0]) {
            if let Some(Expression::Arrow(arrow)) = &decl.declarations[0].init {
                match &arrow.body {
                    ArrowBody::Block(stmts) => assert!(!stmts.is_empty()),
                    ArrowBody::Expression(_) => panic!("Expected block body"),
                }
            } else {
                panic!("Expected arrow function expression");
            }
        } else {
            panic!("Expected variable declaration");
        }
    }

    #[test]
    fn test_parse_let_const() {
        let mut parser = Parser::new("let x = 1; const y = 2;");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 2);
        if let Statement::VariableDeclaration(decl) = get_stmt(&program.body[0]) {
            assert_eq!(decl.kind, VariableKind::Let);
        }
        if let Statement::VariableDeclaration(decl) = get_stmt(&program.body[1]) {
            assert_eq!(decl.kind, VariableKind::Const);
        }
    }

    #[test]
    fn test_parse_template_literal_no_substitution() {
        let mut parser = Parser::new("const s = `hello world`;");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
        if let Statement::VariableDeclaration(decl) = get_stmt(&program.body[0]) {
            if let Some(Expression::TemplateLiteral(template)) = &decl.declarations[0].init {
                assert_eq!(template.quasis.len(), 1);
                assert_eq!(template.quasis[0].cooked, "hello world");
                assert!(template.expressions.is_empty());
            } else {
                panic!("Expected template literal");
            }
        } else {
            panic!("Expected variable declaration");
        }
    }

    #[test]
    fn test_parse_default_parameters() {
        let mut parser = Parser::new("function foo(a = 1, b = 2) {}");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
        if let Statement::FunctionDeclaration(func) = get_stmt(&program.body[0]) {
            assert_eq!(func.params.len(), 2);
            assert!(func.params[0].default.is_some());
            assert!(func.params[1].default.is_some());
        } else {
            panic!("Expected function declaration");
        }
    }

    #[test]
    fn test_parse_rest_parameter() {
        let mut parser = Parser::new("function foo(a, ...rest) {}");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
        if let Statement::FunctionDeclaration(func) = get_stmt(&program.body[0]) {
            assert_eq!(func.params.len(), 2);
            if let BindingPattern::Rest(_) = &func.params[1].pattern {
                // Success
            } else {
                panic!("Expected rest pattern");
            }
        } else {
            panic!("Expected function declaration");
        }
    }

    #[test]
    fn test_parse_spread_array() {
        let mut parser = Parser::new("const arr = [...a, ...b];");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
        if let Statement::VariableDeclaration(decl) = get_stmt(&program.body[0]) {
            if let Some(Expression::Array(arr)) = &decl.declarations[0].init {
                assert_eq!(arr.elements.len(), 2);
                // Both should be spread elements
                for elem in &arr.elements {
                    if let Some(ArrayElement::Spread(_)) = elem {
                        // Success
                    } else {
                        panic!("Expected spread element");
                    }
                }
            } else {
                panic!("Expected array expression");
            }
        } else {
            panic!("Expected variable declaration");
        }
    }

    #[test]
    fn test_parse_spread_call() {
        let mut parser = Parser::new("foo(...args);");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
        if let Statement::Expression(expr) = get_stmt(&program.body[0]) {
            if let Expression::Call(call) = &expr.expression {
                assert_eq!(call.arguments.len(), 1);
                if let Argument::Spread(_) = &call.arguments[0] {
                    // Success
                } else {
                    panic!("Expected spread argument");
                }
            } else {
                panic!("Expected call expression");
            }
        } else {
            panic!("Expected expression statement");
        }
    }

    #[test]
    fn test_parse_spread_object() {
        let mut parser = Parser::new("const obj = {...a, x: 1};");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
        if let Statement::VariableDeclaration(decl) = get_stmt(&program.body[0]) {
            if let Some(Expression::Object(obj)) = &decl.declarations[0].init {
                assert_eq!(obj.properties.len(), 2);
                if let ObjectProperty::Spread(_) = &obj.properties[0] {
                    // Success
                } else {
                    panic!("Expected spread property");
                }
            } else {
                panic!("Expected object expression");
            }
        } else {
            panic!("Expected variable declaration");
        }
    }

    #[test]
    fn test_parse_array_destructuring() {
        let mut parser = Parser::new("const [a, b, c] = arr;");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
        if let Statement::VariableDeclaration(decl) = get_stmt(&program.body[0]) {
            if let BindingPattern::Array(arr) = &decl.declarations[0].id {
                assert_eq!(arr.elements.len(), 3);
            } else {
                panic!("Expected array binding pattern");
            }
        } else {
            panic!("Expected variable declaration");
        }
    }

    #[test]
    fn test_parse_object_destructuring() {
        let mut parser = Parser::new("const {x, y} = obj;");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
        if let Statement::VariableDeclaration(decl) = get_stmt(&program.body[0]) {
            if let BindingPattern::Object(obj) = &decl.declarations[0].id {
                assert_eq!(obj.properties.len(), 2);
            } else {
                panic!("Expected object binding pattern");
            }
        } else {
            panic!("Expected variable declaration");
        }
    }

    #[test]
    fn test_parse_destructuring_with_defaults() {
        let mut parser = Parser::new("const [a = 1, b = 2] = arr;");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
        if let Statement::VariableDeclaration(decl) = get_stmt(&program.body[0]) {
            if let BindingPattern::Array(arr) = &decl.declarations[0].id {
                assert!(arr.elements[0].as_ref().unwrap().default.is_some());
                assert!(arr.elements[1].as_ref().unwrap().default.is_some());
            } else {
                panic!("Expected array binding pattern");
            }
        } else {
            panic!("Expected variable declaration");
        }
    }

    #[test]
    fn test_parse_nested_destructuring() {
        let mut parser = Parser::new("const {a: {b}} = obj;");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
        if let Statement::VariableDeclaration(decl) = get_stmt(&program.body[0]) {
            if let BindingPattern::Object(obj) = &decl.declarations[0].id {
                assert_eq!(obj.properties.len(), 1);
                if let BindingPattern::Object(_) = &obj.properties[0].value {
                    // Success - nested object pattern
                } else {
                    panic!("Expected nested object binding pattern");
                }
            } else {
                panic!("Expected object binding pattern");
            }
        } else {
            panic!("Expected variable declaration");
        }
    }

    #[test]
    fn test_parse_class_declaration() {
        let mut parser = Parser::new("class Foo { constructor() {} }");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
        if let Statement::ClassDeclaration(class) = get_stmt(&program.body[0]) {
            assert_eq!(class.id.name, "Foo");
            assert!(class.super_class.is_none());
            assert_eq!(class.body.body.len(), 1);
        } else {
            panic!("Expected class declaration");
        }
    }

    #[test]
    fn test_parse_class_with_extends() {
        let mut parser = Parser::new("class Bar extends Foo { }");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
        if let Statement::ClassDeclaration(class) = get_stmt(&program.body[0]) {
            assert_eq!(class.id.name, "Bar");
            assert!(class.super_class.is_some());
        } else {
            panic!("Expected class declaration");
        }
    }

    #[test]
    fn test_parse_class_methods() {
        let mut parser = Parser::new("class Foo { method() {} static staticMethod() {} }");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
        if let Statement::ClassDeclaration(class) = get_stmt(&program.body[0]) {
            assert_eq!(class.body.body.len(), 2);
            if let ClassElement::MethodDefinition(m) = &class.body.body[0] {
                assert!(!m.is_static);
                assert_eq!(m.kind, MethodKind::Method);
            } else {
                panic!("Expected method definition");
            }
            if let ClassElement::MethodDefinition(m) = &class.body.body[1] {
                assert!(m.is_static);
            } else {
                panic!("Expected static method definition");
            }
        } else {
            panic!("Expected class declaration");
        }
    }

    #[test]
    fn test_parse_class_fields() {
        let mut parser = Parser::new("class Foo { x = 1; static y = 2; }");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
        if let Statement::ClassDeclaration(class) = get_stmt(&program.body[0]) {
            assert_eq!(class.body.body.len(), 2);
            if let ClassElement::PropertyDefinition(p) = &class.body.body[0] {
                assert!(!p.is_static);
                assert!(p.value.is_some());
            } else {
                panic!("Expected property definition");
            }
            if let ClassElement::PropertyDefinition(p) = &class.body.body[1] {
                assert!(p.is_static);
            } else {
                panic!("Expected static property definition");
            }
        } else {
            panic!("Expected class declaration");
        }
    }

    #[test]
    fn test_parse_class_getters_setters() {
        let mut parser =
            Parser::new("class Foo { get x() { return this._x; } set x(v) { this._x = v; } }");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
        if let Statement::ClassDeclaration(class) = get_stmt(&program.body[0]) {
            assert_eq!(class.body.body.len(), 2);
            if let ClassElement::MethodDefinition(m) = &class.body.body[0] {
                assert_eq!(m.kind, MethodKind::Get);
            } else {
                panic!("Expected getter");
            }
            if let ClassElement::MethodDefinition(m) = &class.body.body[1] {
                assert_eq!(m.kind, MethodKind::Set);
            } else {
                panic!("Expected setter");
            }
        } else {
            panic!("Expected class declaration");
        }
    }
}
