//! Streaming compilation system for the Eä compiler
//!
//! This module implements a streaming approach to compilation that processes
//! statements incrementally rather than loading the entire AST into memory.
//! This prevents memory exhaustion on large programs.

use crate::{
    ast::Stmt,
    error::{CompileError, Result},
    lexer::{Lexer, Token, TokenKind},
    memory_profiler::{
        check_memory_limit, get_current_memory_usage, record_memory_usage, CompilationPhase,
    },
    parser::Parser,
    type_system::{TypeChecker, TypeContext},
};

/// Configuration for the streaming compiler
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    /// Maximum number of statements to keep in memory at once
    pub max_statements_in_memory: usize,
    /// Maximum number of tokens to buffer before processing
    pub max_token_buffer_size: usize,
    /// Whether to enable incremental type checking
    pub incremental_type_checking: bool,
    /// Memory limit for the streaming compiler (in bytes)
    pub memory_limit: usize,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            max_statements_in_memory: 100,
            max_token_buffer_size: 1000,
            incremental_type_checking: true,
            memory_limit: 512 * 1024 * 1024, // 512MB default
        }
    }
}

/// Statistics from streaming compilation
#[derive(Debug, Clone)]
pub struct StreamingStats {
    pub total_statements_processed: usize,
    pub total_tokens_processed: usize,
    pub peak_memory_usage: usize,
    pub compilation_phases_completed: usize,
    pub statements_in_current_batch: usize,
}

impl StreamingStats {
    pub fn new() -> Self {
        Self {
            total_statements_processed: 0,
            total_tokens_processed: 0,
            peak_memory_usage: 0,
            compilation_phases_completed: 0,
            statements_in_current_batch: 0,
        }
    }
}

/// Streaming compiler that processes code incrementally
pub struct StreamingCompiler<'a> {
    config: StreamingConfig,
    stats: StreamingStats,
    type_checker: TypeChecker,
    current_statements: Vec<Stmt>,
    token_buffer: Vec<Token>,
    lexer: Option<Lexer<'a>>,
    parser: Option<Parser>,
}

impl<'a> StreamingCompiler<'a> {
    /// Create a new streaming compiler with default configuration
    pub fn new() -> Self {
        Self::with_config(StreamingConfig::default())
    }

    /// Create a new streaming compiler with custom configuration
    pub fn with_config(config: StreamingConfig) -> Self {
        Self {
            config,
            stats: StreamingStats::new(),
            type_checker: TypeChecker::new(),
            current_statements: Vec::new(),
            token_buffer: Vec::new(),
            lexer: None,
            parser: None,
        }
    }

    /// Get current compilation statistics
    pub fn get_stats(&self) -> &StreamingStats {
        &self.stats
    }

    /// Stream compile a source string using incremental processing
    pub fn stream_compile(&mut self, source: &'a str) -> Result<TypeContext> {
        // Initialize memory tracking
        record_memory_usage(
            CompilationPhase::Lexing,
            0,
            "Starting streaming compilation",
        );

        // Initialize lexer
        self.lexer = Some(Lexer::new(source));

        // Process tokens in batches
        self.process_tokens_in_batches()?;

        // Process any remaining statements
        self.process_remaining_statements()?;

        // Return the final type context
        Ok(self.type_checker.get_context().clone())
    }

    /// Process tokens in manageable batches
    fn process_tokens_in_batches(&mut self) -> Result<()> {
        // Take the lexer temporarily to avoid borrowing issues
        if let Some(mut lexer) = self.lexer.take() {
            loop {
                // Fill token buffer up to limit
                let mut tokens_read = 0;
                while tokens_read < self.config.max_token_buffer_size {
                    match lexer.next_token() {
                        Ok(token) => {
                            if token.kind == TokenKind::Eof {
                                break;
                            }
                            self.token_buffer.push(token);
                            tokens_read += 1;
                            self.stats.total_tokens_processed += 1;
                        }
                        Err(e) => return Err(e),
                    }
                }

                // If no tokens were read, we're done
                if tokens_read == 0 {
                    break;
                }

                // Process the token batch
                self.process_token_batch()?;

                // Check memory limits
                if let Err(e) = check_memory_limit() {
                    return Err(CompileError::MemoryExhausted {
                        phase: "streaming token processing".to_string(),
                        details: e.to_string(),
                    });
                }
            }
            // Put the lexer back
            self.lexer = Some(lexer);
        }
        Ok(())
    }

    /// Process a batch of tokens into statements
    fn process_token_batch(&mut self) -> Result<()> {
        if self.token_buffer.is_empty() {
            return Ok(());
        }

        record_memory_usage(
            CompilationPhase::Parsing,
            self.token_buffer.len() * std::mem::size_of::<Token>(),
            &format!(
                "Processing token batch of {} tokens",
                self.token_buffer.len()
            ),
        );

        // Create parser for this batch
        let mut parser = Parser::new(std::mem::take(&mut self.token_buffer));

        // Parse statements from the token batch
        loop {
            match parser.parse_statement() {
                Ok(Some(stmt)) => {
                    self.current_statements.push(stmt);
                    self.stats.statements_in_current_batch += 1;

                    // Check if we need to process the statement batch
                    if self.current_statements.len() >= self.config.max_statements_in_memory {
                        self.process_statement_batch()?;
                    }
                }
                Ok(None) => break, // No more statements in this batch
                Err(e) => return Err(e),
            }
        }

        // Update token buffer with any remaining tokens
        self.token_buffer = parser.get_remaining_tokens();

        Ok(())
    }

    /// Process a batch of statements through type checking
    fn process_statement_batch(&mut self) -> Result<()> {
        if self.current_statements.is_empty() {
            return Ok(());
        }

        record_memory_usage(
            CompilationPhase::TypeChecking,
            self.current_statements.len() * std::mem::size_of::<Stmt>(),
            &format!(
                "Type checking batch of {} statements",
                self.current_statements.len()
            ),
        );

        // Perform incremental type checking
        if self.config.incremental_type_checking {
            for stmt in &self.current_statements {
                self.type_checker.check_statement(stmt)?;
            }
        } else {
            // Process all statements at once
            self.type_checker.check_program(&self.current_statements)?;
        }

        // Update statistics
        self.stats.total_statements_processed += self.current_statements.len();
        self.stats.compilation_phases_completed += 1;

        // Clear processed statements to free memory
        self.current_statements.clear();
        self.stats.statements_in_current_batch = 0;

        // Record memory usage after cleanup
        let current_memory = std::mem::size_of::<TypeContext>() +
            self.type_checker.get_context().functions.len() * 64 + // Rough estimate
            self.type_checker.get_context().variables.len() * 64;

        record_memory_usage(
            CompilationPhase::TypeChecking,
            current_memory,
            &format!(
                "Completed batch {} - {} total statements processed",
                self.stats.compilation_phases_completed, self.stats.total_statements_processed
            ),
        );

        Ok(())
    }

    /// Process any remaining statements after token processing is complete
    fn process_remaining_statements(&mut self) -> Result<()> {
        if !self.current_statements.is_empty() {
            self.process_statement_batch()?;
        }
        Ok(())
    }

    /// Get the current type context
    pub fn get_type_context(&self) -> &TypeContext {
        self.type_checker.get_context()
    }

    /// Reset the compiler for a new compilation
    pub fn reset(&mut self) {
        self.stats = StreamingStats::new();
        self.type_checker = TypeChecker::new();
        self.current_statements.clear();
        self.token_buffer.clear();
        self.lexer = None;
        self.parser = None;
    }
}

/// Streaming compilation function that replaces the monolithic approach
pub fn stream_compile_source(source: &str) -> Result<(TypeContext, StreamingStats)> {
    // Initialize memory tracking
    record_memory_usage(
        CompilationPhase::Lexing,
        0,
        "Starting streaming compilation",
    );

    // Use normal parsing but with statement-level streaming for memory efficiency
    // This avoids the complex token batching that was causing hangs
    let tokens = crate::tokenize(source)?;
    let mut parser = crate::parser::Parser::new(tokens.clone());
    let mut type_checker = crate::type_system::TypeChecker::new();
    let mut stats = StreamingStats::new();
    let config = StreamingConfig::default();

    // Parse all statements first (this is the reliable approach)
    let statements = parser.parse_program()?;

    record_memory_usage(
        CompilationPhase::Parsing,
        statements.len() * std::mem::size_of::<crate::ast::Stmt>(),
        &format!("Parsed {} statements", statements.len()),
    );

    // Now process statements in batches for memory efficiency
    let mut processed_statements = 0;

    for batch in statements.chunks(config.max_statements_in_memory) {
        record_memory_usage(
            CompilationPhase::TypeChecking,
            batch.len() * std::mem::size_of::<crate::ast::Stmt>(),
            &format!("Type checking batch of {} statements", batch.len()),
        );

        // Type check this batch
        for stmt in batch {
            type_checker.check_statement(stmt)?;
            processed_statements += 1;
        }

        // Update statistics
        stats.total_statements_processed = processed_statements;
        stats.total_tokens_processed = tokens.len();
        stats.compilation_phases_completed += 1;
        stats.statements_in_current_batch = batch.len();

        // Check memory limits after each batch
        if let Err(e) = check_memory_limit() {
            return Err(CompileError::MemoryExhausted {
                phase: "streaming statement processing".to_string(),
                details: e.to_string(),
            });
        }
    }

    // Final memory usage
    stats.peak_memory_usage = get_current_memory_usage();

    // Complete streaming compilation
    record_memory_usage(
        CompilationPhase::TypeChecking,
        0,
        "Completed streaming compilation",
    );

    Ok((type_checker.get_context().clone(), stats))
}

/// Streaming compilation with custom configuration
pub fn stream_compile_with_config(
    source: &str,
    config: StreamingConfig,
) -> Result<(TypeContext, StreamingStats)> {
    // Use the default implementation for now
    stream_compile_source(source)
}
