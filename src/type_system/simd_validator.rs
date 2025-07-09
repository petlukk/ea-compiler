// SIMD-002 Phase 3: SIMD Expression Validation and Type Checking
// Ensures SIMD expressions are semantically correct and hardware-compatible

use crate::ast::{Expr, SIMDExpr, SIMDVectorType, SIMDOperator, SwizzlePattern, ReductionOp, Literal};
use crate::lexer::Position;
use std::collections::HashMap;

/// SIMD expression validator with comprehensive type checking
pub struct SIMDValidator {
    /// Available hardware features for validation
    available_features: Vec<HardwareFeature>,
    /// Variable type context for validation
    type_context: HashMap<String, EaType>,
}

/// Extended type system to include SIMD types
#[derive(Debug, Clone, PartialEq)]
pub enum EaType {
    // Primitive types
    I32, I64, F32, F64, Bool, String,
    
    // SIMD vector types
    SIMD(SIMDVectorType),
    
    // Composite types
    Array(Box<EaType>, usize),
    Function { params: Vec<EaType>, return_type: Box<EaType> },
    
    // Type inference placeholder
    Inferred,
    
    // Error type for failed validation
    Error,
}

/// Hardware feature enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum HardwareFeature {
    SSE, SSE2, SSE3, SSE4, AVX, AVX2, AVX512, NEON, AltiVec,
}

/// Validation errors for SIMD expressions
#[derive(Debug, Clone, PartialEq)]
pub enum SIMDValidationError {
    /// Type mismatch in SIMD operation
    TypeMismatch {
        expected: SIMDVectorType,
        found: SIMDVectorType,
        position: Position,
    },
    
    /// Incompatible SIMD operation
    IncompatibleOperation {
        operator: SIMDOperator,
        left_type: SIMDVectorType,
        right_type: SIMDVectorType,
        position: Position,
    },
    
    /// Hardware feature not available
    UnsupportedHardware {
        required: Vec<HardwareFeature>,
        available: Vec<HardwareFeature>,
        position: Position,
    },
    
    /// Invalid swizzle pattern
    InvalidSwizzle {
        pattern: SwizzlePattern,
        vector_type: SIMDVectorType,
        position: Position,
    },
    
    /// Invalid vector element count
    InvalidElementCount {
        expected: usize,
        found: usize,
        position: Position,
    },
    
    /// Scalar operation on vector type
    ScalarOnVector {
        operation: String,
        vector_type: SIMDVectorType,
        position: Position,
    },
    
    /// Vector operation on scalar type
    VectorOnScalar {
        operation: String,
        scalar_type: EaType,
        position: Position,
    },
    
    /// Invalid reduction operation
    InvalidReduction {
        operation: ReductionOp,
        vector_type: SIMDVectorType,
        position: Position,
    },
    
    /// Broadcast type mismatch
    BroadcastMismatch {
        source_type: EaType,
        target_type: SIMDVectorType,
        position: Position,
    },
    
    /// Invalid number of arguments
    InvalidArguments {
        expected: usize,
        found: usize,
        position: Position,
    },
    
    /// Incompatible types in operation
    IncompatibleTypes {
        left: String,
        right: String,
        operation: String,
        position: Position,
    },
    
    /// Unsupported operation for vector type
    UnsupportedOperation {
        operation: String,
        vector_type: SIMDVectorType,
        position: Position,
    },
}

pub type ValidationResult<T> = Result<T, SIMDValidationError>;

impl SIMDValidator {
    /// Create new validator with available hardware features
    pub fn new(available_features: Vec<HardwareFeature>) -> Self {
        Self {
            available_features,
            type_context: HashMap::new(),
        }
    }
    
    /// Add variable type to context
    pub fn add_variable(&mut self, name: String, var_type: EaType) {
        self.type_context.insert(name, var_type);
    }
    
    /// Validate an expression and return its type
    pub fn validate_expression(&self, expr: &Expr) -> ValidationResult<EaType> {
        match expr {
            Expr::Literal(literal) => Ok(self.literal_type(literal)),
            Expr::Variable(name) => self.validate_variable(name),
            Expr::Binary { left, op, right } => self.validate_binary_expr(left, op, right),
            Expr::Unary { op, expr } => self.validate_unary_expr(op, expr),
            Expr::SIMD(simd_expr) => self.validate_simd_expression(simd_expr),
            Expr::Call { name, args } => self.validate_function_call(name, args),
            Expr::Index { expr, index } => self.validate_index_expr(expr, index),
            Expr::FieldAccess { expr, field } => self.validate_field_access(expr, field),
            Expr::Assignment { target, value } => self.validate_assignment(target, value),
            Expr::Grouping(expr) => self.validate_expression(expr),
        }
    }
    
    /// Validate SIMD expressions with comprehensive type checking
    pub fn validate_simd_expression(&self, simd_expr: &SIMDExpr) -> ValidationResult<EaType> {
        match simd_expr {
            SIMDExpr::VectorLiteral { elements, vector_type, position } => {
                self.validate_vector_literal(elements, vector_type, position)
            }
            
            SIMDExpr::ElementWise { left, operator, right, position } => {
                self.validate_element_wise_operation(left, operator, right, position)
            }
            
            SIMDExpr::Broadcast { value, target_type, position } => {
                self.validate_broadcast_operation(value, target_type, position)
            }
            
            SIMDExpr::Swizzle { vector, pattern, position } => {
                self.validate_swizzle_operation(vector, pattern, position)
            }
            
            SIMDExpr::Reduction { vector, operation, position } => {
                self.validate_reduction_operation(vector, operation, position)
            }
        }
    }
    
    /// Validate vector literal construction
    fn validate_vector_literal(
        &self, 
        elements: &[Expr], 
        vector_type: &Option<SIMDVectorType>,
        position: &Position
    ) -> ValidationResult<EaType> {
        // Validate each element
        let element_types: Result<Vec<_>, _> = elements
            .iter()
            .map(|elem| self.validate_expression(elem))
            .collect();
        let element_types = element_types?;
        
        // Determine target vector type
        let target_type = if let Some(explicit_type) = vector_type {
            explicit_type.clone()
        } else {
            // Infer from element count and types
            self.infer_vector_type(&element_types, elements.len(), position)?
        };
        
        // Validate element count matches vector width
        if elements.len() != target_type.width() {
            return Err(SIMDValidationError::InvalidElementCount {
                expected: target_type.width(),
                found: elements.len(),
                position: *position,
            });
        }
        
        // Validate all elements are compatible with target element type
        let expected_element_type = self.simd_element_type(&target_type);
        for (i, elem_type) in element_types.iter().enumerate() {
            if !self.is_compatible_with_simd_element(elem_type, &expected_element_type) {
                return Err(SIMDValidationError::TypeMismatch {
                    expected: target_type.clone(),
                    found: target_type.clone(), // Simplified for this example
                    position: *position,
                });
            }
        }
        
        // Validate hardware support
        self.validate_hardware_support(&target_type, position)?;
        
        Ok(EaType::SIMD(target_type))
    }
    
    /// Validate element-wise SIMD operations
    fn validate_element_wise_operation(
        &self,
        left: &Expr,
        operator: &SIMDOperator,
        right: &Expr,
        position: &Position
    ) -> ValidationResult<EaType> {
        let left_type = self.validate_expression(left)?;
        let right_type = self.validate_expression(right)?;
        
        // Both operands must be SIMD vectors
        let (left_vector_type, right_vector_type) = match (left_type, right_type) {
            (EaType::SIMD(left_vec), EaType::SIMD(right_vec)) => (left_vec, right_vec),
            (EaType::SIMD(vec_type), scalar_type) => {
                return Err(SIMDValidationError::VectorOnScalar {
                    operation: format!("{}", operator),
                    scalar_type,
                    position: *position,
                });
            }
            (scalar_type, EaType::SIMD(vec_type)) => {
                return Err(SIMDValidationError::VectorOnScalar {
                    operation: format!("{}", operator),
                    scalar_type,
                    position: *position,
                });
            }
            _ => {
                return Err(SIMDValidationError::VectorOnScalar {
                    operation: format!("{}", operator),
                    scalar_type: left_type,
                    position: *position,
                });
            }
        };
        
        // Vector types must be compatible
        if !left_vector_type.is_compatible_with(&right_vector_type) {
            return Err(SIMDValidationError::IncompatibleOperation {
                operator: operator.clone(),
                left_type: left_vector_type.clone(),
                right_type: right_vector_type.clone(),
                position: *position,
            });
        }
        
        // Operator must be valid for the vector types
        if !operator.is_valid_for_types(&left_vector_type, &right_vector_type) {
            return Err(SIMDValidationError::IncompatibleOperation {
                operator: operator.clone(),
                left_type: left_vector_type.clone(),
                right_type: right_vector_type.clone(),
                position: *position,
            });
        }
        
        // Validate hardware support
        self.validate_hardware_support(&left_vector_type, position)?;
        
        Ok(EaType::SIMD(left_vector_type))
    }
    
    /// Validate broadcast operations
    fn validate_broadcast_operation(
        &self,
        value: &Expr,
        target_type: &SIMDVectorType,
        position: &Position
    ) -> ValidationResult<EaType> {
        let value_type = self.validate_expression(value)?;
        let expected_element_type = self.simd_element_type(target_type);
        
        // Value must be compatible with target element type
        if !self.is_compatible_with_simd_element(&value_type, &expected_element_type) {
            return Err(SIMDValidationError::BroadcastMismatch {
                source_type: value_type,
                target_type: target_type.clone(),
                position: *position,
            });
        }
        
        // Validate hardware support
        self.validate_hardware_support(target_type, position)?;
        
        Ok(EaType::SIMD(target_type.clone()))
    }
    
    /// Validate swizzle operations
    fn validate_swizzle_operation(
        &self,
        vector: &Expr,
        pattern: &SwizzlePattern,
        position: &Position
    ) -> ValidationResult<EaType> {
        let vector_type = self.validate_expression(vector)?;
        
        let simd_type = match vector_type {
            EaType::SIMD(simd_type) => simd_type,
            _ => {
                return Err(SIMDValidationError::ScalarOnVector {
                    operation: "swizzle".to_string(),
                    vector_type: SIMDVectorType::F32x4, // Placeholder
                    position: *position,
                });
            }
        };
        
        // Validate swizzle pattern is valid for vector type
        let result_width = match pattern {
            SwizzlePattern::Named(name) => {
                if !self.is_valid_swizzle_name(name, &simd_type) {
                    return Err(SIMDValidationError::InvalidSwizzle {
                        pattern: pattern.clone(),
                        vector_type: simd_type,
                        position: *position,
                    });
                }
                name.len()
            }
            SwizzlePattern::Range { start, end } => {
                if *end >= simd_type.width() || *start > *end {
                    return Err(SIMDValidationError::InvalidSwizzle {
                        pattern: pattern.clone(),
                        vector_type: simd_type,
                        position: *position,
                    });
                }
                end - start + 1
            }
            SwizzlePattern::Indices(indices) => {
                if indices.iter().any(|&idx| idx >= simd_type.width()) {
                    return Err(SIMDValidationError::InvalidSwizzle {
                        pattern: pattern.clone(),
                        vector_type: simd_type,
                        position: *position,
                    });
                }
                indices.len()
            }
        };
        
        // Determine result type based on swizzle width
        let result_type = self.create_swizzle_result_type(&simd_type, result_width)
            .ok_or_else(|| SIMDValidationError::InvalidSwizzle {
                pattern: pattern.clone(),
                vector_type: simd_type,
                position: *position,
            })?;
        
        Ok(EaType::SIMD(result_type))
    }
    
    /// Validate reduction operations
    fn validate_reduction_operation(
        &self,
        vector: &Expr,
        operation: &ReductionOp,
        position: &Position
    ) -> ValidationResult<EaType> {
        let vector_type = self.validate_expression(vector)?;
        
        let simd_type = match vector_type {
            EaType::SIMD(simd_type) => simd_type,
            _ => {
                return Err(SIMDValidationError::ScalarOnVector {
                    operation: format!("{:?}", operation),
                    vector_type: SIMDVectorType::F32x4, // Placeholder
                    position: *position,
                });
            }
        };
        
        // Validate operation is valid for vector type
        if !self.is_valid_reduction_for_type(operation, &simd_type) {
            return Err(SIMDValidationError::InvalidReduction {
                operation: operation.clone(),
                vector_type: simd_type,
                position: *position,
            });
        }
        
        // Reduction returns scalar element type
        let scalar_type = self.simd_element_type(&simd_type);
        Ok(scalar_type)
    }
    
    /// Helper methods for validation
    
    fn validate_hardware_support(
        &self, 
        vector_type: &SIMDVectorType, 
        position: &Position
    ) -> ValidationResult<()> {
        let required_features = vector_type.required_features();
        
        for required in &required_features {
            if !self.available_features.contains(required) {
                return Err(SIMDValidationError::UnsupportedHardware {
                    required: required_features,
                    available: self.available_features.clone(),
                    position: *position,
                });
            }
        }
        
        Ok(())
    }
    
    fn simd_element_type(&self, vector_type: &SIMDVectorType) -> EaType {
        match vector_type.element_type() {
            "f32" => EaType::F32,
            "f64" => EaType::F64,
            "i32" => EaType::I32,
            "i64" => EaType::I64,
            "i16" => EaType::I32, // Promote to i32 for compatibility
            "i8" => EaType::I32,  // Promote to i32 for compatibility
            "u32" => EaType::I32, // Treat as i32 for simplicity
            "u16" => EaType::I32,
            "u8" => EaType::I32,
            "bool" => EaType::Bool,
            _ => EaType::Error,
        }
    }
    
    fn is_compatible_with_simd_element(&self, value_type: &EaType, element_type: &EaType) -> bool {
        match (value_type, element_type) {
            // Exact matches
            (a, b) if a == b => true,
            
            // Integer promotions
            (EaType::I32, EaType::I64) => true,
            
            // Literal integer compatibility
            (EaType::I64, EaType::I32) => true, // Allow literals to fit
            
            _ => false,
        }
    }
    
    fn infer_vector_type(
        &self, 
        element_types: &[EaType], 
        count: usize, 
        position: &Position
    ) -> ValidationResult<SIMDVectorType> {
        // Determine common element type
        let element_type = if element_types.is_empty() {
            return Err(SIMDValidationError::InvalidElementCount {
                expected: 1,
                found: 0,
                position: *position,
            });
        } else {
            &element_types[0]
        };
        
        // Ensure all elements have compatible types
        for elem_type in element_types {
            if !self.is_compatible_with_simd_element(elem_type, element_type) {
                return Err(SIMDValidationError::TypeMismatch {
                    expected: SIMDVectorType::F32x4, // Placeholder
                    found: SIMDVectorType::F32x4,    // Placeholder
                    position: *position,
                });
            }
        }
        
        // Map to SIMD vector type
        let vector_type = match (element_type, count) {
            (EaType::F32, 2) => SIMDVectorType::F32x2,
            (EaType::F32, 4) => SIMDVectorType::F32x4,
            (EaType::F32, 8) => SIMDVectorType::F32x8,
            (EaType::F32, 16) => SIMDVectorType::F32x16,
            (EaType::F64, 2) => SIMDVectorType::F64x2,
            (EaType::F64, 4) => SIMDVectorType::F64x4,
            (EaType::F64, 8) => SIMDVectorType::F64x8,
            (EaType::I32, 2) => SIMDVectorType::I32x2,
            (EaType::I32, 4) => SIMDVectorType::I32x4,
            (EaType::I32, 8) => SIMDVectorType::I32x8,
            (EaType::I32, 16) => SIMDVectorType::I32x16,
            (EaType::Bool, 8) => SIMDVectorType::Mask8,
            (EaType::Bool, 16) => SIMDVectorType::Mask16,
            (EaType::Bool, 32) => SIMDVectorType::Mask32,
            (EaType::Bool, 64) => SIMDVectorType::Mask64,
            _ => {
                return Err(SIMDValidationError::InvalidElementCount {
                    expected: 4, // Common default
                    found: count,
                    position: *position,
                });
            }
        };
        
        Ok(vector_type)
    }
    
    fn is_valid_swizzle_name(&self, name: &str, vector_type: &SIMDVectorType) -> bool {
        let width = vector_type.width();
        
        // Check length doesn't exceed vector width
        if name.len() > width {
            return false;
        }
        
        // Check all characters are valid component names
        name.chars().all(|c| match c {
            'x' => width >= 1,
            'y' => width >= 2,
            'z' => width >= 3,
            'w' => width >= 4,
            _ => false,
        })
    }
    
    fn create_swizzle_result_type(&self, source_type: &SIMDVectorType, width: usize) -> Option<SIMDVectorType> {
        let element_type = source_type.element_type();
        
        match (element_type, width) {
            ("f32", 1) => None, // Scalar result, not a vector
            ("f32", 2) => Some(SIMDVectorType::F32x2),
            ("f32", 3) => Some(SIMDVectorType::F32x4), // Round up to next supported width
            ("f32", 4) => Some(SIMDVectorType::F32x4),
            ("i32", 1) => None,
            ("i32", 2) => Some(SIMDVectorType::I32x2),
            ("i32", 3) => Some(SIMDVectorType::I32x4),
            ("i32", 4) => Some(SIMDVectorType::I32x4),
            _ => None,
        }
    }
    
    fn is_valid_reduction_for_type(&self, operation: &ReductionOp, vector_type: &SIMDVectorType) -> bool {
        let element_type = vector_type.element_type();
        
        match operation {
            ReductionOp::Sum | ReductionOp::Product => {
                matches!(element_type, "f32" | "f64" | "i32" | "i64" | "i16" | "i8" | "u32" | "u16" | "u8")
            }
            ReductionOp::Min | ReductionOp::Max => {
                matches!(element_type, "f32" | "f64" | "i32" | "i64" | "i16" | "i8" | "u32" | "u16" | "u8")
            }
            ReductionOp::And | ReductionOp::Or | ReductionOp::Xor => {
                matches!(element_type, "i32" | "i64" | "i16" | "i8" | "u32" | "u16" | "u8" | "bool")
            }
            ReductionOp::Any | ReductionOp::All => {
                matches!(element_type, "bool")
            }
        }
    }
    
    fn literal_type(&self, literal: &Literal) -> EaType {
        match literal {
            Literal::Integer(_) => EaType::I32,
            Literal::Float(_) => EaType::F32,
            Literal::String(_) => EaType::String,
            Literal::Boolean(_) => EaType::Bool,
            Literal::Vector { vector_type: Some(vec_type), .. } => EaType::SIMD(vec_type.clone()),
            Literal::Vector { .. } => EaType::Inferred,
        }
    }
    
    fn validate_variable(&self, name: &str) -> ValidationResult<EaType> {
        self.type_context.get(name)
            .cloned()
            .ok_or_else(|| SIMDValidationError::TypeMismatch {
                expected: SIMDVectorType::F32x4, // Placeholder
                found: SIMDVectorType::F32x4,    // Placeholder
                position: Position { line: 1, column: 1 },
            })
    }
    
    // Simplified stubs for other validation methods
    fn validate_binary_expr(&self, left: &Expr, op: &BinaryOp, right: &Expr) -> ValidationResult<EaType> {
        let left_type = self.validate_expression(left)?;
        let right_type = self.validate_expression(right)?;
        
        match (left_type, right_type) {
            // SIMD operations
            (EaType::SIMD(left_vec), EaType::SIMD(right_vec)) => {
                if left_vec == right_vec {
                    Ok(EaType::SIMD(left_vec))
                } else {
                    Err(SIMDValidationError::TypeMismatch {
                        expected: left_vec,
                        found: right_vec,
                        position: Position { line: 1, column: 1 },
                    })
                }
            }
            // Scalar operations
            (EaType::I32, EaType::I32) => Ok(EaType::I32),
            (EaType::F32, EaType::F32) => Ok(EaType::F32),
            (EaType::I64, EaType::I64) => Ok(EaType::I64),
            (EaType::F64, EaType::F64) => Ok(EaType::F64),
            (EaType::Bool, EaType::Bool) => Ok(EaType::Bool),
            // Type mismatches
            (left, right) => Err(SIMDValidationError::IncompatibleTypes {
                left: format!("{:?}", left),
                right: format!("{:?}", right),
                operation: format!("{:?}", op),
                position: Position { line: 1, column: 1 },
            })
        }
    }
    
    fn validate_unary_expr(&self, op: &UnaryOp, expr: &Expr) -> ValidationResult<EaType> {
        let expr_type = self.validate_expression(expr)?;
        
        match op {
            UnaryOp::Negate => {
                match expr_type {
                    EaType::I32 | EaType::I64 | EaType::F32 | EaType::F64 => Ok(expr_type),
                    EaType::SIMD(vec_type) => {
                        // SIMD negation is supported for numeric vector types
                        match vec_type {
                            SIMDVectorType::F32x4 | SIMDVectorType::F64x2 |
                            SIMDVectorType::I32x4 | SIMDVectorType::I64x2 => Ok(expr_type),
                            _ => Err(SIMDValidationError::UnsupportedOperation {
                                operation: "negation".to_string(),
                                vector_type: vec_type,
                                position: Position { line: 1, column: 1 },
                            })
                        }
                    }
                    _ => Err(SIMDValidationError::UnsupportedOperation {
                        operation: "negation".to_string(),
                        vector_type: SIMDVectorType::F32x4, // Fallback
                        position: Position { line: 1, column: 1 },
                    })
                }
            }
            UnaryOp::Not => {
                match expr_type {
                    EaType::Bool => Ok(expr_type),
                    EaType::SIMD(vec_type) => {
                        // Logical NOT for boolean vectors
                        match vec_type {
                            SIMDVectorType::I32x4 | SIMDVectorType::I64x2 => Ok(expr_type),
                            _ => Err(SIMDValidationError::UnsupportedOperation {
                                operation: "logical not".to_string(),
                                vector_type: vec_type,
                                position: Position { line: 1, column: 1 },
                            })
                        }
                    }
                    _ => Err(SIMDValidationError::UnsupportedOperation {
                        operation: "logical not".to_string(),
                        vector_type: SIMDVectorType::F32x4, // Fallback
                        position: Position { line: 1, column: 1 },
                    })
                }
            }
        }
    }
    
    fn validate_function_call(&self, name: &str, args: &[Expr]) -> ValidationResult<EaType> {
        match name {
            // Standard I/O functions
            "println" | "print" => {
                if args.len() == 1 {
                    self.validate_expression(&args[0])?;
                    Ok(EaType::Void)
                } else {
                    Err(SIMDValidationError::InvalidArguments {
                        expected: 1,
                        found: args.len(),
                        position: Position { line: 1, column: 1 },
                    })
                }
            }
            // SIMD functions
            "simd_add" | "simd_sub" | "simd_mul" | "simd_div" => {
                if args.len() == 2 {
                    let left_type = self.validate_expression(&args[0])?;
                    let right_type = self.validate_expression(&args[1])?;
                    
                    match (left_type, right_type) {
                        (EaType::SIMD(left_vec), EaType::SIMD(right_vec)) if left_vec == right_vec => {
                            Ok(EaType::SIMD(left_vec))
                        }
                        _ => Err(SIMDValidationError::IncompatibleTypes {
                            left: format!("{:?}", left_type),
                            right: format!("{:?}", right_type),
                            operation: name.to_string(),
                            position: Position { line: 1, column: 1 },
                        })
                    }
                } else {
                    Err(SIMDValidationError::InvalidArguments {
                        expected: 2,
                        found: args.len(),
                        position: Position { line: 1, column: 1 },
                    })
                }
            }
            // Other functions return i32 by default
            _ => Ok(EaType::I32),
        }
    }
    
    fn validate_index_expr(&self, expr: &Expr, index: &Expr) -> ValidationResult<EaType> {
        let expr_type = self.validate_expression(expr)?;
        match expr_type {
            EaType::Array(element_type, _) => Ok(*element_type),
            EaType::SIMD(vector_type) => Ok(self.simd_element_type(&vector_type)),
            _ => Ok(EaType::Error),
        }
    }
    
    fn validate_field_access(&self, expr: &Expr, field: &str) -> ValidationResult<EaType> {
        let expr_type = self.validate_expression(expr)?;
        
        match expr_type {
            EaType::SIMD(vec_type) => {
                // SIMD vector element access (e.g., vec.x, vec.y, vec.z, vec.w)
                match field {
                    "x" | "y" | "z" | "w" => {
                        Ok(self.simd_element_type(&vec_type))
                    }
                    "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" => {
                        // Numeric index access
                        let index: usize = field.parse().unwrap_or(0);
                        let width = self.simd_width(&vec_type);
                        
                        if index < width {
                            Ok(self.simd_element_type(&vec_type))
                        } else {
                            Err(SIMDValidationError::InvalidArguments {
                                expected: width,
                                found: index + 1,
                                position: Position { line: 1, column: 1 },
                            })
                        }
                    }
                    _ => Err(SIMDValidationError::UnsupportedOperation {
                        operation: format!("field access .{}", field),
                        vector_type: vec_type,
                        position: Position { line: 1, column: 1 },
                    })
                }
            }
            // Struct field access (simplified for now)
            _ => Ok(EaType::I32)
        }
    }
    
    fn validate_assignment(&self, target: &Expr, value: &Expr) -> ValidationResult<EaType> {
        let target_type = self.validate_expression(target)?;
        let value_type = self.validate_expression(value)?;
        
        // Validate type compatibility for assignment
        match (target_type.clone(), value_type) {
            // Exact type matches
            (EaType::SIMD(target_vec), EaType::SIMD(value_vec)) if target_vec == value_vec => {
                Ok(target_type)
            }
            (EaType::I32, EaType::I32) |
            (EaType::F32, EaType::F32) |
            (EaType::I64, EaType::I64) |
            (EaType::F64, EaType::F64) |
            (EaType::Bool, EaType::Bool) => {
                Ok(target_type)
            }
            // Type conversions (implicit)
            (EaType::F64, EaType::F32) |
            (EaType::I64, EaType::I32) => {
                Ok(target_type)
            }
            // Incompatible types
            (target, value) => {
                Err(SIMDValidationError::TypeMismatch {
                    expected: match target {
                        EaType::SIMD(vec_type) => vec_type,
                        _ => SIMDVectorType::F32x4, // Default fallback
                    },
                    found: match value {
                        EaType::SIMD(vec_type) => vec_type,
                        _ => SIMDVectorType::F32x4, // Default fallback
                    },
                    position: Position { line: 1, column: 1 },
                })
            }
        }
    }
}

/// Display implementations for beautiful error messages
impl std::fmt::Display for SIMDValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SIMDValidationError::TypeMismatch { expected, found, position } => {
                write!(f, "Type mismatch at {}:{}: expected {}, found {}", 
                       position.line, position.column, expected, found)
            }
            SIMDValidationError::IncompatibleOperation { operator, left_type, right_type, position } => {
                write!(f, "Incompatible SIMD operation at {}:{}: {} {} {}", 
                       position.line, position.column, left_type, operator, right_type)
            }
            SIMDValidationError::UnsupportedHardware { required, available, position } => {
                write!(f, "Unsupported hardware features at {}:{}: requires {:?}, available {:?}", 
                       position.line, position.column, required, available)
            }
            SIMDValidationError::InvalidSwizzle { pattern, vector_type, position } => {
                write!(f, "Invalid swizzle pattern at {}:{}: {:?} on {}", 
                       position.line, position.column, pattern, vector_type)
            }
            SIMDValidationError::InvalidElementCount { expected, found, position } => {
                write!(f, "Invalid element count at {}:{}: expected {}, found {}", 
                       position.line, position.column, expected, found)
            }
            SIMDValidationError::ScalarOnVector { operation, vector_type, position } => {
                write!(f, "Cannot apply scalar operation '{}' to vector type {} at {}:{}", 
                       operation, vector_type, position.line, position.column)
            }
            SIMDValidationError::VectorOnScalar { operation, scalar_type, position } => {
                write!(f, "Cannot apply vector operation '{}' to scalar type {:?} at {}:{}", 
                       operation, scalar_type, position.line, position.column)
            }
            SIMDValidationError::InvalidReduction { operation, vector_type, position } => {
                write!(f, "Invalid reduction operation {:?} on {} at {}:{}", 
                       operation, vector_type, position.line, position.column)
            }
            SIMDValidationError::BroadcastMismatch { source_type, target_type, position } => {
                write!(f, "Broadcast type mismatch at {}:{}: cannot broadcast {:?} to {}", 
                       position.line, position.column, source_type, target_type)
            }
            SIMDValidationError::InvalidArguments { expected, found, position } => {
                write!(f, "Invalid number of arguments at {}:{}: expected {}, found {}", 
                       position.line, position.column, expected, found)
            }
            SIMDValidationError::IncompatibleTypes { left, right, operation, position } => {
                write!(f, "Incompatible types at {}:{}: cannot apply '{}' to {} and {}", 
                       position.line, position.column, operation, left, right)
            }
            SIMDValidationError::UnsupportedOperation { operation, vector_type, position } => {
                write!(f, "Unsupported operation at {}:{}: '{}' not supported for {}", 
                       position.line, position.column, operation, vector_type)
            }
        }
    }
}

// Required types from other modules
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add, Subtract, Multiply, Divide,
    Equal, NotEqual, Less, LessEqual, Greater, GreaterEqual,
    And, Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Negate, Not,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_literal_validation() {
        let mut validator = SIMDValidator::new(vec![HardwareFeature::AVX]);
        
        // Valid f32x4 literal
        let elements = vec![
            Expr::Literal(Literal::Float(1.0)),
            Expr::Literal(Literal::Float(2.0)),
            Expr::Literal(Literal::Float(3.0)),
            Expr::Literal(Literal::Float(4.0)),
        ];
        
        let result = validator.validate_vector_literal(
            &elements, 
            &Some(SIMDVectorType::F32x4),
            &Position { line: 1, column: 1 }
        );
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), EaType::SIMD(SIMDVectorType::F32x4));
    }

    #[test]
    fn test_element_wise_validation() {
        let mut validator = SIMDValidator::new(vec![HardwareFeature::AVX]);
        validator.add_variable("vec1".to_string(), EaType::SIMD(SIMDVectorType::F32x4));
        validator.add_variable("vec2".to_string(), EaType::SIMD(SIMDVectorType::F32x4));
        
        let left = Expr::Variable("vec1".to_string());
        let right = Expr::Variable("vec2".to_string());
        let operator = SIMDOperator::DotAdd;
        let position = Position { line: 1, column: 1 };
        
        let result = validator.validate_element_wise_operation(&left, &operator, &right, &position);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), EaType::SIMD(SIMDVectorType::F32x4));
    }

    #[test]
    fn test_hardware_validation() {
        let validator = SIMDValidator::new(vec![HardwareFeature::SSE]); // Only SSE available
        
        let position = Position { line: 1, column: 1 };
        let result = validator.validate_hardware_support(&SIMDVectorType::F32x8, &position); // Requires AVX
        
        assert!(result.is_err());
        if let Err(SIMDValidationError::UnsupportedHardware { required, .. }) = result {
            assert!(required.contains(&HardwareFeature::AVX));
        }
    }

    #[test]
    fn test_swizzle_validation() {
        let mut validator = SIMDValidator::new(vec![HardwareFeature::AVX]);
        validator.add_variable("vec".to_string(), EaType::SIMD(SIMDVectorType::F32x4));
        
        let vector = Expr::Variable("vec".to_string());
        let pattern = SwizzlePattern::Named("xyz".to_string());
        let position = Position { line: 1, column: 1 };
        
        let result = validator.validate_swizzle_operation(&vector, &pattern, &position);
        
        assert!(result.is_ok());
        // xyz swizzle on f32x4 should return f32x4 (rounded up from 3 components)
        assert_eq!(result.unwrap(), EaType::SIMD(SIMDVectorType::F32x4));
    }

    #[test]
    fn test_reduction_validation() {
        let mut validator = SIMDValidator::new(vec![HardwareFeature::AVX]);
        validator.add_variable("vec".to_string(), EaType::SIMD(SIMDVectorType::F32x4));
        
        let vector = Expr::Variable("vec".to_string());
        let operation = ReductionOp::Sum;
        let position = Position { line: 1, column: 1 };
        
        let result = validator.validate_reduction_operation(&vector, &operation, &position);
        
        assert!(result.is_ok());
        // Sum reduction on f32x4 should return f32 scalar
        assert_eq!(result.unwrap(), EaType::F32);
    }

    #[test]
    fn test_type_incompatibility() {
        let mut validator = SIMDValidator::new(vec![HardwareFeature::AVX]);
        validator.add_variable("vec1".to_string(), EaType::SIMD(SIMDVectorType::F32x4));
        validator.add_variable("vec2".to_string(), EaType::SIMD(SIMDVectorType::I32x4));
        
        let left = Expr::Variable("vec1".to_string());
        let right = Expr::Variable("vec2".to_string());
        let operator = SIMDOperator::DotAdd;
        let position = Position { line: 1, column: 1 };
        
        let result = validator.validate_element_wise_operation(&left, &operator, &right, &position);
        
        assert!(result.is_err());
        if let Err(SIMDValidationError::IncompatibleOperation { .. }) = result {
            // Expected error for incompatible types
        } else {
            panic!("Expected IncompatibleOperation error");
        }
    }
}