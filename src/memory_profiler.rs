//! Memory profiling infrastructure for Eä compiler
//! 
//! This module provides comprehensive memory tracking and profiling capabilities
//! to identify and fix memory exhaustion issues during compilation.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Global memory profiler instance
static MEMORY_PROFILER: std::sync::LazyLock<Arc<Mutex<MemoryProfiler>>> = 
    std::sync::LazyLock::new(|| Arc::new(Mutex::new(MemoryProfiler::new())));

/// Memory profiler for tracking compilation phase memory usage
#[derive(Debug)]
pub struct MemoryProfiler {
    /// Current memory usage by compilation phase
    phase_usage: HashMap<CompilationPhase, usize>,
    /// Peak memory usage by phase
    peak_usage: HashMap<CompilationPhase, usize>,
    /// Total memory allocations
    total_allocations: usize,
    /// Memory usage timeline
    timeline: Vec<MemorySnapshot>,
    /// Memory limit (in bytes)
    memory_limit: usize,
    /// Start time for current compilation
    start_time: Instant,
}

/// Compilation phases for memory tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompilationPhase {
    Lexing,
    Parsing,
    TypeChecking,
    CodeGeneration,
    Optimization,
    Linking,
    JIT,
    Total,
}

/// Memory snapshot at a specific point in time
#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    pub timestamp: Instant,
    pub phase: CompilationPhase,
    pub memory_usage: usize,
    pub description: String,
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub current_usage: usize,
    pub peak_usage: usize,
    pub total_allocations: usize,
    pub timeline: Vec<MemorySnapshot>,
}

impl MemoryProfiler {
    /// Create a new memory profiler
    pub fn new() -> Self {
        Self {
            phase_usage: HashMap::new(),
            peak_usage: HashMap::new(),
            total_allocations: 0,
            timeline: Vec::new(),
            memory_limit: 1024 * 1024 * 1024, // 1GB default limit
            start_time: Instant::now(),
        }
    }

    /// Set memory limit in bytes
    pub fn set_memory_limit(&mut self, limit: usize) {
        self.memory_limit = limit;
    }

    /// Record memory usage for a specific phase
    pub fn record_usage(&mut self, phase: CompilationPhase, usage: usize, description: String) {
        // Update current usage
        self.phase_usage.insert(phase, usage);
        
        // Update peak usage
        let current_peak = self.peak_usage.get(&phase).unwrap_or(&0);
        if usage > *current_peak {
            self.peak_usage.insert(phase, usage);
        }
        
        // Add to timeline
        self.timeline.push(MemorySnapshot {
            timestamp: Instant::now(),
            phase,
            memory_usage: usage,
            description,
        });
        
        // Update total allocations
        self.total_allocations += usage;
    }

    /// Check if current memory usage exceeds limit
    pub fn check_memory_limit(&self) -> Result<(), MemoryError> {
        let total_usage = self.get_total_usage();
        if total_usage > self.memory_limit {
            Err(MemoryError::LimitExceeded {
                current: total_usage,
                limit: self.memory_limit,
                phase: self.get_current_phase(),
            })
        } else {
            Ok(())
        }
    }

    /// Get total memory usage across all phases
    pub fn get_total_usage(&self) -> usize {
        self.phase_usage.values().sum()
    }

    /// Get current compilation phase (latest recorded)
    pub fn get_current_phase(&self) -> CompilationPhase {
        self.timeline.last()
            .map(|snapshot| snapshot.phase)
            .unwrap_or(CompilationPhase::Total)
    }

    /// Get memory statistics
    pub fn get_stats(&self) -> MemoryStats {
        MemoryStats {
            current_usage: self.get_total_usage(),
            peak_usage: self.peak_usage.values().sum(),
            total_allocations: self.total_allocations,
            timeline: self.timeline.clone(),
        }
    }

    /// Reset profiler for new compilation
    pub fn reset(&mut self) {
        self.phase_usage.clear();
        self.peak_usage.clear();
        self.total_allocations = 0;
        self.timeline.clear();
        self.start_time = Instant::now();
    }

    /// Generate memory usage report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("=== Memory Usage Report ===\n");
        report.push_str(&format!("Total Memory Usage: {:.2} MB\n", 
            self.get_total_usage() as f64 / 1024.0 / 1024.0));
        report.push_str(&format!("Memory Limit: {:.2} MB\n", 
            self.memory_limit as f64 / 1024.0 / 1024.0));
        report.push_str(&format!("Total Allocations: {}\n", self.total_allocations));
        
        report.push_str("\nMemory Usage by Phase:\n");
        for (phase, usage) in &self.phase_usage {
            let peak = self.peak_usage.get(phase).unwrap_or(&0);
            report.push_str(&format!("  {:?}: {:.2} MB (peak: {:.2} MB)\n", 
                phase, 
                *usage as f64 / 1024.0 / 1024.0,
                *peak as f64 / 1024.0 / 1024.0));
        }
        
        report.push_str("\nMemory Timeline (last 10 events):\n");
        for snapshot in self.timeline.iter().rev().take(10) {
            let elapsed = snapshot.timestamp.duration_since(self.start_time);
            report.push_str(&format!("  {:.2}s - {:?}: {:.2} MB - {}\n",
                elapsed.as_secs_f64(),
                snapshot.phase,
                snapshot.memory_usage as f64 / 1024.0 / 1024.0,
                snapshot.description));
        }
        
        report
    }
}

/// Memory-related errors
#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    #[error("Memory limit exceeded: {current} bytes used, limit is {limit} bytes (phase: {phase:?})")]
    LimitExceeded {
        current: usize,
        limit: usize,
        phase: CompilationPhase,
    },
    #[error("Memory allocation failed: {message}")]
    AllocationFailed { message: String },
    #[error("Memory corruption detected: {details}")]
    CorruptionDetected { details: String },
}

/// Global memory profiler access functions
pub fn get_memory_profiler() -> Arc<Mutex<MemoryProfiler>> {
    MEMORY_PROFILER.clone()
}

/// Record memory usage for current compilation phase
pub fn record_memory_usage(phase: CompilationPhase, usage: usize, description: &str) {
    if let Ok(mut profiler) = get_memory_profiler().lock() {
        profiler.record_usage(phase, usage, description.to_string());
    }
}

/// Check if memory usage is within limits
pub fn check_memory_limit() -> Result<(), MemoryError> {
    if let Ok(profiler) = get_memory_profiler().lock() {
        profiler.check_memory_limit()
    } else {
        Ok(()) // If we can't lock, assume it's okay
    }
}


/// Set global memory limit
pub fn set_memory_limit(limit: usize) {
    if let Ok(mut profiler) = get_memory_profiler().lock() {
        profiler.set_memory_limit(limit);
    }
}

/// Reset profiler for new compilation
pub fn reset_profiler() {
    if let Ok(mut profiler) = get_memory_profiler().lock() {
        profiler.reset();
    }
}

/// Generate memory usage report
pub fn generate_memory_report() -> String {
    if let Ok(profiler) = get_memory_profiler().lock() {
        profiler.generate_report()
    } else {
        "Error: Could not access memory profiler".to_string()
    }
}

/// Get current memory usage for resource management
pub fn get_current_memory_usage() -> usize {
    if let Ok(profiler) = get_memory_profiler().lock() {
        // Sum current usage across all phases
        profiler.phase_usage.values().sum()
    } else {
        0
    }
}

/// Get current memory limit for resource management
pub fn get_memory_limit() -> usize {
    if let Ok(profiler) = get_memory_profiler().lock() {
        profiler.memory_limit
    } else {
        1024 * 1024 * 1024 // Default 1GB
    }
}

/// Memory tracking wrapper for large allocations
pub struct MemoryTracker<T> {
    data: T,
    size: usize,
    phase: CompilationPhase,
}

impl<T> MemoryTracker<T> {
    /// Create a new memory tracker
    pub fn new(data: T, phase: CompilationPhase) -> Self {
        let size = std::mem::size_of::<T>();
        record_memory_usage(phase, size, &format!("Allocated {}", std::any::type_name::<T>()));
        
        Self {
            data,
            size,
            phase,
        }
    }

    /// Get the wrapped data
    pub fn get(&self) -> &T {
        &self.data
    }

    /// Get mutable reference to wrapped data
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.data
    }

    /// Get the size of the tracked data
    pub fn size(&self) -> usize {
        self.size
    }
}

impl<T> Drop for MemoryTracker<T> {
    fn drop(&mut self) {
        // Record memory being freed
        if let Ok(mut profiler) = get_memory_profiler().lock() {
            // Subtract this memory from the phase usage
            if let Some(current_usage) = profiler.phase_usage.get(&self.phase) {
                let new_usage = current_usage.saturating_sub(self.size);
                profiler.phase_usage.insert(self.phase, new_usage);
            }
            
            // Record the deallocation event
            profiler.record_usage(
                self.phase, 
                0, 
                format!("Freed {} bytes of {}", self.size, std::any::type_name::<T>())
            );
        }
    }
}

/// Macro for easy memory tracking
#[macro_export]
macro_rules! track_memory {
    ($phase:expr, $expr:expr) => {{
        let result = $expr;
        let size = std::mem::size_of_val(&result);
        $crate::memory_profiler::record_memory_usage($phase, size, &format!("Allocated at {}:{}", file!(), line!()));
        result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_profiler_basic() {
        let mut profiler = MemoryProfiler::new();
        
        // Record some usage
        profiler.record_usage(CompilationPhase::Lexing, 1024, "Test allocation".to_string());
        profiler.record_usage(CompilationPhase::Parsing, 2048, "Parser allocation".to_string());
        
        // Check totals
        assert_eq!(profiler.get_total_usage(), 3072);
        assert_eq!(profiler.timeline.len(), 2);
    }

    #[test]
    fn test_memory_limit_check() {
        let mut profiler = MemoryProfiler::new();
        profiler.set_memory_limit(1024);
        
        // Should be okay under limit
        profiler.record_usage(CompilationPhase::Lexing, 512, "Small allocation".to_string());
        assert!(profiler.check_memory_limit().is_ok());
        
        // Should fail over limit
        profiler.record_usage(CompilationPhase::Parsing, 1024, "Large allocation".to_string());
        assert!(profiler.check_memory_limit().is_err());
    }

    #[test]
    fn test_memory_tracker() {
        let data = vec![1, 2, 3, 4, 5];
        let _tracker = MemoryTracker::new(data, CompilationPhase::Lexing);
        
        // Memory should be recorded
        assert!(get_current_memory_usage() > 0);
    }
}