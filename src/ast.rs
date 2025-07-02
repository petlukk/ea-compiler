//! Abstract Syntax Tree (AST) definitions for the Eä programming language.

use std::fmt;

/// Represents a binary operator in an expression
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    // Arithmetic
    Add,       // +
    Subtract,  // -
    Multiply,  // *
    Divide,    // /
    Modulo,    // %
    
    // Comparison
    Equal,          // ==
    NotEqual,       // !=
    Less,           // <
    LessEqual,      // <=
    Greater,        // >
    GreaterEqual,   // >=
    
    // Logical
    And,    // &&
    Or,     // ||
    
    // Assignment
    Assign,         // =
    PlusAssign,     // +=
    MinusAssign,    // -=
    MultiplyAssign, // *=
    DivideAssign,   // /=
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Subtract => write!(f, "-"),
            BinaryOp::Multiply => write!(f, "*"),
            BinaryOp::Divide => write!(f, "/"),
            BinaryOp::Modulo => write!(f, "%"),
            BinaryOp::Equal => write!(f, "=="),
            BinaryOp::NotEqual => write!(f, "!="),
            BinaryOp::Less => write!(f, "<"),
            BinaryOp::LessEqual => write!(f, "<="),
            BinaryOp::Greater => write!(f, ">"),
            BinaryOp::GreaterEqual => write!(f, ">="),
            BinaryOp::And => write!(f, "&&"),
            BinaryOp::Or => write!(f, "||"),
            BinaryOp::Assign => write!(f, "="),
            BinaryOp::PlusAssign => write!(f, "+="),
            BinaryOp::MinusAssign => write!(f, "-="),
            BinaryOp::MultiplyAssign => write!(f, "*="),
            BinaryOp::DivideAssign => write!(f, "/="),
        }
    }
}

/// Represents a unary operator in an expression
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Negate,    // -
    Not,       // !
    Reference, // &
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOp::Negate => write!(f, "-"),
            UnaryOp::Not => write!(f, "!"),
            UnaryOp::Reference => write!(f, "&"),
        }
    }
}

/// Represents a literal value in the AST
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Vector {
        elements: Vec<Literal>,
        vector_type: Option<SIMDVectorType>,
    },
    
}
use crate::lexer::Position; // Add this import

/// SIMD Expression types for industry-first SIMD support
#[derive(Debug, Clone, PartialEq)]
pub enum SIMDExpr {
    /// SIMD vector literal: [1.0, 2.0, 3.0, 4.0]
    VectorLiteral {
        elements: Vec<Expr>,
        vector_type: Option<SIMDVectorType>,
        position: Position,
    },
    
    /// SIMD element-wise operations: vec1 .* vec2
    ElementWise {
        left: Box<Expr>,
        operator: SIMDOperator,
        right: Box<Expr>,
        position: Position,
    },
    
    /// SIMD broadcast operation: broadcast(scalar, f32x4)
    Broadcast {
        value: Box<Expr>,
        target_type: SIMDVectorType,
        position: Position,
    },
    
    /// SIMD swizzle operation: vec.xyz or vec[0:3]
    Swizzle {
        vector: Box<Expr>,
        pattern: SwizzlePattern,
        position: Position,
    },
    
    /// SIMD reduction operation: sum(vector), max(vector)
    Reduction {
        vector: Box<Expr>,
        operation: ReductionOp,
        position: Position,
    },
    
    /// SIMD dot product: dot_product(a, b)
    DotProduct {
        left: Box<Expr>,
        right: Box<Expr>,
        position: Position,
    },
    
    /// SIMD vector load from memory: load_vector(address, alignment)
    VectorLoad {
        address: Box<Expr>,
        vector_type: SIMDVectorType,
        alignment: Option<u32>, // Optional alignment in bytes (16, 32, etc.)
        position: Position,
    },
    
    /// SIMD vector store to memory: store_vector(address, vector, alignment)
    VectorStore {
        address: Box<Expr>,
        vector: Box<Expr>,
        alignment: Option<u32>, // Optional alignment in bytes
        position: Position,
    },
}

/// SIMD vector types - all 32 types from SIMD-001
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SIMDVectorType {
    // Float vectors
    F32x2, F32x4, F32x8, F32x16,
    F64x2, F64x4, F64x8,
    
    // Integer vectors
    I32x2, I32x4, I32x8, I32x16,
    I64x2, I64x4, I64x8,
    I16x4, I16x8, I16x16, I16x32,
    I8x8, I8x16, I8x32, I8x64,
    
    // Unsigned integer vectors
    U32x4, U32x8,
    U16x8, U16x16,
    U8x16, U8x32,
    
    // Mask types
    Mask8, Mask16, Mask32, Mask64,
}

/// SIMD operators - comprehensive set for element-wise operations
#[derive(Debug, Clone, PartialEq)]
pub enum SIMDOperator {
    // Arithmetic
    DotAdd,       // .+
    DotSubtract,  // .-
    DotMultiply,  // .*
    DotDivide,    // ./
    
    // Bitwise
    DotAnd,       // .&
    DotOr,        // .|
    DotXor,       // .^
    
    // Comparison
    DotEqual,     // .==
    DotNotEqual,  // .!=
    DotLess,      // .<
    DotGreater,   // .>
    DotLessEqual, // .<=
    DotGreaterEqual, // .>=
}

/// Swizzle patterns for SIMD vector element selection
#[derive(Debug, Clone, PartialEq)]
pub enum SwizzlePattern {
    /// Named swizzle: .x, .xy, .xyz, .xyzw
    Named(String),
    /// Index range: [0:3], [2:5]
    Range { start: usize, end: usize },
    /// Individual indices: [0, 2, 1, 3]
    Indices(Vec<usize>),
}

/// SIMD reduction operations
#[derive(Debug, Clone, PartialEq)]
pub enum ReductionOp {
    Sum, Product, Min, Max,
    And, Or, Xor,
    Any, All,
}

// Implementation methods for SIMD types
impl SIMDVectorType {
    /// Get the element type of a SIMD vector
    pub fn element_type(&self) -> &'static str {
        match self {
            SIMDVectorType::F32x2 | SIMDVectorType::F32x4 | 
            SIMDVectorType::F32x8 | SIMDVectorType::F32x16 => "f32",
            
            SIMDVectorType::F64x2 | SIMDVectorType::F64x4 | 
            SIMDVectorType::F64x8 => "f64",
            
            SIMDVectorType::I32x2 | SIMDVectorType::I32x4 | 
            SIMDVectorType::I32x8 | SIMDVectorType::I32x16 => "i32",
            
            SIMDVectorType::I64x2 | SIMDVectorType::I64x4 | 
            SIMDVectorType::I64x8 => "i64",
            
            SIMDVectorType::I16x4 | SIMDVectorType::I16x8 | 
            SIMDVectorType::I16x16 | SIMDVectorType::I16x32 => "i16",
            
            SIMDVectorType::I8x8 | SIMDVectorType::I8x16 | 
            SIMDVectorType::I8x32 | SIMDVectorType::I8x64 => "i8",
            
            SIMDVectorType::U32x4 | SIMDVectorType::U32x8 => "u32",
            SIMDVectorType::U16x8 | SIMDVectorType::U16x16 => "u16",
            SIMDVectorType::U8x16 | SIMDVectorType::U8x32 => "u8",
            
            SIMDVectorType::Mask8 | SIMDVectorType::Mask16 | 
            SIMDVectorType::Mask32 | SIMDVectorType::Mask64 => "bool",
        }
    }
    
    /// Get the vector width (number of elements)
    pub fn width(&self) -> usize {
        match self {
            SIMDVectorType::F32x2 | SIMDVectorType::F64x2 | 
            SIMDVectorType::I32x2 | SIMDVectorType::I64x2 => 2,
            
            SIMDVectorType::F32x4 | SIMDVectorType::F64x4 | 
            SIMDVectorType::I32x4 | SIMDVectorType::I64x4 |
            SIMDVectorType::I16x4 | SIMDVectorType::U32x4 => 4,
            
            SIMDVectorType::F32x8 | SIMDVectorType::F64x8 | 
            SIMDVectorType::I32x8 | SIMDVectorType::I64x8 |
            SIMDVectorType::I16x8 | SIMDVectorType::I8x8 |
            SIMDVectorType::U32x8 | SIMDVectorType::U16x8 => 8,
            
            SIMDVectorType::F32x16 | SIMDVectorType::I32x16 |
            SIMDVectorType::I16x16 | SIMDVectorType::I8x16 |
            SIMDVectorType::U16x16 | SIMDVectorType::U8x16 => 16,
            
            SIMDVectorType::I16x32 | SIMDVectorType::I8x32 |
            SIMDVectorType::U8x32 => 32,
            
            SIMDVectorType::I8x64 => 64,
            
            SIMDVectorType::Mask8 => 8,
            SIMDVectorType::Mask16 => 16,
            SIMDVectorType::Mask32 => 32,
            SIMDVectorType::Mask64 => 64,
        }
    }
    
    /// Check if two SIMD types are compatible for operations
    pub fn is_compatible_with(&self, other: &SIMDVectorType) -> bool {
        // Same type is always compatible
        if self == other {
            return true;
        }
        
        // Check if element types and widths match
        self.element_type() == other.element_type() && self.width() == other.width()
    }
}

impl SIMDOperator {
    /// Check if operator is valid for given vector types
    pub fn is_valid_for_types(&self, left: &SIMDVectorType, right: &SIMDVectorType) -> bool {
        match self {
            // Arithmetic operators require same element type and width
            SIMDOperator::DotAdd | SIMDOperator::DotSubtract | 
            SIMDOperator::DotMultiply | SIMDOperator::DotDivide => {
                left.is_compatible_with(right) && 
                matches!(left.element_type(), "f32" | "f64" | "i32" | "i64" | "i16" | "i8" | "u32" | "u16" | "u8")
            }
            
            // Bitwise operators work on integer types and masks
            SIMDOperator::DotAnd | SIMDOperator::DotOr | SIMDOperator::DotXor => {
                left.is_compatible_with(right) && 
                !matches!(left.element_type(), "f32" | "f64")
            }
            
            // Comparison operators work on all types, produce mask vectors
            SIMDOperator::DotEqual | SIMDOperator::DotNotEqual |
            SIMDOperator::DotLess | SIMDOperator::DotGreater |
            SIMDOperator::DotLessEqual | SIMDOperator::DotGreaterEqual => {
                left.is_compatible_with(right)
            }
        }
    }
}

// Display implementations for beautiful error messages
impl std::fmt::Display for SIMDVectorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            SIMDVectorType::F32x2 => "f32x2",
            SIMDVectorType::F32x4 => "f32x4",
            SIMDVectorType::F32x8 => "f32x8",
            SIMDVectorType::F32x16 => "f32x16",
            SIMDVectorType::F64x2 => "f64x2",
            SIMDVectorType::F64x4 => "f64x4",
            SIMDVectorType::F64x8 => "f64x8",
            SIMDVectorType::I32x2 => "i32x2",
            SIMDVectorType::I32x4 => "i32x4",
            SIMDVectorType::I32x8 => "i32x8",
            SIMDVectorType::I32x16 => "i32x16",
            SIMDVectorType::I64x2 => "i64x2",
            SIMDVectorType::I64x4 => "i64x4",
            SIMDVectorType::I64x8 => "i64x8",
            SIMDVectorType::I16x4 => "i16x4",
            SIMDVectorType::I16x8 => "i16x8",
            SIMDVectorType::I16x16 => "i16x16",
            SIMDVectorType::I16x32 => "i16x32",
            SIMDVectorType::I8x8 => "i8x8",
            SIMDVectorType::I8x16 => "i8x16",
            SIMDVectorType::I8x32 => "i8x32",
            SIMDVectorType::I8x64 => "i8x64",
            SIMDVectorType::U32x4 => "u32x4",
            SIMDVectorType::U32x8 => "u32x8",
            SIMDVectorType::U16x8 => "u16x8",
            SIMDVectorType::U16x16 => "u16x16",
            SIMDVectorType::U8x16 => "u8x16",
            SIMDVectorType::U8x32 => "u8x32",
            SIMDVectorType::Mask8 => "mask8",
            SIMDVectorType::Mask16 => "mask16",
            SIMDVectorType::Mask32 => "mask32",
            SIMDVectorType::Mask64 => "mask64",
        };
        write!(f, "{}", name)
    }
}

impl std::fmt::Display for SIMDOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let op = match self {
            // Arithmetic
            SIMDOperator::DotAdd => ".+",
            SIMDOperator::DotSubtract => ".-",
            SIMDOperator::DotMultiply => ".*",
            SIMDOperator::DotDivide => "./",
            
            // Bitwise
            SIMDOperator::DotAnd => ".&",
            SIMDOperator::DotOr => ".|",
            SIMDOperator::DotXor => ".^",
            
            // Comparison
            SIMDOperator::DotEqual => ".==",
            SIMDOperator::DotNotEqual => ".!=",
            SIMDOperator::DotLess => ".<",
            SIMDOperator::DotGreater => ".>",
            SIMDOperator::DotLessEqual => ".<=",
            SIMDOperator::DotGreaterEqual => ".>=",
        };
        write!(f, "{}", op)
    }
}

impl std::fmt::Display for SIMDExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SIMDExpr::VectorLiteral { elements, vector_type, .. } => {
                if let Some(vtype) = vector_type {
                    write!(f, "{}(", vtype)?;
                } else {
                    write!(f, "[")?;
                }
                
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", elem)?;
                }
                
                if vector_type.is_some() {
                    write!(f, ")")
                } else {
                    write!(f, "]")
                }
            }
            SIMDExpr::ElementWise { left, operator, right, .. } => {
                write!(f, "({} {} {})", left, operator, right)
            }
            SIMDExpr::Broadcast { value, target_type, .. } => {
                write!(f, "broadcast({}, {})", value, target_type)
            }
            SIMDExpr::Swizzle { vector, pattern, .. } => {
                match pattern {
                    SwizzlePattern::Named(name) => write!(f, "{}.{}", vector, name),
                    SwizzlePattern::Range { start, end } => write!(f, "{}[{}:{}]", vector, start, end),
                    SwizzlePattern::Indices(indices) => {
                        write!(f, "{}[", vector)?;
                        for (i, idx) in indices.iter().enumerate() {
                            if i > 0 { write!(f, ", ")?; }
                            write!(f, "{}", idx)?;
                        }
                        write!(f, "]")
                    }
                }
            }
            SIMDExpr::Reduction { vector, operation, .. } => {
                write!(f, "{:?}({})", operation, vector)
            }
            SIMDExpr::DotProduct { left, right, .. } => {
                write!(f, "dot_product({}, {})", left, right)
            }
            SIMDExpr::VectorLoad { address, vector_type, alignment, .. } => {
                if let Some(align) = alignment {
                    write!(f, "load_vector({}, {}, {})", address, vector_type, align)
                } else {
                    write!(f, "load_vector({}, {})", address, vector_type)
                }
            }
            SIMDExpr::VectorStore { address, vector, alignment, .. } => {
                if let Some(align) = alignment {
                    write!(f, "store_vector({}, {}, {})", address, vector, align)
                } else {
                    write!(f, "store_vector({}, {})", address, vector)
                }
            }
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Integer(i) => write!(f, "{}", i),
            Literal::Float(fl) => write!(f, "{}", fl),
            Literal::String(s) => write!(f, "\"{}\"", s),
            Literal::Boolean(b) => write!(f, "{}", b),
            Literal::Vector { elements, vector_type } => {
                if let Some(vtype) = vector_type {
                    write!(f, "{}(", vtype)?;
                } else {
                    write!(f, "[")?;
                }
                
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", elem)?;
                }
                
                if vector_type.is_some() {
                    write!(f, ")")
                } else {
                    write!(f, "]")
                }
            }
        }
    }
}

/// Represents an expression in the AST
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A literal value (number, string, boolean)
    Literal(Literal),
    
    /// Variable reference
    Variable(String),
    
    /// Unary operation: !x, -x, &x
    Unary(UnaryOp, Box<Expr>),
    
    /// Binary operation: a + b, a * b, etc.
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
    
    /// Grouping with parentheses: (expr)
    Grouping(Box<Expr>),
    
    /// Function call: func(arg1, arg2)
    Call(Box<Expr>, Vec<Expr>),
    
    /// Array indexing: array[index]
    Index(Box<Expr>, Box<Expr>),
    
    /// Field access: object.field
    FieldAccess(Box<Expr>, String),

    SIMD(SIMDExpr),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Literal(lit) => write!(f, "{}", lit),
            Expr::Variable(name) => write!(f, "{}", name),
            Expr::Unary(op, expr) => write!(f, "{}({})", op, expr),
            Expr::Binary(left, op, right) => write!(f, "({} {} {})", left, op, right),
            Expr::Grouping(expr) => write!(f, "({})", expr),
            Expr::Call(callee, args) => {
                write!(f, "{}(", callee)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            },
            Expr::Index(array, index) => write!(f, "{}[{}]", array, index),
            Expr::FieldAccess(object, field) => write!(f, "{}.{}", object, field),
            Expr::SIMD(simd_expr) => write!(f, "{}", simd_expr),
        }
    }
}

/// Type annotation in the AST
#[derive(Debug, Clone, PartialEq)]
pub struct TypeAnnotation {
    pub name: String,
    pub is_mutable: bool,
}

impl fmt::Display for TypeAnnotation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_mutable {
            write!(f, "mut {}", self.name)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

/// Function parameter in a function declaration
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub type_annotation: TypeAnnotation,
}

impl fmt::Display for Parameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.type_annotation)
    }
}

/// Represents a statement in the AST
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// Expression statement, e.g., `foo()`
    Expression(Expr),
    
    /// Variable declaration, e.g., `let x = 5` or `let mut y: i32 = 10`
    VarDeclaration {
        name: String,
        type_annotation: Option<TypeAnnotation>,
        initializer: Option<Expr>,
    },
    
    /// Block of statements enclosed in braces: `{ ... }`
    Block(Vec<Stmt>),
    
    /// Function declaration: `func name(params) -> return_type { body }`
    FunctionDeclaration {
        name: String,
        params: Vec<Parameter>,
        return_type: Option<TypeAnnotation>,
        body: Box<Stmt>, // Block statement
    },
    
    /// Return statement: `return expr`
    Return(Option<Expr>),
    
    /// If statement: `if condition { then_branch } else { else_branch }`
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    
    /// While loop: `while condition { body }`
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    
    /// For loop: `for init; condition; increment { body }`
    For {
        initializer: Option<Box<Stmt>>,
        condition: Option<Expr>,
        increment: Option<Expr>,
        body: Box<Stmt>,
    },
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stmt::Expression(expr) => write!(f, "{};", expr),
            Stmt::VarDeclaration { name, type_annotation, initializer } => {
                write!(f, "let ")?;
                
                if let Some(type_ann) = type_annotation {
                    if type_ann.is_mutable {
                        write!(f, "mut ")?;
                    }
                    write!(f, "{}: {}", name, type_ann.name)?;
                } else {
                    write!(f, "{}", name)?;
                }
                
                if let Some(init) = initializer {
                    write!(f, " = {}", init)?;
                }
                
                write!(f, ";")
            },
            Stmt::Block(statements) => {
                writeln!(f, "{{")?;
                for stmt in statements {
                    writeln!(f, "    {}", stmt)?;
                }
                write!(f, "}}")
            },
            Stmt::FunctionDeclaration { name, params, return_type, body } => {
                write!(f, "func {}(", name)?;
                
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                
                write!(f, ")")?;
                
                if let Some(ret_type) = return_type {
                    write!(f, " -> {}", ret_type)?;
                }
                
                write!(f, " {}", body)
            },
            Stmt::Return(expr) => {
                if let Some(e) = expr {
                    write!(f, "return {};", e)
                } else {
                    write!(f, "return;")
                }
            },
            Stmt::If { condition, then_branch, else_branch } => {
                write!(f, "if {} {}", condition, then_branch)?;
                
                if let Some(else_stmt) = else_branch {
                    write!(f, " else {}", else_stmt)?;
                }
                
                Ok(())
            },
            Stmt::While { condition, body } => {
                write!(f, "while {} {}", condition, body)
            },
            Stmt::For { initializer, condition, increment, body } => {
                write!(f, "for ")?;
                
                if let Some(init) = initializer {
                    write!(f, "{}", init)?;
                } else {
                    write!(f, ";")?;
                }
                
                if let Some(cond) = condition {
                    write!(f, " {};", cond)?;
                } else {
                    write!(f, ";")?;
                }
                
                if let Some(inc) = increment {
                    write!(f, " {}", inc)?;
                }
                
                write!(f, " {}", body)
            },
        }
    }
}