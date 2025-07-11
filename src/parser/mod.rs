//! Parser for the Eä programming language.
//!
//! This module is responsible for transforming a sequence of tokens into an
//! Abstract Syntax Tree (AST) that represents the structure of the source code.

// SIMD-002 Phase 2: Core SIMD Expression Parser Implementation
// Extends existing recursive descent parser with industry-first SIMD support

use crate::{
    ast::{
        BinaryOp, EnumVariant, Expr, Literal, MatchArm, Parameter, Pattern, ReductionOp, SIMDExpr,
        SIMDOperator, SIMDVectorType, Stmt, TypeAnnotation, UnaryOp,
    }, // Added Pattern and MatchArm imports
    error::{CompileError, Result},
    lexer::{Token, TokenKind, Position}, // Re-added Position for error recovery
    memory_profiler::{record_memory_usage, CompilationPhase, check_memory_limit},
    parser_optimization::{enter_parse_recursion, exit_parse_recursion, time_parsing_operation},
};

/// Error suggestions for common mistakes
#[derive(Debug, Clone)]
pub struct ErrorSuggestion {
    pub message: String,
    pub suggested_fix: Option<String>,
}

/// Recovery action to take after a parse error
#[derive(Debug, Clone)]
pub enum RecoveryAction {
    Skip,           // Skip current token and continue
    Synchronize,    // Skip to next statement boundary
    Insert(TokenKind), // Insert missing token
    Replace(TokenKind), // Replace current token
}

/// Error context for better error messages
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub expected: Vec<TokenKind>,
    pub found: TokenKind,
    pub position: Position,
    pub context: String,
}

/// The parser converts a sequence of tokens into an Abstract Syntax Tree (AST).
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    errors: Vec<CompileError>,     // Collect multiple errors
    in_recovery: bool,             // Flag to prevent cascading errors
}

impl Parser {
    /// Creates a new parser for the given tokens.
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { 
            tokens, 
            current: 0,
            errors: Vec::new(),
            in_recovery: false,
        }
    }

    /// Get all collected errors
    pub fn get_errors(&self) -> &[CompileError] {
        &self.errors
    }

    /// Parses the tokens and returns the resulting program as a list of statements.
    pub fn parse_program(&mut self) -> Result<Vec<Stmt>> {
        eprintln!("🏗️ Starting parse_program...");
        
        // Record initial memory usage for parsing
        let initial_memory = std::mem::size_of::<Vec<Stmt>>();
        record_memory_usage(CompilationPhase::Parsing, initial_memory, "Started parsing");

        let mut statements = Vec::new();
        let mut loop_count = 0;
        let mut last_position = self.current;
        let mut position_stuck_count = 0;

        eprintln!("🔄 Starting parsing loop...");
        while !self.is_at_end() {
            loop_count += 1;
            eprintln!("🔄 Parse loop iteration {}, current position: {:?}", loop_count, self.current);
            
            // Check if we're stuck at the same position
            if self.current == last_position {
                position_stuck_count += 1;
                if position_stuck_count > 5 {
                    eprintln!("❌ Parser stuck at position {} for {} iterations, forcing advance", self.current, position_stuck_count);
                    // Force advance to prevent infinite loop
                    self.advance();
                    self.synchronize();
                    position_stuck_count = 0;
                }
            } else {
                position_stuck_count = 0;
            }
            last_position = self.current;
            
            if loop_count > 1000 {
                eprintln!("❌ Parse loop detected (over 1000 iterations), breaking");
                return Err(CompileError::ParseError {
                    message: "Infinite loop detected in parser".to_string(),
                    position: self.tokens[self.current].position.clone(),
                });
            }
            
            eprintln!("🔄 Calling declaration()...");
            match self.declaration() {
                Ok(stmt) => {
                    eprintln!("✅ Declaration successful, got statement");
                    statements.push(stmt);
                    self.in_recovery = false; // Reset recovery flag on success
                    
                    // Check memory usage periodically
                    if statements.len() % 100 == 0 {
                        let current_memory = statements.len() * std::mem::size_of::<Stmt>();
                        record_memory_usage(CompilationPhase::Parsing, current_memory, 
                            &format!("Parsing progress: {} statements", statements.len()));
                        
                        // Check memory limits
                        if let Err(e) = check_memory_limit() {
                            return Err(CompileError::MemoryExhausted { 
                                phase: "parsing".to_string(), 
                                details: e.to_string() 
                            });
                        }
                    }
                }
                Err(error) => {
                    eprintln!("❌ Declaration failed: {:?}", error);
                    self.errors.push(error.clone());
                    if !self.in_recovery {
                        self.in_recovery = true;
                        self.synchronize(); // Try to recover and continue parsing
                    }
                }
            }
        }

        // Record final memory usage
        let final_memory = statements.len() * std::mem::size_of::<Stmt>();
        record_memory_usage(CompilationPhase::Parsing, final_memory, 
            &format!("Completed parsing: {} statements", statements.len()));

        // Return the first error if any occurred, but we've collected all errors
        if !self.errors.is_empty() {
            return Err(self.errors[0].clone());
        }

        Ok(statements)
    }

    /// Parses a declaration statement (function, variable, or regular statement).
    fn declaration(&mut self) -> Result<Stmt> {
        if self.match_tokens(&[TokenKind::Func]) {
            return self.function_declaration("function");
        }

        if self.match_tokens(&[TokenKind::Struct]) {
            return self.struct_declaration();
        }

        if self.match_tokens(&[TokenKind::Enum]) {
            return self.enum_declaration();
        }

        if self.match_tokens(&[TokenKind::Let]) {
            return self.var_declaration();
        }

        self.statement()
    }

    /// Parses a function declaration.
    fn function_declaration(&mut self, kind: &str) -> Result<Stmt> {
        let name = self.consume_identifier(format!("Expected {kind} name"))?;

        self.consume(
            TokenKind::LeftParen,
            format!("Expected '(' after {kind} name"),
        )?;

        let mut parameters = Vec::new();
        if !self.check(&TokenKind::RightParen) {
            loop {
                // Parse parameter name
                let param_name = self.consume_identifier("Expected parameter name".to_string())?;

                // Parse parameter type
                self.consume(
                    TokenKind::Colon,
                    "Expected ':' after parameter name".to_string(),
                )?;

                let is_mutable = self.match_tokens(&[TokenKind::Mut]);
                let type_name = self.consume_type_name("Expected parameter type".to_string())?;

                let param = Parameter {
                    name: param_name,
                    type_annotation: TypeAnnotation {
                        name: type_name,
                        is_mutable,
                    },
                };

                parameters.push(param);

                if !self.match_tokens(&[TokenKind::Comma]) {
                    break;
                }
            }
        }

        self.consume(
            TokenKind::RightParen,
            "Expected ')' after parameters".to_string(),
        )?;

        // Parse optional return type
        let return_type = if self.match_tokens(&[TokenKind::Arrow]) {
            let is_mutable = self.match_tokens(&[TokenKind::Mut]);
            if self.match_tokens(&[TokenKind::LeftParen]) {
                // Unit type ()
                self.consume(
                    TokenKind::RightParen,
                    "Expected ')' for unit return type".to_string(),
                )?;
                Some(TypeAnnotation {
                    name: "()".to_string(),
                    is_mutable,
                })
            } else {
                let type_name = self.consume_type_name("Expected return type".to_string())?;
                Some(TypeAnnotation {
                    name: type_name,
                    is_mutable,
                })
            }
        } else {
            None
        };

        // Parse function body
        self.consume(
            TokenKind::LeftBrace,
            format!("Expected '{{' before {kind} body"),
        )?;

        let body = self.block()?;

        Ok(Stmt::FunctionDeclaration {
            name,
            params: parameters,
            return_type,
            body: Box::new(body),
            attributes: Vec::new(), // Note: Attributes not yet implemented
        })
    }

    /// Parses a struct declaration.
    fn struct_declaration(&mut self) -> Result<Stmt> {
        let name = self.consume_identifier("Expected struct name".to_string())?;

        self.consume(
            TokenKind::LeftBrace,
            "Expected '{' after struct name".to_string(),
        )?;

        let mut fields = Vec::new();

        if !self.check(&TokenKind::RightBrace) {
            loop {
                let field_name = self.consume_identifier("Expected field name".to_string())?;

                self.consume(
                    TokenKind::Colon,
                    "Expected ':' after field name".to_string(),
                )?;

                let field_type = self.consume_type_name("Expected field type".to_string())?;

                fields.push(crate::ast::StructField {
                    name: field_name,
                    type_annotation: crate::ast::TypeAnnotation {
                        name: field_type,
                        is_mutable: false,
                    },
                });

                if !self.match_tokens(&[TokenKind::Comma]) {
                    break;
                }

                // Allow trailing comma
                if self.check(&TokenKind::RightBrace) {
                    break;
                }
            }
        }

        self.consume(
            TokenKind::RightBrace,
            "Expected '}' after struct fields".to_string(),
        )?;

        Ok(Stmt::StructDeclaration { name, fields })
    }

    /// Parses an enum declaration.
    fn enum_declaration(&mut self) -> Result<Stmt> {
        let name = self.consume_identifier("Expected enum name".to_string())?;

        self.consume(
            TokenKind::LeftBrace,
            "Expected '{' after enum name".to_string(),
        )?;

        let mut variants = Vec::new();

        if !self.check(&TokenKind::RightBrace) {
            loop {
                let variant_name = self.consume_identifier("Expected variant name".to_string())?;

                let data = if self.match_tokens(&[TokenKind::LeftParen]) {
                    let mut type_list = Vec::new();

                    if !self.check(&TokenKind::RightParen) {
                        loop {
                            let type_name =
                                self.consume_type_name("Expected type in variant".to_string())?;
                            type_list.push(TypeAnnotation {
                                name: type_name,
                                is_mutable: false,
                            });

                            if !self.match_tokens(&[TokenKind::Comma]) {
                                break;
                            }
                        }
                    }

                    self.consume(
                        TokenKind::RightParen,
                        "Expected ')' after variant types".to_string(),
                    )?;

                    Some(type_list)
                } else {
                    None
                };

                variants.push(EnumVariant {
                    name: variant_name,
                    data,
                });

                if !self.match_tokens(&[TokenKind::Comma]) {
                    break;
                }

                // Allow trailing comma
                if self.check(&TokenKind::RightBrace) {
                    break;
                }
            }
        }

        self.consume(
            TokenKind::RightBrace,
            "Expected '}' after enum variants".to_string(),
        )?;

        Ok(Stmt::EnumDeclaration { name, variants })
    }

    /// Parses a variable declaration.
    fn var_declaration(&mut self) -> Result<Stmt> {
        let is_mutable = self.match_tokens(&[TokenKind::Mut]);
        let name = self.consume_identifier("Expected variable name".to_string())?;

        // Parse optional type annotation
        let type_annotation = if self.match_tokens(&[TokenKind::Colon]) {
            let type_name = self.consume_type_name("Expected type after ':'".to_string())?;
            Some(TypeAnnotation {
                name: type_name,
                is_mutable,
            })
        } else {
            None
        };

        // Parse optional initializer
        let initializer = if self.match_tokens(&[TokenKind::Assign]) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(
            TokenKind::Semicolon,
            "Expected ';' after variable declaration".to_string(),
        )?;

        Ok(Stmt::VarDeclaration {
            name,
            type_annotation,
            initializer,
        })
    }

    /// Parses a regular statement.
    fn statement(&mut self) -> Result<Stmt> {
        if self.match_tokens(&[TokenKind::Return]) {
            return self.return_statement();
        }

        if self.match_tokens(&[TokenKind::LeftBrace]) {
            return self.block();
        }

        if self.match_tokens(&[TokenKind::If]) {
            return self.if_statement();
        }

        if self.match_tokens(&[TokenKind::While]) {
            return self.while_statement();
        }

        if self.match_tokens(&[TokenKind::For]) {
            return self.for_statement();
        }

        self.expression_statement()
    }

    /// Parses a return statement.
    fn return_statement(&mut self) -> Result<Stmt> {
        let expr = if !self.check(&TokenKind::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(
            TokenKind::Semicolon,
            "Expected ';' after return value".to_string(),
        )?;

        Ok(Stmt::Return(expr))
    }

    /// Parses a block statement.
    fn block(&mut self) -> Result<Stmt> {
        let mut statements = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(
            TokenKind::RightBrace,
            "Expected '}' after block".to_string(),
        )?;

        Ok(Stmt::Block(statements))
    }

    /// Parses an if statement.
    fn if_statement(&mut self) -> Result<Stmt> {
        self.consume(TokenKind::LeftParen, "Expected '(' after 'if'".to_string())?;

        let condition = self.expression()?;

        self.consume(
            TokenKind::RightParen,
            "Expected ')' after if condition".to_string(),
        )?;

        let then_branch = Box::new(self.statement()?);

        let else_branch = if self.match_tokens(&[TokenKind::Else]) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    /// Parses a while statement.
    fn while_statement(&mut self) -> Result<Stmt> {
        self.consume(
            TokenKind::LeftParen,
            "Expected '(' after 'while'".to_string(),
        )?;

        let condition = self.expression()?;

        self.consume(
            TokenKind::RightParen,
            "Expected ')' after while condition".to_string(),
        )?;

        let body = Box::new(self.statement()?);

        Ok(Stmt::While { condition, body })
    }

    /// Parses a for statement.
    fn for_statement(&mut self) -> Result<Stmt> {
        // Check if this is a for-in loop by looking for the pattern: identifier 'in'
        // We can peek ahead without consuming tokens

        // Check if the next token is an identifier
        if matches!(self.peek().kind, TokenKind::Identifier(_)) {
            // Look at the token after the identifier to see if it's 'in'
            if self.current + 1 < self.tokens.len() {
                if let TokenKind::In = self.tokens[self.current + 1].kind {
                    // This is a for-in loop
                    let variable = self
                        .consume_identifier("Expected variable name in for-in loop".to_string())?;
                    self.consume(TokenKind::In, "Expected 'in' keyword".to_string())?;
                    let iterable = self.expression()?;
                    let body = Box::new(self.statement()?);

                    return Ok(Stmt::ForIn {
                        variable,
                        iterable,
                        body,
                    });
                }
            }
        }

        // Parse as traditional for loop

        self.consume(TokenKind::LeftParen, "Expected '(' after 'for'".to_string())?;

        // Initializer
        let initializer = if self.match_tokens(&[TokenKind::Semicolon]) {
            None
        } else if self.match_tokens(&[TokenKind::Let]) {
            Some(Box::new(self.var_declaration()?))
        } else {
            Some(Box::new(self.expression_statement()?))
        };

        // Condition
        let condition = if !self.check(&TokenKind::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(
            TokenKind::Semicolon,
            "Expected ';' after loop condition".to_string(),
        )?;

        // Increment
        let increment = if !self.check(&TokenKind::RightParen) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(
            TokenKind::RightParen,
            "Expected ')' after for clauses".to_string(),
        )?;

        // Body
        let body = Box::new(self.statement()?);

        Ok(Stmt::For {
            initializer,
            condition,
            increment,
            body,
        })
    }

    /// Parses an expression statement.
    fn expression_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;

        self.consume(
            TokenKind::Semicolon,
            "Expected ';' after expression".to_string(),
        )?;

        Ok(Stmt::Expression(expr))
    }

    /// Parses an expression.
    pub fn parse(&mut self) -> Result<Expr> {
        self.expression()
    }

    /// Parses an expression.
    fn expression(&mut self) -> Result<Expr> {
        enter_parse_recursion()?;
        let result = time_parsing_operation(|| self.match_expression());
        exit_parse_recursion();
        result
    }

    /// Parse match expressions or delegate to assignment
    fn match_expression(&mut self) -> Result<Expr> {
        if self.match_tokens(&[TokenKind::Match]) {
            return self.parse_match_expression();
        }

        self.assignment()
    }

    /// Parses an assignment expression.
    fn assignment(&mut self) -> Result<Expr> {
        enter_parse_recursion()?;
        let expr = self.logical_or()?;

        if self.match_tokens(&[
            TokenKind::Assign,
            TokenKind::PlusAssign,
            TokenKind::MinusAssign,
            TokenKind::StarAssign,
            TokenKind::SlashAssign,
        ]) {
            let operator = self.previous().clone();
            let value = time_parsing_operation(|| self.assignment())?;

            // Only variables can be assigned to
            if let Expr::Variable(name) = expr {
                let op = match operator.kind {
                    TokenKind::Assign => BinaryOp::Assign,
                    TokenKind::PlusAssign => BinaryOp::PlusAssign,
                    TokenKind::MinusAssign => BinaryOp::MinusAssign,
                    TokenKind::StarAssign => BinaryOp::MultiplyAssign,
                    TokenKind::SlashAssign => BinaryOp::DivideAssign,
                    _ => unreachable!(),
                };

                return Ok(Expr::Binary(
                    Box::new(Expr::Variable(name)),
                    op,
                    Box::new(value),
                ));
            }

            // If we get here, the left side wasn't a valid assignment target
            exit_parse_recursion();
            return Err(CompileError::parse_error(
                "Invalid assignment target".to_string(),
                operator.position,
            ));
        }

        exit_parse_recursion();
        Ok(expr)
    }

    /// Parses a logical OR expression.
    fn logical_or(&mut self) -> Result<Expr> {
        let mut expr = self.logical_and()?;

        while self.match_tokens(&[TokenKind::Or]) {
            let right = self.logical_and()?;
            expr = Expr::Binary(Box::new(expr), BinaryOp::Or, Box::new(right));
        }

        Ok(expr)
    }

    /// Parses a logical AND expression.
    fn logical_and(&mut self) -> Result<Expr> {
        let mut expr = self.simd_or()?;

        while self.match_tokens(&[TokenKind::And]) {
            let right = self.simd_or()?;
            expr = Expr::Binary(Box::new(expr), BinaryOp::And, Box::new(right));
        }

        Ok(expr)
    }

    /// Parses an equality expression.
    fn equality(&mut self) -> Result<Expr> {
        let mut expr = self.comparison()?;

        while self.match_tokens(&[
            TokenKind::Equal,
            TokenKind::NotEqual,
            TokenKind::DotEqual,
            TokenKind::DotNotEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;

            match operator.kind {
                TokenKind::Equal => {
                    expr = Expr::Binary(Box::new(expr), BinaryOp::Equal, Box::new(right));
                }
                TokenKind::NotEqual => {
                    expr = Expr::Binary(Box::new(expr), BinaryOp::NotEqual, Box::new(right));
                }
                TokenKind::DotEqual => {
                    expr = Expr::SIMD(SIMDExpr::ElementWise {
                        left: Box::new(expr),
                        operator: SIMDOperator::DotEqual,
                        right: Box::new(right),
                        position: operator.position,
                    });
                }
                TokenKind::DotNotEqual => {
                    expr = Expr::SIMD(SIMDExpr::ElementWise {
                        left: Box::new(expr),
                        operator: SIMDOperator::DotNotEqual,
                        right: Box::new(right),
                        position: operator.position,
                    });
                }
                _ => unreachable!(),
            }
        }

        Ok(expr)
    }

    /// Parses a comparison expression.
    fn comparison(&mut self) -> Result<Expr> {
        let mut expr = self.enhanced_term()?;

        while self.match_tokens(&[
            TokenKind::Less,
            TokenKind::LessEqual,
            TokenKind::Greater,
            TokenKind::GreaterEqual,
            TokenKind::DotLess,
            TokenKind::DotLessEqual,
            TokenKind::DotGreater,
            TokenKind::DotGreaterEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.enhanced_term()?;

            match operator.kind {
                TokenKind::Less => {
                    expr = Expr::Binary(Box::new(expr), BinaryOp::Less, Box::new(right));
                }
                TokenKind::LessEqual => {
                    expr = Expr::Binary(Box::new(expr), BinaryOp::LessEqual, Box::new(right));
                }
                TokenKind::Greater => {
                    expr = Expr::Binary(Box::new(expr), BinaryOp::Greater, Box::new(right));
                }
                TokenKind::GreaterEqual => {
                    expr = Expr::Binary(Box::new(expr), BinaryOp::GreaterEqual, Box::new(right));
                }
                TokenKind::DotLess => {
                    expr = Expr::SIMD(SIMDExpr::ElementWise {
                        left: Box::new(expr),
                        operator: SIMDOperator::DotLess,
                        right: Box::new(right),
                        position: operator.position,
                    });
                }
                TokenKind::DotLessEqual => {
                    expr = Expr::SIMD(SIMDExpr::ElementWise {
                        left: Box::new(expr),
                        operator: SIMDOperator::DotLessEqual,
                        right: Box::new(right),
                        position: operator.position,
                    });
                }
                TokenKind::DotGreater => {
                    expr = Expr::SIMD(SIMDExpr::ElementWise {
                        left: Box::new(expr),
                        operator: SIMDOperator::DotGreater,
                        right: Box::new(right),
                        position: operator.position,
                    });
                }
                TokenKind::DotGreaterEqual => {
                    expr = Expr::SIMD(SIMDExpr::ElementWise {
                        left: Box::new(expr),
                        operator: SIMDOperator::DotGreaterEqual,
                        right: Box::new(right),
                        position: operator.position,
                    });
                }
                _ => unreachable!(),
            }
        }

        Ok(expr)
    }

    /// Parses a term (addition, subtraction).
    fn term(&mut self) -> Result<Expr> {
        let mut expr = self.factor()?;

        while self.match_tokens(&[TokenKind::Plus, TokenKind::Minus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;

            let op = match operator.kind {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Subtract,
                _ => unreachable!(),
            };

            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    /// Parses a factor (multiplication, division).
    fn factor(&mut self) -> Result<Expr> {
        let mut expr = self.unary()?;

        while self.match_tokens(&[TokenKind::Star, TokenKind::Slash, TokenKind::Percent]) {
            let operator = self.previous().clone();
            let right = self.unary()?;

            let op = match operator.kind {
                TokenKind::Star => BinaryOp::Multiply,
                TokenKind::Slash => BinaryOp::Divide,
                TokenKind::Percent => BinaryOp::Modulo,
                _ => unreachable!(),
            };

            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    /// Parses a unary expression.
    fn unary(&mut self) -> Result<Expr> {
        if self.match_tokens(&[TokenKind::Minus, TokenKind::Not, TokenKind::Ampersand]) {
            let operator = self.previous().clone();
            let right = self.unary()?;

            let op = match operator.kind {
                TokenKind::Minus => UnaryOp::Negate,
                TokenKind::Not => UnaryOp::Not,
                TokenKind::Ampersand => UnaryOp::Reference,
                _ => unreachable!(),
            };

            return Ok(Expr::Unary(op, Box::new(right)));
        }

        self.call()
    }

    /// Parses a function call or primary expression.
    fn call(&mut self) -> Result<Expr> {
        let mut expr = self.primary()?;

        loop {
            if self.match_tokens(&[TokenKind::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.match_tokens(&[TokenKind::LeftBracket]) {
                let first_expr = self.expression()?;

                // Check if this is a slice (arr[start:end]) or regular index (arr[index])
                if self.check(&TokenKind::Colon) {
                    self.advance(); // consume ':'
                    let end_expr = self.expression()?;
                    let _ = self.consume_with_recovery(
                        TokenKind::RightBracket,
                        "Expected ']' after slice end".to_string(),
                        "slice_close",
                    )?;
                    expr = Expr::Slice {
                        array: Box::new(expr),
                        start: Box::new(first_expr),
                        end: Box::new(end_expr),
                    };
                } else {
                    // Regular indexing
                    let _ = self.consume_with_recovery(
                        TokenKind::RightBracket,
                        "Expected ']' after array index".to_string(),
                        "array_index_close",
                    )?;
                    expr = Expr::Index(Box::new(expr), Box::new(first_expr));
                }
            } else if self.match_tokens(&[TokenKind::Dot]) {
                let name =
                    self.consume_identifier("Expected property name after '.'".to_string())?;
                expr = Expr::FieldAccess(Box::new(expr), name);
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// Parses the arguments of a function call.
    fn finish_call(&mut self, callee: Expr) -> Result<Expr> {
        let mut arguments = Vec::new();

        // Handle empty argument lists
        if !self.check(&TokenKind::RightParen) {
            loop {
                arguments.push(self.expression()?);
                if !self.match_tokens(&[TokenKind::Comma]) {
                    break;
                }
            }
        }

        let _ = self.consume(
            TokenKind::RightParen,
            "Expected ')' after arguments".to_string(),
        )?;

        Ok(Expr::Call(Box::new(callee), arguments))
    }

    /// Parses primary expressions: literals, variables, parentheses, SIMD vectors, and arrays
    fn primary(&mut self) -> Result<Expr> {
        // Handle literals and variables
        if self.match_tokens(&[TokenKind::True]) {
            return Ok(Expr::Literal(Literal::Boolean(true)));
        }

        if self.match_tokens(&[TokenKind::False]) {
            return Ok(Expr::Literal(Literal::Boolean(false)));
        }

        if let Some(token) = self.match_tokens_and_get(&[TokenKind::Integer(0)]) {
            if let TokenKind::Integer(n) = token.kind {
                return Ok(Expr::Literal(Literal::Integer(n)));
            }
        }

        if let Some(token) = self.match_tokens_and_get(&[TokenKind::Float(0.0)]) {
            if let TokenKind::Float(f) = token.kind {
                return Ok(Expr::Literal(Literal::Float(f)));
            }
        }

        if let Some(token) = self.match_tokens_and_get(&[TokenKind::StringLiteral("".to_string())])
        {
            if let TokenKind::StringLiteral(s) = token.kind {
                return Ok(Expr::Literal(Literal::String(s)));
            }
        }

        // Handle built-in functions (print, println, etc.)
        if !self.is_at_end() && matches!(self.peek().kind, TokenKind::Print) {
            let token = self.advance().clone();
            let func_name = "print";
            
            // This should be a function call
            if self.check(&TokenKind::LeftParen) {
                let var_expr = Expr::Variable(func_name.to_string());
                self.advance(); // consume '('
                return self.finish_call(var_expr);
            }
            
            // If not a function call, treat as variable reference  
            return Ok(Expr::Variable(func_name.to_string()));
        }

        // Handle println function
        if !self.is_at_end() && matches!(self.peek().kind, TokenKind::Println) {
            let token = self.advance().clone();
            let func_name = "println";
            
            // This should be a function call
            if self.check(&TokenKind::LeftParen) {
                let var_expr = Expr::Variable(func_name.to_string());
                self.advance(); // consume '('
                return self.finish_call(var_expr);
            }
            
            // If not a function call, treat as variable reference  
            return Ok(Expr::Variable(func_name.to_string()));
        }

        // Handle standard library types (Vec, HashMap, etc.)
        if !self.is_at_end() && matches!(self.peek().kind, 
            TokenKind::VecType | TokenKind::HashMapType | TokenKind::HashSetType | 
            TokenKind::StringType | TokenKind::FileType) {
            let token = self.advance().clone();
            let type_name = match &token.kind {
                TokenKind::VecType => "Vec",
                TokenKind::HashMapType => "HashMap", 
                TokenKind::HashSetType => "HashSet",
                TokenKind::StringType => "String",
                TokenKind::FileType => "File",
                _ => unreachable!(),
            };
            
            // Check if this is a module-scoped call (Vec::new())
            if !self.is_at_end() && matches!(self.peek().kind, TokenKind::DoubleColon) {
                self.advance(); // consume '::'
                let second_name = self.consume_identifier("Expected identifier after '::'".to_string())?;
                
                // Check if this is a function call (Vec::new())
                if self.check(&TokenKind::LeftParen) {
                    // This is a module-scoped function call like Vec::new()
                    let module_expr = Expr::Variable(type_name.to_string());
                    let field_access = Expr::FieldAccess(Box::new(module_expr), second_name);
                    self.advance(); // consume '('
                    return self.finish_call(field_access);
                }
            }
            
            // If not a static method call, treat as type literal
            return Ok(Expr::Variable(type_name.to_string()));
        }

        // Handle identifiers (variables, function calls, struct literals, enum literals)
        if !self.is_at_end() && matches!(self.peek().kind, TokenKind::Identifier(_)) {
            let token = self.advance().clone();
            if let TokenKind::Identifier(name) = token.kind {
                // Check if this is a module-scoped call or enum literal (Name::Something)
                if !self.is_at_end() && matches!(self.peek().kind, TokenKind::DoubleColon) {
                    self.advance(); // consume '::'
                    let second_name = self.consume_identifier("Expected identifier after '::'".to_string())?;
                    
                    // Check if this is a function call (Vec::new() or HashMap::new())
                    if self.check(&TokenKind::LeftParen) {
                        // This is a module-scoped function call like Vec::new()
                        let module_expr = Expr::Variable(name);
                        let field_access = Expr::FieldAccess(Box::new(module_expr), second_name);
                        self.advance(); // consume '('
                        return self.finish_call(field_access);
                    } else {
                        // This is an enum literal (EnumName::Variant) - parse it without backtracking
                        let mut args = Vec::new();
                        if self.check(&TokenKind::LeftParen) {
                            self.advance(); // consume '('
                            if !self.check(&TokenKind::RightParen) {
                                loop {
                                    args.push(self.expression()?);
                                    if !self.match_tokens(&[TokenKind::Comma]) {
                                        break;
                                    }
                                }
                            }
                            self.consume(
                                TokenKind::RightParen,
                                "Expected ')' after enum variant arguments".to_string(),
                            )?;
                        }
                        return Ok(Expr::EnumLiteral {
                            enum_name: name,
                            variant: second_name,
                            args,
                        });
                    }
                }

                // Check if this is a function call
                if self.check(&TokenKind::LeftParen) {
                    return self.parse_function_call(name);
                }

                // Check if this is a struct literal
                if self.check(&TokenKind::LeftBrace) {
                    return self.parse_struct_literal(name);
                }

                return Ok(Expr::Variable(name));
            }
        }

        // Handle built-in SIMD reduction functions
        if self.match_tokens(&[TokenKind::HorizontalSum]) {
            return self.parse_reduction_function(ReductionOp::Sum);
        }

        if self.match_tokens(&[TokenKind::HorizontalMin]) {
            return self.parse_reduction_function(ReductionOp::Min);
        }

        if self.match_tokens(&[TokenKind::HorizontalMax]) {
            return self.parse_reduction_function(ReductionOp::Max);
        }

        if self.match_tokens(&[TokenKind::DotProduct]) {
            return self.parse_dot_product_function();
        }

        if self.match_tokens(&[TokenKind::LoadVector]) {
            return self.parse_load_vector_function();
        }

        if self.match_tokens(&[TokenKind::StoreVector]) {
            return self.parse_store_vector_function();
        }

        // Handle SIMD literals that already come as a single token
        if let Some(token) = self.match_tokens_and_get(&[TokenKind::SimdLiteral("".to_string())]) {
            if let TokenKind::SimdLiteral(s) = token.kind {
                return Ok(Expr::Literal(Literal::String(s))); // Temporary - will be enhanced in SIMD type system
            }
        }

        // Handle grouping with parentheses
        if self.match_tokens(&[TokenKind::LeftParen]) {
            let expr = self.expression()?;
            self.consume(
                TokenKind::RightParen,
                "Expected ')' after expression".to_string(),
            )?;
            return Ok(Expr::Grouping(Box::new(expr)));
        }

        // Handle array literals and SIMD vector literals starting with [
        if self.match_tokens(&[TokenKind::LeftBracket]) {
            return self.parse_array_or_simd_literal();
        }

        // Handle block expressions
        if self.match_tokens(&[TokenKind::LeftBrace]) {
            return self.parse_block_expression();
        }

        // If we get here, we couldn't match any expression
        Err(CompileError::parse_error(
            format!("Expected expression, got {:?}", self.peek().kind),
            self.peek().position.clone(),
        ))
    }

    /// Parse SIMD logical OR operations (.|)
    fn simd_or(&mut self) -> Result<Expr> {
        let mut expr = self.simd_xor()?;

        while self.match_tokens(&[TokenKind::DotOr]) {
            let operator = SIMDOperator::DotOr;
            let right = self.simd_xor()?;
            let position = self.previous().position.clone();

            expr = Expr::SIMD(SIMDExpr::ElementWise {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                position,
            });
        }

        Ok(expr)
    }

    /// Parse SIMD logical XOR operations (.^)
    fn simd_xor(&mut self) -> Result<Expr> {
        let mut expr = self.simd_and()?;

        while self.match_tokens(&[TokenKind::DotXor]) {
            let operator = SIMDOperator::DotXor;
            let right = self.simd_and()?;
            let position = self.previous().position.clone();

            expr = Expr::SIMD(SIMDExpr::ElementWise {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                position,
            });
        }

        Ok(expr)
    }

    /// Parse SIMD logical AND operations (.&)
    fn simd_and(&mut self) -> Result<Expr> {
        let mut expr = self.equality()?; // Connect back to existing chain

        while self.match_tokens(&[TokenKind::DotAnd]) {
            let operator = SIMDOperator::DotAnd;
            let right = self.equality()?;
            let position = self.previous().position.clone();

            expr = Expr::SIMD(SIMDExpr::ElementWise {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                position,
            });
        }

        Ok(expr)
    }

    /// Enhanced term parsing with SIMD addition
    fn enhanced_term(&mut self) -> Result<Expr> {
        let mut expr = self.enhanced_factor()?;

        while !self.is_at_end() {
            // Fix: Check bounds instead of using while let
            let token = self.peek();
            match token.kind {
                TokenKind::DotAdd => {
                    self.advance();
                    let operator = SIMDOperator::DotAdd;
                    let right = self.enhanced_factor()?;
                    let position = self.previous().position.clone();

                    expr = Expr::SIMD(SIMDExpr::ElementWise {
                        left: Box::new(expr),
                        operator,
                        right: Box::new(right),
                        position,
                    });
                }
                TokenKind::DotSubtract => {
                    self.advance();
                    let operator = SIMDOperator::DotSubtract;
                    let right = self.enhanced_factor()?;
                    let position = self.previous().position.clone();

                    expr = Expr::SIMD(SIMDExpr::ElementWise {
                        left: Box::new(expr),
                        operator,
                        right: Box::new(right),
                        position,
                    });
                }
                TokenKind::Plus | TokenKind::Minus => {
                    let op = if matches!(token.kind, TokenKind::Plus) {
                        BinaryOp::Add
                    } else {
                        BinaryOp::Subtract
                    };
                    self.advance();
                    let right = self.enhanced_factor()?;
                    expr = Expr::Binary(Box::new(expr), op, Box::new(right));
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    /// Enhanced factor parsing with SIMD multiplication
    fn enhanced_factor(&mut self) -> Result<Expr> {
        let mut expr = self.unary()?;

        while !self.is_at_end() {
            // Fix: Check bounds instead of using while let
            let token = self.peek();
            match token.kind {
                TokenKind::DotMultiply => {
                    self.advance();
                    let operator = SIMDOperator::DotMultiply;
                    let right = self.unary()?;
                    let position = self.previous().position.clone();

                    expr = Expr::SIMD(SIMDExpr::ElementWise {
                        left: Box::new(expr),
                        operator,
                        right: Box::new(right),
                        position,
                    });
                }
                TokenKind::DotDivide => {
                    self.advance();
                    let operator = SIMDOperator::DotDivide;
                    let right = self.unary()?;
                    let position = self.previous().position.clone();

                    expr = Expr::SIMD(SIMDExpr::ElementWise {
                        left: Box::new(expr),
                        operator,
                        right: Box::new(right),
                        position,
                    });
                }
                TokenKind::Star | TokenKind::Slash | TokenKind::Percent => {
                    let operator = self.advance();
                    let op = match operator.kind {
                        TokenKind::Star => BinaryOp::Multiply,
                        TokenKind::Slash => BinaryOp::Divide,
                        TokenKind::Percent => BinaryOp::Modulo,
                        _ => unreachable!(),
                    };
                    let right = self.unary()?;
                    expr = Expr::Binary(Box::new(expr), op, Box::new(right));
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    /// Consumes the current token if it matches any of the given types.
    fn match_tokens(&mut self, types: &[TokenKind]) -> bool {
        for token_type in types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    /// Checks if the current token is of the given type.
    fn check(&self, token_type: &TokenKind) -> bool {
        if self.is_at_end() {
            return false;
        }

        // Special handling for tokens with associated values
        match (&self.peek().kind, token_type) {
            (TokenKind::Integer(_), TokenKind::Integer(_)) => true,
            (TokenKind::Float(_), TokenKind::Float(_)) => true,
            (TokenKind::StringLiteral(_), TokenKind::StringLiteral(_)) => true,
            (TokenKind::Identifier(_), TokenKind::Identifier(_)) => true,
            _ => &self.peek().kind == token_type,
        }
    }

    /// Returns the current token and advances to the next.
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    /// Consumes the current token if it's of the expected type, or errors.
    fn consume(&mut self, token_type: TokenKind, message: String) -> Result<&Token> {
        if self.check(&token_type) {
            return Ok(self.advance());
        }

        Err(CompileError::parse_error(
            message,
            self.peek().position.clone(),
        ))
    }

    /// Consumes the current token if it's an identifier, or errors.
    fn consume_identifier(&mut self, message: String) -> Result<String> {
        if self.check(&TokenKind::Identifier(String::new())) {
            let token = self.advance();
            if let TokenKind::Identifier(name) = &token.kind {
                return Ok(name.clone());
            }
        }

        Err(CompileError::parse_error(
            message,
            self.peek().position.clone(),
        ))
    }

    /// Consumes a type name (either identifier or built-in type).
    fn consume_type_name(&mut self, message: String) -> Result<String> {
        // Check for built-in type tokens first
        if self.match_tokens(&[
            TokenKind::I8,
            TokenKind::I16,
            TokenKind::I32,
            TokenKind::I64,
            TokenKind::U8,
            TokenKind::U16,
            TokenKind::U32,
            TokenKind::U64,
            TokenKind::F32,
            TokenKind::F64,
            TokenKind::Bool,
            TokenKind::String,
            // Standard library types
            TokenKind::VecType,
            TokenKind::HashMapType,
            TokenKind::HashSetType,
            TokenKind::StringType,
            TokenKind::FileType,
            // SIMD vector types
            TokenKind::F32x2,
            TokenKind::F32x4,
            TokenKind::F32x8,
            TokenKind::F32x16,
            TokenKind::F64x2,
            TokenKind::F64x4,
            TokenKind::F64x8,
            TokenKind::I32x2,
            TokenKind::I32x4,
            TokenKind::I32x8,
            TokenKind::I32x16,
            TokenKind::I64x2,
            TokenKind::I64x4,
            TokenKind::I64x8,
            TokenKind::I16x4,
            TokenKind::I16x8,
            TokenKind::I16x16,
            TokenKind::I16x32,
            TokenKind::I8x8,
            TokenKind::I8x16,
            TokenKind::I8x32,
            TokenKind::I8x64,
            TokenKind::U32x4,
            TokenKind::U32x8,
            TokenKind::U16x8,
            TokenKind::U16x16,
            TokenKind::U8x16,
            TokenKind::U8x32,
            TokenKind::Mask8,
            TokenKind::Mask16,
            TokenKind::Mask32,
            TokenKind::Mask64,
        ]) {
            let token = self.previous();
            let type_name = match &token.kind {
                TokenKind::I8 => "i8",
                TokenKind::I16 => "i16",
                TokenKind::I32 => "i32",
                TokenKind::I64 => "i64",
                TokenKind::U8 => "u8",
                TokenKind::U16 => "u16",
                TokenKind::U32 => "u32",
                TokenKind::U64 => "u64",
                TokenKind::F32 => "f32",
                TokenKind::F64 => "f64",
                TokenKind::Bool => "bool",
                TokenKind::String => "string",
                // Standard library types
                TokenKind::VecType => "Vec",
                TokenKind::HashMapType => "HashMap",
                TokenKind::HashSetType => "HashSet",
                TokenKind::StringType => "String",
                TokenKind::FileType => "File",
                // SIMD vector types
                TokenKind::F32x2 => "f32x2",
                TokenKind::F32x4 => "f32x4",
                TokenKind::F32x8 => "f32x8",
                TokenKind::F32x16 => "f32x16",
                TokenKind::F64x2 => "f64x2",
                TokenKind::F64x4 => "f64x4",
                TokenKind::F64x8 => "f64x8",
                TokenKind::I32x2 => "i32x2",
                TokenKind::I32x4 => "i32x4",
                TokenKind::I32x8 => "i32x8",
                TokenKind::I32x16 => "i32x16",
                TokenKind::I64x2 => "i64x2",
                TokenKind::I64x4 => "i64x4",
                TokenKind::I64x8 => "i64x8",
                TokenKind::I16x4 => "i16x4",
                TokenKind::I16x8 => "i16x8",
                TokenKind::I16x16 => "i16x16",
                TokenKind::I16x32 => "i16x32",
                TokenKind::I8x8 => "i8x8",
                TokenKind::I8x16 => "i8x16",
                TokenKind::I8x32 => "i8x32",
                TokenKind::I8x64 => "i8x64",
                TokenKind::U32x4 => "u32x4",
                TokenKind::U32x8 => "u32x8",
                TokenKind::U16x8 => "u16x8",
                TokenKind::U16x16 => "u16x16",
                TokenKind::U8x16 => "u8x16",
                TokenKind::U8x32 => "u8x32",
                TokenKind::Mask8 => "mask8",
                TokenKind::Mask16 => "mask16",
                TokenKind::Mask32 => "mask32",
                TokenKind::Mask64 => "mask64",
                _ => unreachable!(),
            };
            return Ok(type_name.to_string());
        }

        // Fall back to identifier for custom types
        if self.check(&TokenKind::Identifier(String::new())) {
            let token = self.advance();
            if let TokenKind::Identifier(name) = &token.kind {
                return Ok(name.clone());
            }
        }

        Err(CompileError::parse_error(
            message,
            self.peek().position.clone(),
        ))
    }

    /// Checks if we're at the end of the token stream.
    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }

    /// Parse a single statement for streaming compilation
    pub fn parse_statement(&mut self) -> Result<Option<Stmt>> {
        if self.is_at_end() {
            return Ok(None);
        }
        
        match self.declaration() {
            Ok(stmt) => {
                self.in_recovery = false;
                Ok(Some(stmt))
            }
            Err(error) => {
                self.errors.push(error.clone());
                if !self.in_recovery {
                    self.in_recovery = true;
                    self.synchronize();
                }
                // Return the error for streaming compiler to handle
                Err(error)
            }
        }
    }

    /// Get remaining tokens for streaming compilation
    pub fn get_remaining_tokens(&self) -> Vec<Token> {
        if self.current < self.tokens.len() {
            self.tokens[self.current..].to_vec()
        } else {
            Vec::new()
        }
    }

    /// Check if parser has more tokens to process
    pub fn has_more_tokens(&self) -> bool {
        self.current < self.tokens.len() && !self.is_at_end()
    }

    /// Returns the current token without consuming it.
    fn peek(&self) -> &Token {
        if self.current < self.tokens.len() {
            &self.tokens[self.current]
        } else if !self.tokens.is_empty() {
            // Return the last token if we're at the end
            &self.tokens[self.tokens.len() - 1]
        } else {
            // This should never happen but add a fallback
            panic!("Parser: No tokens available")
        }
    }

    /// Returns the previous token.
    fn previous(&self) -> &Token {
        if self.current > 0 && self.current <= self.tokens.len() {
            &self.tokens[self.current - 1]
        } else if !self.tokens.is_empty() {
            &self.tokens[0]
        } else {
            // This should never happen but add a fallback
            panic!("Parser: No tokens available")
        }
    }

    /// Parse array literals or SIMD vector literals starting with [
    fn parse_array_or_simd_literal(&mut self) -> Result<Expr> {
        let mut expressions = Vec::new();

        // Handle empty array/vector
        if self.check(&TokenKind::RightBracket) {
            self.advance(); // consume ]
            return Ok(Expr::Literal(Literal::Vector {
                elements: vec![],
                vector_type: None,
            }));
        }

        // Parse elements
        loop {
            expressions.push(self.expression()?);

            if !self.match_tokens(&[TokenKind::Comma]) {
                break;
            }
        }

        self.consume_with_recovery(
            TokenKind::RightBracket,
            "Expected ']' after array elements".to_string(),
            "array_literal_close",
        )?;

        // Check if this is a SIMD vector literal with type annotation
        if self.check_simd_type() {
            let simd_type_token_kind = self.advance().kind.clone();
            let simd_type = self.parse_simd_vector_type(&simd_type_token_kind)?;
            return Ok(Expr::SIMD(SIMDExpr::VectorLiteral {
                elements: expressions,
                vector_type: Some(simd_type),
                position: self.previous().position.clone(),
            }));
        }

        // Regular array literal - convert expressions to literals where possible
        let mut literal_elements = Vec::new();
        for expr in expressions {
            match expr {
                Expr::Literal(lit) => literal_elements.push(lit),
                _ => {
                    return Err(CompileError::parse_error(
                        "Array literals can only contain literal values".to_string(),
                        self.previous().position.clone(),
                    ));
                }
            }
        }

        Ok(Expr::Literal(Literal::Vector {
            elements: literal_elements,
            vector_type: None,
        }))
    }

    /// Check if current token is a SIMD type
    fn check_simd_type(&self) -> bool {
        if self.is_at_end() {
            return false;
        }

        matches!(
            self.peek().kind,
            TokenKind::F32x2
                | TokenKind::F32x4
                | TokenKind::F32x8
                | TokenKind::F32x16
                | TokenKind::F64x2
                | TokenKind::F64x4
                | TokenKind::F64x8
                | TokenKind::I32x2
                | TokenKind::I32x4
                | TokenKind::I32x8
                | TokenKind::I32x16
                | TokenKind::I64x2
                | TokenKind::I64x4
                | TokenKind::I64x8
                | TokenKind::I16x4
                | TokenKind::I16x8
                | TokenKind::I16x16
                | TokenKind::I16x32
                | TokenKind::I8x8
                | TokenKind::I8x16
                | TokenKind::I8x32
                | TokenKind::I8x64
                | TokenKind::U32x4
                | TokenKind::U32x8
                | TokenKind::U16x8
                | TokenKind::U16x16
                | TokenKind::U8x16
                | TokenKind::U8x32
                | TokenKind::Mask8
                | TokenKind::Mask16
                | TokenKind::Mask32
                | TokenKind::Mask64
        )
    }

    /// Helper function to match tokens and get the matched token
    fn match_tokens_and_get(&mut self, types: &[TokenKind]) -> Option<Token> {
        for token_type in types {
            if self.check_token_type(token_type) {
                return Some(self.advance().clone());
            }
        }
        None
    }

    /// Check if current token matches a specific token type (handles variants)
    fn check_token_type(&self, token_type: &TokenKind) -> bool {
        if self.is_at_end() {
            return false;
        }

        match (token_type, &self.peek().kind) {
            (TokenKind::Integer(_), TokenKind::Integer(_)) => true,
            (TokenKind::Float(_), TokenKind::Float(_)) => true,
            (TokenKind::StringLiteral(_), TokenKind::StringLiteral(_)) => true,
            (TokenKind::Identifier(_), TokenKind::Identifier(_)) => true,
            (TokenKind::SimdLiteral(_), TokenKind::SimdLiteral(_)) => true,
            (a, b) => a == b,
        }
    }

    /// Parse SIMD vector type from token
    fn parse_simd_vector_type(&self, token_kind: &TokenKind) -> Result<SIMDVectorType> {
        use crate::ast::SIMDVectorType;

        let simd_type = match token_kind {
            TokenKind::F32x2 => SIMDVectorType::F32x2,
            TokenKind::F32x4 => SIMDVectorType::F32x4,
            TokenKind::F32x8 => SIMDVectorType::F32x8,
            TokenKind::F32x16 => SIMDVectorType::F32x16,
            TokenKind::F64x2 => SIMDVectorType::F64x2,
            TokenKind::F64x4 => SIMDVectorType::F64x4,
            TokenKind::F64x8 => SIMDVectorType::F64x8,
            TokenKind::I32x2 => SIMDVectorType::I32x2,
            TokenKind::I32x4 => SIMDVectorType::I32x4,
            TokenKind::I32x8 => SIMDVectorType::I32x8,
            TokenKind::I32x16 => SIMDVectorType::I32x16,
            TokenKind::I64x2 => SIMDVectorType::I64x2,
            TokenKind::I64x4 => SIMDVectorType::I64x4,
            TokenKind::I64x8 => SIMDVectorType::I64x8,
            TokenKind::I16x4 => SIMDVectorType::I16x4,
            TokenKind::I16x8 => SIMDVectorType::I16x8,
            TokenKind::I16x16 => SIMDVectorType::I16x16,
            TokenKind::I16x32 => SIMDVectorType::I16x32,
            TokenKind::I8x8 => SIMDVectorType::I8x8,
            TokenKind::I8x16 => SIMDVectorType::I8x16,
            TokenKind::I8x32 => SIMDVectorType::I8x32,
            TokenKind::I8x64 => SIMDVectorType::I8x64,
            TokenKind::U32x4 => SIMDVectorType::U32x4,
            TokenKind::U32x8 => SIMDVectorType::U32x8,
            TokenKind::U16x8 => SIMDVectorType::U16x8,
            TokenKind::U16x16 => SIMDVectorType::U16x16,
            TokenKind::U8x16 => SIMDVectorType::U8x16,
            TokenKind::U8x32 => SIMDVectorType::U8x32,
            TokenKind::Mask8 => SIMDVectorType::Mask8,
            TokenKind::Mask16 => SIMDVectorType::Mask16,
            TokenKind::Mask32 => SIMDVectorType::Mask32,
            TokenKind::Mask64 => SIMDVectorType::Mask64,
            _ => {
                return Err(CompileError::parse_error(
                    format!("Invalid SIMD vector type: {:?}", token_kind),
                    self.previous().position.clone(),
                ));
            }
        };

        Ok(simd_type)
    }

    /// Parse a function call expression
    fn parse_function_call(&mut self, name: String) -> Result<Expr> {
        self.consume_with_recovery(
            TokenKind::LeftParen,
            "Expected '(' after function name".to_string(),
            "function_call_open",
        )?;

        let mut arguments = Vec::new();

        if !self.check(&TokenKind::RightParen) {
            loop {
                arguments.push(self.expression()?);
                if !self.match_tokens(&[TokenKind::Comma]) {
                    break;
                }
            }
        }

        self.consume_with_recovery(
            TokenKind::RightParen,
            "Expected ')' after arguments".to_string(),
            "function_call_close",
        )?;

        Ok(Expr::Call(Box::new(Expr::Variable(name)), arguments))
    }

    /// Parse a SIMD reduction function call
    fn parse_reduction_function(&mut self, operation: ReductionOp) -> Result<Expr> {
        self.consume_with_recovery(
            TokenKind::LeftParen,
            "Expected '(' after reduction function".to_string(),
            "reduction_function_open",
        )?;

        let vector = self.expression()?;

        self.consume_with_recovery(
            TokenKind::RightParen,
            "Expected ')' after vector argument".to_string(),
            "reduction_function_close",
        )?;

        let position = self.previous().position.clone();

        Ok(Expr::SIMD(SIMDExpr::Reduction {
            vector: Box::new(vector),
            operation,
            position,
        }))
    }

    /// Parse a SIMD dot product function call
    fn parse_dot_product_function(&mut self) -> Result<Expr> {
        self.consume(
            TokenKind::LeftParen,
            "Expected '(' after dot_product".to_string(),
        )?;

        let left = self.expression()?;

        self.consume(
            TokenKind::Comma,
            "Expected ',' between dot product arguments".to_string(),
        )?;

        let right = self.expression()?;

        self.consume(
            TokenKind::RightParen,
            "Expected ')' after dot product arguments".to_string(),
        )?;

        let position = self.previous().position.clone();

        Ok(Expr::SIMD(SIMDExpr::DotProduct {
            left: Box::new(left),
            right: Box::new(right),
            position,
        }))
    }

    /// Parse load_vector(address, vector_type) function call
    fn parse_load_vector_function(&mut self) -> Result<Expr> {
        self.consume(
            TokenKind::LeftParen,
            "Expected '(' after load_vector".to_string(),
        )?;

        let address = self.expression()?;

        self.consume(
            TokenKind::Comma,
            "Expected ',' between load_vector arguments".to_string(),
        )?;

        // Parse the vector type identifier
        let vector_type = if let Some(token) =
            self.match_tokens_and_get(&[TokenKind::Identifier("".to_string())])
        {
            self.parse_simd_vector_type(&token.kind)?
        } else {
            return Err(CompileError::parse_error(
                "Expected vector type identifier".to_string(),
                self.peek().position.clone(),
            ));
        };

        // Optional alignment parameter
        let alignment = if self.match_tokens(&[TokenKind::Comma]) {
            Some(self.parse_alignment()?)
        } else {
            None
        };

        self.consume(
            TokenKind::RightParen,
            "Expected ')' after load_vector arguments".to_string(),
        )?;

        let position = self.previous().position.clone();

        Ok(Expr::SIMD(SIMDExpr::VectorLoad {
            address: Box::new(address),
            vector_type,
            alignment,
            position,
        }))
    }

    /// Parse store_vector(address, vector) function call
    fn parse_store_vector_function(&mut self) -> Result<Expr> {
        self.consume(
            TokenKind::LeftParen,
            "Expected '(' after store_vector".to_string(),
        )?;

        let address = self.expression()?;

        self.consume(
            TokenKind::Comma,
            "Expected ',' between store_vector arguments".to_string(),
        )?;

        let vector = self.expression()?;

        // Optional alignment parameter
        let alignment = if self.match_tokens(&[TokenKind::Comma]) {
            Some(self.parse_alignment()?)
        } else {
            None
        };

        self.consume(
            TokenKind::RightParen,
            "Expected ')' after store_vector arguments".to_string(),
        )?;

        let position = self.previous().position.clone();

        Ok(Expr::SIMD(SIMDExpr::VectorStore {
            address: Box::new(address),
            vector: Box::new(vector),
            alignment,
            position,
        }))
    }

    /// Parse alignment parameter (expects integer literal)
    fn parse_alignment(&mut self) -> Result<u32> {
        if let Some(token) = self.match_tokens_and_get(&[TokenKind::Integer(0)]) {
            if let TokenKind::Integer(value) = token.kind {
                let alignment = value as u32;
                // Validate alignment is power of 2 and reasonable
                if alignment.is_power_of_two() && alignment >= 1 && alignment <= 64 {
                    Ok(alignment)
                } else {
                    Err(CompileError::parse_error(
                        "Alignment must be a power of 2 between 1 and 64".to_string(),
                        token.position,
                    ))
                }
            } else {
                Err(CompileError::parse_error(
                    "Invalid alignment value".to_string(),
                    token.position,
                ))
            }
        } else {
            Err(CompileError::parse_error(
                "Expected alignment value".to_string(),
                self.peek().position.clone(),
            ))
        }
    }

    /// Parse struct literal: StructName { field1: value1, field2: value2 }
    fn parse_struct_literal(&mut self, struct_name: String) -> Result<Expr> {
        self.consume(
            TokenKind::LeftBrace,
            "Expected '{' for struct literal".to_string(),
        )?;

        let mut fields = Vec::new();

        if !self.check(&TokenKind::RightBrace) {
            loop {
                let field_name = self.consume_identifier("Expected field name".to_string())?;

                self.consume(
                    TokenKind::Colon,
                    "Expected ':' after field name".to_string(),
                )?;

                let field_value = self.expression()?;

                fields.push(crate::ast::StructFieldInit {
                    name: field_name,
                    value: field_value,
                });

                if !self.match_tokens(&[TokenKind::Comma]) {
                    break;
                }
            }
        }

        self.consume(
            TokenKind::RightBrace,
            "Expected '}' after struct fields".to_string(),
        )?;

        Ok(Expr::StructLiteral {
            name: struct_name,
            fields,
        })
    }

    /// Parse enum literal: EnumName::Variant or EnumName::Variant(args)
    fn parse_enum_literal(&mut self, enum_name: String) -> Result<Expr> {
        // Consume the DoubleColon token directly since check() has issues with it
        if !self.is_at_end() && matches!(self.peek().kind, TokenKind::DoubleColon) {
            self.advance();
        } else {
            return Err(CompileError::parse_error(
                "Expected '::' for enum variant".to_string(),
                self.peek().position.clone(),
            ));
        }

        let variant_name = self.consume_identifier("Expected variant name".to_string())?;

        let mut args = Vec::new();

        // Check if variant has arguments
        if self.match_tokens(&[TokenKind::LeftParen]) {
            if !self.check(&TokenKind::RightParen) {
                loop {
                    args.push(self.expression()?);

                    if !self.match_tokens(&[TokenKind::Comma]) {
                        break;
                    }
                }
            }

            self.consume(
                TokenKind::RightParen,
                "Expected ')' after enum variant arguments".to_string(),
            )?;
        }

        Ok(Expr::EnumLiteral {
            enum_name,
            variant: variant_name,
            args,
        })
    }

    /// Parse a match value (without struct literal lookahead that conflicts with match arms)
    fn match_value(&mut self) -> Result<Expr> {
        self.match_value_logical_or()
    }

    /// Logical OR for match values (avoids struct literal parsing)
    fn match_value_logical_or(&mut self) -> Result<Expr> {
        let mut expr = self.match_value_logical_and()?;

        while self.match_tokens(&[TokenKind::Or]) {
            let right = self.match_value_logical_and()?;
            expr = Expr::Binary(Box::new(expr), BinaryOp::Or, Box::new(right));
        }

        Ok(expr)
    }

    /// Logical AND for match values (avoids struct literal parsing)
    fn match_value_logical_and(&mut self) -> Result<Expr> {
        let mut expr = self.match_value_simd_or()?;

        while self.match_tokens(&[TokenKind::And]) {
            let right = self.match_value_simd_or()?;
            expr = Expr::Binary(Box::new(expr), BinaryOp::And, Box::new(right));
        }

        Ok(expr)
    }

    /// SIMD OR for match values (avoids struct literal parsing)
    fn match_value_simd_or(&mut self) -> Result<Expr> {
        let mut expr = self.match_value_equality()?;

        while self.match_tokens(&[TokenKind::DotOr]) {
            let right = self.match_value_equality()?;
            expr = Expr::SIMD(SIMDExpr::ElementWise {
                left: Box::new(expr),
                operator: SIMDOperator::DotOr,
                right: Box::new(right),
                position: self.previous().position.clone(),
            });
        }

        Ok(expr)
    }

    /// Equality for match values (avoids struct literal parsing)
    fn match_value_equality(&mut self) -> Result<Expr> {
        let mut expr = self.match_value_comparison()?;

        while self.match_tokens(&[
            TokenKind::Equal,
            TokenKind::NotEqual,
            TokenKind::DotEqual,
            TokenKind::DotNotEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.match_value_comparison()?;

            match operator.kind {
                TokenKind::Equal => {
                    expr = Expr::Binary(Box::new(expr), BinaryOp::Equal, Box::new(right));
                }
                TokenKind::NotEqual => {
                    expr = Expr::Binary(Box::new(expr), BinaryOp::NotEqual, Box::new(right));
                }
                TokenKind::DotEqual => {
                    expr = Expr::SIMD(SIMDExpr::ElementWise {
                        left: Box::new(expr),
                        operator: SIMDOperator::DotEqual,
                        right: Box::new(right),
                        position: operator.position,
                    });
                }
                TokenKind::DotNotEqual => {
                    expr = Expr::SIMD(SIMDExpr::ElementWise {
                        left: Box::new(expr),
                        operator: SIMDOperator::DotNotEqual,
                        right: Box::new(right),
                        position: operator.position,
                    });
                }
                _ => unreachable!(),
            }
        }

        Ok(expr)
    }

    /// Comparison for match values (avoids struct literal parsing)
    fn match_value_comparison(&mut self) -> Result<Expr> {
        let mut expr = self.match_value_primary()?;

        while self.match_tokens(&[
            TokenKind::Greater,
            TokenKind::GreaterEqual,
            TokenKind::Less,
            TokenKind::LessEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.match_value_primary()?;

            let binary_op = match operator.kind {
                TokenKind::Greater => BinaryOp::Greater,
                TokenKind::GreaterEqual => BinaryOp::GreaterEqual,
                TokenKind::Less => BinaryOp::Less,
                TokenKind::LessEqual => BinaryOp::LessEqual,
                _ => unreachable!(),
            };

            expr = Expr::Binary(Box::new(expr), binary_op, Box::new(right));
        }

        Ok(expr)
    }

    /// Primary expression for match values (WITHOUT struct literal lookahead)
    fn match_value_primary(&mut self) -> Result<Expr> {
        // Handle literals
        if let Some(token) = self.match_tokens_and_get(&[TokenKind::Integer(0)]) {
            if let TokenKind::Integer(value) = token.kind {
                return Ok(Expr::Literal(Literal::Integer(value)));
            }
        }

        if let Some(token) = self.match_tokens_and_get(&[TokenKind::Float(0.0)]) {
            if let TokenKind::Float(value) = token.kind {
                return Ok(Expr::Literal(Literal::Float(value)));
            }
        }

        if let Some(token) = self.match_tokens_and_get(&[TokenKind::StringLiteral("".to_string())])
        {
            if let TokenKind::StringLiteral(value) = token.kind {
                return Ok(Expr::Literal(Literal::String(value)));
            }
        }

        if self.match_tokens(&[TokenKind::True]) {
            return Ok(Expr::Literal(Literal::Boolean(true)));
        }

        if self.match_tokens(&[TokenKind::False]) {
            return Ok(Expr::Literal(Literal::Boolean(false)));
        }

        // Handle identifiers (variables, function calls, enum literals)
        if let Some(token) = self.match_tokens_and_get(&[TokenKind::Identifier("".to_string())]) {
            if let TokenKind::Identifier(name) = token.kind {
                // Check for enum literal with ::
                if self.match_tokens(&[TokenKind::DoubleColon]) {
                    let variant_token =
                        self.consume_identifier("Expected variant name after '::'".to_string())?;

                    // Check for variant with arguments: EnumName::Variant(args)
                    if self.check(&TokenKind::LeftParen) {
                        self.advance();
                        let mut args = Vec::new();

                        if !self.check(&TokenKind::RightParen) {
                            loop {
                                args.push(self.expression()?);
                                if !self.match_tokens(&[TokenKind::Comma]) {
                                    break;
                                }
                            }
                        }

                        self.consume(
                            TokenKind::RightParen,
                            "Expected ')' after enum variant arguments".to_string(),
                        )?;

                        return Ok(Expr::EnumLiteral {
                            enum_name: name,
                            variant: variant_token,
                            args,
                        });
                    } else {
                        // Simple enum literal without arguments
                        return Ok(Expr::EnumLiteral {
                            enum_name: name,
                            variant: variant_token,
                            args: vec![],
                        });
                    }
                }

                // Check for function call
                if self.check(&TokenKind::LeftParen) {
                    return self.parse_function_call(name);
                }

                // NOTE: NO struct literal check here - this is the key difference

                return Ok(Expr::Variable(name));
            }
        }

        // Handle grouping with parentheses
        if self.match_tokens(&[TokenKind::LeftParen]) {
            let expr = self.expression()?;
            self.consume(
                TokenKind::RightParen,
                "Expected ')' after expression".to_string(),
            )?;
            return Ok(expr);
        }

        // Handle unary expressions
        if self.match_tokens(&[TokenKind::Minus, TokenKind::Not, TokenKind::Ampersand]) {
            let operator = self.previous().clone();
            let expr = self.match_value_primary()?;

            let unary_op = match operator.kind {
                TokenKind::Minus => UnaryOp::Negate,
                TokenKind::Not => UnaryOp::Not,
                TokenKind::Ampersand => UnaryOp::Reference,
                _ => unreachable!(),
            };

            return Ok(Expr::Unary(unary_op, Box::new(expr)));
        }

        Err(CompileError::parse_error(
            "Expected expression".to_string(),
            self.peek().position.clone(),
        ))
    }

    /// Parse match expression: match value { pattern => expr, ... }
    fn parse_match_expression(&mut self) -> Result<Expr> {
        let value = Box::new(self.match_value()?);

        self.consume(
            TokenKind::LeftBrace,
            "Expected '{' after match value".to_string(),
        )?;

        let mut arms = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            let pattern = self.parse_pattern()?;

            self.consume(
                TokenKind::FatArrow, // =>
                "Expected '=>' after pattern".to_string(),
            )?;

            let expression = self.expression()?;

            arms.push(MatchArm {
                pattern,
                expression,
            });

            // Optional comma between arms
            if self.match_tokens(&[TokenKind::Comma]) {
                // Allow trailing comma
                if self.check(&TokenKind::RightBrace) {
                    break;
                }
            }
        }

        self.consume(
            TokenKind::RightBrace,
            "Expected '}' after match arms".to_string(),
        )?;

        Ok(Expr::Match { value, arms })
    }

    /// Parse a pattern in a match arm
    fn parse_pattern(&mut self) -> Result<Pattern> {
        // Handle identifier patterns (variable, wildcard, enum variant)
        if self.check_token_type(&TokenKind::Identifier("".to_string())) {
            let token = self.advance().clone();
            if let TokenKind::Identifier(name) = token.kind {
                if name == "_" {
                    return Ok(Pattern::Wildcard);
                }

                // Check if this is an enum variant pattern (Name::Variant)
                if self.check(&TokenKind::DoubleColon) {
                    return self.parse_enum_variant_pattern(name);
                }

                // Otherwise, it's a variable pattern
                return Ok(Pattern::Variable(name));
            }
        }

        // Handle literals
        if self.match_tokens(&[TokenKind::True]) {
            return Ok(Pattern::Literal(Literal::Boolean(true)));
        }

        if self.match_tokens(&[TokenKind::False]) {
            return Ok(Pattern::Literal(Literal::Boolean(false)));
        }

        if let Some(token) = self.match_tokens_and_get(&[TokenKind::Integer(0)]) {
            if let TokenKind::Integer(n) = token.kind {
                return Ok(Pattern::Literal(Literal::Integer(n)));
            }
        }

        if let Some(token) = self.match_tokens_and_get(&[TokenKind::StringLiteral("".to_string())])
        {
            if let TokenKind::StringLiteral(s) = token.kind {
                return Ok(Pattern::Literal(Literal::String(s)));
            }
        }

        Err(CompileError::parse_error(
            "Expected pattern".to_string(),
            self.peek().position.clone(),
        ))
    }

    /// Parse a block expression: { statements... }
    fn parse_block_expression(&mut self) -> Result<Expr> {
        let mut statements = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(
            TokenKind::RightBrace,
            "Expected '}' after block".to_string(),
        )?;

        Ok(Expr::Block(statements))
    }

    /// Parse enum variant pattern: EnumName::Variant(patterns...)
    fn parse_enum_variant_pattern(&mut self, enum_name: String) -> Result<Pattern> {
        self.consume(
            TokenKind::DoubleColon,
            "Expected '::' for enum variant pattern".to_string(),
        )?;

        let variant_name = self.consume_identifier("Expected variant name".to_string())?;

        let mut sub_patterns = Vec::new();

        // Check if variant has sub-patterns
        if self.match_tokens(&[TokenKind::LeftParen]) {
            if !self.check(&TokenKind::RightParen) {
                loop {
                    sub_patterns.push(self.parse_pattern()?);

                    if !self.match_tokens(&[TokenKind::Comma]) {
                        break;
                    }
                }
            }

            self.consume(
                TokenKind::RightParen,
                "Expected ')' after variant patterns".to_string(),
            )?;
        }

        Ok(Pattern::EnumVariant {
            enum_name,
            variant: variant_name,
            patterns: sub_patterns,
        })
    }

    // ==================== ERROR RECOVERY METHODS ====================

    /// Synchronize to a statement boundary for error recovery
    fn synchronize(&mut self) {
        while !self.is_at_end() {
            // Skip to next statement boundary
            if matches!(self.previous().kind, TokenKind::Semicolon) {
                return;
            }

            // Or to next declaration keyword
            if matches!(
                self.peek().kind,
                TokenKind::Func
                    | TokenKind::Let
                    | TokenKind::Struct
                    | TokenKind::Enum
                    | TokenKind::If
                    | TokenKind::While
                    | TokenKind::For
                    | TokenKind::Return
            ) {
                return;
            }

            self.advance();
        }
    }

    /// Recover from a parse error with suggestions
    fn recover_from_parse_error(&mut self, error: CompileError) -> RecoveryAction {
        let suggestions = self.suggest_fixes(&error);
        
        // Log suggestions (in a real implementation, these would be shown to the user)
        for suggestion in suggestions {
            eprintln!("💡 Suggestion: {}", suggestion.message);
            if let Some(fix) = suggestion.suggested_fix {
                eprintln!("   Try: {}", fix);
            }
        }

        // Determine recovery action based on error context
        match &error {
            CompileError::ParseError { message, .. } => {
                if message.contains("Expected ';'") {
                    RecoveryAction::Insert(TokenKind::Semicolon)
                } else if message.contains("Expected ')'") {
                    RecoveryAction::Insert(TokenKind::RightParen)
                } else if message.contains("Expected '}'") {
                    RecoveryAction::Insert(TokenKind::RightBrace)
                } else if message.contains("Expected ']'") {
                    RecoveryAction::Insert(TokenKind::RightBracket)
                } else {
                    RecoveryAction::Synchronize
                }
            }
            _ => RecoveryAction::Synchronize,
        }
    }

    /// Generate intelligent error suggestions
    fn suggest_fixes(&self, error: &CompileError) -> Vec<ErrorSuggestion> {
        let mut suggestions = Vec::new();

        match error {
            CompileError::ParseError { message, position: _ } => {
                // Common typo corrections
                if message.contains("Expected identifier") {
                    suggestions.push(ErrorSuggestion {
                        message: "Check for typos in variable or function names".to_string(),
                        suggested_fix: None,
                    });
                }

                if message.contains("Expected ';'") {
                    suggestions.push(ErrorSuggestion {
                        message: "Missing semicolon after statement".to_string(),
                        suggested_fix: Some("Add ';' at the end of the statement".to_string()),
                    });
                }

                if message.contains("Expected ')'") {
                    suggestions.push(ErrorSuggestion {
                        message: "Unmatched parenthesis".to_string(),
                        suggested_fix: Some("Add closing ')' or check for extra opening '('".to_string()),
                    });
                }

                if message.contains("Expected '}'") {
                    suggestions.push(ErrorSuggestion {
                        message: "Unmatched brace".to_string(),
                        suggested_fix: Some("Add closing '}' or check for extra opening '{'".to_string()),
                    });
                }

                if message.contains("Expected type") {
                    suggestions.push(ErrorSuggestion {
                        message: "Type annotation required".to_string(),
                        suggested_fix: Some("Add type annotation like ': i32' or ': f32'".to_string()),
                    });
                }

                // SIMD-specific suggestions
                if message.contains("SIMD") {
                    suggestions.push(ErrorSuggestion {
                        message: "SIMD syntax error".to_string(),
                        suggested_fix: Some("Use SIMD vector types like f32x4, i32x4, or element-wise operators like .+, .*, .-".to_string()),
                    });
                }

                // Function syntax suggestions
                if message.contains("function") || message.contains("func") {
                    suggestions.push(ErrorSuggestion {
                        message: "Function declaration syntax".to_string(),
                        suggested_fix: Some("Use 'func name(param: type) -> return_type { ... }'".to_string()),
                    });
                }

                // Variable declaration suggestions
                if message.contains("variable") || message.contains("let") {
                    suggestions.push(ErrorSuggestion {
                        message: "Variable declaration syntax".to_string(),
                        suggested_fix: Some("Use 'let name: type = value;' or 'let name = value;'".to_string()),
                    });
                }

                // Control flow suggestions
                if message.contains("if") || message.contains("while") || message.contains("for") {
                    suggestions.push(ErrorSuggestion {
                        message: "Control flow syntax".to_string(),
                        suggested_fix: Some("Check condition syntax and braces: 'if (condition) { ... }'".to_string()),
                    });
                }
            }
            _ => {}
        }

        // Add general suggestions if no specific ones found
        if suggestions.is_empty() {
            suggestions.push(ErrorSuggestion {
                message: "Check syntax and refer to Eä language documentation".to_string(),
                suggested_fix: None,
            });
        }

        suggestions
    }

    /// Attempt to recover and continue parsing after an error
    fn attempt_recovery(&mut self, expected: &[TokenKind], _context: &str) -> Option<RecoveryAction> {
        // Try common recovery strategies
        
        // 1. Check for missing semicolon
        if self.check(&TokenKind::Identifier("".to_string())) || 
           self.check(&TokenKind::Let) || 
           self.check(&TokenKind::Func) {
            return Some(RecoveryAction::Insert(TokenKind::Semicolon));
        }

        // 2. Check for missing closing punctuation
        for expected_token in expected {
            if matches!(expected_token, 
                TokenKind::RightParen | 
                TokenKind::RightBrace | 
                TokenKind::RightBracket | 
                TokenKind::Semicolon) {
                return Some(RecoveryAction::Insert(expected_token.clone()));
            }
        }

        // 3. Check for typos in keywords
        if let TokenKind::Identifier(name) = &self.peek().kind {
            let suggestions = self.suggest_keyword_corrections(name);
            if !suggestions.is_empty() {
                // For now, just skip the incorrect identifier
                return Some(RecoveryAction::Skip);
            }
        }

        // 4. Default to synchronization
        Some(RecoveryAction::Synchronize)
    }

    /// Suggest corrections for mistyped keywords
    fn suggest_keyword_corrections(&self, identifier: &str) -> Vec<String> {
        let keywords = vec![
            "func", "let", "if", "else", "while", "for", "return", "struct", "enum",
            "match", "true", "false", "vectorize", "unroll", "align", "reduce",
            "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64",
            "bool", "string", "f32x4", "i32x4", "f64x2"
        ];
        
        let mut suggestions = Vec::new();
        
        for keyword in keywords {
            if Self::levenshtein_distance(identifier, keyword) <= 2 {
                suggestions.push(keyword.to_string());
            }
        }
        
        suggestions
    }

    /// Calculate Levenshtein distance for typo detection
    pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
        let len1 = s1.len();
        let len2 = s2.len();
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        for (i, c1) in s1.chars().enumerate() {
            for (j, c2) in s2.chars().enumerate() {
                let cost = if c1 == c2 { 0 } else { 1 };
                matrix[i + 1][j + 1] = std::cmp::min(
                    std::cmp::min(
                        matrix[i][j + 1] + 1,     // deletion
                        matrix[i + 1][j] + 1,     // insertion
                    ),
                    matrix[i][j] + cost,          // substitution
                );
            }
        }

        matrix[len1][len2]
    }

    /// Enhanced consume method with better error reporting
    fn consume_with_recovery(&mut self, expected: TokenKind, message: String, context: &str) -> Result<&Token> {
        if self.check(&expected) {
            return Ok(self.advance());
        }

        // Create detailed error context
        let _error_context = ErrorContext {
            expected: vec![expected.clone()],
            found: self.peek().kind.clone(),
            position: self.peek().position.clone(),
            context: context.to_string(),
        };

        // Try recovery
        if let Some(recovery_action) = self.attempt_recovery(&[expected.clone()], context) {
            match recovery_action {
                RecoveryAction::Insert(token_kind) => {
                    eprintln!("🔧 Auto-inserting missing {:?}", token_kind);
                    // Note: In a real implementation, we might insert a synthetic token
                    // For now, we'll just continue
                }
                RecoveryAction::Skip => {
                    eprintln!("⏭️  Skipping unexpected token: {:?}", self.peek().kind);
                    self.advance();
                    return self.consume_with_recovery(expected, message, context);
                }
                RecoveryAction::Synchronize => {
                    self.synchronize();
                }
                RecoveryAction::Replace(_) => {
                    eprintln!("🔄 Replacing token and continuing");
                    self.advance();
                }
            }
        }

        Err(CompileError::parse_error(message, self.peek().position.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BinaryOp, Expr, Literal, UnaryOp};
    use crate::lexer::Lexer;

    fn parse_expr(source: &str) -> Result<Expr> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize_all()?;
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    #[test]
    fn test_parse_integer_literal() {
        let expr = parse_expr("42").unwrap();
        assert_eq!(expr, Expr::Literal(Literal::Integer(42)));
    }

    #[test]
    fn test_parse_float_literal() {
        let expr = parse_expr("3.14").unwrap();
        assert_eq!(expr, Expr::Literal(Literal::Float(3.14)));
    }

    #[test]
    fn test_parse_string_literal() {
        let expr = parse_expr(r#""hello""#).unwrap();
        assert_eq!(expr, Expr::Literal(Literal::String("hello".to_string())));
    }

    #[test]
    fn test_parse_boolean_literal() {
        let expr1 = parse_expr("true").unwrap();
        let expr2 = parse_expr("false").unwrap();
        assert_eq!(expr1, Expr::Literal(Literal::Boolean(true)));
        assert_eq!(expr2, Expr::Literal(Literal::Boolean(false)));
    }

    #[test]
    fn test_parse_variable() {
        let expr = parse_expr("foo").unwrap();
        assert_eq!(expr, Expr::Variable("foo".to_string()));
    }

    #[test]
    fn test_parse_unary_expression() {
        let expr1 = parse_expr("-42").unwrap();
        let expr2 = parse_expr("!true").unwrap();

        assert_eq!(
            expr1,
            Expr::Unary(
                UnaryOp::Negate,
                Box::new(Expr::Literal(Literal::Integer(42)))
            )
        );

        assert_eq!(
            expr2,
            Expr::Unary(
                UnaryOp::Not,
                Box::new(Expr::Literal(Literal::Boolean(true)))
            )
        );
    }

    #[test]
    fn test_parse_binary_expression() {
        let expr = parse_expr("1 + 2").unwrap();

        assert_eq!(
            expr,
            Expr::Binary(
                Box::new(Expr::Literal(Literal::Integer(1))),
                BinaryOp::Add,
                Box::new(Expr::Literal(Literal::Integer(2)))
            )
        );
    }

    #[test]
    fn test_parse_complex_expression() {
        let expr = parse_expr("1 + 2 * 3").unwrap();

        // Should be parsed as 1 + (2 * 3) due to precedence
        assert_eq!(
            expr,
            Expr::Binary(
                Box::new(Expr::Literal(Literal::Integer(1))),
                BinaryOp::Add,
                Box::new(Expr::Binary(
                    Box::new(Expr::Literal(Literal::Integer(2))),
                    BinaryOp::Multiply,
                    Box::new(Expr::Literal(Literal::Integer(3)))
                ))
            )
        );
    }

    #[test]
    fn test_parse_grouping() {
        let expr = parse_expr("(1 + 2) * 3").unwrap();

        // Should be parsed as (1 + 2) * 3
        assert_eq!(
            expr,
            Expr::Binary(
                Box::new(Expr::Grouping(Box::new(Expr::Binary(
                    Box::new(Expr::Literal(Literal::Integer(1))),
                    BinaryOp::Add,
                    Box::new(Expr::Literal(Literal::Integer(2)))
                )))),
                BinaryOp::Multiply,
                Box::new(Expr::Literal(Literal::Integer(3)))
            )
        );
    }

    #[test]
    fn test_parse_function_call() {
        let expr = parse_expr("foo(1, 2)").unwrap();

        assert_eq!(
            expr,
            Expr::Call(
                Box::new(Expr::Variable("foo".to_string())),
                vec![
                    Expr::Literal(Literal::Integer(1)),
                    Expr::Literal(Literal::Integer(2))
                ]
            )
        );
    }

    #[test]
    fn test_parse_array_indexing() {
        let expr = parse_expr("array[1 + 2]").unwrap();

        assert_eq!(
            expr,
            Expr::Index(
                Box::new(Expr::Variable("array".to_string())),
                Box::new(Expr::Binary(
                    Box::new(Expr::Literal(Literal::Integer(1))),
                    BinaryOp::Add,
                    Box::new(Expr::Literal(Literal::Integer(2)))
                ))
            )
        );
    }

    #[test]
    fn test_parse_field_access() {
        let expr = parse_expr("object.field").unwrap();

        assert_eq!(
            expr,
            Expr::FieldAccess(
                Box::new(Expr::Variable("object".to_string())),
                "field".to_string()
            )
        );
    }

    #[test]
    fn test_parse_assignment() {
        let expr = parse_expr("x = 42").unwrap();

        assert_eq!(
            expr,
            Expr::Binary(
                Box::new(Expr::Variable("x".to_string())),
                BinaryOp::Assign,
                Box::new(Expr::Literal(Literal::Integer(42)))
            )
        );
    }

    #[test]
    fn test_parse_compound_assignment() {
        let expr1 = parse_expr("x += 5").unwrap();
        let expr2 = parse_expr("y *= z").unwrap();

        assert_eq!(
            expr1,
            Expr::Binary(
                Box::new(Expr::Variable("x".to_string())),
                BinaryOp::PlusAssign,
                Box::new(Expr::Literal(Literal::Integer(5)))
            )
        );

        assert_eq!(
            expr2,
            Expr::Binary(
                Box::new(Expr::Variable("y".to_string())),
                BinaryOp::MultiplyAssign,
                Box::new(Expr::Variable("z".to_string()))
            )
        );
    }

    // Helper function to parse a single statement
    fn parse_statement(source: &str) -> Result<Stmt> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize_all()?;
        let mut parser = Parser::new(tokens);
        parser.declaration()
    }

    #[test]
    fn test_parse_variable_declaration() {
        let source = "let x = 42;";
        let result = parse_statement(source).unwrap();

        if let Stmt::VarDeclaration {
            name,
            type_annotation,
            initializer,
        } = result
        {
            assert_eq!(name, "x");
            assert!(type_annotation.is_none());
            assert_eq!(initializer.unwrap(), Expr::Literal(Literal::Integer(42)));
        } else {
            panic!("Expected variable declaration, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_typed_variable_declaration() {
        let source = "let mut y: i32 = 10;";
        let result = parse_statement(source).unwrap();

        if let Stmt::VarDeclaration {
            name,
            type_annotation,
            initializer,
        } = result
        {
            assert_eq!(name, "y");

            assert!(type_annotation.is_some());
            let type_ann = type_annotation.unwrap();
            assert_eq!(type_ann.name, "i32");
            assert!(type_ann.is_mutable);

            assert_eq!(initializer.unwrap(), Expr::Literal(Literal::Integer(10)));
        } else {
            panic!("Expected variable declaration, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_expression_statement() {
        let source = "foo(1, 2);";
        let result = parse_statement(source).unwrap();

        if let Stmt::Expression(expr) = result {
            if let Expr::Call(callee, args) = expr {
                assert_eq!(*callee, Expr::Variable("foo".to_string()));
                assert_eq!(args.len(), 2);
                assert_eq!(args[0], Expr::Literal(Literal::Integer(1)));
                assert_eq!(args[1], Expr::Literal(Literal::Integer(2)));
            } else {
                panic!("Expected call expression, got {:?}", expr);
            }
        } else {
            panic!("Expected expression statement, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_return_statement() {
        let source = "return 42;";
        let result = parse_statement(source).unwrap();

        if let Stmt::Return(expr) = result {
            assert!(expr.is_some());
            assert_eq!(expr.unwrap(), Expr::Literal(Literal::Integer(42)));
        } else {
            panic!("Expected return statement, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_empty_return_statement() {
        let source = "return;";
        let result = parse_statement(source).unwrap();

        if let Stmt::Return(expr) = result {
            assert!(expr.is_none());
        } else {
            panic!("Expected return statement, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_block_statement() {
        let source = "{ let x = 1; let y = 2; }";
        let result = parse_statement(source).unwrap();

        if let Stmt::Block(statements) = result {
            assert_eq!(statements.len(), 2);

            if let Stmt::VarDeclaration { name, .. } = &statements[0] {
                assert_eq!(name, "x");
            } else {
                panic!("Expected variable declaration");
            }

            if let Stmt::VarDeclaration { name, .. } = &statements[1] {
                assert_eq!(name, "y");
            } else {
                panic!("Expected variable declaration");
            }
        } else {
            panic!("Expected block statement, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_if_statement() {
        let source = "if (x > 10) { return true; } else { return false; }";
        let result = parse_statement(source).unwrap();

        if let Stmt::If {
            condition,
            then_branch,
            else_branch,
        } = result
        {
            // Check condition
            assert!(matches!(condition, Expr::Binary(_, BinaryOp::Greater, _)));

            // Check then branch
            if let Stmt::Block(stmts) = *then_branch {
                assert_eq!(stmts.len(), 1);
                assert!(matches!(stmts[0], Stmt::Return(_)));
            } else {
                panic!("Expected block in then branch");
            }

            // Check else branch
            assert!(else_branch.is_some());
            if let Stmt::Block(stmts) = *else_branch.unwrap() {
                assert_eq!(stmts.len(), 1);
                assert!(matches!(stmts[0], Stmt::Return(_)));
            } else {
                panic!("Expected block in else branch");
            }
        } else {
            panic!("Expected if statement, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_while_statement() {
        let source = "while (i < 10) { i += 1; }";
        let result = parse_statement(source).unwrap();

        if let Stmt::While { condition, body } = result {
            // Check condition
            assert!(matches!(condition, Expr::Binary(_, BinaryOp::Less, _)));

            // Check body
            if let Stmt::Block(stmts) = *body {
                assert_eq!(stmts.len(), 1);
                assert!(matches!(stmts[0], Stmt::Expression(_)));
            } else {
                panic!("Expected block in while body");
            }
        } else {
            panic!("Expected while statement, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_for_statement() {
        let source = "for (let i = 0; i < 10; i += 1) { print(i); }";
        let result = parse_statement(source).unwrap();

        if let Stmt::For {
            initializer,
            condition,
            increment,
            body,
        } = result
        {
            // Check initializer
            assert!(initializer.is_some());
            assert!(matches!(*initializer.unwrap(), Stmt::VarDeclaration { .. }));

            // Check condition
            assert!(condition.is_some());
            assert!(matches!(
                condition.unwrap(),
                Expr::Binary(_, BinaryOp::Less, _)
            ));

            // Check increment
            assert!(increment.is_some());
            assert!(matches!(
                increment.unwrap(),
                Expr::Binary(_, BinaryOp::PlusAssign, _)
            ));

            // Check body
            if let Stmt::Block(stmts) = *body {
                assert_eq!(stmts.len(), 1);
                assert!(matches!(stmts[0], Stmt::Expression(_)));
            } else {
                panic!("Expected block in for body");
            }
        } else {
            panic!("Expected for statement, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_function_declaration() {
        let source = "func add(a: i32, b: i32) -> i32 { return a + b; }";
        let result = parse_statement(source).unwrap();

        if let Stmt::FunctionDeclaration {
            name,
            params,
            return_type,
            body,
            ..
        } = result
        {
            // Check name
            assert_eq!(name, "add");

            // Check parameters
            assert_eq!(params.len(), 2);
            assert_eq!(params[0].name, "a");
            assert_eq!(params[0].type_annotation.name, "i32");
            assert_eq!(params[1].name, "b");
            assert_eq!(params[1].type_annotation.name, "i32");

            // Check return type
            assert!(return_type.is_some());
            assert_eq!(return_type.unwrap().name, "i32");

            // Check body
            if let Stmt::Block(stmts) = *body {
                assert_eq!(stmts.len(), 1);
                assert!(matches!(stmts[0], Stmt::Return(_)));
            } else {
                panic!("Expected block in function body");
            }
        } else {
            panic!("Expected function declaration, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_void_function_declaration() {
        let source = "func greet(name: string) -> () { print(name); return; }";
        let result = parse_statement(source).unwrap();

        if let Stmt::FunctionDeclaration {
            name,
            params,
            return_type,
            body,
            ..
        } = result
        {
            // Check name
            assert_eq!(name, "greet");

            // Check parameters
            assert_eq!(params.len(), 1);
            assert_eq!(params[0].name, "name");
            assert_eq!(params[0].type_annotation.name, "string");

            // Check return type (should be unit type '()')
            assert!(return_type.is_some());
            assert_eq!(return_type.unwrap().name, "()");

            // Check body
            if let Stmt::Block(stmts) = *body {
                assert_eq!(stmts.len(), 2);
                assert!(matches!(stmts[0], Stmt::Expression(_)));
                assert!(matches!(stmts[1], Stmt::Return(None)));
            } else {
                panic!("Expected block in function body");
            }
        } else {
            panic!("Expected function declaration, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_complex_program() {
        let source = r#"
            func fibonacci(n: i32) -> i32 {
                if (n <= 1) {
                    return n;
                }
                return fibonacci(n - 1) + fibonacci(n - 2);
            }

            func main() -> () {
                let result = fibonacci(10);
                print(result);
                return;
            }
        "#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize_all().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();

        // Check that we have two function declarations
        assert_eq!(program.len(), 2);
        assert!(matches!(program[0], Stmt::FunctionDeclaration { .. }));
        assert!(matches!(program[1], Stmt::FunctionDeclaration { .. }));

        // Verify fibonacci function
        if let Stmt::FunctionDeclaration { name, .. } = &program[0] {
            assert_eq!(name, "fibonacci");
        }

        // Verify main function
        if let Stmt::FunctionDeclaration { name, .. } = &program[1] {
            assert_eq!(name, "main");
        }
    }

    #[test]
    fn test_parse_array_indexing_fixed() {
        let source = "array[1 + 2]";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize_all().expect("Lexing should succeed");
        let mut parser = Parser::new(tokens);

        let expr = parser.expression().expect("Parsing should succeed");

        // Verify the structure
        match expr {
            Expr::Index(array, index) => {
                assert!(matches!(*array, Expr::Variable(_)));
                assert!(matches!(*index, Expr::Binary(_, _, _)));
            }
            _ => panic!("Expected Index expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_simd_vector_literal_parsing() {
        let source = "[1.0, 2.0, 3.0, 4.0]f32x4";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize_all().expect("Lexing should succeed");
        let mut parser = Parser::new(tokens);

        let expr = parser.expression().expect("Parsing should succeed");

        // Verify SIMD vector literal structure
        match expr {
            Expr::SIMD(SIMDExpr::VectorLiteral {
                elements,
                vector_type,
                ..
            }) => {
                assert_eq!(elements.len(), 4);
                assert_eq!(vector_type, Some(SIMDVectorType::F32x4));
            }
            _ => panic!("Expected SIMD VectorLiteral, got {:?}", expr),
        }
    }

    #[test]
    fn test_regular_array_literal() {
        let source = "[1, 2, 3]";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize_all().expect("Lexing should succeed");
        let mut parser = Parser::new(tokens);

        let expr = parser.expression().expect("Parsing should succeed");

        match expr {
            Expr::Literal(Literal::Vector {
                elements,
                vector_type,
            }) => {
                assert_eq!(elements.len(), 3);
                assert_eq!(vector_type, None);
            }
            _ => panic!("Expected Vector literal, got {:?}", expr),
        }
    }

    #[test]
    fn test_empty_array() {
        let source = "[]";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize_all().expect("Lexing should succeed");
        let mut parser = Parser::new(tokens);

        let expr = parser.expression().expect("Parsing should succeed");

        match expr {
            Expr::Literal(Literal::Vector {
                elements,
                vector_type,
            }) => {
                assert_eq!(elements.len(), 0);
                assert_eq!(vector_type, None);
            }
            _ => panic!("Expected empty Vector literal, got {:?}", expr),
        }
    }
}
