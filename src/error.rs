//! Error handling for the Eä compiler.

use crate::lexer::Position;
use std::fmt;

/// Result type for compiler operations.
pub type Result<T> = std::result::Result<T, CompileError>;

/// Represents errors that can occur during compilation.
#[derive(Debug, Clone)]
pub enum CompileError {
    /// Lexical analysis error
    LexError { message: String, position: Position },
    /// Parsing error
    ParseError { message: String, position: Position },
    /// Type checking error
    TypeError { message: String, position: Position },
    /// Code generation error
    CodeGenError {
        message: String,
        position: Option<Position>,
    },
    /// Memory exhaustion error
    MemoryExhausted { phase: String, details: String },
}

impl CompileError {
    /// Creates a new lexical error
    pub fn lex_error(message: String, position: Position) -> Self {
        Self::LexError { message, position }
    }

    /// Creates a new parsing error
    pub fn parse_error(message: String, position: Position) -> Self {
        Self::ParseError { message, position }
    }

    /// Creates a new type error
    pub fn type_error(message: String, position: Position) -> Self {
        Self::TypeError { message, position }
    }

    /// Creates a new code generation error
    pub fn codegen_error(message: String, position: Option<Position>) -> Self {
        Self::CodeGenError { message, position }
    }

    /// Creates a new memory exhaustion error
    pub fn memory_exhausted(phase: String, details: String) -> Self {
        Self::MemoryExhausted { phase, details }
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::LexError { message, position } => {
                write!(
                    f,
                    "Lexical error at {}:{}: {}",
                    position.line, position.column, message
                )
            }
            CompileError::ParseError { message, position } => {
                write!(
                    f,
                    "Parse error at {}:{}: {}",
                    position.line, position.column, message
                )
            }
            CompileError::TypeError { message, position } => {
                write!(
                    f,
                    "Type error at {}:{}: {}",
                    position.line, position.column, message
                )
            }
            CompileError::CodeGenError { message, position } => {
                if let Some(pos) = position {
                    write!(
                        f,
                        "Code generation error at {}:{}: {}",
                        pos.line, pos.column, message
                    )
                } else {
                    write!(f, "Code generation error: {}", message)
                }
            }
            CompileError::MemoryExhausted { phase, details } => {
                write!(f, "Memory exhausted during {}: {}", phase, details)
            }
        }
    }
}

impl std::error::Error for CompileError {}
