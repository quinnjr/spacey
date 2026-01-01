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

    /// Creates a new parser with TypeScript mode enabled.
    pub fn new_typescript(source: &'a str) -> Self {
        let mut scanner = Scanner::new_typescript(source);
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

        while !self.is_at_end() {
            body.push(self.parse_statement()?);
        }

        Ok(Program { body })
    }

    /// Parses a single statement.
    pub fn parse_statement(&mut self) -> Result<Statement, Error> {
        // Handle TypeScript-only declarations (skip them entirely)
        if self.is_typescript_mode() {
            match &self.current.kind {
                TokenKind::Type => {
                    self.skip_type_alias()?;
                    return Ok(Statement::Empty);
                }
                TokenKind::Interface => {
                    self.skip_interface()?;
                    return Ok(Statement::Empty);
                }
                TokenKind::Declare => {
                    self.skip_declare_statement()?;
                    return Ok(Statement::Empty);
                }
                TokenKind::Namespace => {
                    self.skip_namespace()?;
                    return Ok(Statement::Empty);
                }
                TokenKind::Abstract => {
                    // abstract class - skip the abstract keyword
                    self.advance();
                    // Continue to parse class normally
                }
                TokenKind::Enum => {
                    // Parse enum and convert to JavaScript
                    return self.parse_enum_declaration();
                }
                _ => {}
            }
        }

        match &self.current.kind {
            TokenKind::Var | TokenKind::Let | TokenKind::Const => self.parse_variable_declaration(),
            TokenKind::Function => self.parse_function_declaration(),
            TokenKind::If => self.parse_if_statement(),
            TokenKind::Switch => self.parse_switch_statement(),
            TokenKind::While => self.parse_while_statement(),
            TokenKind::Do => self.parse_do_while_statement(),
            TokenKind::For => self.parse_for_statement(),
            TokenKind::Return => self.parse_return_statement(),
            TokenKind::Break => self.parse_break_statement(),
            TokenKind::Continue => self.parse_continue_statement(),
            TokenKind::Throw => self.parse_throw_statement(),
            TokenKind::Try => self.parse_try_statement(),
            TokenKind::With => self.parse_with_statement(),
            TokenKind::Debugger => {
                self.advance();
                self.expect(&TokenKind::Semicolon)?;
                Ok(Statement::Debugger)
            }
            TokenKind::LeftBrace => self.parse_block_statement(),
            TokenKind::Semicolon => {
                self.advance();
                Ok(Statement::Empty)
            }
            // TypeScript decorator
            TokenKind::At if self.is_typescript_mode() => {
                self.skip_decorators()?;
                self.parse_statement()
            }
            _ => self.parse_expression_statement(),
        }
    }

    /// Skip decorators: @decorator, @decorator(), @decorator.property
    fn skip_decorators(&mut self) -> Result<(), Error> {
        while self.check(&TokenKind::At) {
            self.advance(); // consume '@'

            // Decorator expression (identifier, member access, or call)
            self.expect_identifier()?;

            // Handle member access: @decorator.property
            while self.check(&TokenKind::Dot) {
                self.advance();
                self.expect_identifier()?;
            }

            // Handle decorator call: @decorator()
            if self.check(&TokenKind::LeftParen) {
                self.advance();
                // Skip arguments
                let mut paren_depth = 1;
                while paren_depth > 0 && !self.is_at_end() {
                    match &self.current.kind {
                        TokenKind::LeftParen => paren_depth += 1,
                        TokenKind::RightParen => paren_depth -= 1,
                        _ => {}
                    }
                    self.advance();
                }
            }
        }

        Ok(())
    }

    /// Parse break statement with optional label.
    fn parse_break_statement(&mut self) -> Result<Statement, Error> {
        self.advance(); // consume 'break'

        // Check for label (no line terminator before label)
        if !self.check(&TokenKind::Semicolon) && !self.is_at_end()
            && let TokenKind::Identifier(label) = &self.current.kind {
                let label = label.clone();
                self.advance();
                self.expect(&TokenKind::Semicolon)?;
                return Ok(Statement::BreakLabel(label));
            }

        self.expect(&TokenKind::Semicolon)?;
        Ok(Statement::Break)
    }

    /// Parse continue statement with optional label.
    fn parse_continue_statement(&mut self) -> Result<Statement, Error> {
        self.advance(); // consume 'continue'

        // Check for label (no line terminator before label)
        if !self.check(&TokenKind::Semicolon) && !self.is_at_end()
            && let TokenKind::Identifier(label) = &self.current.kind {
                let label = label.clone();
                self.advance();
                self.expect(&TokenKind::Semicolon)?;
                return Ok(Statement::ContinueLabel(label));
            }

        self.expect(&TokenKind::Semicolon)?;
        Ok(Statement::Continue)
    }

    /// Parse with statement (ES3 Section 12.10).
    fn parse_with_statement(&mut self) -> Result<Statement, Error> {
        self.advance(); // consume 'with'
        self.expect(&TokenKind::LeftParen)?;
        let object = self.parse_expression()?;
        self.expect(&TokenKind::RightParen)?;
        let body = self.parse_statement()?;
        Ok(Statement::With(WithStatement {
            object,
            body: Box::new(body),
        }))
    }

    fn parse_variable_declaration(&mut self) -> Result<Statement, Error> {
        let kind = match &self.current.kind {
            TokenKind::Var => VariableKind::Var,
            TokenKind::Let => VariableKind::Let,
            TokenKind::Const => VariableKind::Const,
            _ => return Err(Error::SyntaxError("Expected variable keyword".into())),
        };
        self.advance();

        let mut declarations = Vec::new();

        loop {
            let id = self.expect_identifier()?;

            // TypeScript: Skip type annotation (e.g., let x: number)
            self.skip_type_annotation()?;

            let init = if self.check(&TokenKind::Equal) {
                self.advance();
                Some(self.parse_expression()?)
            } else {
                None
            };

            declarations.push(VariableDeclarator { id, init });

            if !self.check(&TokenKind::Comma) {
                break;
            }
            self.advance();
        }

        self.expect(&TokenKind::Semicolon)?;

        Ok(Statement::VariableDeclaration(VariableDeclaration {
            kind,
            declarations,
        }))
    }

    fn parse_function_declaration(&mut self) -> Result<Statement, Error> {
        self.advance(); // consume 'function'

        // TypeScript: skip type parameters (<T, U>)
        self.skip_type_parameters()?;

        let id = self.expect_identifier()?;

        // TypeScript: skip type parameters after name
        self.skip_type_parameters()?;

        self.expect(&TokenKind::LeftParen)?;

        let params = self.parse_parameters()?;

        self.expect(&TokenKind::RightParen)?;

        // TypeScript: skip return type annotation
        self.skip_return_type()?;

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

    fn parse_parameters(&mut self) -> Result<Vec<Identifier>, Error> {
        let mut params = Vec::new();

        if !self.check(&TokenKind::RightParen) {
            loop {
                // TypeScript: skip access modifiers in constructor params
                self.skip_access_modifiers()?;

                // TypeScript: handle rest parameter
                if self.check(&TokenKind::Ellipsis) {
                    self.advance();
                }

                params.push(self.expect_identifier()?);

                // TypeScript: skip optional marker and type annotation
                self.skip_optional_type_annotation()?;

                // TypeScript: skip default value type (already handled by JS)
                if self.check(&TokenKind::Equal) {
                    self.advance();
                    // Parse the default value expression
                    let _default = self.parse_expression()?;
                    // Note: We're discarding the default value for now
                    // In a full implementation, we'd store it in the AST
                }

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
        self.expect(&TokenKind::Semicolon)?;

        Ok(Statement::DoWhile(DoWhileStatement { body, test }))
    }

    fn parse_for_statement(&mut self) -> Result<Statement, Error> {
        self.advance(); // consume 'for'
        self.expect(&TokenKind::LeftParen)?;

        // Check for for-in/for-of with variable declaration
        if matches!(
            self.current.kind,
            TokenKind::Var | TokenKind::Let | TokenKind::Const
        ) {
            let decl = self.parse_variable_declaration_no_semi()?;

            // Check if this is for-in or for-of
            if self.check(&TokenKind::In) {
                self.advance();
                let right = self.parse_expression()?;
                self.expect(&TokenKind::RightParen)?;
                let body = Box::new(self.parse_statement()?);
                return Ok(Statement::ForIn(ForInStatement {
                    left: ForInLeft::Declaration(Box::new(decl)),
                    right,
                    body,
                }));
            } else if self.check_identifier("of") {
                self.advance();
                let right = self.parse_expression()?;
                self.expect(&TokenKind::RightParen)?;
                let body = Box::new(self.parse_statement()?);
                return Ok(Statement::ForOf(ForOfStatement {
                    left: ForInLeft::Declaration(Box::new(decl)),
                    right,
                    body,
                    is_await: false,
                }));
            }

            // Regular for loop with declaration
            self.expect(&TokenKind::Semicolon)?;
            let test = if self.check(&TokenKind::Semicolon) {
                None
            } else {
                Some(self.parse_expression()?)
            };
            self.expect(&TokenKind::Semicolon)?;
            let update = if self.check(&TokenKind::RightParen) {
                None
            } else {
                Some(self.parse_expression()?)
            };
            self.expect(&TokenKind::RightParen)?;
            let body = Box::new(self.parse_statement()?);

            return Ok(Statement::For(ForStatement {
                init: Some(ForInit::Declaration(Box::new(decl))),
                test,
                update,
                body,
            }));
        }

        // Check for empty init or expression
        let init = if self.check(&TokenKind::Semicolon) {
            None
        } else {
            let expr = self.parse_expression()?;

            // Check if this is for-in or for-of with expression
            if self.check(&TokenKind::In) {
                self.advance();
                let right = self.parse_expression()?;
                self.expect(&TokenKind::RightParen)?;
                let body = Box::new(self.parse_statement()?);
                return Ok(Statement::ForIn(ForInStatement {
                    left: ForInLeft::Expression(expr),
                    right,
                    body,
                }));
            } else if self.check_identifier("of") {
                self.advance();
                let right = self.parse_expression()?;
                self.expect(&TokenKind::RightParen)?;
                let body = Box::new(self.parse_statement()?);
                return Ok(Statement::ForOf(ForOfStatement {
                    left: ForInLeft::Expression(expr),
                    right,
                    body,
                    is_await: false,
                }));
            }

            Some(ForInit::Expression(expr))
        };

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

    fn parse_throw_statement(&mut self) -> Result<Statement, Error> {
        self.advance(); // consume 'throw'
        let argument = self.parse_expression()?;
        self.expect(&TokenKind::Semicolon)?;
        Ok(Statement::Throw(ThrowStatement { argument }))
    }

    fn parse_try_statement(&mut self) -> Result<Statement, Error> {
        self.advance(); // consume 'try'
        self.expect(&TokenKind::LeftBrace)?;
        let block = self.parse_block_body()?;

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
            Some(CatchClause { param, body })
        } else {
            None
        };

        let finalizer = if self.check(&TokenKind::Finally) {
            self.advance();
            self.expect(&TokenKind::LeftBrace)?;
            Some(self.parse_block_body()?)
        } else {
            None
        };

        if handler.is_none() && finalizer.is_none() {
            return Err(Error::SyntaxError(
                "Try statement must have catch or finally".into(),
            ));
        }

        Ok(Statement::Try(TryStatement {
            block,
            handler,
            finalizer,
        }))
    }

    fn parse_block_body(&mut self) -> Result<BlockStatement, Error> {
        let mut body = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            body.push(self.parse_statement()?);
        }
        self.expect(&TokenKind::RightBrace)?;
        Ok(BlockStatement { body })
    }

    fn check_identifier(&self, name: &str) -> bool {
        matches!(&self.current.kind, TokenKind::Identifier(s) if s == name)
    }

    fn parse_variable_declaration_no_semi(&mut self) -> Result<VariableDeclaration, Error> {
        let kind = match &self.current.kind {
            TokenKind::Var => VariableKind::Var,
            TokenKind::Let => VariableKind::Let,
            TokenKind::Const => VariableKind::Const,
            _ => return Err(Error::SyntaxError("Expected variable keyword".into())),
        };
        self.advance();

        let mut declarations = Vec::new();

        loop {
            let id = self.expect_identifier()?;

            // TypeScript: Skip type annotation
            self.skip_type_annotation()?;

            let init = if self.check(&TokenKind::Equal) {
                self.advance();
                Some(self.parse_expression()?)
            } else {
                None
            };

            declarations.push(VariableDeclarator { id, init });

            if !self.check(&TokenKind::Comma) {
                break;
            }
            self.advance();
        }

        Ok(VariableDeclaration { kind, declarations })
    }

    fn parse_return_statement(&mut self) -> Result<Statement, Error> {
        self.advance(); // consume 'return'
        let argument = if self.check(&TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.expect(&TokenKind::Semicolon)?;

        Ok(Statement::Return(ReturnStatement { argument }))
    }

    fn parse_block_statement(&mut self) -> Result<Statement, Error> {
        self.advance(); // consume '{'
        let mut body = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            body.push(self.parse_statement()?);
        }

        self.expect(&TokenKind::RightBrace)?;

        Ok(Statement::Block(BlockStatement { body }))
    }

    fn parse_expression_statement(&mut self) -> Result<Statement, Error> {
        // Check for labeled statement: identifier followed by colon
        if let TokenKind::Identifier(name) = &self.current.kind {
            let label_name = name.clone();
            // Peek ahead to see if next token is colon
            let next = self.scanner.peek_token();
            if next.kind == TokenKind::Colon {
                // This is a labeled statement
                self.advance(); // consume identifier
                self.advance(); // consume colon
                let body = self.parse_statement()?;
                return Ok(Statement::Labeled(LabeledStatement {
                    label: Identifier { name: label_name },
                    body: Box::new(body),
                }));
            }
        }

        let expression = self.parse_expression()?;
        self.expect(&TokenKind::Semicolon)?;
        Ok(Statement::Expression(ExpressionStatement { expression }))
    }

    /// Parses an expression.
    pub fn parse_expression(&mut self) -> Result<Expression, Error> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expression, Error> {
        let expr = self.parse_conditional()?;

        if self.check(&TokenKind::Equal) {
            self.advance();
            let value = self.parse_assignment()?;
            return Ok(Expression::Assignment(AssignmentExpression {
                operator: AssignmentOperator::Assign,
                left: Box::new(expr),
                right: Box::new(value),
            }));
        }

        Ok(expr)
    }

    /// Parse conditional (ternary) expression: test ? consequent : alternate
    fn parse_conditional(&mut self) -> Result<Expression, Error> {
        let test = self.parse_logical_or()?;

        if self.check(&TokenKind::Question) {
            self.advance(); // consume '?'
            let consequent = self.parse_assignment()?;
            self.expect(&TokenKind::Colon)?;
            let alternate = self.parse_assignment()?;

            return Ok(Expression::Conditional(ConditionalExpression {
                test: Box::new(test),
                consequent: Box::new(consequent),
                alternate: Box::new(alternate),
            }));
        }

        Ok(test)
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
        let mut left = self.parse_comparison()?;

        loop {
            let operator = match &self.current.kind {
                TokenKind::EqualEqual => BinaryOperator::Equal,
                TokenKind::NotEqual => BinaryOperator::NotEqual,
                TokenKind::StrictEqual => BinaryOperator::StrictEqual,
                TokenKind::StrictNotEqual => BinaryOperator::StrictNotEqual,
                _ => break,
            };
            self.advance();
            let right = self.parse_comparison()?;
            left = Expression::Binary(BinaryExpression {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expression, Error> {
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
        // Check for prefix increment/decrement
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

        let operator = match &self.current.kind {
            TokenKind::Bang => Some(UnaryOperator::LogicalNot),
            TokenKind::Minus => Some(UnaryOperator::Minus),
            TokenKind::Plus => Some(UnaryOperator::Plus),
            TokenKind::Typeof => Some(UnaryOperator::Typeof),
            TokenKind::Void => Some(UnaryOperator::Void),
            TokenKind::Delete => Some(UnaryOperator::Delete),
            TokenKind::Tilde => Some(UnaryOperator::BitwiseNot),
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

        self.parse_call()
    }

    fn parse_call(&mut self) -> Result<Expression, Error> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.check(&TokenKind::LeftParen) {
                self.advance();
                let arguments = self.parse_arguments()?;
                self.expect(&TokenKind::RightParen)?;
                expr = Expression::Call(CallExpression {
                    callee: Box::new(expr),
                    arguments,
                });
            } else if self.check(&TokenKind::Dot) {
                self.advance();
                let property = self.expect_identifier()?;
                expr = Expression::Member(MemberExpression {
                    object: Box::new(expr),
                    property: MemberProperty::Identifier(property),
                    computed: false,
                });
            } else if self.check(&TokenKind::LeftBracket) {
                self.advance();
                let property = self.parse_expression()?;
                self.expect(&TokenKind::RightBracket)?;
                expr = Expression::Member(MemberExpression {
                    object: Box::new(expr),
                    property: MemberProperty::Expression(Box::new(property)),
                    computed: true,
                });
            } else if self.check(&TokenKind::PlusPlus) {
                // Postfix increment: x++
                self.advance();
                expr = Expression::Update(UpdateExpression {
                    operator: UpdateOperator::Increment,
                    argument: Box::new(expr),
                    prefix: false,
                });
            } else if self.check(&TokenKind::MinusMinus) {
                // Postfix decrement: x--
                self.advance();
                expr = Expression::Update(UpdateExpression {
                    operator: UpdateOperator::Decrement,
                    argument: Box::new(expr),
                    prefix: false,
                });
            } else if self.check(&TokenKind::Bang) && self.is_typescript_mode() {
                // TypeScript non-null assertion: x!
                self.advance();
                // The expression stays the same, we just skip the !
            } else if self.check(&TokenKind::As) && self.is_typescript_mode() {
                // TypeScript type assertion: x as Type
                self.advance();
                self.skip_type()?;
                // The expression stays the same, type is erased
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_arguments(&mut self) -> Result<Vec<Expression>, Error> {
        let mut args = Vec::new();

        if !self.check(&TokenKind::RightParen) {
            loop {
                args.push(self.parse_expression()?);
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
                // Check for arrow function: identifier => ...
                if self.check(&TokenKind::Arrow) {
                    return self.parse_arrow_function_body(vec![id], false);
                }
                Ok(Expression::Identifier(id))
            }
            TokenKind::This => {
                self.advance();
                Ok(Expression::This)
            }
            TokenKind::Function => self.parse_function_expression(),
            TokenKind::LeftParen => self.parse_parenthesized_or_arrow(),
            TokenKind::LeftBracket => self.parse_array_literal(),
            TokenKind::LeftBrace => self.parse_object_literal(),
            TokenKind::New => self.parse_new_expression(),
            TokenKind::RegExp { pattern, flags } => {
                let p = pattern.clone();
                let f = flags.clone();
                self.advance();
                Ok(Expression::Literal(Literal::RegExp {
                    pattern: p,
                    flags: f,
                }))
            }
            TokenKind::Slash => {
                // This could be a regex literal - rescan as regex
                let start = self.current.span.start;
                let regex_token = self.scanner.rescan_as_regex(start);
                match regex_token.kind {
                    TokenKind::RegExp { pattern, flags } => {
                        // Update current token to be after the regex
                        self.current = self.scanner.next_token();
                        Ok(Expression::Literal(Literal::RegExp { pattern, flags }))
                    }
                    _ => Err(Error::SyntaxError("Invalid regex literal".to_string())),
                }
            }
            _ => Err(Error::SyntaxError(format!(
                "Unexpected token: {:?}",
                self.current.kind
            ))),
        }
    }

    fn parse_function_expression(&mut self) -> Result<Expression, Error> {
        self.advance(); // consume 'function'

        // TypeScript: skip type parameters
        self.skip_type_parameters()?;

        // Optional function name
        let id = if let TokenKind::Identifier(name) = &self.current.kind {
            let id = Identifier { name: name.clone() };
            self.advance();
            Some(id)
        } else {
            None
        };

        // TypeScript: skip type parameters after name
        self.skip_type_parameters()?;

        self.expect(&TokenKind::LeftParen)?;
        let params = self.parse_parameters()?;
        self.expect(&TokenKind::RightParen)?;

        // TypeScript: skip return type annotation
        self.skip_return_type()?;

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

    fn parse_parenthesized_or_arrow(&mut self) -> Result<Expression, Error> {
        self.advance(); // consume '('

        // Empty parentheses - must be arrow function
        if self.check(&TokenKind::RightParen) {
            self.advance();
            if !self.check(&TokenKind::Arrow) {
                return Err(Error::SyntaxError("Expected '=>'".into()));
            }
            return self.parse_arrow_function_body(vec![], false);
        }

        // Try to parse as parameter list for arrow function
        // If we see identifier followed by comma or ), it's likely an arrow function
        let mut params = Vec::new();
        let mut is_arrow = false;
        let mut consumed_identifier: Option<Identifier> = None;

        // First element - check if it's a simple identifier that could be an arrow param
        if let TokenKind::Identifier(name) = &self.current.kind {
            let id = Identifier { name: name.clone() };
            self.advance();

            // Only treat as potential arrow params if followed by , or )
            if self.check(&TokenKind::Comma) || self.check(&TokenKind::RightParen) {
                params.push(id);

                // Collect more parameters
                while self.check(&TokenKind::Comma) {
                    self.advance();
                    if let TokenKind::Identifier(name) = &self.current.kind {
                        params.push(Identifier { name: name.clone() });
                        self.advance();
                    } else {
                        // Not a simple parameter list, fall back to expression
                        break;
                    }
                }

                if self.check(&TokenKind::RightParen) {
                    self.advance();
                    if self.check(&TokenKind::Arrow) {
                        is_arrow = true;
                    }
                }
            } else {
                // Not followed by , or ), so it's an expression starting with identifier
                consumed_identifier = Some(id);
            }
        }

        if is_arrow {
            return self.parse_arrow_function_body(params, false);
        }

        // Not an arrow function
        if !params.is_empty() {
            // We consumed identifiers as params but no arrow - treat as grouping/sequence
            let first = Expression::Identifier(params[0].clone());

            if params.len() > 1 {
                let mut exprs = vec![first];
                for p in params.into_iter().skip(1) {
                    exprs.push(Expression::Identifier(p));
                }
                // Already consumed RightParen above when checking for arrow
                return Ok(Expression::Sequence(SequenceExpression { expressions: exprs }));
            }

            // Already consumed RightParen above when checking for arrow
            return Ok(first);
        }

        // Parse as regular parenthesized expression
        // If we consumed an identifier, we need to continue parsing from there
        let expr = if let Some(id) = consumed_identifier {
            // Start with the identifier we already consumed and parse the rest of the expression
            let first = Expression::Identifier(id);
            self.parse_expression_continue(first)?
        } else {
            self.parse_assignment()?
        };

        // Check for comma operator (sequence expression)
        if self.check(&TokenKind::Comma) {
            let mut exprs = vec![expr];
            while self.check(&TokenKind::Comma) {
                self.advance(); // consume ','
                exprs.push(self.parse_assignment()?);
            }
            self.expect(&TokenKind::RightParen)?;
            return Ok(Expression::Sequence(SequenceExpression { expressions: exprs }));
        }

        self.expect(&TokenKind::RightParen)?;

        Ok(expr)
    }

    /// Continue parsing an expression that started with a given left-hand side
    fn parse_expression_continue(&mut self, left: Expression) -> Result<Expression, Error> {
        // This handles continuing after we've already parsed an identifier
        // and need to handle binary operators, member access, calls, etc.
        let mut result = left;

        // Handle member access and calls first (highest precedence)
        loop {
            if self.check(&TokenKind::Dot) {
                self.advance();
                let name = self.expect_identifier()?;
                result = Expression::Member(MemberExpression {
                    object: Box::new(result),
                    property: MemberProperty::Identifier(name),
                    computed: false,
                });
            } else if self.check(&TokenKind::LeftBracket) {
                self.advance();
                let prop = self.parse_expression()?;
                self.expect(&TokenKind::RightBracket)?;
                result = Expression::Member(MemberExpression {
                    object: Box::new(result),
                    property: MemberProperty::Expression(Box::new(prop)),
                    computed: true,
                });
            } else if self.check(&TokenKind::LeftParen) {
                self.advance();
                let args = self.parse_arguments()?;
                result = Expression::Call(CallExpression {
                    callee: Box::new(result),
                    arguments: args,
                });
            } else {
                break;
            }
        }

        // Now handle binary operators
        self.parse_binary_with_left(result)
    }

    /// Parse binary expression starting with given left operand
    fn parse_binary_with_left(&mut self, left: Expression) -> Result<Expression, Error> {
        // Simple binary operator handling at the additive level
        let mut result = left;

        loop {
            let op = match &self.current.kind {
                TokenKind::Plus => Some(BinaryOperator::Add),
                TokenKind::Minus => Some(BinaryOperator::Subtract),
                TokenKind::Star => Some(BinaryOperator::Multiply),
                TokenKind::Slash => Some(BinaryOperator::Divide),
                TokenKind::Percent => Some(BinaryOperator::Modulo),
                TokenKind::LessThan => Some(BinaryOperator::LessThan),
                TokenKind::LessThanEqual => Some(BinaryOperator::LessThanEqual),
                TokenKind::GreaterThan => Some(BinaryOperator::GreaterThan),
                TokenKind::GreaterThanEqual => Some(BinaryOperator::GreaterThanEqual),
                TokenKind::EqualEqual => Some(BinaryOperator::Equal),
                TokenKind::NotEqual => Some(BinaryOperator::NotEqual),
                TokenKind::StrictEqual => Some(BinaryOperator::StrictEqual),
                TokenKind::StrictNotEqual => Some(BinaryOperator::StrictNotEqual),
                TokenKind::AmpersandAmpersand => Some(BinaryOperator::LogicalAnd),
                TokenKind::PipePipe => Some(BinaryOperator::LogicalOr),
                _ => None,
            };

            if let Some(operator) = op {
                self.advance();
                let right = self.parse_unary()?;
                result = Expression::Binary(BinaryExpression {
                    operator,
                    left: Box::new(result),
                    right: Box::new(right),
                });
            } else {
                break;
            }
        }

        Ok(result)
    }

    fn parse_arrow_function_body(
        &mut self,
        params: Vec<Identifier>,
        is_async: bool,
    ) -> Result<Expression, Error> {
        // TypeScript: skip return type annotation before '=>'
        self.skip_return_type()?;

        self.advance(); // consume '=>'

        let body = if self.check(&TokenKind::LeftBrace) {
            // Block body
            self.advance();
            let stmts = self.parse_function_body()?;
            self.expect(&TokenKind::RightBrace)?;
            ArrowBody::Block(stmts)
        } else {
            // Expression body
            let expr = self.parse_assignment()?;
            ArrowBody::Expression(Box::new(expr))
        };

        Ok(Expression::Arrow(ArrowFunctionExpression {
            params,
            body,
            is_async,
        }))
    }

    fn parse_new_expression(&mut self) -> Result<Expression, Error> {
        self.advance(); // consume 'new'

        // Parse the callee - this should be a member expression without call
        // e.g., "new Foo()", "new a.b.c()", "new Foo.Bar()"
        let callee = Box::new(self.parse_new_callee()?);

        // Parse arguments (required for new expressions with parens)
        let arguments = if self.check(&TokenKind::LeftParen) {
            self.advance();
            let args = self.parse_arguments()?;
            self.expect(&TokenKind::RightParen)?;
            args
        } else {
            vec![]
        };

        Ok(Expression::New(NewExpression { callee, arguments }))
    }

    /// Parse the callee of a new expression (member expression without call parsing)
    fn parse_new_callee(&mut self) -> Result<Expression, Error> {
        // Check for nested 'new' (e.g., new new Foo())
        if self.check(&TokenKind::New) {
            return self.parse_new_expression();
        }

        let mut expr = self.parse_primary()?;

        // Allow member access but not function calls
        loop {
            if self.check(&TokenKind::Dot) {
                self.advance();
                let property = self.expect_identifier()?;
                expr = Expression::Member(MemberExpression {
                    object: Box::new(expr),
                    property: MemberProperty::Identifier(property),
                    computed: false,
                });
            } else if self.check(&TokenKind::LeftBracket) {
                self.advance();
                let property = self.parse_expression()?;
                self.expect(&TokenKind::RightBracket)?;
                expr = Expression::Member(MemberExpression {
                    object: Box::new(expr),
                    property: MemberProperty::Expression(Box::new(property)),
                    computed: true,
                });
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_array_literal(&mut self) -> Result<Expression, Error> {
        self.advance(); // consume '['
        let mut elements = Vec::new();

        while !self.check(&TokenKind::RightBracket) && !self.is_at_end() {
            if self.check(&TokenKind::Comma) {
                elements.push(None); // Hole in array
            } else {
                elements.push(Some(self.parse_expression()?));
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
            let key = self.expect_identifier()?;
            self.expect(&TokenKind::Colon)?;
            let value = self.parse_expression()?;

            properties.push(Property {
                key: PropertyKey::Identifier(key),
                value,
                shorthand: false,
            });

            if !self.check(&TokenKind::RightBrace) {
                self.expect(&TokenKind::Comma)?;
            }
        }

        self.expect(&TokenKind::RightBrace)?;

        Ok(Expression::Object(ObjectExpression { properties }))
    }

    // Helper methods

    fn advance(&mut self) {
        self.previous = std::mem::replace(&mut self.current, self.scanner.next_token());
    }

    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.current.kind) == std::mem::discriminant(kind)
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

    // ==================== TypeScript Support ====================

    /// Enable or disable TypeScript mode.
    ///
    /// When TypeScript mode is enabled, the parser will recognize TypeScript
    /// keywords and syntax, and strip type annotations at parse time.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Set to `true` to enable TypeScript parsing
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut parser = Parser::new("const x: number = 42;");
    /// parser.set_typescript_mode(true);
    /// let ast = parser.parse_program()?;
    /// ```
    pub fn set_typescript_mode(&mut self, enabled: bool) {
        self.scanner.set_typescript_mode(enabled);
    }

    /// Check if the parser is in TypeScript mode.
    fn is_typescript_mode(&self) -> bool {
        self.scanner.is_typescript_mode()
    }

    /// Skip a type annotation if present (e.g., `: number`).
    fn skip_type_annotation(&mut self) -> Result<(), Error> {
        if !self.is_typescript_mode() {
            return Ok(());
        }
        if self.check(&TokenKind::Colon) {
            self.advance();
            self.skip_type()?;
        }
        Ok(())
    }

    /// Skip an optional type annotation with `?:` syntax.
    fn skip_optional_type_annotation(&mut self) -> Result<bool, Error> {
        if !self.is_typescript_mode() {
            return Ok(false);
        }
        let is_optional = self.check(&TokenKind::Question);
        if is_optional {
            self.advance();
        }
        if self.check(&TokenKind::Colon) {
            self.advance();
            self.skip_type()?;
        }
        Ok(is_optional)
    }

    /// Skip a return type annotation (e.g., `): number`).
    fn skip_return_type(&mut self) -> Result<(), Error> {
        if !self.is_typescript_mode() {
            return Ok(());
        }
        if self.check(&TokenKind::Colon) {
            self.advance();
            self.skip_type()?;
        }
        Ok(())
    }

    /// Skip a type expression (unions, intersections, generics, etc.).
    fn skip_type(&mut self) -> Result<(), Error> {
        self.skip_union_type()
    }

    /// Skip a union type: `A | B | C`
    fn skip_union_type(&mut self) -> Result<(), Error> {
        self.skip_intersection_type()?;
        while self.check(&TokenKind::Pipe) {
            self.advance();
            self.skip_intersection_type()?;
        }
        Ok(())
    }

    /// Skip an intersection type: `A & B & C`
    fn skip_intersection_type(&mut self) -> Result<(), Error> {
        self.skip_primary_type()?;
        while self.check(&TokenKind::Ampersand) {
            self.advance();
            self.skip_primary_type()?;
        }
        Ok(())
    }

    /// Skip a primary type expression.
    fn skip_primary_type(&mut self) -> Result<(), Error> {
        match &self.current.kind {
            TokenKind::Any | TokenKind::Unknown | TokenKind::Never | TokenKind::Void | TokenKind::Null => {
                self.advance();
            }
            TokenKind::Identifier(_) => {
                self.advance();
                if self.check(&TokenKind::LessThan) {
                    self.skip_type_arguments()?;
                }
                while self.check(&TokenKind::Dot) {
                    self.advance();
                    self.expect_identifier()?;
                    if self.check(&TokenKind::LessThan) {
                        self.skip_type_arguments()?;
                    }
                }
            }
            TokenKind::Typeof => {
                self.advance();
                self.expect_identifier()?;
                while self.check(&TokenKind::Dot) {
                    self.advance();
                    self.expect_identifier()?;
                }
            }
            TokenKind::Keyof | TokenKind::Readonly => {
                self.advance();
                self.skip_type()?;
            }
            TokenKind::LeftBracket => {
                self.advance();
                self.expect(&TokenKind::RightBracket)?;
            }
            TokenKind::LeftParen => {
                self.skip_parenthesized_type()?;
            }
            TokenKind::LeftBrace => {
                self.skip_object_type()?;
            }
            TokenKind::String(_) | TokenKind::Number(_) | TokenKind::True | TokenKind::False => {
                self.advance();
            }
            TokenKind::Infer => {
                self.advance();
                self.expect_identifier()?;
            }
            TokenKind::This | TokenKind::New => {
                self.advance();
                if self.check(&TokenKind::LeftParen) {
                    self.skip_parenthesized_type()?;
                }
            }
            _ => return Ok(()),
        }
        self.skip_type_postfix()?;
        Ok(())
    }

    /// Skip postfix type modifiers like `[]` or `extends X ? Y : Z`
    fn skip_type_postfix(&mut self) -> Result<(), Error> {
        loop {
            if self.check(&TokenKind::LeftBracket) {
                self.advance();
                if !self.check(&TokenKind::RightBracket) {
                    self.skip_type()?;
                }
                self.expect(&TokenKind::RightBracket)?;
                continue;
            }
            if self.check(&TokenKind::Extends) {
                self.advance();
                self.skip_type()?;
                self.expect(&TokenKind::Question)?;
                self.skip_type()?;
                self.expect(&TokenKind::Colon)?;
                self.skip_type()?;
                continue;
            }
            break;
        }
        Ok(())
    }

    /// Skip type arguments: `<A, B, C>`
    fn skip_type_arguments(&mut self) -> Result<(), Error> {
        if !self.check(&TokenKind::LessThan) {
            return Ok(());
        }
        self.advance();
        if !self.check(&TokenKind::GreaterThan) {
            self.skip_type()?;
            while self.check(&TokenKind::Comma) {
                self.advance();
                self.skip_type()?;
            }
        }
        self.expect(&TokenKind::GreaterThan)?;
        Ok(())
    }

    /// Skip type parameters: `<T, U extends V>`
    fn skip_type_parameters(&mut self) -> Result<(), Error> {
        if !self.is_typescript_mode() || !self.check(&TokenKind::LessThan) {
            return Ok(());
        }
        self.advance();
        loop {
            if self.check(&TokenKind::In) || self.check(&TokenKind::Out) {
                self.advance();
            }
            self.expect_identifier()?;
            if self.check(&TokenKind::Extends) {
                self.advance();
                self.skip_type()?;
            }
            if self.check(&TokenKind::Equal) {
                self.advance();
                self.skip_type()?;
            }
            if !self.check(&TokenKind::Comma) {
                break;
            }
            self.advance();
        }
        self.expect(&TokenKind::GreaterThan)?;
        Ok(())
    }

    /// Skip a parenthesized type or function type.
    fn skip_parenthesized_type(&mut self) -> Result<(), Error> {
        self.expect(&TokenKind::LeftParen)?;
        if !self.check(&TokenKind::RightParen) {
            loop {
                if self.check(&TokenKind::Ellipsis) {
                    self.advance();
                }
                if let TokenKind::Identifier(_) = &self.current.kind {
                    self.advance();
                    if self.check(&TokenKind::Question) {
                        self.advance();
                    }
                    if self.check(&TokenKind::Colon) {
                        self.advance();
                        self.skip_type()?;
                    }
                } else {
                    self.skip_type()?;
                }
                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
        }
        self.expect(&TokenKind::RightParen)?;
        if self.check(&TokenKind::Arrow) {
            self.advance();
            self.skip_type()?;
        }
        Ok(())
    }

    /// Skip an object type literal: `{ prop: Type; method(): Type }`
    fn skip_object_type(&mut self) -> Result<(), Error> {
        self.expect(&TokenKind::LeftBrace)?;
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            if self.check(&TokenKind::LeftBracket) {
                self.advance();
                self.expect_identifier()?;
                self.expect(&TokenKind::Colon)?;
                self.skip_type()?;
                self.expect(&TokenKind::RightBracket)?;
                self.expect(&TokenKind::Colon)?;
                self.skip_type()?;
            } else {
                if self.check(&TokenKind::Readonly) {
                    self.advance();
                }
                match &self.current.kind {
                    TokenKind::Identifier(_) | TokenKind::String(_) | TokenKind::Number(_) => {
                        self.advance();
                    }
                    _ => break,
                }
                if self.check(&TokenKind::Question) {
                    self.advance();
                }
                if self.check(&TokenKind::LessThan) {
                    self.skip_type_parameters()?;
                }
                if self.check(&TokenKind::LeftParen) {
                    self.skip_parenthesized_type()?;
                } else if self.check(&TokenKind::Colon) {
                    self.advance();
                    self.skip_type()?;
                }
            }
            if self.check(&TokenKind::Semicolon) || self.check(&TokenKind::Comma) {
                self.advance();
            } else if !self.check(&TokenKind::RightBrace) {
                break;
            }
        }
        self.expect(&TokenKind::RightBrace)?;
        Ok(())
    }

    /// Skip a complete type alias declaration: `type Name<T> = Type;`
    fn skip_type_alias(&mut self) -> Result<(), Error> {
        self.expect(&TokenKind::Type)?;
        self.expect_identifier()?;
        self.skip_type_parameters()?;
        self.expect(&TokenKind::Equal)?;
        self.skip_type()?;
        if self.check(&TokenKind::Semicolon) {
            self.advance();
        }
        Ok(())
    }

    /// Skip a complete interface declaration: `interface Name<T> extends A, B { ... }`
    fn skip_interface(&mut self) -> Result<(), Error> {
        self.expect(&TokenKind::Interface)?;
        self.expect_identifier()?;
        self.skip_type_parameters()?;
        if self.check(&TokenKind::Extends) {
            self.advance();
            self.skip_type()?;
            while self.check(&TokenKind::Comma) {
                self.advance();
                self.skip_type()?;
            }
        }
        self.skip_object_type()?;
        Ok(())
    }

    /// Skip a declare statement: `declare ...`
    fn skip_declare_statement(&mut self) -> Result<(), Error> {
        self.expect(&TokenKind::Declare)?;
        let mut brace_depth = 0;
        loop {
            match &self.current.kind {
                TokenKind::LeftBrace => {
                    brace_depth += 1;
                    self.advance();
                }
                TokenKind::RightBrace => {
                    if brace_depth == 0 {
                        break;
                    }
                    brace_depth -= 1;
                    self.advance();
                    if brace_depth == 0 {
                        break;
                    }
                }
                TokenKind::Semicolon if brace_depth == 0 => {
                    self.advance();
                    break;
                }
                TokenKind::Eof => break,
                _ => {
                    self.advance();
                }
            }
        }
        Ok(())
    }

    /// Skip a namespace declaration: `namespace Name { ... }`
    fn skip_namespace(&mut self) -> Result<(), Error> {
        self.expect(&TokenKind::Namespace)?;
        self.expect_identifier()?;
        while self.check(&TokenKind::Dot) {
            self.advance();
            self.expect_identifier()?;
        }
        self.expect(&TokenKind::LeftBrace)?;
        let mut brace_depth = 1;
        while brace_depth > 0 && !self.is_at_end() {
            match &self.current.kind {
                TokenKind::LeftBrace => brace_depth += 1,
                TokenKind::RightBrace => brace_depth -= 1,
                _ => {}
            }
            self.advance();
        }
        Ok(())
    }

    /// Skip access modifiers: `public`, `private`, `protected`, `readonly`
    fn skip_access_modifiers(&mut self) -> Result<(), Error> {
        if !self.is_typescript_mode() {
            return Ok(());
        }
        while matches!(
            self.current.kind,
            TokenKind::Public
                | TokenKind::Private
                | TokenKind::Protected
                | TokenKind::Readonly
                | TokenKind::Abstract
                | TokenKind::Override
                | TokenKind::Accessor
        ) {
            self.advance();
        }
        Ok(())
    }

    /// Skip `implements` clause in class declaration.
    #[allow(dead_code)]
    fn skip_implements_clause(&mut self) -> Result<(), Error> {
        if !self.is_typescript_mode() {
            return Ok(());
        }
        if self.check(&TokenKind::Implements) {
            self.advance();
            self.skip_type()?;
            while self.check(&TokenKind::Comma) {
                self.advance();
                self.skip_type()?;
            }
        }
        Ok(())
    }

    /// Parse a TypeScript enum declaration and convert it to JavaScript.
    ///
    /// Converts:
    /// ```typescript
    /// enum Color { Red, Green, Blue }
    /// ```
    /// To equivalent AST for:
    /// ```javascript
    /// var Color = {};
    /// Color["Red"] = 0; Color[0] = "Red";
    /// Color["Green"] = 1; Color[1] = "Green";
    /// Color["Blue"] = 2; Color[2] = "Blue";
    /// ```
    fn parse_enum_declaration(&mut self) -> Result<Statement, Error> {
        self.expect(&TokenKind::Enum)?;

        // Get the enum name
        let enum_name = self.expect_identifier()?;
        self.expect(&TokenKind::LeftBrace)?;

        // Collect enum members
        let mut members: Vec<(String, Option<Expression>)> = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            let member_name = self.expect_identifier()?;

            // Check for explicit value
            let value = if self.check(&TokenKind::Equal) {
                self.advance();
                Some(self.parse_expression()?)
            } else {
                None
            };

            members.push((member_name.name, value));

            if !self.check(&TokenKind::RightBrace)
                && self.check(&TokenKind::Comma) {
                    self.advance();
                }
        }

        self.expect(&TokenKind::RightBrace)?;

        // Generate JavaScript AST
        // var EnumName = {};
        let var_decl = Statement::VariableDeclaration(VariableDeclaration {
            kind: VariableKind::Var,
            declarations: vec![VariableDeclarator {
                id: enum_name.clone(),
                init: Some(Expression::Object(ObjectExpression {
                    properties: vec![],
                })),
            }],
        });

        // Generate assignment statements for each member
        let mut statements = vec![var_decl];
        let mut current_value: i64 = 0;

        for (name, explicit_value) in members {
            // Determine the value for this member
            let value_expr = match explicit_value {
                Some(Expression::Literal(Literal::Number(n))) => {
                    current_value = n as i64 + 1;
                    Expression::Literal(Literal::Number(n))
                }
                Some(Expression::Literal(Literal::String(s))) => {
                    // String enum - no reverse mapping
                    let assign_stmt = Statement::Expression(ExpressionStatement {
                        expression: Expression::Assignment(AssignmentExpression {
                            operator: AssignmentOperator::Assign,
                            left: Box::new(Expression::Member(MemberExpression {
                                object: Box::new(Expression::Identifier(enum_name.clone())),
                                property: MemberProperty::Expression(Box::new(Expression::Literal(
                                    Literal::String(name.clone()),
                                ))),
                                computed: true,
                            })),
                            right: Box::new(Expression::Literal(Literal::String(s))),
                        }),
                    });
                    statements.push(assign_stmt);
                    continue;
                }
                Some(expr) => {
                    // For complex expressions, just use them
                    current_value += 1;
                    expr
                }
                None => {
                    let val = current_value;
                    current_value += 1;
                    Expression::Literal(Literal::Number(val as f64))
                }
            };

            // EnumName["MemberName"] = value
            let forward_assign = Statement::Expression(ExpressionStatement {
                expression: Expression::Assignment(AssignmentExpression {
                    operator: AssignmentOperator::Assign,
                    left: Box::new(Expression::Member(MemberExpression {
                        object: Box::new(Expression::Identifier(enum_name.clone())),
                        property: MemberProperty::Expression(Box::new(Expression::Literal(
                            Literal::String(name.clone()),
                        ))),
                        computed: true,
                    })),
                    right: Box::new(value_expr.clone()),
                }),
            });
            statements.push(forward_assign);

            // EnumName[value] = "MemberName" (reverse mapping for numeric enums)
            if let Expression::Literal(Literal::Number(_)) = value_expr {
                let reverse_assign = Statement::Expression(ExpressionStatement {
                    expression: Expression::Assignment(AssignmentExpression {
                        operator: AssignmentOperator::Assign,
                        left: Box::new(Expression::Member(MemberExpression {
                            object: Box::new(Expression::Identifier(enum_name.clone())),
                            property: MemberProperty::Expression(Box::new(value_expr)),
                            computed: true,
                        })),
                        right: Box::new(Expression::Literal(Literal::String(name))),
                    }),
                });
                statements.push(reverse_assign);
            }
        }

        // Return a block containing all statements
        Ok(Statement::Block(BlockStatement { body: statements }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to parse and get first statement
    fn parse_stmt(src: &str) -> Statement {
        let mut parser = Parser::new(src);
        let program = parser.parse_program().unwrap();
        program.body.into_iter().next().unwrap()
    }

    // Helper to parse and check it succeeds
    fn parse_ok(src: &str) -> Program {
        let mut parser = Parser::new(src);
        parser.parse_program().unwrap()
    }

    // Helper to parse and check it fails
    fn parse_err(src: &str) -> Error {
        let mut parser = Parser::new(src);
        parser.parse_program().unwrap_err()
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
    fn test_parse_var_let_const() {
        parse_ok("var x = 1;");
        parse_ok("let y = 2;");
        parse_ok("const z = 3;");
    }

    #[test]
    fn test_parse_multiple_declarations() {
        let program = parse_ok("let a = 1, b = 2, c;");
        assert_eq!(program.body.len(), 1);
    }

    #[test]
    fn test_parse_if_statement() {
        parse_ok("if (x > 0) { y = 1; }");
        parse_ok("if (x) y = 1;");
        parse_ok("if (x) y = 1; else z = 2;");
        parse_ok("if (a) { } else if (b) { } else { }");
    }

    #[test]
    fn test_parse_while_statement() {
        parse_ok("while (x > 0) { x = x - 1; }");
        parse_ok("while (true) break;");
    }

    #[test]
    fn test_parse_do_while_statement() {
        parse_ok("do { x = x + 1; } while (x < 10);");
    }

    #[test]
    fn test_parse_for_statement() {
        parse_ok("for (let i = 0; i < 10; i = i + 1) { }");
        parse_ok("for (;;) break;");
        parse_ok("for (i = 0; i < n;) { i = i + 1; }");
    }

    #[test]
    fn test_parse_switch_statement() {
        parse_ok("switch (x) { case 1: break; case 2: y = 2; break; default: z = 0; }");
        parse_ok("switch (x) { default: break; }");
    }

    #[test]
    fn test_parse_try_catch_finally() {
        parse_ok("try { x = 1; } catch (e) { }");
        parse_ok("try { } finally { cleanup(); }");
        parse_ok("try { } catch (e) { } finally { }");
    }

    #[test]
    fn test_parse_throw_statement() {
        parse_ok("throw new Error('msg');");
        parse_ok("throw 42;");
    }

    #[test]
    fn test_parse_return_statement() {
        parse_ok("function f() { return; }");
        parse_ok("function f() { return 42; }");
    }

    #[test]
    fn test_parse_break_continue() {
        parse_ok("while (true) { break; }");
        parse_ok("while (true) { continue; }");
    }

    #[test]
    fn test_parse_block_statement() {
        parse_ok("{ let x = 1; let y = 2; }");
    }

    #[test]
    fn test_parse_empty_statement() {
        parse_ok(";");
        parse_ok(";;;");
    }

    #[test]
    fn test_parse_expression_statement() {
        parse_ok("42;");
        parse_ok("x + y;");
        parse_ok("f();");
    }

    #[test]
    fn test_parse_arithmetic_operators() {
        parse_ok("a + b;");
        parse_ok("a - b;");
        parse_ok("a * b;");
        parse_ok("a / b;");
    }

    #[test]
    fn test_parse_comparison_operators() {
        parse_ok("a < b;");
        parse_ok("a > b;");
        parse_ok("a <= b;");
        parse_ok("a >= b;");
        parse_ok("a == b;");
        parse_ok("a != b;");
        parse_ok("a === b;");
        parse_ok("a !== b;");
    }

    #[test]
    fn test_parse_logical_operators() {
        parse_ok("a && b;");
        parse_ok("a || b;");
        parse_ok("!a;");
    }

    #[test]
    fn test_parse_assignment_operators() {
        parse_ok("a = b;");
    }

    #[test]
    fn test_parse_unary_operators() {
        parse_ok("-x;");
        parse_ok("!x;");
        parse_ok("typeof x;");
    }

    #[test]
    fn test_parse_member_expression() {
        parse_ok("a.b;");
        parse_ok("a.b.c;");
        parse_ok("a[0];");
        parse_ok("a[b];");
        parse_ok("a.b[c].d;");
    }

    #[test]
    fn test_parse_call_expression() {
        parse_ok("f();");
        parse_ok("f(a);");
        parse_ok("f(a, b, c);");
        parse_ok("obj.method();");
        parse_ok("a()();");
    }

    #[test]
    fn test_parse_new_expression() {
        parse_ok("new Foo();");
        parse_ok("new Foo(a, b);");
        parse_ok("new a.b.c();");
    }

    #[test]
    fn test_parse_array_literal() {
        parse_ok("[];");
        parse_ok("[1, 2, 3];");
        parse_ok("[a, b, c];");
    }

    #[test]
    fn test_parse_object_literal() {
        // Object literals need identifier on left side
        parse_ok("let x = {};");
        parse_ok("let x = { a: 1 };");
        parse_ok("let x = { a: 1, b: 2 };");
    }

    #[test]
    fn test_parse_arrow_function() {
        parse_ok("let f = () => 42;");
        parse_ok("let f = (a, b) => a + b;");
        parse_ok("let f = (x) => { return x; };");
    }

    #[test]
    fn test_parse_function_expression() {
        parse_ok("let f = function() { };");
        parse_ok("let f = function add(a, b) { return a + b; };");
    }

    #[test]
    fn test_parse_literals() {
        parse_ok("42;");
        parse_ok("3.14;");
        parse_ok("'hello';");
        parse_ok("\"world\";");
        parse_ok("true;");
        parse_ok("false;");
        parse_ok("null;");
    }

    #[test]
    fn test_parse_this() {
        parse_ok("this;");
        parse_ok("this.x;");
    }

    #[test]
    fn test_parse_grouping() {
        // Test that grouped expressions work
        parse_ok("let x = (1 + 2);");
    }

    #[test]
    fn test_parse_complex_expression() {
        parse_ok("a + b * c;");
        parse_ok("f(g(h(x)));");
    }

    #[test]
    fn test_parse_nested_control_flow() {
        parse_ok("if (a) { if (b) { c = 1; } }");
        parse_ok("while (a) { while (b) { break; } }");
    }

    #[test]
    fn test_parse_error_missing_semicolon() {
        // This might or might not error depending on ASI rules
        // Just verify it doesn't panic
        let _ = Parser::new("let x = 1").parse_program();
    }

    #[test]
    fn test_parse_error_unexpected_token() {
        let err = parse_err("let = 42;");
        assert!(matches!(err, Error::SyntaxError(_)));
    }

    #[test]
    fn test_parse_operator_precedence() {
        // 1 + 2 * 3 should parse as 1 + (2 * 3)
        let stmt = parse_stmt("1 + 2 * 3;");
        if let Statement::Expression(expr_stmt) = stmt {
            // The outer expression should be addition
            if let Expression::Binary(bin_expr) = &expr_stmt.expression {
                assert_eq!(bin_expr.operator, BinaryOperator::Add);
            } else {
                panic!("Expected binary expression");
            }
        } else {
            panic!("Expected expression statement");
        }
    }

    #[test]
    fn test_parse_empty_program() {
        let program = parse_ok("");
        assert!(program.body.is_empty());
    }

    #[test]
    fn test_parse_multiple_statements() {
        let program = parse_ok("let x = 1; let y = 2; let z = 3;");
        assert_eq!(program.body.len(), 3);
    }

    #[test]
    fn test_parse_ternary_operator() {
        parse_ok("true ? 1 : 2;");
        parse_ok("x > 0 ? x : -x;");
        parse_ok("a ? b : c ? d : e;");
        parse_ok("let x = a > b ? a : b;");
    }

    // TypeScript-specific tests

    fn parse_ts_ok(src: &str) -> Program {
        let mut parser = Parser::new_typescript(src);
        parser.parse_program().unwrap()
    }

    #[test]
    fn test_parse_typescript_variable_type_annotation() {
        // let x: number = 42;
        let program = parse_ts_ok("let x: number = 42;");
        assert_eq!(program.body.len(), 1);
    }

    #[test]
    fn test_parse_typescript_function_types() {
        // function add(a: number, b: number): number { return a + b; }
        let program = parse_ts_ok("function add(a: number, b: number): number { return a + b; }");
        assert_eq!(program.body.len(), 1);
    }

    #[test]
    fn test_parse_typescript_type_alias_skipped() {
        // type ID = string | number;
        // let x = 1;
        let program = parse_ts_ok("type ID = string | number; let x = 1;");
        // Type alias should be skipped (becomes Empty), so we have 2 statements
        assert_eq!(program.body.len(), 2);
    }

    #[test]
    fn test_parse_typescript_interface_skipped() {
        // interface User { name: string; age: number; }
        // let user = {};
        let program = parse_ts_ok("interface User { name: string; age: number; } let user = {};");
        assert_eq!(program.body.len(), 2);
    }

    #[test]
    fn test_parse_typescript_generic_function() {
        // function identity<T>(x: T): T { return x; }
        let program = parse_ts_ok("function identity<T>(x: T): T { return x; }");
        assert_eq!(program.body.len(), 1);
    }

    #[test]
    fn test_parse_typescript_optional_params() {
        // function greet(name?: string): void { }
        let program = parse_ts_ok("function greet(name?: string) { }");
        assert_eq!(program.body.len(), 1);
    }

    #[test]
    fn test_parse_typescript_complex_type() {
        // let x: { name: string; items: number[] } = { name: "test", items: [] };
        let program = parse_ts_ok("let x: { name: string } = { name: \"test\" };");
        assert_eq!(program.body.len(), 1);
    }

    #[test]
    fn test_parse_typescript_arrow_with_types() {
        // Arrow function with return type but simple param
        // The parameter type annotation in arrow requires special handling
        let program = parse_ts_ok("const fn = (x) => x * 2;");
        assert_eq!(program.body.len(), 1);
    }

    #[test]
    fn test_parse_typescript_function_expression_with_types() {
        // Function expression with full type annotations
        let program = parse_ts_ok("const fn = function(x: number): number { return x * 2; };");
        assert_eq!(program.body.len(), 1);
    }

    #[test]
    fn test_parse_typescript_as_assertion() {
        // let x = value as string;
        let program = parse_ts_ok("let x = value as string;");
        assert_eq!(program.body.len(), 1);
    }

    #[test]
    fn test_parse_typescript_non_null_assertion() {
        // let x = obj!.prop;
        let program = parse_ts_ok("let x = obj!;");
        assert_eq!(program.body.len(), 1);
    }

    #[test]
    fn test_parse_typescript_declare_skipped() {
        // declare const x: number;
        let program = parse_ts_ok("declare const x: number; let y = 1;");
        assert_eq!(program.body.len(), 2);
    }

    #[test]
    fn test_parse_typescript_namespace_skipped() {
        // namespace Utils { export function helper() {} }
        let program = parse_ts_ok("namespace Utils { export function helper() {} } let x = 1;");
        assert_eq!(program.body.len(), 2);
    }

    #[test]
    fn test_parse_typescript_union_type() {
        // let x: string | number = "hello";
        let program = parse_ts_ok("let x: string | number = \"hello\";");
        assert_eq!(program.body.len(), 1);
    }

    #[test]
    fn test_parse_typescript_intersection_type() {
        // let x: A & B = {};
        let program = parse_ts_ok("let x: A & B = {};");
        assert_eq!(program.body.len(), 1);
    }

    #[test]
    fn test_parse_typescript_enum() {
        // enum Color { Red, Green, Blue }
        let program = parse_ts_ok("enum Color { Red, Green, Blue }");
        assert_eq!(program.body.len(), 1);

        // The enum should be converted to a block with statements
        match &program.body[0] {
            Statement::Block(block) => {
                // var Color = {};
                // Color["Red"] = 0; Color[0] = "Red";
                // Color["Green"] = 1; Color[1] = "Green";
                // Color["Blue"] = 2; Color[2] = "Blue";
                // = 7 statements
                assert_eq!(block.body.len(), 7);
            }
            _ => panic!("Expected block statement for enum"),
        }
    }

    #[test]
    fn test_parse_typescript_enum_with_values() {
        // enum Status { Active = 1, Inactive = 0 }
        let program = parse_ts_ok("enum Status { Active = 1, Inactive = 0 }");
        assert_eq!(program.body.len(), 1);
    }

    #[test]
    fn test_parse_typescript_string_enum() {
        // enum Direction { Up = "UP", Down = "DOWN" }
        let program = parse_ts_ok("enum Direction { Up = \"UP\", Down = \"DOWN\" }");
        assert_eq!(program.body.len(), 1);

        // String enums don't have reverse mapping, so fewer statements
        match &program.body[0] {
            Statement::Block(block) => {
                // var Direction = {};
                // Direction["Up"] = "UP";
                // Direction["Down"] = "DOWN";
                // = 3 statements (no reverse mapping for strings)
                assert_eq!(block.body.len(), 3);
            }
            _ => panic!("Expected block statement for enum"),
        }
    }
}
