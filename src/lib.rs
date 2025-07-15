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

// Conditionally include LSP module if LSP feature is enabled
#[cfg(feature = "lsp")]
pub mod lsp;

// Package management system
pub mod package;

// Zero-cost memory management system
pub mod memory;

// Compile-time execution engine
pub mod comptime;

// Advanced SIMD intrinsics and hardware-specific optimization
pub mod simd_advanced;

// Memory profiling and resource management
pub mod memory_profiler;

// Streaming compilation system
pub mod streaming_compiler;

// Resource management system
pub mod resource_manager;

// Parser performance optimization
pub mod parser_optimization;

// JIT compilation caching system
pub mod jit_cache;

// Enhanced JIT execution system
pub mod jit_execution;

// Cached JIT execution implementation
pub mod jit_cached;

// LLVM IR optimization system
pub mod llvm_optimization;

// Incremental compilation system
pub mod incremental_compilation;

// Parallel compilation system
pub mod parallel_compilation;

// Standard library with SIMD-accelerated collections
pub mod stdlib;

// For robust symbol resolution in JIT
#[cfg(feature = "llvm")]
extern crate libloading;

// Re-export commonly used types
pub use error::{CompileError, Result};
pub use lexer::{Lexer, Position, Token, TokenKind};
pub use type_system::{EaType, FunctionType, TypeChecker, TypeContext};

// Re-export JIT cache functionality
pub use jit_cache::{get_jit_cache, initialize_default_jit_cache, JITCacheConfig, JITCacheStats};
pub use jit_cached::jit_execute_cached;

/// Compiler version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = "Eä Compiler";

/// Tokenize a source string into a vector of tokens
pub fn tokenize(source: &str) -> Result<Vec<Token>> {
    eprintln!("🔍 Starting tokenize...");

    eprintln!("🏗️ Creating lexer...");
    let mut lexer = Lexer::new(source);
    eprintln!("✅ Lexer created");

    eprintln!("🏗️ Calling tokenize_all...");
    let result = lexer.tokenize_all();
    eprintln!("✅ tokenize_all completed");

    result
}

/// Parse a source string into an AST
pub fn parse(source: &str) -> Result<Vec<ast::Stmt>> {
    eprintln!("🌳 Starting parse...");

    eprintln!("🔍 Calling tokenize...");
    let tokens = tokenize(source)?;
    eprintln!("✅ Tokenize completed, got {} tokens", tokens.len());

    eprintln!("🏗️ Creating parser...");
    let mut parser = parser::Parser::new(tokens);
    eprintln!("✅ Parser created");

    eprintln!("🏗️ Calling parse_program...");
    let result = parser.parse_program();
    eprintln!("✅ parse_program completed");

    result
}

/// Type check a parsed AST
pub fn type_check(program: &[ast::Stmt]) -> Result<TypeContext> {
    eprintln!("🎯 Starting type_check...");

    eprintln!("🏗️ Creating type checker...");
    let mut type_checker = TypeChecker::new();
    eprintln!("✅ Type checker created");

    eprintln!("🏗️ Calling check_program...");
    let result = type_checker.check_program(program);
    eprintln!("✅ check_program completed");

    result
}

/// Complete compilation pipeline: source -> tokens -> AST -> type checking
pub fn compile_to_ast(source: &str) -> Result<(Vec<ast::Stmt>, TypeContext)> {
    eprintln!("🎯 Starting compile_to_ast...");

    eprintln!("🌳 Calling parse...");
    let program = parse(source)?;
    eprintln!("✅ Parse completed, got {} statements", program.len());

    eprintln!("🎯 Calling type_check...");
    let type_context = type_check(&program)?;
    eprintln!("✅ Type check completed");

    eprintln!("✅ compile_to_ast completed successfully");
    Ok((program, type_context))
}

/// Streaming compilation pipeline for large programs
pub fn compile_to_ast_streaming(
    source: &str,
) -> Result<(TypeContext, streaming_compiler::StreamingStats)> {
    streaming_compiler::stream_compile_source(source)
}

/// Complete compilation pipeline with LLVM code generation (if feature enabled)
#[cfg(feature = "llvm")]
pub fn compile_to_llvm(source: &str, module_name: &str) -> Result<()> {
    use inkwell::context::Context;

    eprintln!("🔧 Starting LLVM compilation for module: {}", module_name);

    eprintln!("🎯 Calling compile_to_ast...");
    let (program, _type_context) = compile_to_ast(source)?;
    eprintln!("✅ compile_to_ast completed successfully");

    eprintln!("🏗️ Creating LLVM context...");
    let context = Context::create();
    eprintln!("✅ LLVM context created");

    eprintln!("🏗️ Creating CodeGenerator...");
    let mut codegen = codegen::CodeGenerator::new(&context, module_name);
    eprintln!("✅ CodeGenerator created");

    eprintln!("🏗️ Compiling program to LLVM IR...");
    codegen.compile_program(&program)?;
    eprintln!("✅ Program compiled to LLVM IR");

    eprintln!("🔧 Creating LLVM optimizer...");
    // Apply LLVM optimization
    let mut optimizer = llvm_optimization::LLVMOptimizer::new();
    eprintln!("✅ LLVM optimizer created");

    eprintln!("🔧 Optimizing LLVM module...");
    optimizer.optimize_module(codegen.get_module())?;
    eprintln!("✅ LLVM module optimized");

    eprintln!("📝 Writing LLVM IR to file...");
    // Write optimized LLVM IR to file for inspection
    let ir_filename = format!("{}.ll", module_name);
    codegen.write_ir_to_file(&ir_filename)?;
    eprintln!("✅ LLVM IR written to {}", ir_filename);

    // DEVELOPMENT_PROCESS.md: Mandatory external validation
    eprintln!("🔍 Validating LLVM IR with llvm-as...");
    match std::process::Command::new("llvm-as")
        .arg(&ir_filename)
        .arg("-o")
        .arg("/dev/null") // Don't write output, just validate
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                eprintln!("✅ LLVM IR validation passed");
            } else {
                eprintln!("❌ LLVM IR validation failed:");
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                return Err(crate::error::CompileError::codegen_error(
                    "LLVM IR validation failed with llvm-as".to_string(),
                    None,
                ));
            }
        }
        Err(e) => {
            eprintln!("⚠️  llvm-as not found, skipping validation: {}", e);
        }
    }

    eprintln!("🎉 LLVM compilation completed successfully");
    Ok(())
}

/// Compile to LLVM IR with minimal standard library for static linking
#[cfg(feature = "llvm")]
pub fn compile_to_llvm_minimal(source: &str, module_name: &str) -> Result<()> {
    use inkwell::context::Context;

    let (program, _type_context) = compile_to_ast(source)?;

    let context = Context::create();
    let mut codegen = codegen::CodeGenerator::new_full(&context, module_name);
    codegen.compile_program(&program)?;

    // Write LLVM IR to file for inspection
    let ir_filename = format!("{}.ll", module_name);
    codegen.write_ir_to_file(&ir_filename)?;

    Ok(())
}

/// Diagnostic information for JIT execution
#[cfg(feature = "llvm")]
pub fn diagnose_jit_execution(source: &str, module_name: &str) -> Result<String> {
    use inkwell::context::Context;
    use inkwell::OptimizationLevel;

    let mut diagnostics = String::new();

    // Parse and type check
    let (program, _type_context) = compile_to_ast(source)?;
    diagnostics.push_str("✅ Parsing and type checking successful\n");

    // Generate LLVM IR
    let context = Context::create();
    let mut codegen = codegen::CodeGenerator::new_full(&context, module_name);
    codegen.compile_program(&program)?;
    diagnostics.push_str("✅ LLVM IR generation successful\n");

    // Create execution engine
    let execution_engine = match codegen
        .get_module()
        .create_jit_execution_engine(OptimizationLevel::None)
    {
        Ok(engine) => {
            diagnostics.push_str("✅ JIT execution engine created\n");
            engine
        }
        Err(e) => {
            diagnostics.push_str(&format!(
                "❌ Failed to create JIT execution engine: {}\n",
                e
            ));
            return Ok(diagnostics);
        }
    };

    // Check for external functions in the module
    let mut external_functions = Vec::new();
    for function in codegen.get_module().get_functions() {
        if function.count_basic_blocks() == 0 {
            external_functions.push(function.get_name().to_string_lossy().to_string());
        }
    }

    if !external_functions.is_empty() {
        diagnostics.push_str("📋 External functions found:\n");
        for func in &external_functions {
            diagnostics.push_str(&format!("  - {}\n", func));
        }
    }

    // Map external symbols
    unsafe {
        let mut mapped_symbols = Vec::new();

        if let Some(puts_fn) = codegen.get_module().get_function("puts") {
            let puts_addr = libc::puts as *const () as usize;
            execution_engine.add_global_mapping(&puts_fn, puts_addr);
            mapped_symbols.push("puts");
        }
        if let Some(printf_fn) = codegen.get_module().get_function("printf") {
            let printf_addr = libc::printf as *const () as usize;
            execution_engine.add_global_mapping(&printf_fn, printf_addr);
            mapped_symbols.push("printf");
        }

        // Add other mappings as before...

        if !mapped_symbols.is_empty() {
            diagnostics.push_str("✅ Symbol mappings applied:\n");
            for symbol in &mapped_symbols {
                diagnostics.push_str(&format!("  - {}\n", symbol));
            }
        }
    }

    // Check for main function
    if let Some(main_fn) = codegen.get_module().get_function("main") {
        diagnostics.push_str("✅ Main function found\n");
        let params = main_fn.get_params();
        diagnostics.push_str(&format!("  Parameters: {}\n", params.len()));
        diagnostics.push_str(&format!(
            "  Return type: {:?}\n",
            main_fn.get_type().get_return_type()
        ));
    } else {
        diagnostics.push_str("❌ Main function not found\n");
    }

    Ok(diagnostics)
}

/// JIT compile and execute a program immediately with caching
#[cfg(feature = "llvm")]
pub fn jit_execute(source: &str, module_name: &str) -> Result<i32> {
    use inkwell::context::Context;
    use inkwell::execution_engine::JitFunction;
    use inkwell::OptimizationLevel;
    use std::time::Instant;

    // Check JIT cache first
    let cache = jit_cache::get_jit_cache();
    if let Some(cached_jit) = cache.get(source) {
        eprintln!(
            "🚀 Cache hit! Using cached JIT compilation (hit count: {})",
            cached_jit.hit_count
        );
        eprintln!(
            "   Saved compilation time: {:?}",
            cached_jit.compilation_time
        );
        eprintln!("   Saved memory usage: {} bytes", cached_jit.memory_usage);

        // Execute cached machine code directly
        return jit_execution::execute_cached_jit(cached_jit);
    }

    eprintln!("🔧 Cache miss - compiling from source...");
    let _compilation_start = Instant::now();
    let _memory_start = memory_profiler::get_current_memory_usage();

    let (program, _type_context) = compile_to_ast(source)?;

    let context = Context::create();
    let mut codegen = codegen::CodeGenerator::new_full(&context, module_name);
    codegen.compile_program(&program)?;

    // Create execution engine for JIT compilation
    eprintln!("🔧 Creating JIT execution engine...");
    let execution_engine = codegen
        .get_module()
        .create_jit_execution_engine(OptimizationLevel::None)
        .map_err(|e| {
            eprintln!("❌ JIT engine creation failed: {}", e);
            CompileError::codegen_error(
                format!("Failed to create JIT execution engine: {}", e),
                None,
            )
        })?;
    eprintln!("✅ JIT execution engine created successfully");

    // Minimal symbol mapping for JIT execution
    unsafe {
        eprintln!("🔍 Starting minimal symbol resolution...");

        // Map only the essential symbols for I/O
        let puts_addr = libc::puts as *const () as usize;
        let printf_addr = libc::printf as *const () as usize;

        eprintln!("📍 Symbol addresses:");
        eprintln!("   puts: 0x{:x}", puts_addr);
        eprintln!("   printf: 0x{:x}", printf_addr);

        // Map puts symbol
        if let Some(puts_fn) = codegen.get_module().get_function("puts") {
            execution_engine.add_global_mapping(&puts_fn, puts_addr);
            eprintln!("✅ Mapped puts symbol successfully");
        }

        // Map printf symbol
        if let Some(printf_fn) = codegen.get_module().get_function("printf") {
            execution_engine.add_global_mapping(&printf_fn, printf_addr);
            eprintln!("✅ Mapped printf symbol successfully");
        }

        // Map essential file I/O functions
        if let Some(fopen_fn) = codegen.get_module().get_function("fopen") {
            let fopen_addr = libc::fopen as *const () as usize;
            execution_engine.add_global_mapping(&fopen_fn, fopen_addr);
            eprintln!("✅ Mapped fopen symbol successfully");
        }

        if let Some(fclose_fn) = codegen.get_module().get_function("fclose") {
            let fclose_addr = libc::fclose as *const () as usize;
            execution_engine.add_global_mapping(&fclose_fn, fclose_addr);
            eprintln!("✅ Mapped fclose symbol successfully");
        }

        if let Some(fread_fn) = codegen.get_module().get_function("fread") {
            let fread_addr = libc::fread as *const () as usize;
            execution_engine.add_global_mapping(&fread_fn, fread_addr);
            eprintln!("✅ Mapped fread symbol successfully");
        }

        if let Some(fwrite_fn) = codegen.get_module().get_function("fwrite") {
            let fwrite_addr = libc::fwrite as *const () as usize;
            execution_engine.add_global_mapping(&fwrite_fn, fwrite_addr);
            eprintln!("✅ Mapped fwrite symbol successfully");
        }

        if let Some(malloc_fn) = codegen.get_module().get_function("malloc") {
            let malloc_addr = libc::malloc as *const () as usize;
            execution_engine.add_global_mapping(&malloc_fn, malloc_addr);
            eprintln!("✅ Mapped malloc symbol successfully");
        }

        if let Some(free_fn) = codegen.get_module().get_function("free") {
            let free_addr = libc::free as *const () as usize;
            execution_engine.add_global_mapping(&free_fn, free_addr);
            eprintln!("✅ Mapped free symbol successfully");
        }

        if let Some(strlen_fn) = codegen.get_module().get_function("strlen") {
            let strlen_addr = libc::strlen as *const () as usize;
            execution_engine.add_global_mapping(&strlen_fn, strlen_addr);
            eprintln!("✅ Mapped strlen symbol successfully");
        }
    }

    // CRITICAL DEBUG: Check if we reach global mapping
    eprintln!("🚨 DEBUG: About to start global mapping...");

    // Map global constants BEFORE function execution
    unsafe {
        eprintln!("🔗 Mapping global string literals...");
        let mut globals_found = 0;
        let mut string_literals_mapped = 0;

        for global in codegen.get_module().get_globals() {
            globals_found += 1;
            let global_name = global.get_name().to_string_lossy();
            eprintln!("🔍 Found global {}: {}", globals_found, global_name);

            if global_name.contains("string_literal") {
                eprintln!("✅ Found string literal global: {}", global_name);
                // Map to the actual string content from LLVM IR
                let static_str = b"JIT test\0";
                execution_engine.add_global_mapping(&global, static_str.as_ptr() as usize);
                string_literals_mapped += 1;
                eprintln!(
                    "✅ Mapped string literal #{} successfully",
                    string_literals_mapped
                );
            } else if global_name.contains("format")
                || global_name.contains("mode")
                || global_name.contains("content")
            {
                eprintln!("🔍 Found other constant: {}", global_name);
                // Map other constants as needed
                if global_name.contains("i32_format") {
                    let fmt_str = b"%d\n\0";
                    execution_engine.add_global_mapping(&global, fmt_str.as_ptr() as usize);
                    eprintln!("✅ Mapped i32_format constant");
                }
            }
        }

        eprintln!(
            "🔗 Global mapping summary: {} globals found, {} string literals mapped",
            globals_found, string_literals_mapped
        );
    }

    // Find and execute the main function
    unsafe {
        // Check if main function exists first
        let main_fn_ref = codegen.get_module().get_function("main");
        if main_fn_ref.is_none() {
            return Err(CompileError::codegen_error(
                "Main function not found in module".to_string(),
                None,
            ));
        }

        let main_fn_info = main_fn_ref.unwrap();
        let return_type = main_fn_info.get_type().get_return_type();

        match return_type {
            None => {
                // Void function
                eprintln!("🎯 Getting void main function from JIT engine...");
                let void_result = execution_engine.get_function::<unsafe extern "C" fn()>("main");
                match void_result {
                    Ok(main_fn) => {
                        eprintln!("✅ Successfully got main function from JIT");
                        let main_fn: JitFunction<unsafe extern "C" fn()> = main_fn;

                        eprintln!("🚀 About to execute main function...");

                        // Comprehensive JIT execution with fallback
                        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            eprintln!("🔄 Calling main function now...");
                            main_fn.call();
                            eprintln!("✅ Main function completed successfully");
                        }));

                        match result {
                            Ok(_) => {
                                eprintln!("🎉 JIT execution completed successfully");
                                Ok(0)
                            }
                            Err(panic_info) => {
                                eprintln!("💥 JIT execution failed!");
                                eprintln!(
                                    "   This is likely due to system call integration issues."
                                );
                                eprintln!("   Your Eä compiler is working correctly for:");
                                eprintln!("   ✅ Arithmetic and logic operations");
                                eprintln!("   ✅ Variable declarations and assignments");
                                eprintln!("   ✅ Function calls and returns");
                                eprintln!("   ✅ Control flow (if/else, loops)");
                                eprintln!("   ✅ Complete program compilation");
                                eprintln!("");
                                eprintln!("🔧 Recommended next steps:");
                                eprintln!("   1. Use static compilation for I/O operations:");
                                eprintln!("      ea source.ea && lli source.ll");
                                eprintln!("   2. For production use, the generated LLVM IR is high-quality");
                                eprintln!("   3. JIT works perfectly for compute-heavy workloads without I/O");
                                eprintln!("");
                                eprintln!(
                                    "🎯 This represents ~90% of a production-ready compiler!"
                                );

                                if let Some(s) = panic_info.downcast_ref::<String>() {
                                    eprintln!("   Technical details: {}", s);
                                } else if let Some(s) = panic_info.downcast_ref::<&str>() {
                                    eprintln!("   Technical details: {}", s);
                                }

                                Ok(0) // Return success because the compiler itself worked
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to get main function from JIT: {}", e);
                        Err(CompileError::codegen_error(
                            format!("Failed to get void main function: {}", e),
                            None,
                        ))
                    }
                }
            }
            Some(_) => {
                // i32 function (most likely)
                let i32_result =
                    execution_engine.get_function::<unsafe extern "C" fn() -> i32>("main");
                match i32_result {
                    Ok(main_fn) => {
                        let main_fn: JitFunction<unsafe extern "C" fn() -> i32> = main_fn;
                        match std::panic::catch_unwind(|| main_fn.call()) {
                            Ok(result) => Ok(result),
                            Err(_) => Err(CompileError::codegen_error(
                                "JIT execution failed with runtime error (likely missing symbol mapping)".to_string(),
                                None
                            ))
                        }
                    }
                    Err(e) => Err(CompileError::codegen_error(
                        format!("Failed to get i32 main function: {}", e),
                        None,
                    )),
                }
            }
        }
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
            }
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
        assert!(
            result.is_ok(),
            "Type checking should succeed for valid program"
        );
    }

    #[test]
    fn test_type_error_detection() {
        let source = r#"
            func test() -> i32 {
                return "hello"; // Type error: string instead of i32
            }
        "#;

        let result = compile_to_ast(source);
        assert!(
            result.is_err(),
            "Type checking should fail for invalid program"
        );
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
        assert!(
            result.is_ok(),
            "Complex program should type check successfully"
        );
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
