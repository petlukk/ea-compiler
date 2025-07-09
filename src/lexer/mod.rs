// src/lexer/mod.rs - FIXED with Hash and Eq traits
//! Lexical analyzer for the Eä programming language with SIMD extensions

use logos::Logos;
use std::fmt;

pub mod tokens;

use crate::error::CompileError;
use crate::memory_profiler::{record_memory_usage, CompilationPhase, check_memory_limit};

/// Position information for tokens
#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

impl Position {
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self {
            line,
            column,
            offset,
        }
    }
}

/// A token with position information
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub position: Position,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: String, position: Position) -> Self {
        Self {
            kind,
            lexeme,
            position,
        }
    }
}

/// Token types for the Eä language with SIMD extensions
/// FIXED: Added custom Hash and Eq implementations for HashMap compatibility
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\f]+")] // Skip whitespace except newlines
pub enum TokenKind {
    // === Core Keywords ===
    #[token("func")]
    Func,
    #[token("struct")]
    Struct,
    #[token("let")]
    Let,
    #[token("mut")]
    Mut,
    #[token("const")]
    Const,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("return")]
    Return,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("while")]
    While,
    #[token("for")]
    For,
    #[token("in")]
    In,
    #[token("enum")]
    Enum,
    #[token("match")]
    Match,

    // === Memory Management Keywords ===
    #[token("mem_region")]
    MemRegion,
    #[token("parallel")]
    Parallel,
    #[token("async")]
    Async,
    #[token("await")]
    Await,

    // === SIMD Keywords (NEW) ===
    #[token("vectorize")]
    Vectorize,
    #[token("simd_width")]
    SimdWidth,
    #[token("simd_auto")]
    SimdAuto,
    #[token("target_feature")]
    TargetFeature,
    #[token("horizontal_sum")]
    HorizontalSum,
    #[token("horizontal_min")]
    HorizontalMin,
    #[token("horizontal_max")]
    HorizontalMax,
    #[token("dot_product")]
    DotProduct,
    #[token("load_vector")]
    LoadVector,
    #[token("store_vector")]
    StoreVector,
    #[token("from_slice")]
    FromSlice,
    #[token("to_array")]
    ToArray,
    #[token("splat")]
    Splat,
    #[token("shuffle")]
    Shuffle,
    #[token("lanes")]
    Lanes,

    // === SIMD Hardware Feature Keywords ===
    #[token("sse")]
    SSE,
    #[token("sse2")]
    SSE2,
    #[token("sse3")]
    SSE3,
    #[token("sse4")]
    SSE4,
    #[token("avx")]
    AVX,
    #[token("avx2")]
    AVX2,
    #[token("avx512")]
    AVX512,
    #[token("neon")]
    NEON,
    #[token("altivec")]
    AltiVec,

    // === Scalar Types ===
    #[token("i8")]
    I8,
    #[token("i16")]
    I16,
    #[token("i32")]
    I32,
    #[token("i64")]
    I64,
    #[token("u8")]
    U8,
    #[token("u16")]
    U16,
    #[token("u32")]
    U32,
    #[token("u64")]
    U64,
    #[token("f32")]
    F32,
    #[token("f64")]
    F64,
    #[token("bool")]
    Bool,
    #[token("string")]
    String,

    // === SIMD Vector Types (NEW) ===
    // 32-bit float SIMD types
    #[token("f32x2")]
    F32x2,
    #[token("f32x4")]
    F32x4,
    #[token("f32x8")]
    F32x8,
    #[token("f32x16")]
    F32x16,

    // 64-bit float SIMD types
    #[token("f64x2")]
    F64x2,
    #[token("f64x4")]
    F64x4,
    #[token("f64x8")]
    F64x8,

    // 32-bit integer SIMD types
    #[token("i32x2")]
    I32x2,
    #[token("i32x4")]
    I32x4,
    #[token("i32x8")]
    I32x8,
    #[token("i32x16")]
    I32x16,

    // 64-bit integer SIMD types
    #[token("i64x2")]
    I64x2,
    #[token("i64x4")]
    I64x4,
    #[token("i64x8")]
    I64x8,

    // 16-bit integer SIMD types
    #[token("i16x4")]
    I16x4,
    #[token("i16x8")]
    I16x8,
    #[token("i16x16")]
    I16x16,
    #[token("i16x32")]
    I16x32,

    // 8-bit integer SIMD types (great for image processing)
    #[token("i8x8")]
    I8x8,
    #[token("i8x16")]
    I8x16,
    #[token("i8x32")]
    I8x32,
    #[token("i8x64")]
    I8x64,

    // Unsigned SIMD types
    #[token("u32x4")]
    U32x4,
    #[token("u32x8")]
    U32x8,
    #[token("u16x8")]
    U16x8,
    #[token("u16x16")]
    U16x16,
    #[token("u8x16")]
    U8x16,
    #[token("u8x32")]
    U8x32,

    // Boolean/mask SIMD types
    #[token("mask8")]
    Mask8,
    #[token("mask16")]
    Mask16,
    #[token("mask32")]
    Mask32,
    #[token("mask64")]
    Mask64,

    // === Operators ===
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("=")]
    Assign,
    #[token("==")]
    Equal,
    #[token("!=")]
    NotEqual,
    #[token("<")]
    Less,
    #[token("<=")]
    LessEqual,
    #[token(">")]
    Greater,
    #[token(">=")]
    GreaterEqual,
    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token("!")]
    Not,
    #[token("&")]
    Ampersand,
    #[token("+=")]
    PlusAssign,
    #[token("-=")]
    MinusAssign,
    #[token("*=")]
    StarAssign,
    #[token("/=")]
    SlashAssign,

    // === SIMD-specific Operators (NEW) ===
    // Arithmetic
    #[token(".+")]
    DotAdd, // Element-wise add
    #[token(".-")]
    DotSubtract, // Element-wise subtract
    #[token(".*")]
    DotMultiply, // Element-wise multiply
    #[token("./")]
    DotDivide, // Element-wise divide

    // Bitwise
    #[token(".&")]
    DotAnd, // Element-wise bitwise AND
    #[token(".|")]
    DotOr, // Element-wise bitwise OR
    #[token(".^")]
    DotXor, // Element-wise bitwise XOR

    // Comparison
    #[token(".==")]
    DotEqual, // Element-wise equal
    #[token(".!=")]
    DotNotEqual, // Element-wise not equal
    #[token(".<")]
    DotLess, // Element-wise less than
    #[token(".>")]
    DotGreater, // Element-wise greater than
    #[token(".<=")]
    DotLessEqual, // Element-wise less than or equal
    #[token(".>=")]
    DotGreaterEqual, // Element-wise greater than or equal

    // === Delimiters ===
    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[token("{")]
    LeftBrace,
    #[token("}")]
    RightBrace,
    #[token("[")]
    LeftBracket,
    #[token("]")]
    RightBracket,
    #[token(",")]
    Comma,
    #[token(";")]
    Semicolon,
    #[token(":")]
    Colon,
    #[token("::")]
    DoubleColon,
    #[token("->")]
    Arrow,
    #[token("=>")]
    FatArrow,
    #[token(".")]
    Dot,

    // === Literals ===
    // Hexadecimal integers
    #[regex(r"0[xX][0-9a-fA-F][0-9a-fA-F_]*", |lex| {
        let s = lex.slice();
        i64::from_str_radix(&s[2..].replace('_', ""), 16).ok()
    })]
    // Binary integers
    #[regex(r"0[bB][01][01_]*", |lex| {
        let s = lex.slice();
        i64::from_str_radix(&s[2..].replace('_', ""), 2).ok()
    })]
    // Decimal integers
    #[regex(r"[0-9][0-9_]*", |lex| {
        lex.slice().replace('_', "").parse::<i64>().ok()
    })]
    Integer(i64),

    // Floating point numbers
    #[regex(r"[0-9][0-9_]*\.[0-9][0-9_]*", |lex| {
        lex.slice().replace('_', "").parse::<f64>().ok()
    })]
    Float(f64),

    // === SIMD Vector Literals (NEW) ===
    // SIMD vector literal syntax: [1.0, 2.0, 3.0, 4.0]f32x4
    // NOTE: Disabled for now to fix bracket parsing - will be handled in parser instead
    // #[regex(r"\[(?:[0-9]+\.?[0-9]*,?\s*)+\](?:f32x2|f32x4|f32x8|f32x16|f64x2|f64x4|f64x8|i32x2|i32x4|i32x8|i32x16|i64x2|i64x4|i64x8|i16x4|i16x8|i16x16|i16x32|i8x8|i8x16|i8x32|i8x64|u32x4|u32x8|u16x8|u16x16|u8x16|u8x32)", |lex| {
    //     Some(lex.slice().to_string())
    // })]
    SimdLiteral(String),

    // String literals
    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        Some(s[1..s.len()-1].to_string())
    })]
    StringLiteral(String),

    // Identifiers
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    // Comments (skip them)
    #[regex(r"//[^\n]*", logos::skip)]
    #[regex(r"/\*([^*]|\*[^/])*\*/", logos::skip)]
    Comment,

    // Newline (track for line numbers)
    #[token("\n")]
    Newline,

    // === Attributes ===
    #[token("@")]
    At,

    // Special tokens
    Eof,
}

// Custom Eq implementation to handle f64 values
impl Eq for TokenKind {}

// Custom Hash implementation to handle f64 values
impl std::hash::Hash for TokenKind {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            TokenKind::Integer(n) => n.hash(state),
            TokenKind::Float(f) => {
                // Convert f64 to bits for hashing to handle NaN consistently
                f.to_bits().hash(state);
            }
            TokenKind::SimdLiteral(s) => s.hash(state),
            TokenKind::StringLiteral(s) => s.hash(state),
            TokenKind::Identifier(s) => s.hash(state),
            _ => {} // Other variants have no data to hash
        }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display = match self {
            // Core keywords
            TokenKind::Func => "func",
            TokenKind::Struct => "struct",
            TokenKind::Let => "let",
            TokenKind::Mut => "mut",
            TokenKind::Const => "const",
            TokenKind::If => "if",
            TokenKind::Else => "else",
            TokenKind::Return => "return",
            TokenKind::True => "true",
            TokenKind::False => "false",
            TokenKind::While => "while",
            TokenKind::For => "for",
            TokenKind::In => "in",
            TokenKind::Enum => "enum",
            TokenKind::Match => "match",

            // Memory management
            TokenKind::MemRegion => "mem_region",
            TokenKind::Parallel => "parallel",
            TokenKind::Async => "async",
            TokenKind::Await => "await",

            // SIMD keywords
            TokenKind::Vectorize => "vectorize",
            TokenKind::SimdWidth => "simd_width",
            TokenKind::SimdAuto => "simd_auto",
            TokenKind::TargetFeature => "target_feature",
            TokenKind::HorizontalSum => "horizontal_sum",
            TokenKind::HorizontalMin => "horizontal_min",
            TokenKind::HorizontalMax => "horizontal_max",
            TokenKind::DotProduct => "dot_product",
            TokenKind::LoadVector => "load_vector",
            TokenKind::StoreVector => "store_vector",
            TokenKind::FromSlice => "from_slice",
            TokenKind::ToArray => "to_array",
            TokenKind::Splat => "splat",
            TokenKind::Shuffle => "shuffle",
            TokenKind::Lanes => "lanes",

            // Hardware features
            TokenKind::SSE => "sse",
            TokenKind::SSE2 => "sse2",
            TokenKind::SSE3 => "sse3",
            TokenKind::SSE4 => "sse4",
            TokenKind::AVX => "avx",
            TokenKind::AVX2 => "avx2",
            TokenKind::AVX512 => "avx512",
            TokenKind::NEON => "neon",
            TokenKind::AltiVec => "altivec",

            // Scalar types
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

            // SIMD types
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

            // Standard operators
            TokenKind::Plus => "+",
            TokenKind::Minus => "-",
            TokenKind::Star => "*",
            TokenKind::Slash => "/",
            TokenKind::Percent => "%",
            TokenKind::Assign => "=",
            TokenKind::Equal => "==",
            TokenKind::NotEqual => "!=",
            TokenKind::Less => "<",
            TokenKind::LessEqual => "<=",
            TokenKind::Greater => ">",
            TokenKind::GreaterEqual => ">=",
            TokenKind::And => "&&",
            TokenKind::Or => "||",
            TokenKind::Not => "!",
            TokenKind::Ampersand => "&",
            TokenKind::PlusAssign => "+=",
            TokenKind::MinusAssign => "-=",
            TokenKind::StarAssign => "*=",
            TokenKind::SlashAssign => "/=",

            // SIMD operators
            TokenKind::DotAdd => ".+",
            TokenKind::DotSubtract => ".-",
            TokenKind::DotMultiply => ".*",
            TokenKind::DotDivide => "./",
            TokenKind::DotAnd => ".&",
            TokenKind::DotOr => ".|",
            TokenKind::DotXor => ".^",
            TokenKind::DotEqual => ".==",
            TokenKind::DotNotEqual => ".!=",
            TokenKind::DotLess => ".<",
            TokenKind::DotGreater => ".>",
            TokenKind::DotLessEqual => ".<=",
            TokenKind::DotGreaterEqual => ".>=",

            // Delimiters
            TokenKind::LeftParen => "(",
            TokenKind::RightParen => ")",
            TokenKind::LeftBrace => "{",
            TokenKind::RightBrace => "}",
            TokenKind::LeftBracket => "[",
            TokenKind::RightBracket => "]",
            TokenKind::Comma => ",",
            TokenKind::Semicolon => ";",
            TokenKind::Colon => ":",
            TokenKind::DoubleColon => "::",
            TokenKind::Arrow => "->",
            TokenKind::FatArrow => "=>",
            TokenKind::Dot => ".",

            // Literals with values
            TokenKind::Integer(n) => return write!(f, "{}", n),
            TokenKind::Float(n) => return write!(f, "{}", n),
            TokenKind::SimdLiteral(s) => return write!(f, "{}", s),
            TokenKind::StringLiteral(s) => return write!(f, "\"{}\"", s),
            TokenKind::Identifier(s) => return write!(f, "{}", s),

            // Special
            TokenKind::Comment => "comment",
            TokenKind::Newline => "\\n",
            TokenKind::At => "@",
            TokenKind::Eof => "EOF",
        };
        write!(f, "{}", display)
    }
}

/// Lexer for the Eä language with SIMD extensions
pub struct Lexer<'source> {
    logos_lexer: logos::Lexer<'source, TokenKind>,
    source: &'source str,
    line: usize,
    column: usize,
}

impl<'source> Lexer<'source> {
    pub fn new(source: &'source str) -> Self {
        Self {
            logos_lexer: TokenKind::lexer(source),
            source,
            line: 1,
            column: 1,
        }
    }

    /// Enhanced keyword detection including SIMD keywords
    /// NOTE: This function is currently unused but kept for potential future use
    #[allow(dead_code)]
    fn keyword_type(text: &str) -> Option<TokenKind> {
        match text {
            // Core language keywords
            "func" => Some(TokenKind::Func),
            "struct" => Some(TokenKind::Struct),
            "let" => Some(TokenKind::Let),
            "mut" => Some(TokenKind::Mut),
            "const" => Some(TokenKind::Const),
            "if" => Some(TokenKind::If),
            "else" => Some(TokenKind::Else),
            "return" => Some(TokenKind::Return),
            "true" => Some(TokenKind::True),
            "false" => Some(TokenKind::False),
            "while" => Some(TokenKind::While),
            "for" => Some(TokenKind::For),
            "in" => Some(TokenKind::In),
            "enum" => Some(TokenKind::Enum),
            "match" => Some(TokenKind::Match),

            // Memory management
            "mem_region" => Some(TokenKind::MemRegion),
            "parallel" => Some(TokenKind::Parallel),
            "async" => Some(TokenKind::Async),
            "await" => Some(TokenKind::Await),

            // SIMD keywords
            "vectorize" => Some(TokenKind::Vectorize),
            "simd_width" => Some(TokenKind::SimdWidth),
            "simd_auto" => Some(TokenKind::SimdAuto),
            "target_feature" => Some(TokenKind::TargetFeature),
            "horizontal_sum" => Some(TokenKind::HorizontalSum),
            "horizontal_min" => Some(TokenKind::HorizontalMin),
            "horizontal_max" => Some(TokenKind::HorizontalMax),
            "dot_product" => Some(TokenKind::DotProduct),
            "load_vector" => Some(TokenKind::LoadVector),
            "store_vector" => Some(TokenKind::StoreVector),
            "from_slice" => Some(TokenKind::FromSlice),
            "to_array" => Some(TokenKind::ToArray),
            "splat" => Some(TokenKind::Splat),
            "shuffle" => Some(TokenKind::Shuffle),
            "lanes" => Some(TokenKind::Lanes),

            // Hardware features
            "sse" => Some(TokenKind::SSE),
            "sse2" => Some(TokenKind::SSE2),
            "sse3" => Some(TokenKind::SSE3),
            "sse4" => Some(TokenKind::SSE4),
            "avx" => Some(TokenKind::AVX),
            "avx2" => Some(TokenKind::AVX2),
            "avx512" => Some(TokenKind::AVX512),
            "neon" => Some(TokenKind::NEON),
            "altivec" => Some(TokenKind::AltiVec),

            _ => None,
        }
    }

    /// Enhanced SIMD literal parsing
    /// NOTE: This function is currently unused but kept for potential future use
    #[allow(dead_code)]
    fn parse_simd_literal(&self, text: &str) -> Option<TokenKind> {
        // Check if it's a SIMD vector literal: [1.0, 2.0, 3.0, 4.0]f32x4
        if text.starts_with('[') && text.contains(']') {
            if let Some(type_start) = text.rfind(']') {
                let type_part = &text[type_start + 1..];

                // Check if it's a valid SIMD type
                match type_part {
                    "f32x2" | "f32x4" | "f32x8" | "f32x16" | "f64x2" | "f64x4" | "f64x8"
                    | "i32x2" | "i32x4" | "i32x8" | "i32x16" | "i64x2" | "i64x4" | "i64x8"
                    | "i16x4" | "i16x8" | "i16x16" | "i16x32" | "i8x8" | "i8x16" | "i8x32"
                    | "i8x64" | "u32x4" | "u32x8" | "u16x8" | "u16x16" | "u8x16" | "u8x32" => {
                        return Some(TokenKind::SimdLiteral(text.to_string()));
                    }
                    _ => {}
                }
            }
        }
        None
    }

    pub fn next_token(&mut self) -> Result<Token, CompileError> {
        loop {
            match self.logos_lexer.next() {
                Some(Ok(token_kind)) => {
                    let lexeme = self.logos_lexer.slice().to_string();
                    let span = self.logos_lexer.span();

                    // Handle newlines specially to track line numbers
                    if token_kind == TokenKind::Newline {
                        self.line += 1;
                        self.column = 1;
                        // Skip newlines - continue to next token
                        continue;
                    }

                    // Calculate the actual column based on the source position
                    let line_start = self.source[..span.start]
                        .rfind('\n')
                        .map(|pos| pos + 1)
                        .unwrap_or(0);
                    let column = span.start - line_start + 1;

                    let position = Position::new(self.line, column, span.start);

                    // Update column position for next token
                    self.column = column + lexeme.len();

                    return Ok(Token::new(token_kind, lexeme, position));
                }
                Some(Err(_)) => {
                    let lexeme = self.logos_lexer.slice().to_string();
                    let span = self.logos_lexer.span();

                    // Calculate the actual column based on the source position
                    let line_start = self.source[..span.start]
                        .rfind('\n')
                        .map(|pos| pos + 1)
                        .unwrap_or(0);
                    let column = span.start - line_start + 1;

                    let position = Position::new(self.line, column, span.start);

                    return Err(CompileError::lex_error(
                        format!("Unexpected character: '{}'", lexeme),
                        position,
                    ));
                }
                None => {
                    // End of input
                    let span_end = self.logos_lexer.span().end;
                    let position = Position::new(self.line, self.column, span_end);
                    return Ok(Token::new(TokenKind::Eof, String::new(), position));
                }
            }
        }
    }

    pub fn tokenize_all(&mut self) -> Result<Vec<Token>, CompileError> {
        // Record initial memory usage
        let initial_memory = std::mem::size_of::<Vec<Token>>();
        record_memory_usage(CompilationPhase::Lexing, initial_memory, "Started lexing");

        let mut tokens = Vec::new();

        loop {
            let token = self.next_token()?;
            let is_eof = token.kind == TokenKind::Eof;
            tokens.push(token);

            // Check memory limits periodically
            if tokens.len() % 1000 == 0 {
                let current_memory = tokens.len() * std::mem::size_of::<Token>();
                record_memory_usage(CompilationPhase::Lexing, current_memory, 
                    &format!("Lexing progress: {} tokens", tokens.len()));
                
                // Check if we're exceeding memory limits
                if let Err(e) = check_memory_limit() {
                    return Err(CompileError::MemoryExhausted { 
                        phase: "lexing".to_string(), 
                        details: e.to_string() 
                    });
                }
            }

            if is_eof {
                break;
            }
        }

        // Record final memory usage
        let final_memory = tokens.len() * std::mem::size_of::<Token>();
        record_memory_usage(CompilationPhase::Lexing, final_memory, 
            &format!("Completed lexing: {} tokens", tokens.len()));

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complex_simd_program() {
        let source = r#"
            func process_vectors(a: f32x8, b: f32x8) -> f32x8 {
                vectorize with simd_width(256) {
                    let result = a .* b .+ f32x8::splat(1.0);
                    return result;
                }
            }
            
            @target_feature(avx2)
            func optimized_compute() -> f32 {
                let vec = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]f32x8;
                return vec.horizontal_sum();
            }
        "#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize_all().unwrap();

        // Verify we can tokenize a complex SIMD program
        assert!(tokens.len() > 20);

        // Find key SIMD tokens
        let token_kinds: Vec<&TokenKind> = tokens.iter().map(|t| &t.kind).collect();

        assert!(token_kinds.contains(&&TokenKind::F32x8));
        assert!(token_kinds.contains(&&TokenKind::Vectorize));
        assert!(token_kinds.contains(&&TokenKind::SimdWidth));
        assert!(token_kinds.contains(&&TokenKind::DotMultiply));
        assert!(token_kinds.contains(&&TokenKind::DotAdd));
        assert!(token_kinds.contains(&&TokenKind::TargetFeature));
        assert!(token_kinds.contains(&&TokenKind::AVX2));
        assert!(token_kinds.contains(&&TokenKind::HorizontalSum));
        assert!(token_kinds.contains(&&TokenKind::Splat));
    }

    #[test]
    fn test_mixed_scalar_and_simd_types() {
        let mut lexer = Lexer::new("i32 f32x4 f64 i16x8 bool u8x16");
        let tokens = lexer.tokenize_all().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::I32); // scalar
        assert_eq!(tokens[1].kind, TokenKind::F32x4); // SIMD
        assert_eq!(tokens[2].kind, TokenKind::F64); // scalar
        assert_eq!(tokens[3].kind, TokenKind::I16x8); // SIMD
        assert_eq!(tokens[4].kind, TokenKind::Bool); // scalar
        assert_eq!(tokens[5].kind, TokenKind::U8x16); // SIMD
    }

    #[test]
    fn test_simd_position_tracking() {
        let source = "vectorize\nf32x8";
        let mut lexer = Lexer::new(source);

        let token1 = lexer.next_token().unwrap();
        assert_eq!(token1.position.line, 1);
        assert_eq!(token1.position.column, 1);
        assert_eq!(token1.lexeme, "vectorize");

        let token2 = lexer.next_token().unwrap();
        assert_eq!(token2.position.line, 2);
        assert_eq!(token2.position.column, 1);
        assert_eq!(token2.lexeme, "f32x8");
    }

    #[test]
    fn test_simd_error_on_invalid_character() {
        let mut lexer = Lexer::new("vectorize $ f32x4");
        let tokens = lexer.tokenize_all();

        assert!(tokens.is_err());
        if let Err(e) = tokens {
            let error_str = e.to_string();
            assert!(error_str.contains("Unexpected character"));
            assert!(error_str.contains("$"));
        }
    }

    #[test]
    fn test_all_simd_vector_widths() {
        let simd_types = [
            "f32x2", "f32x4", "f32x8", "f32x16", "f64x2", "f64x4", "f64x8", "i32x2", "i32x4",
            "i32x8", "i32x16", "i64x2", "i64x4", "i64x8", "i16x4", "i16x8", "i16x16", "i16x32",
            "i8x8", "i8x16", "i8x32", "i8x64", "u32x4", "u32x8", "u16x8", "u16x16", "u8x16",
            "u8x32", "mask8", "mask16", "mask32", "mask64",
        ];

        for simd_type in simd_types {
            let mut lexer = Lexer::new(simd_type);
            let tokens = lexer.tokenize_all().unwrap();

            // Should have exactly 2 tokens: the SIMD type and EOF
            assert_eq!(tokens.len(), 2);
            assert_ne!(tokens[0].kind, TokenKind::Identifier(simd_type.to_string()));
            assert_eq!(tokens[1].kind, TokenKind::Eof);
        }
    }

    #[test]
    fn test_simd_with_underscores_in_numbers() {
        let mut lexer = Lexer::new("let mask = 0xFF_FF_FF_FF; let vec: f32x4;");
        let tokens = lexer.tokenize_all().unwrap();

        // Find the hex literal with underscores
        let hex_token = tokens
            .iter()
            .find(|t| matches!(t.kind, TokenKind::Integer(_)))
            .unwrap();
        if let TokenKind::Integer(value) = hex_token.kind {
            assert_eq!(value, 0xFFFFFFFF);
        }

        // Find the SIMD type
        let simd_token = tokens.iter().find(|t| t.kind == TokenKind::F32x4).unwrap();
        assert_eq!(simd_token.lexeme, "f32x4");
    }

    #[test]
    fn test_simd_keywords() {
        let mut lexer = Lexer::new("vectorize simd_width simd_auto target_feature");
        let tokens = lexer.tokenize_all().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Vectorize);
        assert_eq!(tokens[1].kind, TokenKind::SimdWidth);
        assert_eq!(tokens[2].kind, TokenKind::SimdAuto);
        assert_eq!(tokens[3].kind, TokenKind::TargetFeature);
        assert_eq!(tokens[4].kind, TokenKind::Eof);
    }

    #[test]
    fn test_simd_types() {
        let mut lexer = Lexer::new("f32x4 f64x8 i32x16 u8x32 mask16");
        let tokens = lexer.tokenize_all().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::F32x4);
        assert_eq!(tokens[1].kind, TokenKind::F64x8);
        assert_eq!(tokens[2].kind, TokenKind::I32x16);
        assert_eq!(tokens[3].kind, TokenKind::U8x32);
        assert_eq!(tokens[4].kind, TokenKind::Mask16);
        assert_eq!(tokens[5].kind, TokenKind::Eof);
    }

    #[test]
    fn test_hardware_features() {
        let mut lexer = Lexer::new("avx2 sse4 neon avx512");
        let tokens = lexer.tokenize_all().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::AVX2);
        assert_eq!(tokens[1].kind, TokenKind::SSE4);
        assert_eq!(tokens[2].kind, TokenKind::NEON);
        assert_eq!(tokens[3].kind, TokenKind::AVX512);
    }

    #[test]
    fn test_simd_operators() {
        let mut lexer = Lexer::new(".* .+ ./ .& .| .^");
        let tokens = lexer.tokenize_all().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::DotMultiply);
        assert_eq!(tokens[1].kind, TokenKind::DotAdd);
        assert_eq!(tokens[2].kind, TokenKind::DotDivide);
        assert_eq!(tokens[3].kind, TokenKind::DotAnd);
        assert_eq!(tokens[4].kind, TokenKind::DotOr);
        assert_eq!(tokens[5].kind, TokenKind::DotXor);
    }

    #[test]
    fn test_simd_vector_literals() {
        let input = "[1.0, 2.0, 3.0, 4.0]f32x4";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize_all().unwrap();

        // This will be parsed as separate tokens for now: [ 1.0 , 2.0 , 3.0 , 4.0 ] f32x4
        // We'll handle the full SIMD literal parsing in the parser phase
        assert!(tokens.len() > 1);
        assert_eq!(tokens[0].kind, TokenKind::LeftBracket);
        // Note: The actual SIMD literal parsing will be enhanced in parser phase
    }

    #[test]
    fn test_simd_function_calls() {
        let mut lexer = Lexer::new("horizontal_sum from_slice to_array splat");
        let tokens = lexer.tokenize_all().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::HorizontalSum);
        assert_eq!(tokens[1].kind, TokenKind::FromSlice);
        assert_eq!(tokens[2].kind, TokenKind::ToArray);
        assert_eq!(tokens[3].kind, TokenKind::Splat);
    }

    #[test]
    fn test_enum_and_match_keywords() {
        let mut lexer = Lexer::new("enum Option match value");
        let tokens = lexer.tokenize_all().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Enum);
        assert_eq!(tokens[1].kind, TokenKind::Identifier("Option".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::Match);
        assert_eq!(tokens[3].kind, TokenKind::Identifier("value".to_string()));
        assert_eq!(tokens[4].kind, TokenKind::Eof);
    }
}
