// src/type_system/mod.rs
//! Type system for the Eä programming language.
//!
//! This module implements type checking, type inference, and type compatibility
//! checking for all Eä language constructs.

use crate::ast::{BinaryOp, Expr, Literal, Stmt, TypeAnnotation, UnaryOp};
use crate::error::{CompileError, Result};
use crate::lexer::Position;
pub mod types;
use std::collections::HashMap;
use std::fmt;

pub mod hardware;

/// Simple element types for SIMD vectors to avoid recursive type issues
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SIMDElementType {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
}

/// Represents all types in the Eä programming language.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EaType {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Bool,
    String,
    Unit,
    Array(Box<EaType>),
    Reference(Box<EaType>),
    Function(Box<FunctionType>),
    Struct(String), // Struct type with name
    Custom(String),

    // Enum type
    Enum {
        name: String,
        variants: Vec<String>, // For now, just variant names
    },

    Generic(String),
    Error,

    // SIMD types
    SIMDVector {
        element_type: SIMDElementType, // Use non-recursive element type
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
    pub structs: HashMap<String, HashMap<String, EaType>>, // struct_name -> {field_name -> field_type}
    pub types: HashMap<String, EaType>,                    // enum_name -> EaType::Enum
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
            EaType::Struct(name) => write!(f, "{}", name),
            EaType::Custom(name) => write!(f, "{}", name),
            EaType::Enum { name, .. } => write!(f, "{}", name),
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
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            EaType::I8
                | EaType::I16
                | EaType::I32
                | EaType::I64
                | EaType::U8
                | EaType::U16
                | EaType::U32
                | EaType::U64
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

    pub fn simd_element_type(&self) -> Option<EaType> {
        match self {
            EaType::SIMDVector { element_type, .. } => Some(element_type.to_ea_type()),
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

impl TypeContext {
    pub fn new() -> Self {
        let mut context = Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            structs: HashMap::new(),
            types: HashMap::new(),
            current_function_return: None,
        };

        // Add built-in print function
        context.functions.insert(
            "print".to_string(),
            FunctionType {
                params: vec![EaType::String],
                return_type: Box::new(EaType::Unit),
                is_variadic: false,
            },
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
        checker.add_builtin_types();
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
        self.context
            .define_function("print".to_string(), print_type);

        // print_i32(i32) -> void
        let print_i32_type = FunctionType {
            params: vec![EaType::I32],
            return_type: Box::new(EaType::Unit),
            is_variadic: false,
        };
        self.context
            .define_function("print_i32".to_string(), print_i32_type);

        // print_f32(f32) -> void
        let print_f32_type = FunctionType {
            params: vec![EaType::F32],
            return_type: Box::new(EaType::Unit),
            is_variadic: false,
        };
        self.context
            .define_function("print_f32".to_string(), print_f32_type);

        // println(string) -> void
        let println_type = FunctionType {
            params: vec![EaType::String],
            return_type: Box::new(EaType::Unit),
            is_variadic: false,
        };
        self.context
            .define_function("println".to_string(), println_type);

        // read_line() -> string
        let read_line_type = FunctionType {
            params: vec![],
            return_type: Box::new(EaType::String),
            is_variadic: false,
        };
        self.context
            .define_function("read_line".to_string(), read_line_type);

        // read_file(string) -> Result<string, FileError>
        let read_file_type = FunctionType {
            params: vec![EaType::String],
            return_type: Box::new(EaType::String), // Simplified - should be Result type
            is_variadic: false,
        };
        self.context
            .define_function("read_file".to_string(), read_file_type);

        // write_file(string, string) -> Result<(), FileError>
        let write_file_type = FunctionType {
            params: vec![EaType::String, EaType::String],
            return_type: Box::new(EaType::Unit), // Simplified - should be Result type
            is_variadic: false,
        };
        self.context
            .define_function("write_file".to_string(), write_file_type);

        // file_exists(string) -> bool
        let file_exists_type = FunctionType {
            params: vec![EaType::String],
            return_type: Box::new(EaType::Bool),
            is_variadic: false,
        };
        self.context
            .define_function("file_exists".to_string(), file_exists_type);

        // string_length(string) -> i32
        let string_length_type = FunctionType {
            params: vec![EaType::String],
            return_type: Box::new(EaType::I32),
            is_variadic: false,
        };
        self.context
            .define_function("string_length".to_string(), string_length_type);

        // string_concat(string, string) -> string
        let string_concat_type = FunctionType {
            params: vec![EaType::String, EaType::String],
            return_type: Box::new(EaType::String),
            is_variadic: false,
        };
        self.context
            .define_function("string_concat".to_string(), string_concat_type);

        // string_equals(string, string) -> bool
        let string_equals_type = FunctionType {
            params: vec![EaType::String, EaType::String],
            return_type: Box::new(EaType::Bool),
            is_variadic: false,
        };
        self.context
            .define_function("string_equals".to_string(), string_equals_type);

        // string_contains(string, string) -> bool
        let string_contains_type = FunctionType {
            params: vec![EaType::String, EaType::String],
            return_type: Box::new(EaType::Bool),
            is_variadic: false,
        };
        self.context
            .define_function("string_contains".to_string(), string_contains_type);

        // i32_to_string(i32) -> string
        let i32_to_string_type = FunctionType {
            params: vec![EaType::I32],
            return_type: Box::new(EaType::String),
            is_variadic: false,
        };
        self.context
            .define_function("i32_to_string".to_string(), i32_to_string_type);

        // f32_to_string(f32) -> string
        let f32_to_string_type = FunctionType {
            params: vec![EaType::F32],
            return_type: Box::new(EaType::String),
            is_variadic: false,
        };
        self.context
            .define_function("f32_to_string".to_string(), f32_to_string_type);

        // array_length<T>([]T) -> i32 (support both i32 and i64 arrays)
        let array_length_i32_type = FunctionType {
            params: vec![EaType::Array(Box::new(EaType::I32))],
            return_type: Box::new(EaType::I32),
            is_variadic: false,
        };
        self.context
            .define_function("array_length".to_string(), array_length_i32_type);

        let array_length_i64_type = FunctionType {
            params: vec![EaType::Array(Box::new(EaType::I64))],
            return_type: Box::new(EaType::I32),
            is_variadic: false,
        };
        self.context
            .define_function("array_length".to_string(), array_length_i64_type);

        // array_push<T>(&mut []T, T) -> ()
        let array_push_type = FunctionType {
            params: vec![
                EaType::Reference(Box::new(EaType::Array(Box::new(EaType::I32)))),
                EaType::I32,
            ], // Simplified
            return_type: Box::new(EaType::Unit),
            is_variadic: false,
        };
        self.context
            .define_function("array_push".to_string(), array_push_type);

        // array_pop<T>(&mut []T) -> Option<T>
        let array_pop_type = FunctionType {
            params: vec![EaType::Reference(Box::new(EaType::Array(Box::new(
                EaType::I32,
            ))))], // Simplified
            return_type: Box::new(EaType::I32), // Simplified - should be Option<T>
            is_variadic: false,
        };
        self.context
            .define_function("array_pop".to_string(), array_pop_type);

        // array_get<T>([]T, i32) -> Option<T>
        let array_get_type = FunctionType {
            params: vec![EaType::Array(Box::new(EaType::I32)), EaType::I32], // Simplified
            return_type: Box::new(EaType::I32), // Simplified - should be Option<T>
            is_variadic: false,
        };
        self.context
            .define_function("array_get".to_string(), array_get_type);

        // array_contains<T>([]T, T) -> bool
        let array_contains_type = FunctionType {
            params: vec![EaType::Array(Box::new(EaType::I32)), EaType::I32], // Simplified
            return_type: Box::new(EaType::Bool),
            is_variadic: false,
        };
        self.context
            .define_function("array_contains".to_string(), array_contains_type);

        // Math functions
        // sqrt(f32) -> f32
        let sqrt_type = FunctionType {
            params: vec![EaType::F32],
            return_type: Box::new(EaType::F32),
            is_variadic: false,
        };
        self.context.define_function("sqrt".to_string(), sqrt_type);

        // sin(f32) -> f32
        let sin_type = FunctionType {
            params: vec![EaType::F32],
            return_type: Box::new(EaType::F32),
            is_variadic: false,
        };
        self.context.define_function("sin".to_string(), sin_type);

        // cos(f32) -> f32
        let cos_type = FunctionType {
            params: vec![EaType::F32],
            return_type: Box::new(EaType::F32),
            is_variadic: false,
        };
        self.context.define_function("cos".to_string(), cos_type);

        // abs(f32) -> f32
        let abs_type = FunctionType {
            params: vec![EaType::F32],
            return_type: Box::new(EaType::F32),
            is_variadic: false,
        };
        self.context.define_function("abs".to_string(), abs_type);

        // min(f32, f32) -> f32
        let min_type = FunctionType {
            params: vec![EaType::F32, EaType::F32],
            return_type: Box::new(EaType::F32),
            is_variadic: false,
        };
        self.context.define_function("min".to_string(), min_type);

        // max(f32, f32) -> f32
        let max_type = FunctionType {
            params: vec![EaType::F32, EaType::F32],
            return_type: Box::new(EaType::F32),
            is_variadic: false,
        };
        self.context.define_function("max".to_string(), max_type);

        // pow(f32, f32) -> f32
        let pow_type = FunctionType {
            params: vec![EaType::F32, EaType::F32],
            return_type: Box::new(EaType::F32),
            is_variadic: false,
        };
        self.context.define_function("pow".to_string(), pow_type);

        // printf(string, ...) -> i32 (external)
        let printf_type = FunctionType {
            params: vec![EaType::String],
            return_type: Box::new(EaType::I32),
            is_variadic: true,
        };
        self.context
            .define_function("printf".to_string(), printf_type);

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

    /// Adds built-in enum types like Result<T,E> and Option<T>
    fn add_builtin_types(&mut self) {
        // Result<T, E> enum with Ok(T) and Err(E) variants
        let result_type = EaType::Enum {
            name: "Result".to_string(),
            variants: vec!["Ok".to_string(), "Err".to_string()],
        };
        self.context.types.insert("Result".to_string(), result_type);

        // Option<T> enum with Some(T) and None variants
        let option_type = EaType::Enum {
            name: "Option".to_string(),
            variants: vec!["Some".to_string(), "None".to_string()],
        };
        self.context.types.insert("Option".to_string(), option_type);
    }

    /// Gets a reference to the hardware detector.
    pub fn hardware_detector(&self) -> &hardware::HardwareDetector {
        &self.hardware_detector
    }

    /// Get optimization recommendations for a SIMD vector type.
    pub fn get_simd_optimization_hints(
        &self,
        vector_type: &crate::ast::SIMDVectorType,
    ) -> Vec<String> {
        self.hardware_detector
            .optimization_recommendations(vector_type)
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
            Stmt::FunctionDeclaration {
                name,
                params,
                return_type,
                body,
                attributes: _,
            } => self.check_function_declaration(name, params, return_type, body),
            Stmt::VarDeclaration {
                name,
                type_annotation,
                initializer,
            } => self.check_var_declaration(name, type_annotation, initializer),
            Stmt::Expression(expr) => {
                self.check_expression(expr)?;
                Ok(())
            }
            Stmt::Return(expr) => self.check_return_statement(expr),
            Stmt::Block(stmts) => self.check_block(stmts),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => self.check_if_statement(condition, then_branch, else_branch),
            Stmt::While { condition, body } => self.check_while_statement(condition, body),
            Stmt::For {
                initializer,
                condition,
                increment,
                body,
            } => self.check_for_statement(initializer, condition, increment, body),
            Stmt::ForIn {
                variable,
                iterable,
                body,
            } => self.check_for_in_statement(variable, iterable, body),
            Stmt::StructDeclaration { name, fields } => self.check_struct_declaration(name, fields),
            Stmt::EnumDeclaration { name, variants } => self.check_enum_declaration(name, variants),
        }
    }

    fn check_function_declaration(
        &mut self,
        name: &str,
        params: &[crate::ast::Parameter],
        return_type: &Option<TypeAnnotation>,
        body: &Box<Stmt>,
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
        initializer: &Option<Expr>,
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
            }
            (Some(type_ann), None) => self.annotation_to_type(type_ann)?,
            (None, Some(init)) => self.check_expression(init)?,
            (None, None) => {
                return Err(CompileError::type_error(
                    format!(
                        "Variable '{}' must have either a type annotation or an initializer",
                        name
                    ),
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
        else_branch: &Option<Box<Stmt>>,
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
        body: &Box<Stmt>,
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
                    }
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

    fn check_for_in_statement(
        &mut self,
        variable: &str,
        iterable: &Expr,
        body: &Box<Stmt>,
    ) -> Result<()> {
        // Check the iterable expression type
        let iterable_type = self.check_expression(iterable)?;

        // Ensure the iterable is an array type
        let element_type = match iterable_type {
            EaType::Array(element_type) => *element_type,
            _ => {
                return Err(CompileError::type_error(
                    format!("For-in loop requires array type, got {:?}", iterable_type),
                    Position::new(0, 0, 0),
                ));
            }
        };

        // Create a new scope for the loop body
        let loop_context = self.context.enter_scope();
        let old_context = std::mem::replace(&mut self.context, loop_context);

        // Add the loop variable to the scope with the element type
        self.context
            .define_variable(variable.to_string(), element_type);

        // Check the body
        let result = self.check_statement(body);

        // Restore the previous context
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
            Expr::Slice { array, start, end } => self.check_slice_expression(array, start, end),
            Expr::FieldAccess(object, field) => self.check_field_access(object, field),
            Expr::StructLiteral { name, fields } => self.check_struct_literal(name, fields),
            Expr::EnumLiteral {
                enum_name,
                variant,
                args,
            } => self.check_enum_literal(enum_name, variant, args),
            Expr::Match { value, arms } => self.check_match_expression(value, arms),
            Expr::Block(statements) => self.check_block_expression(statements),
            Expr::SIMD(simd_expr) => self.check_simd_expression(simd_expr),
        }
    }

    fn literal_type(&self, literal: &Literal) -> EaType {
        match literal {
            Literal::Integer(_) => EaType::I64,
            Literal::Float(_) => EaType::F64,
            Literal::String(_) => EaType::String,
            Literal::Boolean(_) => EaType::Bool,
            Literal::Vector {
                elements,
                vector_type,
            } => {
                if let Some(_vtype) = vector_type {
                    // Temporarily treat SIMD vectors as regular arrays to avoid compilation issues
                    if let Some(first_element) = elements.first() {
                        let element_type = self.literal_type(first_element);
                        EaType::Array(Box::new(element_type))
                    } else {
                        EaType::Error
                    }
                } else {
                    // Regular array literal without SIMD type - create Array type
                    if let Some(first_element) = elements.first() {
                        let element_type = self.literal_type(first_element);
                        EaType::Array(Box::new(element_type))
                    } else {
                        EaType::Error
                    }
                }
            }
        }
    }

    fn check_variable(&self, name: &str) -> Result<EaType> {
        self.context
            .get_variable_type(name)
            .cloned()
            .ok_or_else(|| {
                CompileError::type_error(
                    format!("Variable '{}' not found", name),
                    Position::new(0, 0, 0),
                )
            })
    }

    fn check_binary_expression(
        &mut self,
        left: &Box<Expr>,
        op: &BinaryOp,
        right: &Box<Expr>,
    ) -> Result<EaType> {
        let left_type = self.check_expression(left)?;
        let right_type = self.check_expression(right)?;

        match op {
            BinaryOp::Add
            | BinaryOp::Subtract
            | BinaryOp::Multiply
            | BinaryOp::Divide
            | BinaryOp::Modulo => {
                if self.is_numeric_type(&left_type)
                    && self.types_compatible(&left_type, &right_type)
                {
                    Ok(left_type)
                } else {
                    Err(CompileError::type_error(
                        format!("Arithmetic operation {:?} requires compatible numeric types, got {:?} and {:?}", 
                            op, left_type, right_type),
                        Position::new(0, 0, 0),
                    ))
                }
            }

            BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual => {
                if self.is_comparable_type(&left_type)
                    && self.types_compatible(&left_type, &right_type)
                {
                    Ok(EaType::Bool)
                } else {
                    Err(CompileError::type_error(
                        format!("Comparison operation {:?} requires compatible comparable types, got {:?} and {:?}",
                            op, left_type, right_type),
                        Position::new(0, 0, 0),
                    ))
                }
            }

            BinaryOp::Equal | BinaryOp::NotEqual => {
                if self.types_compatible(&left_type, &right_type) {
                    Ok(EaType::Bool)
                } else {
                    Err(CompileError::type_error(
                        format!(
                            "Equality operation {:?} requires compatible types, got {:?} and {:?}",
                            op, left_type, right_type
                        ),
                        Position::new(0, 0, 0),
                    ))
                }
            }

            BinaryOp::And | BinaryOp::Or => {
                if matches!(left_type, EaType::Bool) && matches!(right_type, EaType::Bool) {
                    Ok(EaType::Bool)
                } else {
                    Err(CompileError::type_error(
                        format!(
                            "Logical operation {:?} requires boolean operands, got {:?} and {:?}",
                            op, left_type, right_type
                        ),
                        Position::new(0, 0, 0),
                    ))
                }
            }

            BinaryOp::Assign => {
                if self.types_compatible(&left_type, &right_type) {
                    Ok(left_type)
                } else {
                    Err(CompileError::type_error(
                        format!(
                            "Assignment type mismatch: cannot assign {:?} to {:?}",
                            right_type, left_type
                        ),
                        Position::new(0, 0, 0),
                    ))
                }
            }

            BinaryOp::PlusAssign
            | BinaryOp::MinusAssign
            | BinaryOp::MultiplyAssign
            | BinaryOp::DivideAssign => {
                if self.is_numeric_type(&left_type)
                    && self.types_compatible(&left_type, &right_type)
                {
                    Ok(left_type)
                } else {
                    Err(CompileError::type_error(
                        format!("Compound assignment {:?} requires compatible numeric types, got {:?} and {:?}",
                            op, left_type, right_type),
                        Position::new(0, 0, 0),
                    ))
                }
            }
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
            }
            UnaryOp::Not => {
                if matches!(expr_type, EaType::Bool) {
                    Ok(EaType::Bool)
                } else {
                    Err(CompileError::type_error(
                        format!("Logical not requires boolean type, got {:?}", expr_type),
                        Position::new(0, 0, 0),
                    ))
                }
            }
            UnaryOp::Reference => Ok(EaType::Reference(Box::new(expr_type))),
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
                            func_name,
                            func_type.params.len(),
                            args.len()
                        ),
                        Position::new(0, 0, 0),
                    ));
                }

                for (i, (arg, expected_type)) in
                    args.iter().zip(func_type.params.iter()).enumerate()
                {
                    let arg_type = self.check_expression(arg)?;
                    if !self.types_compatible(expected_type, &arg_type) {
                        return Err(CompileError::type_error(
                            format!(
                                "Argument {} of function '{}': expected {:?}, got {:?}",
                                i + 1,
                                func_name,
                                expected_type,
                                arg_type
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

    fn check_slice_expression(
        &mut self,
        array: &Box<Expr>,
        start: &Box<Expr>,
        end: &Box<Expr>,
    ) -> Result<EaType> {
        let array_type = self.check_expression(array)?;
        let start_type = self.check_expression(start)?;
        let end_type = self.check_expression(end)?;

        if !self.is_integer_type(&start_type) {
            return Err(CompileError::type_error(
                format!(
                    "Array slice start must be integer type, got {:?}",
                    start_type
                ),
                Position::new(0, 0, 0),
            ));
        }

        if !self.is_integer_type(&end_type) {
            return Err(CompileError::type_error(
                format!("Array slice end must be integer type, got {:?}", end_type),
                Position::new(0, 0, 0),
            ));
        }

        match array_type {
            EaType::Array(element_type) => Ok(EaType::Array(element_type)), // Slice returns same array type
            _ => Err(CompileError::type_error(
                format!("Cannot slice non-array type {:?}", array_type),
                Position::new(0, 0, 0),
            )),
        }
    }

    fn check_field_access(&mut self, object: &Box<Expr>, field: &str) -> Result<EaType> {
        let object_type = self.check_expression(object)?;

        match &object_type {
            EaType::Struct(struct_name) => {
                if let Some(struct_fields) = self.context.structs.get(struct_name) {
                    if let Some(field_type) = struct_fields.get(field) {
                        Ok(field_type.clone())
                    } else {
                        Err(CompileError::type_error(
                            format!("Struct '{}' has no field '{}'", struct_name, field),
                            Position::new(0, 0, 0),
                        ))
                    }
                } else {
                    Err(CompileError::type_error(
                        format!("Unknown struct type '{}'", struct_name),
                        Position::new(0, 0, 0),
                    ))
                }
            }
            _ => Err(CompileError::type_error(
                format!("Field access not supported for type {}", object_type),
                Position::new(0, 0, 0),
            )),
        }
    }

    fn check_struct_declaration(
        &mut self,
        name: &str,
        fields: &[crate::ast::StructField],
    ) -> Result<()> {
        let mut field_types = HashMap::new();

        for field in fields {
            let field_type = self.annotation_to_type(&field.type_annotation)?;
            field_types.insert(field.name.clone(), field_type);
        }

        self.context.structs.insert(name.to_string(), field_types);
        Ok(())
    }

    fn check_enum_declaration(
        &mut self,
        name: &str,
        variants: &[crate::ast::EnumVariant],
    ) -> Result<()> {
        let mut variant_names = Vec::new();

        for variant in variants {
            variant_names.push(variant.name.clone());

            // TODO: For now, just validate that variant names don't conflict
            // In future, we'll need to handle variant data types as well
            if let Some(_data) = &variant.data {
                // Validate that the data types are valid
                for type_annotation in _data {
                    self.annotation_to_type(type_annotation)?;
                }
            }
        }

        // Store enum type in context (for now using a simple representation)
        let enum_type = EaType::Enum {
            name: name.to_string(),
            variants: variant_names,
        };

        // Store enum in a dedicated enum map (we'll need to add this to TypeContext)
        // For now, we'll use the custom types map
        self.context.types.insert(name.to_string(), enum_type);
        Ok(())
    }

    fn check_struct_literal(
        &mut self,
        name: &str,
        fields: &[crate::ast::StructFieldInit],
    ) -> Result<EaType> {
        // Check if struct is defined
        let struct_fields = match self.context.structs.get(name) {
            Some(fields) => fields.clone(),
            None => {
                return Err(CompileError::type_error(
                    format!("Undefined struct '{}'", name),
                    Position::new(0, 0, 0),
                ))
            }
        };

        // Check that all required fields are provided
        let mut provided_fields = HashMap::new();
        for field_init in fields {
            let field_type = self.check_expression(&field_init.value)?;
            provided_fields.insert(&field_init.name, field_type);
        }

        // Verify all struct fields are provided with correct types
        for (field_name, expected_type) in &struct_fields {
            match provided_fields.get(field_name) {
                Some(provided_type) => {
                    if !self.types_compatible(expected_type, provided_type) {
                        return Err(CompileError::type_error(
                            format!(
                                "Field '{}' expects type {}, got {}",
                                field_name, expected_type, provided_type
                            ),
                            Position::new(0, 0, 0),
                        ));
                    }
                }
                None => {
                    return Err(CompileError::type_error(
                        format!("Missing field '{}' in struct literal", field_name),
                        Position::new(0, 0, 0),
                    ))
                }
            }
        }

        // Check for extra fields
        for field_init in fields {
            if !struct_fields.contains_key(&field_init.name) {
                return Err(CompileError::type_error(
                    format!("Unknown field '{}' in struct '{}'", field_init.name, name),
                    Position::new(0, 0, 0),
                ));
            }
        }

        Ok(EaType::Struct(name.to_string()))
    }

    fn check_match_expression(
        &mut self,
        value: &Expr,
        arms: &[crate::ast::MatchArm],
    ) -> Result<EaType> {
        // Type check the value being matched
        let value_type = self.check_expression(value)?;

        if arms.is_empty() {
            return Err(CompileError::type_error(
                "Match expression must have at least one arm".to_string(),
                Position::new(0, 0, 0),
            ));
        }

        // All arms must return the same type
        let first_arm_type = self.check_match_arm(&arms[0], &value_type)?;

        for arm in &arms[1..] {
            let arm_type = self.check_match_arm(arm, &value_type)?;
            if arm_type != first_arm_type {
                return Err(CompileError::type_error(
                    format!(
                        "All match arms must return the same type. Expected {}, found {}",
                        first_arm_type, arm_type
                    ),
                    Position::new(0, 0, 0),
                ));
            }
        }

        // TODO: Check for exhaustiveness (all possible patterns covered)

        Ok(first_arm_type)
    }

    fn check_match_arm(
        &mut self,
        arm: &crate::ast::MatchArm,
        value_type: &EaType,
    ) -> Result<EaType> {
        // Type check the pattern against the value type
        self.check_pattern(&arm.pattern, value_type)?;

        // Type check the expression with any variables bound by the pattern
        self.check_expression(&arm.expression)
    }

    fn check_pattern(
        &mut self,
        pattern: &crate::ast::Pattern,
        expected_type: &EaType,
    ) -> Result<()> {
        use crate::ast::Pattern;

        match pattern {
            Pattern::Literal(literal) => {
                let literal_type = self.literal_type(literal);
                if !self.types_compatible(&literal_type, expected_type) {
                    return Err(CompileError::type_error(
                        format!(
                            "Pattern type {} does not match value type {}",
                            literal_type, expected_type
                        ),
                        Position::new(0, 0, 0),
                    ));
                }
                Ok(())
            }
            Pattern::Variable(name) => {
                // Bind the variable to the expected type
                self.context
                    .variables
                    .insert(name.clone(), expected_type.clone());
                Ok(())
            }
            Pattern::EnumVariant {
                enum_name,
                variant,
                patterns,
            } => {
                // Check that the expected type is the correct enum
                match expected_type {
                    EaType::Enum { name, variants } => {
                        if name != enum_name {
                            return Err(CompileError::type_error(
                                format!(
                                    "Pattern enum {} does not match value enum {}",
                                    enum_name, name
                                ),
                                Position::new(0, 0, 0),
                            ));
                        }

                        if !variants.contains(variant) {
                            return Err(CompileError::type_error(
                                format!("Variant {} not found in enum {}", variant, enum_name),
                                Position::new(0, 0, 0),
                            ));
                        }

                        // TODO: Type check variant data patterns when enum variants have data
                        if !patterns.is_empty() {
                            // For now, just accept any sub-patterns
                            // In the future, we'd need to look up the variant's data types
                        }

                        Ok(())
                    }
                    _ => Err(CompileError::type_error(
                        format!(
                            "Cannot match enum pattern against non-enum type {}",
                            expected_type
                        ),
                        Position::new(0, 0, 0),
                    )),
                }
            }
            Pattern::Wildcard => {
                // Wildcard patterns always match
                Ok(())
            }
        }
    }

    fn check_enum_literal(
        &mut self,
        enum_name: &str,
        variant: &str,
        args: &[Expr],
    ) -> Result<EaType> {
        // First, validate arguments without holding any borrows
        for arg in args {
            self.check_expression(arg)?; // Validate args are well-typed
        }

        // Check if enum is defined
        let enum_type = match self.context.types.get(enum_name) {
            Some(EaType::Enum { name, variants }) => {
                // Check if variant exists
                if !variants.contains(&variant.to_string()) {
                    return Err(CompileError::type_error(
                        format!("Unknown variant '{}' in enum '{}'", variant, enum_name),
                        Position::new(0, 0, 0),
                    ));
                }

                EaType::Enum {
                    name: name.clone(),
                    variants: variants.clone(),
                }
            }
            _ => {
                return Err(CompileError::type_error(
                    format!("Undefined enum '{}'", enum_name),
                    Position::new(0, 0, 0),
                ))
            }
        };

        Ok(enum_type)
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
                // Check if it's a defined struct
                if self.context.structs.contains_key(&annotation.name) {
                    Ok(EaType::Struct(annotation.name.clone()))
                } else {
                    // Treat unknown types as Custom for now
                    Ok(EaType::Custom(annotation.name.clone()))
                }
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
            (EaType::U8, EaType::I64) => true, // Allow if the literal fits
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
            (
                EaType::SIMDVector {
                    vector_type: v1, ..
                },
                EaType::SIMDVector {
                    vector_type: v2, ..
                },
            ) => v1.is_compatible_with(v2),

            _ => false,
        }
    }

    fn is_numeric_type(&self, ty: &EaType) -> bool {
        matches!(
            ty,
            EaType::I8
                | EaType::I16
                | EaType::I32
                | EaType::I64
                | EaType::U8
                | EaType::U16
                | EaType::U32
                | EaType::U64
                | EaType::F32
                | EaType::F64
        ) || match ty {
            EaType::Custom(name) => matches!(
                name.as_str(),
                "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "f32" | "f64" | "int"
            ),
            _ => false,
        }
    }

    fn is_integer_type(&self, ty: &EaType) -> bool {
        matches!(
            ty,
            EaType::I8
                | EaType::I16
                | EaType::I32
                | EaType::I64
                | EaType::U8
                | EaType::U16
                | EaType::U32
                | EaType::U64
        ) || match ty {
            EaType::Custom(name) => matches!(
                name.as_str(),
                "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "int"
            ),
            _ => false,
        }
    }

    fn is_comparable_type(&self, ty: &EaType) -> bool {
        matches!(
            ty,
            EaType::I8
                | EaType::I16
                | EaType::I32
                | EaType::I64
                | EaType::U8
                | EaType::U16
                | EaType::U32
                | EaType::U64
                | EaType::F32
                | EaType::F64
                | EaType::String
        ) || match ty {
            EaType::Custom(name) => matches!(
                name.as_str(),
                "i8" | "i16"
                    | "i32"
                    | "i64"
                    | "u8"
                    | "u16"
                    | "u32"
                    | "u64"
                    | "f32"
                    | "f64"
                    | "string"
                    | "int"
            ),
            _ => false,
        }
    }

    /// Type checks SIMD expressions
    fn check_block_expression(&mut self, statements: &Vec<Stmt>) -> Result<EaType> {
        // Check all statements in the block
        for stmt in statements {
            self.check_statement(stmt)?;
        }

        // A block expression returns the type of its last expression statement,
        // or Unit if there are no expression statements or if the last statement is not an expression
        if let Some(last_stmt) = statements.last() {
            if let Stmt::Expression(expr) = last_stmt {
                return self.check_expression(expr);
            }
        }

        Ok(EaType::Unit)
    }

    fn check_simd_expression(&mut self, simd_expr: &crate::ast::SIMDExpr) -> Result<EaType> {
        use crate::ast::SIMDExpr;

        match simd_expr {
            SIMDExpr::VectorLiteral {
                elements,
                vector_type,
                position,
            } => self.check_simd_vector_literal(elements, vector_type, position),
            SIMDExpr::ElementWise {
                left,
                operator,
                right,
                position,
            } => self.check_simd_element_wise(left, operator, right, position),
            SIMDExpr::Broadcast {
                value,
                target_type,
                position,
            } => self.check_simd_broadcast(value, target_type, position),
            SIMDExpr::Swizzle {
                vector,
                pattern,
                position,
            } => self.check_simd_swizzle(vector, pattern, position),
            SIMDExpr::Reduction {
                vector,
                operation,
                position,
            } => self.check_simd_reduction(vector, operation, position),
            SIMDExpr::DotProduct {
                left,
                right,
                position,
            } => self.check_simd_dot_product(left, right, position),
            SIMDExpr::VectorLoad {
                address,
                vector_type,
                alignment: _,
                position,
            } => self.check_simd_vector_load(address, vector_type, position),
            SIMDExpr::VectorStore {
                address,
                vector,
                alignment: _,
                position,
            } => self.check_simd_vector_store(address, vector, position),
        }
    }

    fn check_simd_vector_literal(
        &mut self,
        elements: &[crate::ast::Expr],
        vector_type: &Option<crate::ast::SIMDVectorType>,
        _position: &Position,
    ) -> Result<EaType> {
        if let Some(vtype) = vector_type {
            // Check element count matches vector width
            if elements.len() != vtype.width() {
                return Err(CompileError::type_error(
                    format!(
                        "Vector literal element count mismatch: {} has {} elements, got {}",
                        vtype,
                        vtype.width(),
                        elements.len()
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

            // Convert expected_element_type to SIMDElementType to avoid recursion
            if let Some(simd_element_type) = SIMDElementType::from_ea_type(&expected_element_type) {
                Ok(EaType::SIMDVector {
                    element_type: simd_element_type,
                    width: vtype.width(),
                    vector_type: vtype.clone(),
                })
            } else {
                Err(CompileError::type_error(
                    format!(
                        "Invalid element type for SIMD vector: {}",
                        expected_element_type
                    ),
                    Position::new(0, 0, 0),
                ))
            }
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
        _position: &Position,
    ) -> Result<EaType> {
        let left_type = self.check_expression(left)?;
        let right_type = self.check_expression(right)?;

        // Both operands must be SIMD vectors
        let (left_vector_type, right_vector_type) = match (&left_type, &right_type) {
            (
                EaType::SIMDVector {
                    vector_type: lv, ..
                },
                EaType::SIMDVector {
                    vector_type: rv, ..
                },
            ) => (lv, rv),
            _ => {
                return Err(CompileError::type_error(
                    format!(
                        "Element-wise operations require SIMD vector operands, got {} and {}",
                        left_type, right_type
                    ),
                    Position::new(0, 0, 0),
                ))
            }
        };

        // Check operator is valid for these vector types
        if !operator.is_valid_for_types(left_vector_type, right_vector_type) {
            return Err(CompileError::type_error(
                format!(
                    "Operator {:?} is not valid for {} and {}",
                    operator, left_vector_type, right_vector_type
                ),
                Position::new(0, 0, 0),
            ));
        }

        // Check vectors are compatible
        if !left_vector_type.is_compatible_with(right_vector_type) {
            return Err(CompileError::type_error(
                format!(
                    "Incompatible SIMD vector types for element-wise operation: {} and {}",
                    left_vector_type, right_vector_type
                ),
                Position::new(0, 0, 0),
            ));
        }

        // Check hardware support for vector types
        if !self.hardware_detector.is_supported(left_vector_type) {
            return Err(CompileError::type_error(
                format!(
                    "SIMD vector type {} is not supported on target architecture {}",
                    left_vector_type,
                    self.hardware_detector.target_arch()
                ),
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
        _position: &Position,
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

        // Convert expected_element_type to SIMDElementType to avoid recursion
        if let Some(simd_element_type) = SIMDElementType::from_ea_type(&expected_element_type) {
            Ok(EaType::SIMDVector {
                element_type: simd_element_type,
                width: target_type.width(),
                vector_type: target_type.clone(),
            })
        } else {
            Err(CompileError::type_error(
                format!(
                    "Invalid element type for SIMD vector: {}",
                    expected_element_type
                ),
                Position::new(0, 0, 0),
            ))
        }
    }

    fn check_simd_swizzle(
        &mut self,
        vector: &Box<crate::ast::Expr>,
        _pattern: &crate::ast::SwizzlePattern,
        _position: &Position,
    ) -> Result<EaType> {
        let vector_type = self.check_expression(vector)?;

        match vector_type {
            EaType::SIMDVector { .. } => {
                // For now, return the same vector type
                // TODO: Implement proper swizzle result type inference
                Ok(vector_type)
            }
            _ => Err(CompileError::type_error(
                format!(
                    "Swizzle operation requires SIMD vector, got {}",
                    vector_type
                ),
                Position::new(0, 0, 0),
            )),
        }
    }

    fn check_simd_reduction(
        &mut self,
        vector: &Box<crate::ast::Expr>,
        _operation: &crate::ast::ReductionOp,
        _position: &Position,
    ) -> Result<EaType> {
        let vector_type = self.check_expression(vector)?;

        match vector_type {
            EaType::SIMDVector { element_type, .. } => {
                // Reduction returns scalar of element type
                Ok(element_type.to_ea_type())
            }
            _ => Err(CompileError::type_error(
                format!(
                    "Reduction operation requires SIMD vector, got {}",
                    vector_type
                ),
                Position::new(0, 0, 0),
            )),
        }
    }

    fn check_simd_dot_product(
        &mut self,
        left: &Box<crate::ast::Expr>,
        right: &Box<crate::ast::Expr>,
        _position: &Position,
    ) -> Result<EaType> {
        let left_type = self.check_expression(left)?;
        let right_type = self.check_expression(right)?;

        match (&left_type, &right_type) {
            (
                EaType::SIMDVector {
                    element_type: left_elem,
                    vector_type: left_vec,
                    ..
                },
                EaType::SIMDVector {
                    element_type: right_elem,
                    vector_type: right_vec,
                    ..
                },
            ) => {
                // Check that vector types are compatible
                if left_vec != right_vec {
                    return Err(CompileError::type_error(
                        format!(
                            "Dot product requires vectors of same type, got {} and {}",
                            left_vec, right_vec
                        ),
                        Position::new(0, 0, 0),
                    ));
                }

                // Check that element types are compatible
                if left_elem != right_elem {
                    return Err(CompileError::type_error(
                        format!(
                            "Dot product requires vectors with same element type, got {} and {}",
                            left_elem, right_elem
                        ),
                        Position::new(0, 0, 0),
                    ));
                }

                // Dot product returns scalar of element type
                Ok(left_elem.to_ea_type())
            }
            _ => Err(CompileError::type_error(
                format!(
                    "Dot product requires two SIMD vectors, got {} and {}",
                    left_type, right_type
                ),
                Position::new(0, 0, 0),
            )),
        }
    }

    fn check_simd_vector_load(
        &mut self,
        address: &Box<crate::ast::Expr>,
        vector_type: &crate::ast::SIMDVectorType,
        _position: &Position,
    ) -> Result<EaType> {
        let address_type = self.check_expression(address)?;

        // Check that address is a reference/pointer type
        match address_type {
            EaType::Reference(_) => {
                // Return the vector type being loaded
                let element_type = self.simd_vector_type_to_element_type(vector_type);
                // Convert element_type to SIMDElementType to avoid recursion
                if let Some(simd_element_type) = SIMDElementType::from_ea_type(&element_type) {
                    Ok(EaType::SIMDVector {
                        element_type: simd_element_type,
                        vector_type: vector_type.clone(),
                        width: vector_type.width(),
                    })
                } else {
                    Err(CompileError::type_error(
                        format!("Invalid element type for SIMD vector: {}", element_type),
                        Position::new(0, 0, 0),
                    ))
                }
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
        _position: &Position,
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
                        format!(
                            "Vector store requires SIMD vector value, got {}",
                            vector_type
                        ),
                        Position::new(0, 0, 0),
                    )),
                }
            }
            _ => Err(CompileError::type_error(
                format!(
                    "Vector store requires pointer address, got {}",
                    address_type
                ),
                Position::new(0, 0, 0),
            )),
        }
    }

    /// Convert SIMD vector type to corresponding element type
    pub fn simd_vector_type_to_element_type(
        &self,
        vector_type: &crate::ast::SIMDVectorType,
    ) -> EaType {
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
            (
                EaType::SIMDVector {
                    vector_type: lv, ..
                },
                EaType::SIMDVector {
                    vector_type: rv, ..
                },
            ) => (lv, rv),
            _ => {
                return Err(CompileError::type_error(
                    format!(
                        "SIMD operations require vector operands, got {} and {}",
                        left_type, right_type
                    ),
                    Position::new(0, 0, 0),
                ))
            }
        };

        // Check element type compatibility for the specific operation
        match operator {
            crate::ast::SIMDOperator::DotAdd
            | crate::ast::SIMDOperator::DotSubtract
            | crate::ast::SIMDOperator::DotMultiply => {
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
            crate::ast::SIMDOperator::DotAnd
            | crate::ast::SIMDOperator::DotOr
            | crate::ast::SIMDOperator::DotXor => {
                if !self.simd_types_bitwise_compatible(left_vector_type, right_vector_type) {
                    return Err(CompileError::type_error(
                        format!("Bitwise SIMD operation {:?} requires integer or mask vector types, got {} and {}", 
                            operator, left_vector_type, right_vector_type),
                        Position::new(0, 0, 0),
                    ));
                }
            }
            crate::ast::SIMDOperator::DotEqual
            | crate::ast::SIMDOperator::DotNotEqual
            | crate::ast::SIMDOperator::DotLess
            | crate::ast::SIMDOperator::DotGreater
            | crate::ast::SIMDOperator::DotLessEqual
            | crate::ast::SIMDOperator::DotGreaterEqual => {
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
                format!(
                    "SIMD vector width mismatch: {} has {} elements, {} has {} elements",
                    left_vector_type,
                    left_vector_type.width(),
                    right_vector_type,
                    right_vector_type.width()
                ),
                Position::new(0, 0, 0),
            ));
        }

        Ok(())
    }

    pub fn simd_types_arithmetic_compatible(
        &self,
        left: &crate::ast::SIMDVectorType,
        right: &crate::ast::SIMDVectorType,
    ) -> bool {
        // Arithmetic operations work on numeric types (not masks)
        let left_numeric = !matches!(
            left,
            crate::ast::SIMDVectorType::Mask8
                | crate::ast::SIMDVectorType::Mask16
                | crate::ast::SIMDVectorType::Mask32
                | crate::ast::SIMDVectorType::Mask64
        );
        let right_numeric = !matches!(
            right,
            crate::ast::SIMDVectorType::Mask8
                | crate::ast::SIMDVectorType::Mask16
                | crate::ast::SIMDVectorType::Mask32
                | crate::ast::SIMDVectorType::Mask64
        );

        left_numeric && right_numeric && left.is_compatible_with(right)
    }

    pub fn simd_types_division_compatible(
        &self,
        left: &crate::ast::SIMDVectorType,
        right: &crate::ast::SIMDVectorType,
    ) -> bool {
        // Division is more restrictive - typically only floating point
        let left_float = matches!(left.element_type(), "f32" | "f64");
        let right_float = matches!(right.element_type(), "f32" | "f64");

        // Some implementations allow integer division, but we'll be conservative
        left_float && right_float && left.is_compatible_with(right)
    }

    pub fn simd_types_bitwise_compatible(
        &self,
        left: &crate::ast::SIMDVectorType,
        right: &crate::ast::SIMDVectorType,
    ) -> bool {
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
            crate::ast::SIMDOperator::DotAnd
            | crate::ast::SIMDOperator::DotOr
            | crate::ast::SIMDOperator::DotXor => {
                if matches!(expected_element_type, EaType::F32 | EaType::F64) {
                    return Err(CompileError::type_error(
                        format!(
                            "Bitwise SIMD operation {:?} not valid for floating-point vector {}",
                            operator, vector_type
                        ),
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
                format!(
                    "Invalid SIMD operation between {} and {}",
                    left_type, right_type
                ),
                Position::new(0, 0, 0),
            )),
        }
    }
}
