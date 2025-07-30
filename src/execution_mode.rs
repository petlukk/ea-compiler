// src/execution_mode.rs
//! Execution mode analysis for smart --run implementation
//! 
//! This module analyzes program complexity to determine the optimal execution method:
//! - JIT execution for simple programs (fastest)
//! - Compiled execution for complex programs (most reliable)
//! - Intelligent fallback for medium complexity programs

use crate::ast::{Expr, Stmt, SIMDExpr, TypeAnnotation};
use crate::error::Result;
use crate::lexer::Position;

/// Execution mode determines how a program should be executed
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionMode {
    /// Direct JIT execution - safe for simple programs
    JitSafe,
    /// Try JIT first, fallback to compilation if needed
    JitRisky, 
    /// Skip JIT, use compilation directly for complex programs
    CompileRequired,
}

/// Program complexity analysis results
#[derive(Debug, Default)]
pub struct ComplexityAnalysis {
    /// Program has SIMD function calls with vector parameters
    pub has_simd_function_calls: bool,
    /// Total number of AST statements (proxy for program size)
    pub statement_count: usize,
    /// Maximum recursion depth detected
    pub max_recursion_depth: usize,
    /// Has complex control flow (nested loops, deep conditionals)
    pub complex_control_flow: bool,
    /// Has SIMD vector operations
    pub has_simd_operations: bool,
    /// Number of function definitions
    pub function_count: usize,
}

// Thresholds for execution mode determination
const LARGE_PROGRAM_THRESHOLD: usize = 50;
const COMPLEX_FUNCTION_THRESHOLD: usize = 15;
const MAX_RECURSION_DEPTH_THRESHOLD: usize = 5;

impl ComplexityAnalysis {
    /// Determine the appropriate execution mode based on complexity analysis
    pub fn execution_mode(&self) -> ExecutionMode {
        // Programs with SIMD function calls require compilation
        if self.has_simd_function_calls {
            return ExecutionMode::CompileRequired;
        }
        
        // Very large programs should skip JIT to avoid stack overflow
        if self.statement_count > LARGE_PROGRAM_THRESHOLD {
            return ExecutionMode::CompileRequired;
        }
        
        // Programs with many functions are complex and may hit JIT limitations
        if self.function_count > COMPLEX_FUNCTION_THRESHOLD {
            return ExecutionMode::CompileRequired;
        }
        
        // Programs with deep recursion or complex control flow are risky for JIT
        if self.max_recursion_depth > MAX_RECURSION_DEPTH_THRESHOLD || self.complex_control_flow {
            return ExecutionMode::JitRisky;
        }
        
        // Simple programs are safe for direct JIT execution
        ExecutionMode::JitSafe
    }
    
    /// Get a human-readable reason for the execution mode choice
    pub fn execution_reason(&self) -> &'static str {
        match self.execution_mode() {
            ExecutionMode::CompileRequired => {
                if self.has_simd_function_calls {
                    "SIMD functions detected"
                } else if self.statement_count > LARGE_PROGRAM_THRESHOLD {
                    "large program detected"
                } else if self.function_count > COMPLEX_FUNCTION_THRESHOLD {
                    "many functions detected"
                } else {
                    "complex program structure"
                }
            }
            ExecutionMode::JitRisky => "medium complexity detected",
            ExecutionMode::JitSafe => "simple program",
        }
    }
}

/// Analyze program complexity from AST statements
pub fn analyze_execution_complexity(statements: &[Stmt]) -> Result<ComplexityAnalysis> {
    let mut analysis = ComplexityAnalysis::default();
    
    for stmt in statements {
        analyze_statement(stmt, &mut analysis, 0);
    }
    
    Ok(analysis)
}

/// Recursively analyze a single statement
fn analyze_statement(stmt: &Stmt, analysis: &mut ComplexityAnalysis, depth: usize) {
    analysis.statement_count += 1;
    analysis.max_recursion_depth = analysis.max_recursion_depth.max(depth);
    
    match stmt {
        Stmt::FunctionDeclaration { name: _, params, return_type: _, body, attributes: _ } => {
            analysis.function_count += 1;
            
            // Check if function has vector parameters (indicates SIMD function calls)
            for param in params {
                if is_vector_type(&param.type_annotation) {
                    analysis.has_simd_function_calls = true;
                }
            }
            
            // Analyze function body
            analyze_statement(body, analysis, depth + 1);
        }
        
        Stmt::If { condition, then_branch, else_branch } => {
            analyze_expression(condition, analysis);
            
            // Analyze branches
            analyze_statement(then_branch, analysis, depth + 1);
            
            if let Some(else_stmt) = else_branch {
                analyze_statement(else_stmt, analysis, depth + 1);
            }
            
            // If statements with deep nesting indicate complex control flow
            if depth > 2 {
                analysis.complex_control_flow = true;
            }
        }
        
        Stmt::While { condition, body } => {
            analyze_expression(condition, analysis);
            
            // While loops are complex control flow
            analysis.complex_control_flow = true;
            
            analyze_statement(body, analysis, depth + 1);
        }
        
        Stmt::For { initializer, condition, increment, body } => {
            if let Some(init) = initializer {
                analyze_statement(init, analysis, depth + 1);
            }
            
            if let Some(cond) = condition {
                analyze_expression(cond, analysis);
            }
            
            if let Some(inc) = increment {
                analyze_expression(inc, analysis);
            }
            
            analyze_statement(body, analysis, depth + 1);
            
            // For loops indicate complex iteration
            analysis.complex_control_flow = true;
        }
        
        Stmt::ForIn { variable: _, iterable, body } => {
            analyze_expression(iterable, analysis);
            analyze_statement(body, analysis, depth + 1);
            
            // For-in loops indicate complex iteration
            analysis.complex_control_flow = true;
        }
        
        Stmt::VarDeclaration { pattern: _, type_annotation, initializer } => {
            if let Some(type_ann) = type_annotation {
                if is_vector_type(type_ann) {
                    analysis.has_simd_operations = true;
                }
            }
            if let Some(expr) = initializer {
                analyze_expression(expr, analysis);
            }
        }
        
        Stmt::Block(statements) => {
            for stmt in statements {
                analyze_statement(stmt, analysis, depth + 1);
            }
        }
        
        Stmt::Expression(expr) => {
            analyze_expression(expr, analysis);
        }
        
        Stmt::Return(Some(expr)) => {
            analyze_expression(expr, analysis);
        }
        
        Stmt::Return(None) | Stmt::Break | Stmt::Continue => {
            // Simple control flow statements, no additional analysis needed
        }
        
        Stmt::StructDeclaration { .. } | Stmt::EnumDeclaration { .. } => {
            // Type declarations don't affect execution complexity for our purposes
        }
    }
}

/// Analyze expressions for complexity markers
fn analyze_expression(expr: &Expr, analysis: &mut ComplexityAnalysis) {
    match expr {
        Expr::Call(callee, args) => {
            analyze_expression(callee, analysis);
            
            // Check for function calls with vector arguments
            for arg in args {
                if is_likely_vector_expression(arg) {
                    analysis.has_simd_function_calls = true;
                }
                analyze_expression(arg, analysis);
            }
        }
        
        Expr::SIMD(simd_expr) => {
            analysis.has_simd_operations = true;
            analyze_simd_expression(simd_expr, analysis);
        }
        
        Expr::Binary(left, _op, right) => {
            analyze_expression(left, analysis);
            analyze_expression(right, analysis);
        }
        
        Expr::Unary(_op, operand) => {
            analyze_expression(operand, analysis);
        }
        
        Expr::Variable(_) | Expr::Literal(_) => {
            // Simple expressions, no complexity
        }
        
        Expr::Grouping(expr) => {
            analyze_expression(expr, analysis);
        }
        
        Expr::Index(array, index) => {
            analyze_expression(array, analysis);
            analyze_expression(index, analysis);
        }
        
        Expr::Slice { array, start, end } => {
            analyze_expression(array, analysis);
            analyze_expression(start, analysis);
            analyze_expression(end, analysis);
        }
        
        Expr::FieldAccess(object, _field) => {
            analyze_expression(object, analysis);
        }
        
        Expr::StructLiteral { name: _, fields } => {
            for field in fields {
                analyze_expression(&field.value, analysis);
            }
        }
        
        Expr::EnumLiteral { enum_name: _, variant: _, args } => {
            for arg in args {
                analyze_expression(arg, analysis);
            }
        }
        
        Expr::Match { value, arms } => {
            analyze_expression(value, analysis);
            for arm in arms {
                analyze_expression(&arm.expression, analysis);
            }
        }
        
        Expr::If { condition, then_branch, else_branch } => {
            analyze_expression(condition, analysis);
            analyze_expression(then_branch, analysis);
            if let Some(else_expr) = else_branch {
                analyze_expression(else_expr, analysis);
            }
        }
        
        Expr::Block(statements) => {
            for stmt in statements {
                analyze_statement(stmt, analysis, 0);
            }
        }
        
        Expr::Cast { expr, target_type: _, position: _ } => {
            analyze_expression(expr, analysis);
        }
        
        Expr::Tuple { elements, position: _ } => {
            for element in elements {
                analyze_expression(element, analysis);
            }
        }
    }
}

/// Analyze SIMD expressions specifically
fn analyze_simd_expression(simd_expr: &SIMDExpr, analysis: &mut ComplexityAnalysis) {
    analysis.has_simd_operations = true;
    
    match simd_expr {
        SIMDExpr::VectorLiteral { elements, vector_type: _, position: _ } => {
            for element in elements {
                analyze_expression(element, analysis);
            }
        }
        
        SIMDExpr::ElementWise { left, operator: _, right, position: _ } => {
            analyze_expression(left, analysis);
            analyze_expression(right, analysis);
        }
        
        SIMDExpr::Broadcast { value, target_type: _, position: _ } => {
            analyze_expression(value, analysis);
        }
        
        SIMDExpr::Swizzle { vector, pattern: _, position: _ } => {
            analyze_expression(vector, analysis);
        }
        
        SIMDExpr::Reduction { vector, operation: _, position: _ } => {
            analyze_expression(vector, analysis);
        }
        
        SIMDExpr::DotProduct { left, right, position: _ } => {
            analyze_expression(left, analysis);
            analyze_expression(right, analysis);
        }
        
        SIMDExpr::VectorLoad { address, vector_type: _, alignment: _, position: _ } => {
            analyze_expression(address, analysis);
        }
        
        SIMDExpr::VectorStore { address, vector, alignment: _, position: _ } => {
            analyze_expression(address, analysis);
            analyze_expression(vector, analysis);
        }
    }
}

/// Check if a type annotation represents a vector type
fn is_vector_type(type_annotation: &TypeAnnotation) -> bool {
    // Check if the type name represents a SIMD vector type
    let name = &type_annotation.name;
    name.ends_with("x4") || name.ends_with("x8") || name.ends_with("x16") || name.ends_with("x32") ||
    name.starts_with("f32x") || name.starts_with("f64x") || name.starts_with("i32x") || 
    name.starts_with("i64x") || name.starts_with("u8x") || name.starts_with("u16x") || 
    name.starts_with("u32x") || name.starts_with("u64x")
}

/// Heuristic to detect if an expression is likely to produce a vector value
fn is_likely_vector_expression(expr: &Expr) -> bool {
    match expr {
        Expr::SIMD(_) => true,
        Expr::Variable(name) => {
            // Common vector variable naming patterns
            name.contains("vec") || name.contains("simd") || name.ends_with("x4") || name.ends_with("x8") || name.ends_with("x16")
        }
        Expr::Call(callee, _args) => {
            // Check if the callee is a function that commonly returns vectors
            match callee.as_ref() {
                Expr::Variable(name) => {
                    name.contains("horizontal") || name.contains("simd") || name.contains("vector")
                }
                _ => false,
            }
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Literal, Parameter};
    
    #[test]
    fn test_simple_program_analysis() {
        let statements = vec![
            Stmt::FunctionDeclaration {
                name: "main".to_string(),
                params: vec![],
                return_type: Some(TypeAnnotation {
                    name: "i32".to_string(),
                    is_mutable: false,
                }),
                body: Box::new(Stmt::Block(vec![
                    Stmt::Expression(Expr::Call(
                        Box::new(Expr::Variable("println".to_string())),
                        vec![Expr::Literal(Literal::String("Hello".to_string()))]
                    )),
                    Stmt::Return(Some(Expr::Literal(Literal::Integer(0))))
                ])),
                attributes: vec![],
            }
        ];
        
        let analysis = analyze_execution_complexity(&statements).unwrap();
        assert_eq!(analysis.execution_mode(), ExecutionMode::JitSafe);
        assert_eq!(analysis.function_count, 1);
        assert!(!analysis.has_simd_function_calls);
        assert!(!analysis.complex_control_flow);
    }
    
    #[test]
    fn test_simd_program_analysis() {
        let statements = vec![
            Stmt::FunctionDeclaration {
                name: "simd_func".to_string(),
                params: vec![
                    Parameter {
                        name: "vec".to_string(),
                        type_annotation: TypeAnnotation {
                            name: "f32x4".to_string(),
                            is_mutable: false,
                        },
                    }
                ],
                return_type: Some(TypeAnnotation {
                    name: "f32".to_string(),
                    is_mutable: false,
                }),
                body: Box::new(Stmt::Block(vec![
                    Stmt::Return(Some(Expr::Literal(Literal::Float(1.0))))
                ])),
                attributes: vec![],
            }
        ];
        
        let analysis = analyze_execution_complexity(&statements).unwrap();
        assert_eq!(analysis.execution_mode(), ExecutionMode::CompileRequired);
        assert!(analysis.has_simd_function_calls);
        assert_eq!(analysis.execution_reason(), "SIMD functions detected");
    }
    
    #[test]
    fn test_complex_control_flow() {
        let statements = vec![
            Stmt::FunctionDeclaration {
                name: "complex".to_string(),
                params: vec![],
                return_type: Some(TypeAnnotation {
                    name: "i32".to_string(),
                    is_mutable: false,
                }),
                body: Box::new(Stmt::Block(vec![
                    Stmt::While {
                        condition: Expr::Literal(Literal::Boolean(true)),
                        body: Box::new(Stmt::Block(vec![
                            Stmt::If {
                                condition: Expr::Literal(Literal::Boolean(true)),
                                then_branch: Box::new(Stmt::Block(vec![
                                    Stmt::Expression(Expr::Literal(Literal::Integer(1))),
                                    Stmt::Expression(Expr::Literal(Literal::Integer(2))),
                                    Stmt::Expression(Expr::Literal(Literal::Integer(3))),
                                    Stmt::Expression(Expr::Literal(Literal::Integer(4))),
                                ])),
                                else_branch: None,
                            }
                        ])),
                    }
                ])),
                attributes: vec![],
            }
        ];
        
        let analysis = analyze_execution_complexity(&statements).unwrap();
        assert_eq!(analysis.execution_mode(), ExecutionMode::JitRisky);
        assert!(analysis.complex_control_flow);
    }
}