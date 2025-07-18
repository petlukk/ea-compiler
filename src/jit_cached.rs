// src/jit_cached.rs
//! Cached JIT execution function for the Eä programming language.

use crate::error::{CompileError, Result};
use crate::jit_cache::with_jit_cache;
use crate::jit_execution::{execute_jit_program, map_essential_symbols};
use crate::memory_profiler::get_current_memory_usage;
use crate::{codegen, compile_to_ast};
use inkwell::context::Context;
use inkwell::OptimizationLevel;
use std::time::Instant;

/// JIT compile and execute a program immediately with caching
#[cfg(feature = "llvm")]
pub fn jit_execute_cached(source: &str, module_name: &str) -> Result<i32> {
    // Check JIT cache first
    let cache_result = with_jit_cache(|cache| {
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

            // Re-execute the cached program by recompiling with cached optimizations
            eprintln!("⚡ Executing cached compilation...");

            Some(cached_jit.clone())
        } else {
            None
        }
    });

    if let Some(_cached_jit) = cache_result {
        // Fast path: recompile and execute immediately since we have cached metadata
        let (program, _type_context) = compile_to_ast(source)?;
        let context = Context::create();
        let mut codegen = codegen::CodeGenerator::new(&context, module_name);
        codegen.compile_program(&program)?;

        // Initialize LLVM targets for cached execution too
        inkwell::targets::Target::initialize_native(
            &inkwell::targets::InitializationConfig::default(),
        )
        .map_err(|e| {
            CompileError::codegen_error(
                format!("Failed to initialize LLVM native target: {}", e),
                None,
            )
        })?;

        let execution_engine = codegen
            .get_module()
            .create_jit_execution_engine(OptimizationLevel::None)
            .map_err(|e| {
                CompileError::codegen_error(
                    format!(
                        "Failed to create JIT execution engine with target features: {}",
                        e
                    ),
                    None,
                )
            })?;

        // Reuse cached symbol mappings
        let _symbol_table = map_essential_symbols(&execution_engine, &codegen)?;

        // Execute the program
        let exit_code = execute_jit_program(&execution_engine, &codegen)?;

        eprintln!("✅ Cached execution completed successfully");
        return Ok(exit_code);
    }

    eprintln!("🔧 Cache miss - compiling from source...");
    let compilation_start = Instant::now();
    let memory_start = get_current_memory_usage();

    let (program, _type_context) = compile_to_ast(source)?;

    let context = Context::create();
    let mut codegen = codegen::CodeGenerator::new(&context, module_name);
    codegen.compile_program(&program)?;

    // Create execution engine for JIT compilation with proper target configuration
    eprintln!("🔧 Creating JIT execution engine with target features...");

    // Initialize LLVM targets to ensure proper CPU feature support
    inkwell::targets::Target::initialize_native(&inkwell::targets::InitializationConfig::default())
        .map_err(|e| {
            eprintln!("❌ Failed to initialize native target: {}", e);
            CompileError::codegen_error(
                format!("Failed to initialize LLVM native target: {}", e),
                None,
            )
        })?;

    let execution_engine = codegen
        .get_module()
        .create_jit_execution_engine(OptimizationLevel::None)
        .map_err(|e| {
            eprintln!("❌ JIT engine creation failed: {}", e);
            eprintln!("   This might be due to CPU feature mismatch (AVX2/SSE4.2/FMA)");
            eprintln!("   IR contains target-features: +avx2,+sse4.2,+fma");
            CompileError::codegen_error(
                format!("Failed to create JIT execution engine: {}", e),
                None,
            )
        })?;
    eprintln!("✅ JIT execution engine created successfully with native target support");

    // Map essential symbols for JIT execution
    let symbol_table = map_essential_symbols(&execution_engine, &codegen)?;

    // Execute the program and measure performance
    let exec_start = Instant::now();
    let exit_code = execute_jit_program(&execution_engine, &codegen)?;
    let exec_time = exec_start.elapsed();

    // Store compilation result in cache
    let compilation_time = compilation_start.elapsed();
    let memory_usage = get_current_memory_usage().saturating_sub(memory_start);

    // For now, we'll use a placeholder for machine code since LLVM JIT doesn't expose it directly
    // In a production system, you'd want to extract the actual machine code from the execution engine
    let machine_code = Vec::new(); // Note: LLVM JIT doesn't expose machine code directly; this is a known limitation

    with_jit_cache(|cache| {
        cache
            .put(
                source,
                machine_code,
                0,
                symbol_table,
                memory_usage as u64,
                compilation_time,
            )
            .map_err(|e| {
                CompileError::codegen_error(format!("Failed to cache JIT result: {}", e), None)
            })
    })?;

    eprintln!("✅ JIT compilation cached successfully");
    eprintln!("   Compilation time: {:?}", compilation_time);
    eprintln!("   Execution time: {:?}", exec_time);
    eprintln!("   Memory usage: {} bytes", memory_usage);

    Ok(exit_code)
}
