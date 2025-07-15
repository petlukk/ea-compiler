//! Parser performance optimization utilities
//!
//! This module provides optimizations to improve parser performance for complex programs,
//! including memoization, expression caching, and recursive call optimization.

use crate::{
    ast::{Expr, Stmt},
    error::{CompileError, Result},
    lexer::Token,
    resource_manager::{check_resource_limits, record_nesting_depth},
};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::Instant;

/// Cache key for memoizing parsing results
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ParseCacheKey {
    pub token_range: (usize, usize), // Start and end token indices
    pub parse_type: ParseType,
}

/// Types of parsing operations that can be cached
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ParseType {
    Expression,
    Statement,
    Assignment,
    LogicalOr,
    LogicalAnd,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
}

/// Cached parse result
#[derive(Debug, Clone)]
pub enum ParseResult {
    Expression(Expr),
    Statement(Stmt),
    Error(String),
}

/// Parser performance optimizer with caching and metrics
pub struct ParserOptimizer {
    /// Cache for parsed expressions and statements
    cache: HashMap<ParseCacheKey, ParseResult>,
    /// Performance metrics
    cache_hits: usize,
    cache_misses: usize,
    total_parse_time: std::time::Duration,
    max_recursion_depth: usize,
    current_recursion_depth: usize,
    /// Configuration
    max_cache_size: usize,
    max_recursion_limit: usize,
    enable_caching: bool,
}

impl Default for ParserOptimizer {
    fn default() -> Self {
        Self {
            cache: HashMap::new(),
            cache_hits: 0,
            cache_misses: 0,
            total_parse_time: std::time::Duration::new(0, 0),
            max_recursion_depth: 0,
            current_recursion_depth: 0,
            max_cache_size: 10000,     // Cache up to 10k parse results
            max_recursion_limit: 1000, // Prevent stack overflow
            enable_caching: true,
        }
    }
}

impl ParserOptimizer {
    /// Create a new parser optimizer
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a parser optimizer with custom configuration
    pub fn with_config(max_cache_size: usize, max_recursion_limit: usize) -> Self {
        Self {
            max_cache_size,
            max_recursion_limit,
            ..Self::default()
        }
    }

    /// Try to get a cached parse result
    pub fn get_cached(&mut self, key: &ParseCacheKey) -> Option<&ParseResult> {
        if !self.enable_caching {
            return None;
        }

        if let Some(result) = self.cache.get(key) {
            self.cache_hits += 1;
            Some(result)
        } else {
            self.cache_misses += 1;
            None
        }
    }

    /// Cache a parse result
    pub fn cache_result(&mut self, key: ParseCacheKey, result: ParseResult) {
        if !self.enable_caching {
            return;
        }

        // Prevent cache from growing too large
        if self.cache.len() >= self.max_cache_size {
            // Remove oldest entries (simple LRU approximation)
            let keys_to_remove: Vec<_> = self
                .cache
                .keys()
                .take(self.max_cache_size / 4)
                .cloned()
                .collect();
            for key in keys_to_remove {
                self.cache.remove(&key);
            }
        }

        self.cache.insert(key, result);
    }

    /// Enter a recursive parsing context
    pub fn enter_recursion(&mut self) -> Result<()> {
        self.current_recursion_depth += 1;

        if self.current_recursion_depth > self.max_recursion_depth {
            self.max_recursion_depth = self.current_recursion_depth;
        }

        // Record nesting depth for resource management
        record_nesting_depth(self.current_recursion_depth);

        // Check recursion limit
        if self.current_recursion_depth > self.max_recursion_limit {
            return Err(CompileError::ParseError {
                message: format!(
                    "Maximum recursion depth {} exceeded. Consider simplifying your expressions.",
                    self.max_recursion_limit
                ),
                position: crate::lexer::Position::new(0, 0, 0),
            });
        }

        // Check resource limits
        check_resource_limits()?;

        Ok(())
    }

    /// Exit a recursive parsing context
    pub fn exit_recursion(&mut self) {
        if self.current_recursion_depth > 0 {
            self.current_recursion_depth -= 1;
        }
    }

    /// Measure parse time for a parsing operation
    pub fn time_parse<F, T>(&mut self, operation: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        let start = Instant::now();
        let result = operation();
        let elapsed = start.elapsed();
        self.total_parse_time += elapsed;
        result
    }

    /// Get cache hit ratio
    pub fn get_cache_hit_ratio(&self) -> f64 {
        let total_requests = self.cache_hits + self.cache_misses;
        if total_requests == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total_requests as f64
        }
    }

    /// Generate performance report
    pub fn generate_performance_report(&self) -> String {
        let mut report = String::new();

        report.push_str("=== Parser Performance Report ===\n");
        report.push_str(&format!("Cache Hits: {}\n", self.cache_hits));
        report.push_str(&format!("Cache Misses: {}\n", self.cache_misses));
        report.push_str(&format!(
            "Cache Hit Ratio: {:.2}%\n",
            self.get_cache_hit_ratio() * 100.0
        ));
        report.push_str(&format!("Cache Size: {} entries\n", self.cache.len()));
        report.push_str(&format!(
            "Total Parse Time: {:.2}ms\n",
            self.total_parse_time.as_secs_f64() * 1000.0
        ));
        report.push_str(&format!(
            "Max Recursion Depth: {}\n",
            self.max_recursion_depth
        ));
        report.push_str(&format!("Recursion Limit: {}\n", self.max_recursion_limit));

        if self.max_recursion_depth > self.max_recursion_limit / 2 {
            report.push_str(
                "\n⚠️  Warning: High recursion depth detected. Consider simplifying expressions.\n",
            );
        }

        if self.get_cache_hit_ratio() < 0.1 {
            report
                .push_str("\n💡 Tip: Low cache hit ratio. Consider enabling expression caching.\n");
        }

        report
    }

    /// Clear the cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
        self.cache_hits = 0;
        self.cache_misses = 0;
    }

    /// Disable caching (useful for debugging)
    pub fn disable_caching(&mut self) {
        self.enable_caching = false;
        self.cache.clear();
    }

    /// Enable caching
    pub fn enable_caching(&mut self) {
        self.enable_caching = true;
    }

    /// Reset all metrics
    pub fn reset_metrics(&mut self) {
        self.cache_hits = 0;
        self.cache_misses = 0;
        self.total_parse_time = std::time::Duration::new(0, 0);
        self.max_recursion_depth = 0;
        self.current_recursion_depth = 0;
        self.cache.clear();
    }
}

/// Global parser optimizer instance
static GLOBAL_PARSER_OPTIMIZER: LazyLock<Mutex<Option<ParserOptimizer>>> = LazyLock::new(|| Mutex::new(None));

/// Initialize the global parser optimizer
pub fn initialize_parser_optimizer() {
    if let Ok(mut optimizer) = GLOBAL_PARSER_OPTIMIZER.lock() {
        *optimizer = Some(ParserOptimizer::new());
    }
}

/// Execute a function with the global parser optimizer
pub fn with_parser_optimizer<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut ParserOptimizer) -> R,
{
    if let Ok(mut guard) = GLOBAL_PARSER_OPTIMIZER.lock() {
        if let Some(ref mut optimizer) = *guard {
            Some(f(optimizer))
        } else {
            None
        }
    } else {
        None
    }
}

/// Record entering a recursive parsing context
pub fn enter_parse_recursion() -> Result<()> {
    with_parser_optimizer(|optimizer| optimizer.enter_recursion()).unwrap_or(Ok(()))
}

/// Record exiting a recursive parsing context
pub fn exit_parse_recursion() {
    with_parser_optimizer(|optimizer| optimizer.exit_recursion());
}

/// Time a parsing operation
pub fn time_parsing_operation<F, T>(operation: F) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    // Check if optimizer is available
    if let Ok(mut guard) = GLOBAL_PARSER_OPTIMIZER.lock() {
        if let Some(ref mut optimizer) = *guard {
            // Use the timing function directly
            return optimizer.time_parse(operation);
        }
    }
    
    // Optimizer not available, just run the operation
    operation()
}

/// Generate parser performance report
pub fn generate_parser_performance_report() -> String {
    with_parser_optimizer(|optimizer| optimizer.generate_performance_report())
        .unwrap_or_else(|| "Parser optimizer not initialized".to_string())
}

/// Reset parser performance metrics
pub fn reset_parser_metrics() {
    with_parser_optimizer(|optimizer| {
        optimizer.reset_metrics();
    });
}

/// Expression complexity analyzer
pub struct ExpressionComplexityAnalyzer;

impl ExpressionComplexityAnalyzer {
    /// Analyze expression complexity and suggest optimizations
    pub fn analyze_complexity(tokens: &[Token]) -> ComplexityAnalysis {
        let mut analysis = ComplexityAnalysis::new();

        let mut depth: i32 = 0;
        let mut max_depth: usize = 0;
        let mut operator_count = 0;
        let mut identifier_count = 0;

        for token in tokens {
            match &token.kind {
                crate::lexer::TokenKind::LeftParen => {
                    depth += 1;
                    if depth as usize > max_depth {
                        max_depth = depth as usize;
                    }
                }
                crate::lexer::TokenKind::RightParen => {
                    depth = depth.saturating_sub(1);
                }
                crate::lexer::TokenKind::Plus
                | crate::lexer::TokenKind::Minus
                | crate::lexer::TokenKind::Star
                | crate::lexer::TokenKind::Slash => {
                    operator_count += 1;
                }
                crate::lexer::TokenKind::Identifier(_) => {
                    identifier_count += 1;
                }
                _ => {}
            }
        }

        analysis.max_nesting_depth = max_depth;
        analysis.operator_count = operator_count;
        analysis.identifier_count = identifier_count;
        analysis.token_count = tokens.len();
        analysis.calculate_complexity_score();

        analysis
    }
}

/// Expression complexity analysis result
#[derive(Debug, Clone)]
pub struct ComplexityAnalysis {
    pub max_nesting_depth: usize,
    pub operator_count: usize,
    pub identifier_count: usize,
    pub token_count: usize,
    pub complexity_score: f64,
    pub suggestions: Vec<String>,
}

impl ComplexityAnalysis {
    pub fn new() -> Self {
        Self {
            max_nesting_depth: 0,
            operator_count: 0,
            identifier_count: 0,
            token_count: 0,
            complexity_score: 0.0,
            suggestions: Vec::new(),
        }
    }

    /// Calculate a complexity score based on various factors
    pub fn calculate_complexity_score(&mut self) {
        // Simple complexity scoring algorithm
        let depth_weight = 2.0;
        let operator_weight = 1.0;
        let size_weight = 0.1;

        self.complexity_score = (self.max_nesting_depth as f64 * depth_weight)
            + (self.operator_count as f64 * operator_weight)
            + (self.token_count as f64 * size_weight);

        // Generate suggestions based on complexity
        if self.max_nesting_depth > 10 {
            self.suggestions.push(
                "Consider breaking down deeply nested expressions into multiple statements"
                    .to_string(),
            );
        }

        if self.operator_count > 20 {
            self.suggestions.push(
                "Consider using intermediate variables to simplify complex expressions".to_string(),
            );
        }

        if self.token_count > 100 {
            self.suggestions.push(
                "Large expression detected. Consider using functions or breaking into smaller parts".to_string()
            );
        }
    }

    /// Check if the expression is considered complex
    pub fn is_complex(&self) -> bool {
        self.complexity_score > 50.0 || self.max_nesting_depth > 15
    }
}
