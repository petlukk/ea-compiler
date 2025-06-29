// src/lib.rs
//! Eä programming language compiler
//! 
//! A high-performance systems programming language with built-in SIMD support,
//! adaptive optimization, and memory safety guarantees.

pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod type_system;

// Conditionally include codegen module if LLVM feature is enabled
#[cfg(feature = "llvm")]
pub mod codegen;

// Re-export commonly used types
pub use error::{CompileError, Result};
pub use lexer::{Lexer, Token, TokenKind, Position};
pub use type_system::{TypeChecker, EaType, FunctionType, TypeContext};

/// Compiler version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = "Eä Compiler";

/// Tokenize a source string into a vector of tokens
pub fn tokenize(source: &str) -> Result<Vec<Token>> {
    let mut lexer = Lexer::new(source);
    lexer.tokenize_all()
}

/// Parse a source string into an AST
pub fn parse(source: &str) -> Result<Vec<ast::Stmt>> {
    let tokens = tokenize(source)?;
    let mut parser = parser::Parser::new(tokens);
    parser.parse_program()
}

/// Type check a parsed AST
pub fn type_check(program: &[ast::Stmt]) -> Result<TypeContext> {
    let mut type_checker = TypeChecker::new();
    type_checker.check_program(program)
}

/// Complete compilation pipeline: source -> tokens -> AST -> type checking
pub fn compile_to_ast(source: &str) -> Result<(Vec<ast::Stmt>, TypeContext)> {
    let program = parse(source)?;
    let type_context = type_check(&program)?;
    Ok((program, type_context))
}

/// Complete compilation pipeline with LLVM code generation (if feature enabled)
#[cfg(feature = "llvm")]
pub fn compile_to_llvm(source: &str, module_name: &str) -> Result<()> {
    use inkwell::context::Context;
    
    let (program, _type_context) = compile_to_ast(source)?;
    
    let context = Context::create();
    let mut codegen = codegen::CodeGenerator::new(&context, module_name);
    codegen.compile_program(&program)?;
    
    // Write LLVM IR to file for inspection
    let ir_filename = format!("{}.ll", module_name);
    codegen.write_ir_to_file(&ir_filename)?;
    
    Ok(())
}

/// JIT compile and execute a program immediately
#[cfg(feature = "llvm")]
pub fn jit_execute(source: &str, module_name: &str) -> Result<i32> {
    use inkwell::context::Context;
    use inkwell::execution_engine::JitFunction;
    use inkwell::OptimizationLevel;
    
    let (program, _type_context) = compile_to_ast(source)?;
    
    let context = Context::create();
    let mut codegen = codegen::CodeGenerator::new(&context, module_name);
    codegen.compile_program(&program)?;
    
    // Create execution engine for JIT compilation
    let execution_engine = codegen.get_module()
        .create_jit_execution_engine(OptimizationLevel::None)
        .map_err(|e| CompileError::codegen_error(
            format!("Failed to create JIT execution engine: {}", e),
            None
        ))?;
    
    // Find and execute the main function
    unsafe {
        // Try to get main as a void function first (most common case)
        let void_result = execution_engine.get_function::<unsafe extern "C" fn()>("main");
        if let Ok(main_fn) = void_result {
            let main_fn: JitFunction<unsafe extern "C" fn()> = main_fn;
            main_fn.call();
            return Ok(0); // Return 0 for successful void main
        }
        
        // Try to get main as an i32 function
        let i32_result = execution_engine.get_function::<unsafe extern "C" fn() -> i32>("main");
        if let Ok(main_fn) = i32_result {
            let main_fn: JitFunction<unsafe extern "C" fn() -> i32> = main_fn;
            let result = main_fn.call();
            return Ok(result);
        }
        
        Err(CompileError::codegen_error(
            "Main function not found or has unsupported signature".to_string(),
            None
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokenization() {
        let source = "func main() { let x = 42; }";
        let tokens = tokenize(source).unwrap();
        
        assert!(!tokens.is_empty());
        assert_eq!(tokens[0].kind, TokenKind::Func);
        assert_eq!(tokens.last().unwrap().kind, TokenKind::Eof);
    }

    #[test]
    fn test_basic_parsing() {
        let source = r#"
            func add(a: i32, b: i32) -> i32 {
                return a + b;
            }
        "#;
        
        let program = parse(source).unwrap();
        assert_eq!(program.len(), 1);
        
        match &program[0] {
            ast::Stmt::FunctionDeclaration { name, params, .. } => {
                assert_eq!(name, "add");
                assert_eq!(params.len(), 2);
            },
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_basic_type_checking() {
        let source = r#"
            func add(a: i32, b: i32) -> i32 {
                return a + b;
            }
            
            func main() -> () {
                let result = add(5, 10);
                return;
            }
        "#;
        
        let result = compile_to_ast(source);
        assert!(result.is_ok(), "Type checking should succeed for valid program");
    }

    #[test]
    fn test_type_error_detection() {
        let source = r#"
            func test() -> i32 {
                return "hello"; // Type error: string instead of i32
            }
        "#;
        
        let result = compile_to_ast(source);
        assert!(result.is_err(), "Type checking should fail for invalid program");
    }

    #[test]
    fn test_expression_type_checking() {
        let source = "1 + 2 * 3";
        let tokens = tokenize(source).unwrap();
        let mut parser = parser::Parser::new(tokens);
        let expr = parser.parse().unwrap();
        
        let mut type_checker = TypeChecker::new();
        let expr_type = type_checker.check_expression(&expr).unwrap();
        
        assert_eq!(expr_type, EaType::I64);
    }

    #[cfg(feature = "llvm")]
    #[test]
    fn test_llvm_compilation() {
        let source = r#"
            func add(a: i32, b: i32) -> i32 {
                return a + b;
            }
        "#;
        
        let result = compile_to_llvm(source, "test_module");
        assert!(result.is_ok(), "LLVM compilation should succeed");
    }

    #[test]
    fn test_complex_type_checking() {
        let source = r#"
            func fibonacci(n: i32) -> i32 {
                if (n <= 1) {
                    return n;
                }
                return fibonacci(n - 1) + fibonacci(n - 2);
            }

            func main() -> () {
                let result: i32 = fibonacci(10);
                let is_large: bool = result > 50;
                
                if (is_large) {
                    let message: string = "Result is large";
                }
                
                return;
            }
        "#;
        
        let result = compile_to_ast(source);
        assert!(result.is_ok(), "Complex program should type check successfully");
    }

    #[test]
    fn test_scoping_rules() {
        let source = r#"
            func test_scoping() -> () {
                let x: i32 = 1;
                {
                    let x: string = "shadowed";
                    let y: bool = true;
                }
                let z: i32 = x + 1; // x should be i32 here
                return;
            }
        "#;
        
        let result = compile_to_ast(source);
        assert!(result.is_ok(), "Scoping should work correctly");
    }
}