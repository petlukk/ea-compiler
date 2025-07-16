// src/llvm_optimization.rs
//! LLVM IR optimization system for the Eä programming language.
//!
//! This module provides comprehensive optimization of LLVM IR generation,
//! including passes for performance improvements, code size reduction,
//! and compilation speed optimization.

use crate::error::{CompileError, Result};
use inkwell::module::Module;
use inkwell::passes::{PassManager, PassManagerBuilder};
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::OptimizationLevel;
use std::sync::{LazyLock, Mutex};
use std::time::Instant;

/// LLVM optimization configuration
#[derive(Debug, Clone)]
pub struct LLVMOptimizationConfig {
    /// Optimization level (None, Less, Default, Aggressive)
    pub optimization_level: OptimizationLevel,
    /// Enable function inlining
    pub enable_inlining: bool,
    /// Enable loop optimization
    pub enable_loop_optimization: bool,
    /// Enable vectorization
    pub enable_vectorization: bool,
    /// Enable dead code elimination
    pub enable_dead_code_elimination: bool,
    /// Enable constant propagation
    pub enable_constant_propagation: bool,
    /// Enable tail call optimization
    pub enable_tail_call_optimization: bool,
    /// Enable SIMD optimization
    pub enable_simd_optimization: bool,
    /// Target CPU architecture
    pub target_cpu: String,
    /// Target features (e.g., "+avx2", "+sse4.1")
    pub target_features: String,
}

impl Default for LLVMOptimizationConfig {
    fn default() -> Self {
        Self {
            optimization_level: OptimizationLevel::Default,
            enable_inlining: true,
            enable_loop_optimization: true,
            enable_vectorization: true,
            enable_dead_code_elimination: true,
            enable_constant_propagation: true,
            enable_tail_call_optimization: true,
            enable_simd_optimization: true,
            target_cpu: "x86-64".to_string(),
            target_features: "+avx2,+sse4.1".to_string(),
        }
    }
}

/// LLVM optimization statistics
#[derive(Debug, Default, Clone)]
pub struct LLVMOptimizationStats {
    /// Number of functions optimized
    pub functions_optimized: u64,
    /// Number of instructions before optimization
    pub instructions_before: u64,
    /// Number of instructions after optimization
    pub instructions_after: u64,
    /// Time spent in optimization passes
    pub optimization_time: std::time::Duration,
    /// Number of optimization passes run
    pub passes_run: u64,
    /// Memory usage during optimization
    pub memory_usage: u64,
}

impl LLVMOptimizationStats {
    /// Calculate instruction reduction percentage
    pub fn instruction_reduction(&self) -> f64 {
        if self.instructions_before == 0 {
            0.0
        } else {
            let reduction = self
                .instructions_before
                .saturating_sub(self.instructions_after);
            (reduction as f64 / self.instructions_before as f64) * 100.0
        }
    }
}

/// LLVM optimization engine
pub struct LLVMOptimizer {
    config: LLVMOptimizationConfig,
    stats: LLVMOptimizationStats,
}

impl LLVMOptimizer {
    /// Create a new LLVM optimizer with default configuration
    pub fn new() -> Self {
        Self::with_config(LLVMOptimizationConfig::default())
    }

    /// Create a new LLVM optimizer with custom configuration
    pub fn with_config(config: LLVMOptimizationConfig) -> Self {
        Self {
            config,
            stats: LLVMOptimizationStats::default(),
        }
    }

    /// Optimize an LLVM module
    pub fn optimize_module(&mut self, module: &Module) -> Result<()> {
        let start_time = Instant::now();

        eprintln!("🔧 Starting LLVM optimization...");
        eprintln!(
            "   Optimization level: {:?}",
            self.config.optimization_level
        );
        eprintln!("   Target CPU: {}", self.config.target_cpu);
        eprintln!("   Target features: {}", self.config.target_features);

        // Skip optimization if using None level (for emit-llvm mode)
        if matches!(self.config.optimization_level, OptimizationLevel::None) {
            eprintln!("⚠️  Skipping optimization passes (optimization level: None)");
            self.stats.optimization_time = start_time.elapsed();
            return Ok(());
        }

        // Count instructions before optimization (with safety check)
        eprintln!("🔍 About to count instructions before optimization...");
        self.stats.instructions_before =
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                eprintln!("🔍 Inside count_instructions...");
                self.count_instructions(module)
            })) {
                Ok(count) => {
                    eprintln!("✅ Successfully counted {} instructions", count);
                    count
                }
                Err(_) => {
                    eprintln!("⚠️  Warning: Could not count instructions before optimization");
                    0
                }
            };

        // Create pass manager (with safety check)
        eprintln!("🔍 About to create PassManagerBuilder...");
        let pass_manager_builder = PassManagerBuilder::create();
        eprintln!("✅ PassManagerBuilder created");

        eprintln!("🔍 About to set optimization level...");
        pass_manager_builder.set_optimization_level(self.config.optimization_level);
        eprintln!("✅ Optimization level set");

        if self.config.enable_inlining {
            eprintln!("🔍 About to set inliner...");
            pass_manager_builder.set_inliner_with_threshold(275);
            eprintln!("✅ Inliner set");
        }

        // Loop optimization is handled by individual passes

        // Create function pass manager
        eprintln!("🔍 About to create function pass manager...");
        let function_pass_manager = PassManager::create(module);
        eprintln!("✅ Function pass manager created");

        // Add standard optimization passes
        function_pass_manager.add_instruction_combining_pass();
        function_pass_manager.add_cfg_simplification_pass();
        function_pass_manager.add_dead_store_elimination_pass();
        
        if self.config.enable_constant_propagation {
            function_pass_manager.add_constant_merge_pass();
        }
        
        self.stats.passes_run += 3;
        if self.config.enable_constant_propagation {
            self.stats.passes_run += 1;
        }

        // Initialize and run function passes (with safety checks)
        eprintln!("🔍 About to initialize function pass manager...");
        function_pass_manager.initialize();
        eprintln!("✅ Function pass manager initialized");

        // Run passes on all functions - simplified approach
        eprintln!(
            "🔍 About to run passes on {} functions...",
            module.get_functions().count()
        );

        for function in module.get_functions() {
            // Skip external functions (declarations only)
            if function.count_basic_blocks() == 0 {
                continue;
            }

            let function_name = function.get_name().to_string_lossy();
            eprintln!("🔍 Running passes on function: {}", function_name);

            // Run optimization passes with proper error handling
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                function_pass_manager.run_on(&function);
            })) {
                Ok(_) => {
                    self.stats.functions_optimized += 1;
                    eprintln!("✅ Successfully optimized function: {}", function_name);
                }
                Err(_) => {
                    eprintln!(
                        "⚠️  Optimization failed for function {} - continuing with next function",
                        function_name
                    );
                }
            }
        }

        eprintln!("🔍 About to finalize function pass manager...");
        function_pass_manager.finalize();
        eprintln!("✅ Function pass manager finalized");

        // Count instructions after optimization
        self.stats.instructions_after =
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                self.count_instructions(module)
            })) {
                Ok(count) => count,
                Err(_) => {
                    eprintln!("⚠️  Warning: Could not count instructions after optimization");
                    0
                }
            };

        // Record optimization time
        self.stats.optimization_time = start_time.elapsed();

        eprintln!("✅ LLVM optimization completed");
        eprintln!("   Functions optimized: {}", self.stats.functions_optimized);
        eprintln!("   Instructions before: {}", self.stats.instructions_before);
        eprintln!("   Instructions after: {}", self.stats.instructions_after);
        eprintln!(
            "   Instruction reduction: {:.1}%",
            self.stats.instruction_reduction()
        );
        eprintln!("   Optimization time: {:?}", self.stats.optimization_time);
        eprintln!("   Passes run: {}", self.stats.passes_run);

        Ok(())
    }

    /// Count instructions in a module
    fn count_instructions(&self, module: &Module) -> u64 {
        let mut count = 0;
        for function in module.get_functions() {
            // Skip if function is declaration only (no body)
            if function.count_basic_blocks() == 0 {
                continue;
            }

            for basic_block in function.get_basic_blocks() {
                count += basic_block.get_instructions().count() as u64;
            }
        }
        count
    }

    /// Create optimized target machine
    pub fn create_target_machine(&self) -> Result<TargetMachine> {
        // Initialize targets
        Target::initialize_native(&InitializationConfig::default()).map_err(|e| {
            CompileError::codegen_error(format!("Failed to initialize native target: {}", e), None)
        })?;

        let triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&triple).map_err(|e| {
            CompileError::codegen_error(format!("Failed to create target from triple: {}", e), None)
        })?;

        let machine = target
            .create_target_machine(
                &triple,
                &self.config.target_cpu,
                &self.config.target_features,
                self.config.optimization_level,
                RelocMode::Default,
                CodeModel::Default,
            )
            .ok_or_else(|| {
                CompileError::codegen_error("Failed to create target machine".to_string(), None)
            })?;

        Ok(machine)
    }

    /// Optimize and compile module to object file
    pub fn optimize_and_compile(&mut self, module: &Module, output_path: &str) -> Result<()> {
        eprintln!("🚀 Starting optimized compilation...");

        // First optimize the module
        self.optimize_module(module)?;

        // Create target machine
        let target_machine = self.create_target_machine()?;

        // Compile to object file
        let start_time = Instant::now();
        target_machine
            .write_to_file(module, FileType::Object, std::path::Path::new(output_path))
            .map_err(|e| {
                CompileError::codegen_error(format!("Failed to write object file: {}", e), None)
            })?;

        let compile_time = start_time.elapsed();
        eprintln!("✅ Optimized compilation completed in {:?}", compile_time);

        Ok(())
    }

    /// Get optimization statistics
    pub fn get_stats(&self) -> &LLVMOptimizationStats {
        &self.stats
    }

    /// Reset optimization statistics
    pub fn reset_stats(&mut self) {
        self.stats = LLVMOptimizationStats::default();
    }
}

/// Global LLVM optimizer instance
static GLOBAL_LLVM_OPTIMIZER: LazyLock<Mutex<Option<LLVMOptimizer>>> = LazyLock::new(|| Mutex::new(None));
/// Initialize the global LLVM optimizer
pub fn initialize_llvm_optimizer(config: LLVMOptimizationConfig) {
    let mut optimizer = GLOBAL_LLVM_OPTIMIZER.lock().unwrap();
    if optimizer.is_none() {
        *optimizer = Some(LLVMOptimizer::with_config(config));
    }
}

/// Run operation with the global LLVM optimizer
pub fn with_llvm_optimizer<T>(f: impl FnOnce(&mut LLVMOptimizer) -> T) -> T {
    let mut optimizer = GLOBAL_LLVM_OPTIMIZER.lock().unwrap();
    let optimizer_ref = optimizer.as_mut().expect("LLVM optimizer not initialized");
    f(optimizer_ref)
}

/// Initialize LLVM optimizer with default configuration
pub fn initialize_default_llvm_optimizer() {
    initialize_llvm_optimizer(LLVMOptimizationConfig::default());
}

/// Apply fast optimization preset for development
pub fn apply_fast_optimization_preset() -> LLVMOptimizationConfig {
    LLVMOptimizationConfig {
        optimization_level: OptimizationLevel::Less,
        enable_inlining: true,
        enable_loop_optimization: false,
        enable_vectorization: false,
        enable_dead_code_elimination: true,
        enable_constant_propagation: true,
        enable_tail_call_optimization: false,
        enable_simd_optimization: false,
        target_cpu: "x86-64".to_string(),
        target_features: "+sse4.1".to_string(),
    }
}

/// Apply emit-llvm preset (no optimization for safe IR emission)
pub fn apply_emit_llvm_preset() -> LLVMOptimizationConfig {
    LLVMOptimizationConfig {
        optimization_level: OptimizationLevel::None,
        enable_inlining: false,
        enable_loop_optimization: false,
        enable_vectorization: false,
        enable_dead_code_elimination: false,
        enable_constant_propagation: false,
        enable_tail_call_optimization: false,
        enable_simd_optimization: false,
        target_cpu: "x86-64".to_string(),
        target_features: "+sse4.1".to_string(),
    }
}

/// Apply production optimization preset
pub fn apply_production_optimization_preset() -> LLVMOptimizationConfig {
    LLVMOptimizationConfig {
        optimization_level: OptimizationLevel::Aggressive,
        enable_inlining: true,
        enable_loop_optimization: true,
        enable_vectorization: true,
        enable_dead_code_elimination: true,
        enable_constant_propagation: true,
        enable_tail_call_optimization: true,
        enable_simd_optimization: true,
        target_cpu: "x86-64".to_string(),
        target_features: "+avx2,+sse4.1,+bmi2".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llvm_optimization_config() {
        let config = LLVMOptimizationConfig::default();
        assert_eq!(config.optimization_level, OptimizationLevel::Default);
        assert!(config.enable_inlining);
        assert!(config.enable_vectorization);
    }

    #[test]
    fn test_optimization_presets() {
        let fast = apply_fast_optimization_preset();
        assert_eq!(fast.optimization_level, OptimizationLevel::Less);
        assert!(!fast.enable_vectorization);

        let production = apply_production_optimization_preset();
        assert_eq!(production.optimization_level, OptimizationLevel::Aggressive);
        assert!(production.enable_vectorization);
    }

    #[test]
    fn test_llvm_optimizer_creation() {
        let optimizer = LLVMOptimizer::new();
        assert_eq!(
            optimizer.config.optimization_level,
            OptimizationLevel::Default
        );
        assert_eq!(optimizer.stats.functions_optimized, 0);
    }

    #[test]
    fn test_instruction_reduction_calculation() {
        let mut stats = LLVMOptimizationStats::default();
        stats.instructions_before = 100;
        stats.instructions_after = 80;
        assert_eq!(stats.instruction_reduction(), 20.0);
    }
}
