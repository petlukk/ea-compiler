// src/type_system/mod.rs
//! Type system for the Eä programming language.
//! 
//! This module implements type checking, type inference, and type compatibility
//! checking for all Eä language constructs.

use crate::ast::{Expr, Stmt, Literal, BinaryOp, UnaryOp, TypeAnnotation};
use crate::error::{CompileError, Result};
use crate::lexer::Position;
use std::collections::HashMap;
use std::fmt;

pub mod hardware;

/// Represents all types in the Eä programming language.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EaType {
    I8, I16, I32, I64,
    U8, U16, U32, U64,
    F32, F64,
    Bool,
    String,
    Unit,
    Array(Box<EaType>),
    Reference(Box<EaType>),
    Function(Box<FunctionType>),
    Custom(String),
    Generic(String),
    Error,
    
    // SIMD types
    SIMDVector {
        element_type: Box<EaType>,
        width: usize,
        vector_type: crate::ast::SIMDVectorType,
    },
}

/// Represents a function type with parameters and return type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionType {
    pub params: Vec<EaType>,
    pub return_type: Box<EaType>,
    pub is_variadic: bool,
}

/// Type checking context that maintains type information for variables and functions.
#[derive(Debug, Clone)]
pub struct TypeContext {
    pub variables: HashMap<String, EaType>,
    pub functions: HashMap<String, FunctionType>,
    pub current_function_return: Option<EaType>,
}

/// Main type checker for the Eä language.
pub struct TypeChecker {
    context: TypeContext,
    hardware_detector: hardware::HardwareDetector,
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
            EaType::Reference(inner_type) => write!(f, "&{}", inner_type),
            EaType::Function(func_type) => write!(f, "{}", func_type),
            EaType::Custom(name) => write!(f, "{}", name),
            EaType::Generic(name) => write!(f, "{}", name),
            EaType::Error => write!(f, "<e>"),
            EaType::SIMDVector { vector_type, .. } => write!(f, "{}", vector_type),
        }
    }
}

impl fmt::Display for FunctionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        for (i, param) in self.params.iter().enumerate() {
            if i > 0 { write!(f, ", ")?; }
            write!(f, "{}", param)?;
        }
        if self.is_variadic {
            if !self.params.is_empty() { write!(f, ", ")?; }
            write!(f, "...")?;
        }
        write!(f, ") -> {}", self.return_type)
    }
}

impl EaType {
    pub fn is_integer(&self) -> bool {
        matches!(self, 
            EaType::I8 | EaType::I16 | EaType::I32 | EaType::I64 |
            EaType::U8 | EaType::U16 | EaType::U32 | EaType::U64
        )
    }
    
    pub fn is_float(&self) -> bool {
        matches!(self, EaType::F32 | EaType::F64)
    }
    
    pub fn is_numeric(&self) -> bool {
        self.is_integer() || self.is_float()
    }
    
    pub fn is_comparable(&self) -> bool {
        self.is_numeric() || matches!(self, EaType::String)
    }
    
    pub fn is_simd(&self) -> bool {
        matches!(self, EaType::SIMDVector { .. })
    }
    
    pub fn simd_element_type(&self) -> Option<&EaType> {
        match self {
            EaType::SIMDVector { element_type, .. } => Some(element_type),
            _ => None,
        }
    }
    
    pub fn simd_width(&self) -> Option<usize> {
        match self {
            EaType::SIMDVector { width, .. } => Some(*width),
            _ => None,
        }
    }
    
    pub fn simd_vector_type(&self) -> Option<&crate::ast::SIMDVectorType> {
        match self {
            EaType::SIMDVector { vector_type, .. } => Some(vector_type),
            _ => None,
        }
    }
}

impl TypeContext {
    pub fn new() -> Self {
        let mut context = Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            current_function_return: None,
        };
        
        // Add built-in print function
        context.functions.insert(
            "print".to_string(),
            FunctionType {
                params: vec![EaType::String],
                return_type: Box::new(EaType::Unit),
                is_variadic: false,
            }
        );
        
        context
    }
    
    pub fn enter_scope(&self) -> Self {
        self.clone()
    }
    
    pub fn define_variable(&mut self, name: String, ty: EaType) {
        self.variables.insert(name, ty);
    }
    
    pub fn get_variable_type(&self, name: &str) -> Option<&EaType> {
        self.variables.get(name)
    }
    
    pub fn define_function(&mut self, name: String, func_type: FunctionType) {
        self.functions.insert(name, func_type);
    }
    
    pub fn get_function_type(&self, name: &str) -> Option<&FunctionType> {
        self.functions.get(name)
    }
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut checker = Self {
            context: TypeContext::new(),
            hardware_detector: hardware::HardwareDetector::new(),
        };
        checker.add_builtin_functions();
        checker
    }
    
    /// Adds built-in functions to the type checker
    fn add_builtin_functions(&mut self) {
        // print(string) -> void
        let print_type = FunctionType {
            params: vec![EaType::String],
            return_type: Box::new(EaType::Unit),
            is_variadic: false,
        };
        self.context.define_function("print".to_string(), print_type);
        
        // print_i32(i32) -> void
        let print_i32_type = FunctionType {
            params: vec![EaType::I32],
            return_type: Box::new(EaType::Unit),
            is_variadic: false,
        };
        self.context.define_function("print_i32".to_string(), print_i32_type);
        
        // print_f32(f32) -> void
        let print_f32_type = FunctionType {
            params: vec![EaType::F32],
            return_type: Box::new(EaType::Unit),
            is_variadic: false,
        };
        self.context.define_function("print_f32".to_string(), print_f32_type);
        
        // printf(string, ...) -> i32 (external)
        let printf_type = FunctionType {
            params: vec![EaType::String],
            return_type: Box::new(EaType::I32),
            is_variadic: true,
        };
        self.context.define_function("printf".to_string(), printf_type);
        
        // puts(string) -> i32 (external)
        let puts_type = FunctionType {
            params: vec![EaType::String],
            return_type: Box::new(EaType::I32),
            is_variadic: false,
        };
        self.context.define_function("puts".to_string(), puts_type);
    }
    
    /// Create a type checker for a specific target architecture.
    pub fn for_target(target_arch: &str) -> Self {
        let mut checker = Self {
            context: TypeContext::new(),
            hardware_detector: hardware::HardwareDetector::for_target(target_arch),
        };
        checker.add_builtin_functions();
        checker
    }
    
    /// Gets a reference to the type context (for testing)
    pub fn context(&self) -> &TypeContext {
        &self.context
    }
    
    /// Gets a mutable reference to the type context (for testing)
    pub fn context_mut(&mut self) -> &mut TypeContext {
        &mut self.context
    }
    
    /// Gets a reference to the hardware detector.
    pub fn hardware_detector(&self) -> &hardware::HardwareDetector {
        &self.hardware_detector
    }
    
    /// Get optimization recommendations for a SIMD vector type.
    pub fn get_simd_optimization_hints(&self, vector_type: &crate::ast::SIMDVectorType) -> Vec<String> {
        self.hardware_detector.optimization_recommendations(vector_type)
    }
    
    /// Type checks a complete program.
    pub fn check_program(&mut self, program: &[Stmt]) -> Result<TypeContext> {
        for stmt in program {
            self.check_statement(stmt)?;
        }
        Ok(self.context.clone())
    }
    
    fn check_statement(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::FunctionDeclaration { name, params, return_type, body } => {
                self.check_function_declaration(name, params, return_type, body)
            },
            Stmt::VarDeclaration { name, type_annotation, initializer } => {
                self.check_var_declaration(name, type_annotation, initializer)
            },
            Stmt::Expression(expr) => {
                self.check_expression(expr)?;
                Ok(())
            },
            Stmt::Return(expr) => self.check_return_statement(expr),
            Stmt::Block(stmts) => self.check_block(stmts),
            Stmt::If { condition, then_branch, else_branch } => {
                self.check_if_statement(condition, then_branch, else_branch)
            },
            Stmt::While { condition, body } => {
                self.check_while_statement(condition, body)
            },
            Stmt::For { initializer, condition, increment, body } => {
                self.check_for_statement(initializer, condition, increment, body)
            },
        }
    }
    
    fn check_function_declaration(
        &mut self,
        name: &str,
        params: &[crate::ast::Parameter],
        return_type: &Option<TypeAnnotation>,
        body: &Box<Stmt>
    ) -> Result<()> {
        let mut param_types = Vec::new();
        for param in params {
            let param_type = self.annotation_to_type(&param.type_annotation)?;
            param_types.push(param_type);
        }
        
        let return_ea_type = match return_type {
            Some(type_ann) => self.annotation_to_type(type_ann)?,
            None => EaType::Unit,
        };
        
        let func_type = FunctionType {
            params: param_types.clone(),
            return_type: Box::new(return_ea_type.clone()),
            is_variadic: false,
        };
        
        self.context.define_function(name.to_string(), func_type);
        
        let mut function_context = self.context.enter_scope();
        function_context.current_function_return = Some(return_ea_type);
        
        for (param, param_type) in params.iter().zip(param_types.iter()) {
            function_context.define_variable(param.name.clone(), param_type.clone());
        }
        
        let old_context = std::mem::replace(&mut self.context, function_context);
        let result = self.check_statement(body);
        self.context = old_context;
        
        result
    }
    
    fn check_var_declaration(
        &mut self,
        name: &str,
        type_annotation: &Option<TypeAnnotation>,
        initializer: &Option<Expr>
    ) -> Result<()> {
        let var_type = match (type_annotation, initializer) {
            (Some(type_ann), Some(init)) => {
                let declared_type = self.annotation_to_type(type_ann)?;
                let init_type = self.check_expression(init)?;
                
                if !self.types_compatible(&declared_type, &init_type) {
                    return Err(CompileError::type_error(
                        format!(
                            "Type mismatch in variable '{}': declared as {:?}, initialized with {:?}",
                            name, declared_type, init_type
                        ),
                        Position::new(0, 0, 0),
                    ));
                }
                
                declared_type
            },
            (Some(type_ann), None) => {
                self.annotation_to_type(type_ann)?
            },
            (None, Some(init)) => {
                self.check_expression(init)?
            },
            (None, None) => {
                return Err(CompileError::type_error(
                    format!("Variable '{}' must have either a type annotation or an initializer", name),
                    Position::new(0, 0, 0),
                ));
            }
        };
        
        self.context.define_variable(name.to_string(), var_type);
        Ok(())
    }
    
    fn check_return_statement(&mut self, expr: &Option<Expr>) -> Result<()> {
        let return_type = match expr {
            Some(e) => self.check_expression(e)?,
            None => EaType::Unit,
        };
        
        if let Some(expected_return) = &self.context.current_function_return {
            if !self.types_compatible(expected_return, &return_type) {
                return Err(CompileError::type_error(
                    format!(
                        "Return type mismatch: expected {:?}, got {:?}",
                        expected_return, return_type
                    ),
                    Position::new(0, 0, 0),
                ));
            }
        }
        
        Ok(())
    }
    
    fn check_block(&mut self, stmts: &[Stmt]) -> Result<()> {
        let block_context = self.context.enter_scope();
        let old_context = std::mem::replace(&mut self.context, block_context);
        
        let mut result = Ok(());
        for stmt in stmts {
            if let Err(e) = self.check_statement(stmt) {
                result = Err(e);
                break;
            }
        }
        
        self.context = old_context;
        result
    }
    
    fn check_if_statement(
        &mut self,
        condition: &Expr,
        then_branch: &Box<Stmt>,
        else_branch: &Option<Box<Stmt>>
    ) -> Result<()> {
        let condition_type = self.check_expression(condition)?;
        if !matches!(condition_type, EaType::Bool) {
            return Err(CompileError::type_error(
                format!("If condition must be boolean, got {:?}", condition_type),
                Position::new(0, 0, 0),
            ));
        }
        
        self.check_statement(then_branch)?;
        
        if let Some(else_stmt) = else_branch {
            self.check_statement(else_stmt)?;
        }
        
        Ok(())
    }
    
    fn check_while_statement(&mut self, condition: &Expr, body: &Box<Stmt>) -> Result<()> {
        let condition_type = self.check_expression(condition)?;
        if !matches!(condition_type, EaType::Bool) {
            return Err(CompileError::type_error(
                format!("While condition must be boolean, got {:?}", condition_type),
                Position::new(0, 0, 0),
            ));
        }
        
        self.check_statement(body)
    }
    
    fn check_for_statement(
        &mut self,
        initializer: &Option<Box<Stmt>>,
        condition: &Option<Expr>,
        increment: &Option<Expr>,
        body: &Box<Stmt>
    ) -> Result<()> {
        let loop_context = self.context.enter_scope();
        let old_context = std::mem::replace(&mut self.context, loop_context);
        
        let mut result = Ok(());
        
        if let Some(init) = initializer {
            if let Err(e) = self.check_statement(init) {
                result = Err(e);
            }
        }
        
        if result.is_ok() {
            if let Some(cond) = condition {
                match self.check_expression(cond) {
                    Ok(condition_type) => {
                        if !matches!(condition_type, EaType::Bool) {
                            result = Err(CompileError::type_error(
                                format!("For condition must be boolean, got {:?}", condition_type),
                                Position::new(0, 0, 0),
                            ));
                        }
                    },
                    Err(e) => result = Err(e),
                }
            }
        }
        
        if result.is_ok() {
            if let Some(inc) = increment {
                if let Err(e) = self.check_expression(inc) {
                    result = Err(e);
                }
            }
        }
        
        if result.is_ok() {
            result = self.check_statement(body);
        }
        
        self.context = old_context;
        result
    }
    
    pub fn check_expression(&mut self, expr: &Expr) -> Result<EaType> {
        match expr {
            Expr::Literal(lit) => Ok(self.literal_type(lit)),
            Expr::Variable(name) => self.check_variable(name),
            Expr::Binary(left, op, right) => self.check_binary_expression(left, op, right),
            Expr::Unary(op, expr) => self.check_unary_expression(op, expr),
            Expr::Call(callee, args) => self.check_function_call(callee, args),
            Expr::Grouping(expr) => self.check_expression(expr),
            Expr::Index(array, index) => self.check_index_expression(array, index),
            Expr::FieldAccess(object, field) => self.check_field_access(object, field),
            Expr::SIMD(simd_expr) => self.check_simd_expression(simd_expr),
        }
    }
    
    fn literal_type(&self, literal: &Literal) -> EaType {
        match literal {
            Literal::Integer(_) => EaType::I64,
            Literal::Float(_) => EaType::F64,
            Literal::String(_) => EaType::String,
            Literal::Boolean(_) => EaType::Bool,
            Literal::Vector { elements, vector_type } => {
                if let Some(vtype) = vector_type {
                    // Create proper SIMD vector type
                    let element_type = self.simd_vector_type_to_element_type(vtype);
                    EaType::SIMDVector {
                        element_type: Box::new(element_type),
                        width: vtype.width(),
                        vector_type: vtype.clone(),
                    }
                } else {
                    // Generic vector without specific SIMD type - infer from elements
                    if let Some(first_element) = elements.first() {
                        self.literal_type(first_element)
                    } else {
                        EaType::Error
                    }
                }
            }
        }
    }
    
    fn check_variable(&self, name: &str) -> Result<EaType> {
        self.context.get_variable_type(name)
            .cloned()
            .ok_or_else(|| CompileError::type_error(
                format!("Variable '{}' not found", name),
                Position::new(0, 0, 0),
            ))
    }
    
    fn check_binary_expression(&mut self, left: &Box<Expr>, op: &BinaryOp, right: &Box<Expr>) -> Result<EaType> {
        let left_type = self.check_expression(left)?;
        let right_type = self.check_expression(right)?;
        
        match op {
            BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Modulo => {
                if self.is_numeric_type(&left_type) && self.types_compatible(&left_type, &right_type) {
                    Ok(left_type)
                } else {
                    Err(CompileError::type_error(
                        format!("Arithmetic operation {:?} requires compatible numeric types, got {:?} and {:?}", 
                            op, left_type, right_type),
                        Position::new(0, 0, 0),
                    ))
                }
            },
            
            BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual => {
                if self.is_comparable_type(&left_type) && self.types_compatible(&left_type, &right_type) {
                    Ok(EaType::Bool)
                } else {
                    Err(CompileError::type_error(
                        format!("Comparison operation {:?} requires compatible comparable types, got {:?} and {:?}",
                            op, left_type, right_type),
                        Position::new(0, 0, 0),
                    ))
                }
            },
            
            BinaryOp::Equal | BinaryOp::NotEqual => {
                if self.types_compatible(&left_type, &right_type) {
                    Ok(EaType::Bool)
                } else {
                    Err(CompileError::type_error(
                        format!("Equality operation {:?} requires compatible types, got {:?} and {:?}",
                            op, left_type, right_type),
                        Position::new(0, 0, 0),
                    ))
                }
            },
            
            BinaryOp::And | BinaryOp::Or => {
                if matches!(left_type, EaType::Bool) && matches!(right_type, EaType::Bool) {
                    Ok(EaType::Bool)
                } else {
                    Err(CompileError::type_error(
                        format!("Logical operation {:?} requires boolean operands, got {:?} and {:?}",
                            op, left_type, right_type),
                        Position::new(0, 0, 0),
                    ))
                }
            },
            
            BinaryOp::Assign => {
                if self.types_compatible(&left_type, &right_type) {
                    Ok(left_type)
                } else {
                    Err(CompileError::type_error(
                        format!("Assignment type mismatch: cannot assign {:?} to {:?}",
                            right_type, left_type),
                        Position::new(0, 0, 0),
                    ))
                }
            },
            
            BinaryOp::PlusAssign | BinaryOp::MinusAssign | BinaryOp::MultiplyAssign | BinaryOp::DivideAssign => {
                if self.is_numeric_type(&left_type) && self.types_compatible(&left_type, &right_type) {
                    Ok(left_type)
                } else {
                    Err(CompileError::type_error(
                        format!("Compound assignment {:?} requires compatible numeric types, got {:?} and {:?}",
                            op, left_type, right_type),
                        Position::new(0, 0, 0),
                    ))
                }
            },
        }
    }
    
    fn check_unary_expression(&mut self, op: &UnaryOp, expr: &Box<Expr>) -> Result<EaType> {
        let expr_type = self.check_expression(expr)?;
        
        match op {
            UnaryOp::Negate => {
                if self.is_numeric_type(&expr_type) {
                    Ok(expr_type)
                } else {
                    Err(CompileError::type_error(
                        format!("Negation requires numeric type, got {:?}", expr_type),
                        Position::new(0, 0, 0),
                    ))
                }
            },
            UnaryOp::Not => {
                if matches!(expr_type, EaType::Bool) {
                    Ok(EaType::Bool)
                } else {
                    Err(CompileError::type_error(
                        format!("Logical not requires boolean type, got {:?}", expr_type),
                        Position::new(0, 0, 0),
                    ))
                }
            },
            UnaryOp::Reference => {
                Ok(EaType::Reference(Box::new(expr_type)))
            },
        }
    }
    
    fn check_function_call(&mut self, callee: &Box<Expr>, args: &[Expr]) -> Result<EaType> {
        if let Expr::Variable(func_name) = &**callee {
            // Clone the function type to avoid borrowing issues
            if let Some(func_type) = self.context.get_function_type(func_name).cloned() {
                if args.len() != func_type.params.len() {
                    return Err(CompileError::type_error(
                        format!(
                            "Function '{}' expects {} arguments, got {}",
                            func_name, func_type.params.len(), args.len()
                        ),
                        Position::new(0, 0, 0),
                    ));
                }
                
                for (i, (arg, expected_type)) in args.iter().zip(func_type.params.iter()).enumerate() {
                    let arg_type = self.check_expression(arg)?;
                    if !self.types_compatible(expected_type, &arg_type) {
                        return Err(CompileError::type_error(
                            format!(
                                "Argument {} of function '{}': expected {:?}, got {:?}",
                                i + 1, func_name, expected_type, arg_type
                            ),
                            Position::new(0, 0, 0),
                        ));
                    }
                }
                
                Ok(*func_type.return_type.clone())
            } else {
                Err(CompileError::type_error(
                    format!("Function '{}' not found", func_name),
                    Position::new(0, 0, 0),
                ))
            }
        } else {
            Err(CompileError::type_error(
                "Only direct function calls by name are supported".to_string(),
                Position::new(0, 0, 0),
            ))
        }
    }
    
    fn check_index_expression(&mut self, array: &Box<Expr>, index: &Box<Expr>) -> Result<EaType> {
        let array_type = self.check_expression(array)?;
        let index_type = self.check_expression(index)?;
        
        if !self.is_integer_type(&index_type) {
            return Err(CompileError::type_error(
                format!("Array index must be integer type, got {:?}", index_type),
                Position::new(0, 0, 0),
            ));
        }
        
        match array_type {
            EaType::Array(element_type) => Ok(*element_type),
            _ => Err(CompileError::type_error(
                format!("Cannot index non-array type {:?}", array_type),
                Position::new(0, 0, 0),
            )),
        }
    }
    
    fn check_field_access(&mut self, object: &Box<Expr>, _field: &str) -> Result<EaType> {
        let object_type = self.check_expression(object)?;
        
        Err(CompileError::type_error(
            format!("Field access not yet supported for type {:?}", object_type),
            Position::new(0, 0, 0),
        ))
    }
    
    fn annotation_to_type(&self, annotation: &TypeAnnotation) -> Result<EaType> {
        match annotation.name.as_str() {
            "i8" => Ok(EaType::I8),
            "i16" => Ok(EaType::I16),
            "i32" => Ok(EaType::I32),
            "i64" => Ok(EaType::I64),
            "u8" => Ok(EaType::U8),
            "u16" => Ok(EaType::U16),
            "u32" => Ok(EaType::U32),
            "u64" => Ok(EaType::U64),
            "f32" => Ok(EaType::F32),
            "f64" => Ok(EaType::F64),
            "bool" => Ok(EaType::Bool),
            "string" => Ok(EaType::String),
            "()" => Ok(EaType::Unit),
            // Common type aliases that might help with parsing issues
            "int" => Ok(EaType::I32),
            "float" => Ok(EaType::F64),
            _ => {
                // Instead of erroring, treat unknown types as Custom
                // This helps us work around parser issues temporarily
                Ok(EaType::Custom(annotation.name.clone()))
            }
        }
    }
    
    pub fn types_compatible(&self, expected: &EaType, actual: &EaType) -> bool {
        match (expected, actual) {
            (a, b) if a == b => true,
            
            // Handle custom types that are actually primitive types
            (EaType::I32, EaType::Custom(name)) if name == "i32" => true,
            (EaType::Custom(name), EaType::I32) if name == "i32" => true,
            (EaType::I64, EaType::Custom(name)) if name == "i64" => true,
            (EaType::Custom(name), EaType::I64) if name == "i64" => true,
            (EaType::Bool, EaType::Custom(name)) if name == "bool" => true,
            (EaType::Custom(name), EaType::Bool) if name == "bool" => true,
            (EaType::String, EaType::Custom(name)) if name == "string" => true,
            (EaType::Custom(name), EaType::String) if name == "string" => true,
            (EaType::F32, EaType::Custom(name)) if name == "f32" => true,
            (EaType::Custom(name), EaType::F32) if name == "f32" => true,
            (EaType::F64, EaType::Custom(name)) if name == "f64" => true,
            (EaType::Custom(name), EaType::F64) if name == "f64" => true,
            
            // Allow I64 literals to be used where smaller integer types are expected
            // This is common in programming languages - literal 5 can be used as i32
            (EaType::I8, EaType::I64) => true,
            (EaType::I16, EaType::I64) => true,
            (EaType::I32, EaType::I64) => true,
            (EaType::U8, EaType::I64) => true,  // Allow if the literal fits
            (EaType::U16, EaType::I64) => true,
            (EaType::U32, EaType::I64) => true,
            
            // Allow F64 literals to be used where F32 is expected
            (EaType::F32, EaType::F64) => true,
            
            // Standard integer promotions (smaller to larger)
            (EaType::I16, EaType::I8) => true,
            (EaType::I32, EaType::I8) => true,
            (EaType::I32, EaType::I16) => true,
            (EaType::I64, EaType::I8) => true,
            (EaType::I64, EaType::I16) => true,
            (EaType::I64, EaType::I32) => true,
            
            // Unsigned integer promotions
            (EaType::U16, EaType::U8) => true,
            (EaType::U32, EaType::U8) => true,
            (EaType::U32, EaType::U16) => true,
            (EaType::U64, EaType::U8) => true,
            (EaType::U64, EaType::U16) => true,
            (EaType::U64, EaType::U32) => true,
            
            // Float promotions
            (EaType::F64, EaType::F32) => true,
            
            // SIMD vector compatibility
            (EaType::SIMDVector { vector_type: v1, .. }, EaType::SIMDVector { vector_type: v2, .. }) => {
                v1.is_compatible_with(v2)
            }
            
            _ => false,
        }
    }
    
    fn is_numeric_type(&self, ty: &EaType) -> bool {
        matches!(ty, 
            EaType::I8 | EaType::I16 | EaType::I32 | EaType::I64 |
            EaType::U8 | EaType::U16 | EaType::U32 | EaType::U64 |
            EaType::F32 | EaType::F64
        ) || match ty {
            EaType::Custom(name) => matches!(name.as_str(), 
                "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | 
                "f32" | "f64" | "int"
            ),
            _ => false,
        }
    }
    
    fn is_integer_type(&self, ty: &EaType) -> bool {
        matches!(ty, 
            EaType::I8 | EaType::I16 | EaType::I32 | EaType::I64 |
            EaType::U8 | EaType::U16 | EaType::U32 | EaType::U64
        ) || match ty {
            EaType::Custom(name) => matches!(name.as_str(), 
                "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "int"
            ),
            _ => false,
        }
    }
    
    fn is_comparable_type(&self, ty: &EaType) -> bool {
        matches!(ty, 
            EaType::I8 | EaType::I16 | EaType::I32 | EaType::I64 |
            EaType::U8 | EaType::U16 | EaType::U32 | EaType::U64 |
            EaType::F32 | EaType::F64 | EaType::String
        ) || match ty {
            EaType::Custom(name) => matches!(name.as_str(), 
                "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | 
                "f32" | "f64" | "string" | "int"
            ),
            _ => false,
        }
    }
    
    /// Type checks SIMD expressions
    fn check_simd_expression(&mut self, simd_expr: &crate::ast::SIMDExpr) -> Result<EaType> {
        use crate::ast::SIMDExpr;
        
        match simd_expr {
            SIMDExpr::VectorLiteral { elements, vector_type, position } => {
                self.check_simd_vector_literal(elements, vector_type, position)
            }
            SIMDExpr::ElementWise { left, operator, right, position } => {
                self.check_simd_element_wise(left, operator, right, position)
            }
            SIMDExpr::Broadcast { value, target_type, position } => {
                self.check_simd_broadcast(value, target_type, position)
            }
            SIMDExpr::Swizzle { vector, pattern, position } => {
                self.check_simd_swizzle(vector, pattern, position)
            }
            SIMDExpr::Reduction { vector, operation, position } => {
                self.check_simd_reduction(vector, operation, position)
            }
            SIMDExpr::DotProduct { left, right, position } => {
                self.check_simd_dot_product(left, right, position)
            }
            SIMDExpr::VectorLoad { address, vector_type, alignment: _, position } => {
                self.check_simd_vector_load(address, vector_type, position)
            }
            SIMDExpr::VectorStore { address, vector, alignment: _, position } => {
                self.check_simd_vector_store(address, vector, position)
            }
        }
    }
    
    fn check_simd_vector_literal(
        &mut self,
        elements: &[crate::ast::Expr],
        vector_type: &Option<crate::ast::SIMDVectorType>,
        _position: &Position
    ) -> Result<EaType> {
        if let Some(vtype) = vector_type {
            // Check element count matches vector width
            if elements.len() != vtype.width() {
                return Err(CompileError::type_error(
                    format!(
                        "Vector literal element count mismatch: {} has {} elements, got {}",
                        vtype, vtype.width(), elements.len()
                    ),
                    Position::new(0, 0, 0),
                ));
            }
            
            // Check hardware support for this vector type
            if !self.hardware_detector.is_supported(vtype) {
                let required_features = self.hardware_detector.required_features(vtype);
                return Err(CompileError::type_error(
                    format!(
                        "SIMD vector type {} is not supported on target architecture {}. Required features: {:?}",
                        vtype, self.hardware_detector.target_arch(), required_features
                    ),
                    Position::new(0, 0, 0),
                ));
            }
            
            // Check all elements are compatible with vector element type
            let expected_element_type = self.simd_vector_type_to_element_type(vtype);
            for (i, element) in elements.iter().enumerate() {
                let element_type = self.check_expression(element)?;
                if !self.types_compatible(&expected_element_type, &element_type) {
                    return Err(CompileError::type_error(
                        format!(
                            "Vector literal element {} type mismatch: expected {}, got {}",
                            i, expected_element_type, element_type
                        ),
                        Position::new(0, 0, 0),
                    ));
                }
            }
            
            Ok(EaType::SIMDVector {
                element_type: Box::new(expected_element_type),
                width: vtype.width(),
                vector_type: vtype.clone(),
            })
        } else {
            Err(CompileError::type_error(
                "Vector literal must have explicit type annotation".to_string(),
                Position::new(0, 0, 0),
            ))
        }
    }
    
    fn check_simd_element_wise(
        &mut self,
        left: &Box<crate::ast::Expr>,
        operator: &crate::ast::SIMDOperator,
        right: &Box<crate::ast::Expr>,
        _position: &Position
    ) -> Result<EaType> {
        let left_type = self.check_expression(left)?;
        let right_type = self.check_expression(right)?;
        
        // Both operands must be SIMD vectors
        let (left_vector_type, right_vector_type) = match (&left_type, &right_type) {
            (EaType::SIMDVector { vector_type: lv, .. }, EaType::SIMDVector { vector_type: rv, .. }) => (lv, rv),
            _ => return Err(CompileError::type_error(
                format!("Element-wise operations require SIMD vector operands, got {} and {}", left_type, right_type),
                Position::new(0, 0, 0),
            )),
        };
        
        // Check operator is valid for these vector types
        if !operator.is_valid_for_types(left_vector_type, right_vector_type) {
            return Err(CompileError::type_error(
                format!("Operator {:?} is not valid for {} and {}", operator, left_vector_type, right_vector_type),
                Position::new(0, 0, 0),
            ));
        }
        
        // Check vectors are compatible
        if !left_vector_type.is_compatible_with(right_vector_type) {
            return Err(CompileError::type_error(
                format!("Incompatible SIMD vector types for element-wise operation: {} and {}", left_vector_type, right_vector_type),
                Position::new(0, 0, 0),
            ));
        }
        
        // Check hardware support for vector types
        if !self.hardware_detector.is_supported(left_vector_type) {
            return Err(CompileError::type_error(
                format!("SIMD vector type {} is not supported on target architecture {}", 
                    left_vector_type, self.hardware_detector.target_arch()),
                Position::new(0, 0, 0),
            ));
        }
        
        // Result has same type as operands
        Ok(left_type)
    }
    
    fn check_simd_broadcast(
        &mut self,
        value: &Box<crate::ast::Expr>,
        target_type: &crate::ast::SIMDVectorType,
        _position: &Position
    ) -> Result<EaType> {
        let value_type = self.check_expression(value)?;
        let expected_element_type = self.simd_vector_type_to_element_type(target_type);
        
        if !self.types_compatible(&expected_element_type, &value_type) {
            return Err(CompileError::type_error(
                format!(
                    "Broadcast value type mismatch: expected {}, got {}",
                    expected_element_type, value_type
                ),
                Position::new(0, 0, 0),
            ));
        }
        
        Ok(EaType::SIMDVector {
            element_type: Box::new(expected_element_type),
            width: target_type.width(),
            vector_type: target_type.clone(),
        })
    }
    
    fn check_simd_swizzle(
        &mut self,
        vector: &Box<crate::ast::Expr>,
        _pattern: &crate::ast::SwizzlePattern,
        _position: &Position
    ) -> Result<EaType> {
        let vector_type = self.check_expression(vector)?;
        
        match vector_type {
            EaType::SIMDVector { .. } => {
                // For now, return the same vector type
                // TODO: Implement proper swizzle result type inference
                Ok(vector_type)
            }
            _ => Err(CompileError::type_error(
                format!("Swizzle operation requires SIMD vector, got {}", vector_type),
                Position::new(0, 0, 0),
            )),
        }
    }
    
    fn check_simd_reduction(
        &mut self,
        vector: &Box<crate::ast::Expr>,
        _operation: &crate::ast::ReductionOp,
        _position: &Position
    ) -> Result<EaType> {
        let vector_type = self.check_expression(vector)?;
        
        match vector_type {
            EaType::SIMDVector { element_type, .. } => {
                // Reduction returns scalar of element type
                Ok(*element_type)
            }
            _ => Err(CompileError::type_error(
                format!("Reduction operation requires SIMD vector, got {}", vector_type),
                Position::new(0, 0, 0),
            )),
        }
    }
    
    fn check_simd_dot_product(
        &mut self,
        left: &Box<crate::ast::Expr>,
        right: &Box<crate::ast::Expr>,
        _position: &Position
    ) -> Result<EaType> {
        let left_type = self.check_expression(left)?;
        let right_type = self.check_expression(right)?;
        
        match (&left_type, &right_type) {
            (EaType::SIMDVector { element_type: left_elem, vector_type: left_vec, .. }, 
             EaType::SIMDVector { element_type: right_elem, vector_type: right_vec, .. }) => {
                // Check that vector types are compatible
                if left_vec != right_vec {
                    return Err(CompileError::type_error(
                        format!("Dot product requires vectors of same type, got {} and {}", left_vec, right_vec),
                        Position::new(0, 0, 0),
                    ));
                }
                
                // Check that element types are compatible
                if left_elem != right_elem {
                    return Err(CompileError::type_error(
                        format!("Dot product requires vectors with same element type, got {} and {}", left_elem, right_elem),
                        Position::new(0, 0, 0),
                    ));
                }
                
                // Dot product returns scalar of element type
                Ok((**left_elem).clone())
            }
            _ => Err(CompileError::type_error(
                format!("Dot product requires two SIMD vectors, got {} and {}", left_type, right_type),
                Position::new(0, 0, 0),
            )),
        }
    }
    
    fn check_simd_vector_load(
        &mut self,
        address: &Box<crate::ast::Expr>,
        vector_type: &crate::ast::SIMDVectorType,
        _position: &Position
    ) -> Result<EaType> {
        let address_type = self.check_expression(address)?;
        
        // Check that address is a reference/pointer type
        match address_type {
            EaType::Reference(_) => {
                // Return the vector type being loaded
                let element_type = self.simd_vector_type_to_element_type(vector_type);
                Ok(EaType::SIMDVector {
                    element_type: Box::new(element_type),
                    vector_type: vector_type.clone(),
                    width: vector_type.width(),
                })
            }
            _ => Err(CompileError::type_error(
                format!("Vector load requires pointer address, got {}", address_type),
                Position::new(0, 0, 0),
            )),
        }
    }
    
    fn check_simd_vector_store(
        &mut self,
        address: &Box<crate::ast::Expr>,
        vector: &Box<crate::ast::Expr>,
        _position: &Position
    ) -> Result<EaType> {
        let address_type = self.check_expression(address)?;
        let vector_type = self.check_expression(vector)?;
        
        // Check that address is a reference/pointer type
        match address_type {
            EaType::Reference(_) => {
                // Check that vector is a SIMD vector type
                match vector_type {
                    EaType::SIMDVector { .. } => {
                        // Vector store returns void (unit type)
                        Ok(EaType::Unit)
                    }
                    _ => Err(CompileError::type_error(
                        format!("Vector store requires SIMD vector value, got {}", vector_type),
                        Position::new(0, 0, 0),
                    )),
                }
            }
            _ => Err(CompileError::type_error(
                format!("Vector store requires pointer address, got {}", address_type),
                Position::new(0, 0, 0),
            )),
        }
    }
    
    /// Convert SIMD vector type to corresponding element type
    pub fn simd_vector_type_to_element_type(&self, vector_type: &crate::ast::SIMDVectorType) -> EaType {
        match vector_type.element_type() {
            "f32" => EaType::F32,
            "f64" => EaType::F64,
            "i32" => EaType::I32,
            "i64" => EaType::I64,
            "i16" => EaType::I16,
            "i8" => EaType::I8,
            "u32" => EaType::U32,
            "u16" => EaType::U16,
            "u8" => EaType::U8,
            "bool" => EaType::Bool,
            _ => EaType::Error,
        }
    }
    
    /// Validate SIMD operation type compatibility with detailed error reporting
    fn validate_simd_operation_compatibility(
        &self,
        left_type: &EaType,
        operator: &crate::ast::SIMDOperator,
        right_type: &EaType,
    ) -> Result<()> {
        let (left_vector_type, right_vector_type) = match (left_type, right_type) {
            (EaType::SIMDVector { vector_type: lv, .. }, EaType::SIMDVector { vector_type: rv, .. }) => (lv, rv),
            _ => return Err(CompileError::type_error(
                format!("SIMD operations require vector operands, got {} and {}", left_type, right_type),
                Position::new(0, 0, 0),
            )),
        };
        
        // Check element type compatibility for the specific operation
        match operator {
            crate::ast::SIMDOperator::DotAdd | crate::ast::SIMDOperator::DotSubtract |
            crate::ast::SIMDOperator::DotMultiply => {
                if !self.simd_types_arithmetic_compatible(left_vector_type, right_vector_type) {
                    return Err(CompileError::type_error(
                        format!("Arithmetic SIMD operation {:?} requires compatible numeric vector types, got {} and {}", 
                            operator, left_vector_type, right_vector_type),
                        Position::new(0, 0, 0),
                    ));
                }
            }
            crate::ast::SIMDOperator::DotDivide => {
                if !self.simd_types_division_compatible(left_vector_type, right_vector_type) {
                    return Err(CompileError::type_error(
                        format!("Division SIMD operation requires compatible numeric vector types (no integer division by mask), got {} and {}", 
                            left_vector_type, right_vector_type),
                        Position::new(0, 0, 0),
                    ));
                }
            }
            crate::ast::SIMDOperator::DotAnd | crate::ast::SIMDOperator::DotOr | crate::ast::SIMDOperator::DotXor => {
                if !self.simd_types_bitwise_compatible(left_vector_type, right_vector_type) {
                    return Err(CompileError::type_error(
                        format!("Bitwise SIMD operation {:?} requires integer or mask vector types, got {} and {}", 
                            operator, left_vector_type, right_vector_type),
                        Position::new(0, 0, 0),
                    ));
                }
            }
            crate::ast::SIMDOperator::DotEqual | crate::ast::SIMDOperator::DotNotEqual |
            crate::ast::SIMDOperator::DotLess | crate::ast::SIMDOperator::DotGreater |
            crate::ast::SIMDOperator::DotLessEqual | crate::ast::SIMDOperator::DotGreaterEqual => {
                // Comparison operations work on all types, produce mask vectors
                if left_vector_type.width() != right_vector_type.width() {
                    return Err(CompileError::type_error(
                        format!("Comparison SIMD operation {:?} requires same vector width, got {} and {}", 
                            operator, left_vector_type, right_vector_type),
                        Position::new(0, 0, 0),
                    ));
                }
            }
        }
        
        // Check vector width compatibility
        if left_vector_type.width() != right_vector_type.width() {
            return Err(CompileError::type_error(
                format!("SIMD vector width mismatch: {} has {} elements, {} has {} elements", 
                    left_vector_type, left_vector_type.width(),
                    right_vector_type, right_vector_type.width()),
                Position::new(0, 0, 0),
            ));
        }
        
        Ok(())
    }
    
    pub fn simd_types_arithmetic_compatible(&self, left: &crate::ast::SIMDVectorType, right: &crate::ast::SIMDVectorType) -> bool {
        // Arithmetic operations work on numeric types (not masks)
        let left_numeric = !matches!(left, 
            crate::ast::SIMDVectorType::Mask8 | crate::ast::SIMDVectorType::Mask16 |
            crate::ast::SIMDVectorType::Mask32 | crate::ast::SIMDVectorType::Mask64
        );
        let right_numeric = !matches!(right, 
            crate::ast::SIMDVectorType::Mask8 | crate::ast::SIMDVectorType::Mask16 |
            crate::ast::SIMDVectorType::Mask32 | crate::ast::SIMDVectorType::Mask64
        );
        
        left_numeric && right_numeric && left.is_compatible_with(right)
    }
    
    pub fn simd_types_division_compatible(&self, left: &crate::ast::SIMDVectorType, right: &crate::ast::SIMDVectorType) -> bool {
        // Division is more restrictive - typically only floating point
        let left_float = matches!(left.element_type(), "f32" | "f64");
        let right_float = matches!(right.element_type(), "f32" | "f64");
        
        // Some implementations allow integer division, but we'll be conservative
        left_float && right_float && left.is_compatible_with(right)
    }
    
    pub fn simd_types_bitwise_compatible(&self, left: &crate::ast::SIMDVectorType, right: &crate::ast::SIMDVectorType) -> bool {
        // Bitwise operations work on integers and masks, but not floats
        let left_bitwise = !matches!(left.element_type(), "f32" | "f64");
        let right_bitwise = !matches!(right.element_type(), "f32" | "f64");
        
        left_bitwise && right_bitwise && left.is_compatible_with(right)
    }
    
    /// Check if scalar-vector SIMD operations are valid (broadcasting)
    fn validate_scalar_vector_simd_operation(
        &self,
        scalar_type: &EaType,
        vector_type: &crate::ast::SIMDVectorType,
        operator: &crate::ast::SIMDOperator,
    ) -> Result<()> {
        // Check if scalar type is compatible with vector element type
        let expected_element_type = self.simd_vector_type_to_element_type(vector_type);
        
        if !self.types_compatible(&expected_element_type, scalar_type) {
            return Err(CompileError::type_error(
                format!("Scalar-vector SIMD operation: scalar type {} is not compatible with vector element type {}", 
                    scalar_type, expected_element_type),
                Position::new(0, 0, 0),
            ));
        }
        
        // Check if operation is valid for the types
        match operator {
            crate::ast::SIMDOperator::DotAnd | crate::ast::SIMDOperator::DotOr | crate::ast::SIMDOperator::DotXor => {
                if matches!(expected_element_type, EaType::F32 | EaType::F64) {
                    return Err(CompileError::type_error(
                        format!("Bitwise SIMD operation {:?} not valid for floating-point vector {}", 
                            operator, vector_type),
                        Position::new(0, 0, 0),
                    ));
                }
            }
            crate::ast::SIMDOperator::DotDivide => {
                if !matches!(expected_element_type, EaType::F32 | EaType::F64) {
                    return Err(CompileError::type_error(
                        format!("Division SIMD operation typically requires floating-point vector, got {}", 
                            vector_type),
                        Position::new(0, 0, 0),
                    ));
                }
            }
            _ => {} // Other operations are generally valid
        }
        
        Ok(())
    }
    
    /// Get the result type of a SIMD operation
    fn get_simd_operation_result_type(
        &self,
        left_type: &EaType,
        operator: &crate::ast::SIMDOperator,
        right_type: &EaType,
    ) -> Result<EaType> {
        match (left_type, right_type) {
            // Vector-Vector operations
            (EaType::SIMDVector { .. }, EaType::SIMDVector { .. }) => {
                self.validate_simd_operation_compatibility(left_type, operator, right_type)?;
                Ok(left_type.clone()) // Result has same type as operands
            }
            
            // Scalar-Vector operations (broadcast)
            (scalar, EaType::SIMDVector { vector_type, .. }) if !scalar.is_simd() => {
                self.validate_scalar_vector_simd_operation(scalar, vector_type, operator)?;
                Ok(right_type.clone()) // Result has vector type
            }
            
            // Vector-Scalar operations (broadcast)
            (EaType::SIMDVector { vector_type, .. }, scalar) if !scalar.is_simd() => {
                self.validate_scalar_vector_simd_operation(scalar, vector_type, operator)?;
                Ok(left_type.clone()) // Result has vector type
            }
            
            _ => Err(CompileError::type_error(
                format!("Invalid SIMD operation between {} and {}", left_type, right_type),
                Position::new(0, 0, 0),
            )),
        }
    }
}