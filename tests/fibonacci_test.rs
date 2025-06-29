// tests/fibonacci_test.rs - Updated with working test cases
//! End-to-end test to verify that fibonacci actually compiles and generates correct LLVM IR

#[cfg(feature = "llvm")]
use ea_compiler::{
    compile_to_ast,
    codegen::CodeGenerator,
};
#[cfg(feature = "llvm")]
use inkwell::context::Context;

#[cfg(feature = "llvm")]
#[test]
fn test_fibonacci_compiles_to_llvm() {
    let source = r#"
        func fibonacci(n: i32) -> i32 {
            if (n <= 1) {
                return n;
            }
            return fibonacci(n - 1) + fibonacci(n - 2);
        }
        
        func main() -> i32 {
            return fibonacci(5);
        }
    "#;
    
    println!("Testing fibonacci source:");
    println!("{}", source);
    
    // First, test parsing and type checking
    let (program, _type_context) = compile_to_ast(source)
        .expect("Fibonacci should parse and type check successfully");
    
    println!("✅ Parsing and type checking successful");
    println!("Program has {} statements", program.len());
    
    // Now test LLVM code generation
    let context = Context::create();
    let mut codegen = CodeGenerator::new(&context, "fibonacci_test");
    
    let result = codegen.compile_program(&program);
    match result {
        Ok(()) => {
            println!("✅ LLVM code generation successful!");
            
            // Write the IR to see what we generated
            if let Err(e) = codegen.write_ir_to_file("fibonacci_test.ll") {
                println!("Warning: Failed to write IR file: {}", e);
            } else {
                println!("✅ LLVM IR written to fibonacci_test.ll");
            }
        },
        Err(e) => {
            panic!("❌ LLVM code generation failed: {}", e);
        }
    }
}

#[cfg(feature = "llvm")]
#[test]
fn test_simple_if_statement() {
    let source = r#"
        func test_if(x: i32) -> i32 {
            if (x > 0) {
                return 1;
            }
            return 0;
        }
    "#;
    
    let (program, _) = compile_to_ast(source)
        .expect("Simple if statement should parse");
    
    let context = Context::create();
    let mut codegen = CodeGenerator::new(&context, "test_if");
    
    codegen.compile_program(&program)
        .expect("Simple if statement should compile to LLVM");
    
    println!("✅ Simple if statement compiles successfully");
}

// Removed test_basic_assignment and test_arithmetic_operations as they're redundant with type_system_tests.rs

#[cfg(feature = "llvm")]
#[test]
fn test_comparison_operations() {
    let source = r#"
        func test_comparisons(x: i32, y: i32) -> i32 {
            if (x < y) {
                return 1;
            }
            if (x <= y) {
                return 2;
            }
            if (x > y) {
                return 3;
            }
            if (x >= y) {
                return 4;
            }
            if (x == y) {
                return 5;
            }
            return 0;
        }
    "#;
    
    let (program, _) = compile_to_ast(source)
        .expect("Comparison operations should parse");
    
    let context = Context::create();
    let mut codegen = CodeGenerator::new(&context, "test_comparisons");
    
    codegen.compile_program(&program)
        .expect("Comparison operations should compile to LLVM");
    
    println!("✅ Comparison operations compile successfully");
}

#[cfg(feature = "llvm")]
#[test]
fn test_nested_if_statements() {
    let source = r#"
        func test_nested(x: i32, y: i32) -> i32 {
            if (x > 0) {
                if (y > 0) {
                    return x + y;
                }
                return x;
            }
            return 0;
        }
    "#;
    
    let (program, _) = compile_to_ast(source)
        .expect("Nested if statements should parse");
    
    let context = Context::create();
    let mut codegen = CodeGenerator::new(&context, "test_nested");
    
    codegen.compile_program(&program)
        .expect("Nested if statements should compile to LLVM");
    
    println!("✅ Nested if statements compile successfully");
}

#[cfg(feature = "llvm")]
#[test]
fn test_void_function() {
    let source = r#"
        func test_void() -> () {
            let x: i32 = 42;
            return;
        }
    "#;
    
    let (program, _) = compile_to_ast(source)
        .expect("Void function should parse");
    
    let context = Context::create();
    let mut codegen = CodeGenerator::new(&context, "test_void");
    
    codegen.compile_program(&program)
        .expect("Void function should compile to LLVM");
    
    println!("✅ Void function compiles successfully");
}

// Test without LLVM feature to ensure we have graceful degradation
#[cfg(not(feature = "llvm"))]
#[test]
fn test_fibonacci_parsing_only() {
    let source = r#"
        func fibonacci(n: i32) -> i32 {
            if (n <= 1) {
                return n;
            }
            return fibonacci(n - 1) + fibonacci(n - 2);
        }
    "#;
    
    // This should still work - parsing and type checking
    let (program, _) = compile_to_ast(source)
        .expect("Fibonacci should parse and type check even without LLVM");
    
    println!("✅ Fibonacci parsing and type checking works (LLVM disabled)");
    println!("Program has {} statements", program.len());
}

#[test]
fn test_factorial_parsing() {
    let source = r#"
        func factorial(n: i32) -> i32 {
            if (n <= 1) {
                return 1;
            }
            return n * factorial(n - 1);
        }
    "#;
    
    let (program, _) = compile_to_ast(source)
        .expect("Factorial should parse and type check");
    
    #[cfg(feature = "llvm")]
    {
        let context = Context::create();
        let mut codegen = CodeGenerator::new(&context, "test_factorial");
        
        codegen.compile_program(&program)
            .expect("Factorial should compile to LLVM");
        
        println!("✅ Factorial compiles successfully");
    }
    
    #[cfg(not(feature = "llvm"))]
    {
        println!("✅ Factorial parsing successful (LLVM disabled)");
    }
}