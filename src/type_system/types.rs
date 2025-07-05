// src/type_system/types.rs
//! Type definitions for the Eä programming language type system.

use std::fmt;

/// Simple element types for SIMD vectors to avoid recursive type issues
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SIMDElementType {
    I8, I16, I32, I64,
    U8, U16, U32, U64,
    F32, F64,
}

/// Represents all types in the Eä programming language.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EaType {
    // Primitive integer types
    I8,
    I16,
    I32,
    I64,
    
    // Primitive unsigned integer types
    U8,
    U16,
    U32,
    U64,
    
    // Primitive floating-point types
    F32,
    F64,
    
    // Boolean type
    Bool,
    
    // String type
    String,
    
    // Unit type (void/empty)
    Unit,
    
    // Array type
    Array(Box<EaType>),
    
    // SIMD vector type with non-recursive element type
    SIMDVector {
        element_type: SIMDElementType,  // Use non-recursive element type
        width: usize,
        vector_type: crate::ast::SIMDVectorType,
    },
    
    // Reference type
    Reference(Box<EaType>),
    
    // Function type (separate from FunctionType for expression typing)
    Function(Box<FunctionType>),
    
    // Custom/user-defined types (structs - for future)
    Custom(String),
    
    // Enum type
    Enum {
        name: String,
        variants: Vec<String>, // For now, just variant names
    },
    
    // Generic type parameter (for future generic support)
    Generic(String),
    
    // Error type (for error recovery during type checking)
    Error,
}

impl fmt::Display for EaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EaType::I8 => write!(f, "i8"),
            EaType::I16 => write!(f, "i16"),
            EaType::I32 => write!(f, "i32"),
            EaType::I64 => write!(f, "i64"),
            EaType::U8 => write!(f, "u8"),
            EaType::U16 => write!(f, "u16"),
            EaType::U32 => write!(f, "u32"),
            EaType::U64 => write!(f, "u64"),
            EaType::F32 => write!(f, "f32"),
            EaType::F64 => write!(f, "f64"),
            EaType::Bool => write!(f, "bool"),
            EaType::String => write!(f, "string"),
            EaType::Unit => write!(f, "()"),
            EaType::Array(elem_type) => write!(f, "[{}]", elem_type),
            EaType::SIMDVector { vector_type, .. } => write!(f, "{}", vector_type),
            EaType::Reference(inner_type) => write!(f, "&{}", inner_type),
            EaType::Function(func_type) => write!(f, "{}", func_type),
            EaType::Custom(name) => write!(f, "{}", name),
            EaType::Enum { name, .. } => write!(f, "{}", name),
            EaType::Generic(name) => write!(f, "{}", name),
            EaType::Error => write!(f, "<error>"),
        }
    }
}

/// Represents a function type with parameters and return type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionType {
    /// Parameter types
    pub params: Vec<EaType>,
    /// Return type
    pub return_type: Box<EaType>,
    /// Whether the function accepts variable arguments
    pub is_variadic: bool,
}

impl fmt::Display for FunctionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        for (i, param) in self.params.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", param)?;
        }
        if self.is_variadic {
            if !self.params.is_empty() {
                write!(f, ", ")?;
            }
            write!(f, "...")?;
        }
        write!(f, ") -> {}", self.return_type)
    }
}

impl EaType {
    /// Returns true if this type is a primitive type.
    pub fn is_primitive(&self) -> bool {
        matches!(self, 
            EaType::I8 | EaType::I16 | EaType::I32 | EaType::I64 |
            EaType::U8 | EaType::U16 | EaType::U32 | EaType::U64 |
            EaType::F32 | EaType::F64 | EaType::Bool | EaType::String | EaType::Unit |
            EaType::SIMDVector { .. }
        )
    }
    
    /// Returns true if this type is an integer type.
    pub fn is_integer(&self) -> bool {
        matches!(self, 
            EaType::I8 | EaType::I16 | EaType::I32 | EaType::I64 |
            EaType::U8 | EaType::U16 | EaType::U32 | EaType::U64
        )
    }
    
    /// Returns true if this type is a signed integer type.
    pub fn is_signed_integer(&self) -> bool {
        matches!(self, EaType::I8 | EaType::I16 | EaType::I32 | EaType::I64)
    }
    
    /// Returns true if this type is an unsigned integer type.
    pub fn is_unsigned_integer(&self) -> bool {
        matches!(self, EaType::U8 | EaType::U16 | EaType::U32 | EaType::U64)
    }
    
    /// Returns true if this type is a floating-point type.
    pub fn is_float(&self) -> bool {
        matches!(self, EaType::F32 | EaType::F64)
    }
    
    /// Returns true if this type is numeric (integer or float).
    pub fn is_numeric(&self) -> bool {
        self.is_integer() || self.is_float()
    }
    
    /// Returns true if this type can be compared using ordering operators.
    pub fn is_comparable(&self) -> bool {
        self.is_numeric() || matches!(self, EaType::String)
    }
    
    /// Returns the size of this type in bytes (for primitive types).
    pub fn size_bytes(&self) -> Option<usize> {
        match self {
            EaType::I8 | EaType::U8 => Some(1),
            EaType::I16 | EaType::U16 => Some(2),
            EaType::I32 | EaType::U32 | EaType::F32 => Some(4),
            EaType::I64 | EaType::U64 | EaType::F64 => Some(8),
            EaType::Bool => Some(1),
            EaType::String => Some(8), // Pointer size
            EaType::Unit => Some(0),
            EaType::Reference(_) => Some(8), // Pointer size
            _ => None, // Complex types don't have a fixed size
        }
    }
    
    /// Returns the alignment requirement for this type in bytes.
    pub fn alignment(&self) -> Option<usize> {
        match self {
            EaType::I8 | EaType::U8 | EaType::Bool => Some(1),
            EaType::I16 | EaType::U16 => Some(2),
            EaType::I32 | EaType::U32 | EaType::F32 => Some(4),
            EaType::I64 | EaType::U64 | EaType::F64 => Some(8),
            EaType::String => Some(8), // Pointer alignment
            EaType::Unit => Some(1),
            EaType::Reference(_) => Some(8), // Pointer alignment
            _ => None, // Complex types need custom alignment calculation
        }
    }
    
    /// Returns the default value for this type (if it has one).
    pub fn default_value(&self) -> Option<DefaultValue> {
        match self {
            EaType::I8 | EaType::I16 | EaType::I32 | EaType::I64 => Some(DefaultValue::Integer(0)),
            EaType::U8 | EaType::U16 | EaType::U32 | EaType::U64 => Some(DefaultValue::UnsignedInteger(0)),
            EaType::F32 | EaType::F64 => Some(DefaultValue::Float(0.0)),
            EaType::Bool => Some(DefaultValue::Boolean(false)),
            EaType::String => Some(DefaultValue::String(String::new())),
            EaType::Unit => Some(DefaultValue::Unit),
            _ => None, // Complex types don't have automatic defaults
        }
    }
}

/// Represents default values for types.
#[derive(Debug, Clone, PartialEq)]
pub enum DefaultValue {
    Integer(i64),
    UnsignedInteger(u64),
    Float(f64),
    Boolean(bool),
    String(String),
    Unit,
}

impl fmt::Display for DefaultValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DefaultValue::Integer(i) => write!(f, "{}", i),
            DefaultValue::UnsignedInteger(u) => write!(f, "{}", u),
            DefaultValue::Float(fl) => write!(f, "{}", fl),
            DefaultValue::Boolean(b) => write!(f, "{}", b),
            DefaultValue::String(s) => write!(f, "\"{}\"", s),
            DefaultValue::Unit => write!(f, "()"),
        }
    }
}

/// Type compatibility and conversion rules.
pub struct TypeRules;

impl TypeRules {
    /// Checks if one type can be implicitly converted to another.
    pub fn can_implicitly_convert(from: &EaType, to: &EaType) -> bool {
        // Exact match
        if from == to {
            return true;
        }
        
        match (from, to) {
            // Integer widening conversions
            (EaType::I8, EaType::I16 | EaType::I32 | EaType::I64) => true,
            (EaType::I16, EaType::I32 | EaType::I64) => true,
            (EaType::I32, EaType::I64) => true,
            
            // Unsigned integer widening conversions
            (EaType::U8, EaType::U16 | EaType::U32 | EaType::U64) => true,
            (EaType::U16, EaType::U32 | EaType::U64) => true,
            (EaType::U32, EaType::U64) => true,
            
            // Float widening conversions
            (EaType::F32, EaType::F64) => true,
            
            // Integer to float conversions (with potential precision loss warning)
            (EaType::I8 | EaType::I16 | EaType::I32, EaType::F32 | EaType::F64) => true,
            (EaType::I64, EaType::F64) => true,
            (EaType::U8 | EaType::U16 | EaType::U32, EaType::F32 | EaType::F64) => true,
            (EaType::U64, EaType::F64) => true,
            
            _ => false,
        }
    }
    
    /// Gets the common type for two types in an operation.
    pub fn common_type(left: &EaType, right: &EaType) -> Option<EaType> {
        if left == right {
            return Some(left.clone());
        }
        
        // Numeric type promotion rules
        match (left, right) {
            // Float types take precedence
            (EaType::F64, _) | (_, EaType::F64) if right.is_numeric() || left.is_numeric() => Some(EaType::F64),
            (EaType::F32, _) | (_, EaType::F32) if right.is_numeric() || left.is_numeric() => Some(EaType::F32),
            
            // Integer type promotion (largest type wins)
            (EaType::I64, _) | (_, EaType::I64) if right.is_integer() || left.is_integer() => Some(EaType::I64),
            (EaType::U64, _) | (_, EaType::U64) if right.is_integer() || left.is_integer() => Some(EaType::U64),
            (EaType::I32, _) | (_, EaType::I32) if right.is_integer() || left.is_integer() => Some(EaType::I32),
            (EaType::U32, _) | (_, EaType::U32) if right.is_integer() || left.is_integer() => Some(EaType::U32),
            (EaType::I16, _) | (_, EaType::I16) if right.is_integer() || left.is_integer() => Some(EaType::I16),
            (EaType::U16, _) | (_, EaType::U16) if right.is_integer() || left.is_integer() => Some(EaType::U16),
            (EaType::I8, _) | (_, EaType::I8) if right.is_integer() || left.is_integer() => Some(EaType::I8),
            (EaType::U8, _) | (_, EaType::U8) if right.is_integer() || left.is_integer() => Some(EaType::U8),
            
            _ => None,
        }
    }
    
    /// Checks if a type supports a specific operation.
    pub fn supports_operation(ty: &EaType, op: TypeOperation) -> bool {
        match op {
            TypeOperation::Arithmetic => ty.is_numeric(),
            TypeOperation::Comparison => ty.is_comparable(),
            TypeOperation::Equality => true, // All types support equality comparison
            TypeOperation::LogicalAnd | TypeOperation::LogicalOr => matches!(ty, EaType::Bool),
            TypeOperation::BitwiseAnd | TypeOperation::BitwiseOr | TypeOperation::BitwiseXor => ty.is_integer(),
            TypeOperation::LeftShift | TypeOperation::RightShift => ty.is_integer(),
            TypeOperation::Negation => ty.is_numeric(),
            TypeOperation::LogicalNot => matches!(ty, EaType::Bool),
            TypeOperation::BitwiseNot => ty.is_integer(),
        }
    }
}

/// Operations that can be performed on types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeOperation {
    Arithmetic,      // +, -, *, /, %
    Comparison,      // <, <=, >, >=
    Equality,        // ==, !=
    LogicalAnd,      // &&
    LogicalOr,       // ||
    BitwiseAnd,      // &
    BitwiseOr,       // |
    BitwiseXor,      // ^
    LeftShift,       // <<
    RightShift,      // >>
    Negation,        // -x
    LogicalNot,      // !x
    BitwiseNot,      // ~x
}

impl SIMDElementType {
    /// Convert SIMDElementType to EaType
    pub fn to_ea_type(&self) -> EaType {
        match self {
            SIMDElementType::I8 => EaType::I8,
            SIMDElementType::I16 => EaType::I16,
            SIMDElementType::I32 => EaType::I32,
            SIMDElementType::I64 => EaType::I64,
            SIMDElementType::U8 => EaType::U8,
            SIMDElementType::U16 => EaType::U16,
            SIMDElementType::U32 => EaType::U32,
            SIMDElementType::U64 => EaType::U64,
            SIMDElementType::F32 => EaType::F32,
            SIMDElementType::F64 => EaType::F64,
        }
    }

    /// Convert EaType to SIMDElementType (if possible)
    pub fn from_ea_type(ea_type: &EaType) -> Option<SIMDElementType> {
        match ea_type {
            EaType::I8 => Some(SIMDElementType::I8),
            EaType::I16 => Some(SIMDElementType::I16),
            EaType::I32 => Some(SIMDElementType::I32),
            EaType::I64 => Some(SIMDElementType::I64),
            EaType::U8 => Some(SIMDElementType::U8),
            EaType::U16 => Some(SIMDElementType::U16),
            EaType::U32 => Some(SIMDElementType::U32),
            EaType::U64 => Some(SIMDElementType::U64),
            EaType::F32 => Some(SIMDElementType::F32),
            EaType::F64 => Some(SIMDElementType::F64),
            _ => None,
        }
    }
}

impl fmt::Display for SIMDElementType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SIMDElementType::I8 => write!(f, "i8"),
            SIMDElementType::I16 => write!(f, "i16"),
            SIMDElementType::I32 => write!(f, "i32"),
            SIMDElementType::I64 => write!(f, "i64"),
            SIMDElementType::U8 => write!(f, "u8"),
            SIMDElementType::U16 => write!(f, "u16"),
            SIMDElementType::U32 => write!(f, "u32"),
            SIMDElementType::U64 => write!(f, "u64"),
            SIMDElementType::F32 => write!(f, "f32"),
            SIMDElementType::F64 => write!(f, "f64"),
        }
    }
}