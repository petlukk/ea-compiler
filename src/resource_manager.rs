//! Resource management system for the Eä compiler
//!
//! This module provides comprehensive resource management including memory limits,
//! compilation timeouts, and graceful degradation strategies for large programs.

use crate::{
    error::{CompileError, Result},
    memory_profiler::get_current_memory_usage,
};
use std::time::{Duration, Instant};

/// Resource limits configuration
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Maximum memory usage in bytes
    pub max_memory: usize,
    /// Maximum compilation time in seconds
    pub max_compilation_time: Duration,
    /// Maximum number of tokens to process
    pub max_tokens: usize,
    /// Maximum number of statements to process
    pub max_statements: usize,
    /// Maximum nesting depth for expressions
    pub max_nesting_depth: usize,
    /// Whether to enable graceful degradation
    pub enable_graceful_degradation: bool,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory: 1024 * 1024 * 1024,                 // 1GB
            max_compilation_time: Duration::from_secs(300), // 5 minutes
            max_tokens: 1_000_000,                          // 1M tokens
            max_statements: 100_000,                        // 100K statements
            max_nesting_depth: 100,                         // 100 levels
            enable_graceful_degradation: true,
        }
    }
}

/// Resource usage statistics
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub memory_used: usize,
    pub compilation_time_elapsed: Duration,
    pub tokens_processed: usize,
    pub statements_processed: usize,
    pub max_nesting_depth_reached: usize,
}

impl ResourceUsage {
    pub fn new() -> Self {
        Self {
            memory_used: 0,
            compilation_time_elapsed: Duration::new(0, 0),
            tokens_processed: 0,
            statements_processed: 0,
            max_nesting_depth_reached: 0,
        }
    }
}

/// Graceful degradation strategies
#[derive(Debug, Clone, PartialEq)]
pub enum DegradationStrategy {
    /// Skip complex optimizations
    SkipOptimizations,
    /// Use simpler parsing strategies
    SimplifiedParsing,
    /// Skip non-essential type checking
    ReducedTypeChecking,
    /// Use streaming compilation
    ForceStreaming,
    /// Limit AST depth
    LimitASTDepth,
    /// Abort compilation with helpful error
    AbortWithSuggestions,
}

/// Resource manager that monitors and enforces limits
pub struct ResourceManager {
    limits: ResourceLimits,
    usage: ResourceUsage,
    start_time: Instant,
    degradation_strategies: Vec<DegradationStrategy>,
}

impl ResourceManager {
    /// Create a new resource manager with default limits
    pub fn new() -> Self {
        Self::with_limits(ResourceLimits::default())
    }

    /// Create a new resource manager with custom limits
    pub fn with_limits(limits: ResourceLimits) -> Self {
        Self {
            limits,
            usage: ResourceUsage::new(),
            start_time: Instant::now(),
            degradation_strategies: Vec::new(),
        }
    }

    /// Check if any resource limits have been exceeded
    pub fn check_limits(&mut self) -> Result<()> {
        // Update current usage
        self.update_usage();

        // Check memory limit
        if self.usage.memory_used > self.limits.max_memory {
            if self.limits.enable_graceful_degradation {
                return self.apply_degradation_strategy(DegradationStrategy::ForceStreaming);
            } else {
                return Err(CompileError::MemoryExhausted {
                    phase: "resource management".to_string(),
                    details: format!(
                        "Memory usage {} bytes exceeds limit {} bytes",
                        self.usage.memory_used, self.limits.max_memory
                    ),
                });
            }
        }

        // Check time limit
        if self.usage.compilation_time_elapsed > self.limits.max_compilation_time {
            if self.limits.enable_graceful_degradation {
                return self.apply_degradation_strategy(DegradationStrategy::SkipOptimizations);
            } else {
                return Err(CompileError::CodeGenError {
                    message: format!(
                        "Compilation time {:?} exceeds limit {:?}",
                        self.usage.compilation_time_elapsed, self.limits.max_compilation_time
                    ),
                    position: None,
                });
            }
        }

        // Check token limit
        if self.usage.tokens_processed > self.limits.max_tokens {
            if self.limits.enable_graceful_degradation {
                return self.apply_degradation_strategy(DegradationStrategy::SimplifiedParsing);
            } else {
                return Err(CompileError::ParseError {
                    message: format!(
                        "Token count {} exceeds limit {}",
                        self.usage.tokens_processed, self.limits.max_tokens
                    ),
                    position: crate::lexer::Position::new(0, 0, 0),
                });
            }
        }

        // Check statement limit
        if self.usage.statements_processed > self.limits.max_statements {
            if self.limits.enable_graceful_degradation {
                return self.apply_degradation_strategy(DegradationStrategy::ReducedTypeChecking);
            } else {
                return Err(CompileError::ParseError {
                    message: format!(
                        "Statement count {} exceeds limit {}",
                        self.usage.statements_processed, self.limits.max_statements
                    ),
                    position: crate::lexer::Position::new(0, 0, 0),
                });
            }
        }

        // Check nesting depth
        if self.usage.max_nesting_depth_reached > self.limits.max_nesting_depth {
            if self.limits.enable_graceful_degradation {
                return self.apply_degradation_strategy(DegradationStrategy::LimitASTDepth);
            } else {
                return Err(CompileError::ParseError {
                    message: format!(
                        "Nesting depth {} exceeds limit {}",
                        self.usage.max_nesting_depth_reached, self.limits.max_nesting_depth
                    ),
                    position: crate::lexer::Position::new(0, 0, 0),
                });
            }
        }

        Ok(())
    }

    /// Update current resource usage
    fn update_usage(&mut self) {
        self.usage.memory_used = get_current_memory_usage();
        self.usage.compilation_time_elapsed = self.start_time.elapsed();
    }

    /// Record token processing
    pub fn record_tokens(&mut self, count: usize) {
        self.usage.tokens_processed += count;
    }

    /// Record statement processing
    pub fn record_statements(&mut self, count: usize) {
        self.usage.statements_processed += count;
    }

    /// Record nesting depth
    pub fn record_nesting_depth(&mut self, depth: usize) {
        if depth > self.usage.max_nesting_depth_reached {
            self.usage.max_nesting_depth_reached = depth;
        }
    }

    /// Apply a degradation strategy
    fn apply_degradation_strategy(&mut self, strategy: DegradationStrategy) -> Result<()> {
        if !self.degradation_strategies.contains(&strategy) {
            self.degradation_strategies.push(strategy.clone());
        }

        match strategy {
            DegradationStrategy::SkipOptimizations => {
                eprintln!("⚠️  Warning: Skipping optimizations due to time constraints");
                Ok(())
            }
            DegradationStrategy::SimplifiedParsing => {
                eprintln!("⚠️  Warning: Using simplified parsing due to token count");
                Ok(())
            }
            DegradationStrategy::ReducedTypeChecking => {
                eprintln!("⚠️  Warning: Reducing type checking scope due to statement count");
                Ok(())
            }
            DegradationStrategy::ForceStreaming => {
                eprintln!("⚠️  Warning: Forcing streaming compilation due to memory usage");
                Ok(())
            }
            DegradationStrategy::LimitASTDepth => {
                eprintln!("⚠️  Warning: Limiting AST depth due to nesting complexity");
                Ok(())
            }
            DegradationStrategy::AbortWithSuggestions => Err(CompileError::CodeGenError {
                message: "Compilation aborted due to resource limits. Consider:\n\
                             - Breaking code into smaller modules\n\
                             - Reducing nesting depth\n\
                             - Using streaming compilation (--streaming)\n\
                             - Increasing memory limits"
                    .to_string(),
                position: None,
            }),
        }
    }

    /// Get current resource usage
    pub fn get_usage(&self) -> &ResourceUsage {
        &self.usage
    }

    /// Get applied degradation strategies
    pub fn get_degradation_strategies(&self) -> &[DegradationStrategy] {
        &self.degradation_strategies
    }

    /// Check if a specific degradation strategy is active
    pub fn has_degradation_strategy(&self, strategy: &DegradationStrategy) -> bool {
        self.degradation_strategies.contains(strategy)
    }

    /// Generate a resource usage report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();

        report.push_str("=== Resource Usage Report ===\n");
        report.push_str(&format!(
            "Memory: {:.2} MB / {:.2} MB\n",
            self.usage.memory_used as f64 / (1024.0 * 1024.0),
            self.limits.max_memory as f64 / (1024.0 * 1024.0)
        ));
        report.push_str(&format!(
            "Time: {:.2}s / {:.2}s\n",
            self.usage.compilation_time_elapsed.as_secs_f64(),
            self.limits.max_compilation_time.as_secs_f64()
        ));
        report.push_str(&format!(
            "Tokens: {} / {}\n",
            self.usage.tokens_processed, self.limits.max_tokens
        ));
        report.push_str(&format!(
            "Statements: {} / {}\n",
            self.usage.statements_processed, self.limits.max_statements
        ));
        report.push_str(&format!(
            "Max Nesting: {} / {}\n",
            self.usage.max_nesting_depth_reached, self.limits.max_nesting_depth
        ));

        if !self.degradation_strategies.is_empty() {
            report.push_str("\nDegradation Strategies Applied:\n");
            for strategy in &self.degradation_strategies {
                report.push_str(&format!("  - {:?}\n", strategy));
            }
        }

        report
    }

    /// Reset usage statistics
    pub fn reset(&mut self) {
        self.usage = ResourceUsage::new();
        self.start_time = Instant::now();
        self.degradation_strategies.clear();
    }
}

/// Global resource manager instance
static mut GLOBAL_RESOURCE_MANAGER: Option<ResourceManager> = None;

/// Initialize the global resource manager
pub fn initialize_resource_manager(limits: ResourceLimits) {
    unsafe {
        GLOBAL_RESOURCE_MANAGER = Some(ResourceManager::with_limits(limits));
    }
}

/// Get the global resource manager
pub fn get_resource_manager() -> Option<&'static mut ResourceManager> {
    unsafe { GLOBAL_RESOURCE_MANAGER.as_mut() }
}

/// Check resource limits using the global manager
pub fn check_resource_limits() -> Result<()> {
    if let Some(manager) = get_resource_manager() {
        manager.check_limits()
    } else {
        Ok(())
    }
}

/// Record token processing using the global manager
pub fn record_token_processing(count: usize) {
    if let Some(manager) = get_resource_manager() {
        manager.record_tokens(count);
    }
}

/// Record statement processing using the global manager
pub fn record_statement_processing(count: usize) {
    if let Some(manager) = get_resource_manager() {
        manager.record_statements(count);
    }
}

/// Record nesting depth using the global manager
pub fn record_nesting_depth(depth: usize) {
    if let Some(manager) = get_resource_manager() {
        manager.record_nesting_depth(depth);
    }
}

/// Generate resource usage report using the global manager
pub fn generate_resource_report() -> String {
    if let Some(manager) = get_resource_manager() {
        manager.generate_report()
    } else {
        "Resource manager not initialized".to_string()
    }
}
