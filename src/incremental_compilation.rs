// src/incremental_compilation.rs
//! Incremental compilation system for the Eä programming language.
//!
//! This module provides efficient incremental compilation by tracking file changes,
//! maintaining compilation dependencies, and recompiling only what's necessary.

use crate::ast::Stmt;
use crate::error::{CompileError, Result};
use crate::type_system::TypeContext;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Represents a compilation unit with its dependencies and metadata
#[derive(Debug, Clone)]
pub struct CompilationUnit {
    /// File path of the source file
    pub file_path: PathBuf,
    /// Hash of the file contents
    pub content_hash: u64,
    /// Last modification time
    pub last_modified: SystemTime,
    /// Dependencies on other compilation units
    pub dependencies: HashSet<PathBuf>,
    /// Compiled AST
    pub ast: Option<Vec<Stmt>>,
    /// Type context
    pub type_context: Option<TypeContext>,
    /// Compilation timestamp
    pub compiled_at: SystemTime,
    /// Compilation time taken
    pub compilation_time: Duration,
    /// Whether this unit needs recompilation
    pub needs_recompilation: bool,
}

/// Incremental compilation configuration
#[derive(Debug, Clone)]
pub struct IncrementalConfig {
    /// Maximum number of compilation units to track
    pub max_units: usize,
    /// Enable dependency tracking
    pub enable_dependency_tracking: bool,
    /// Enable content hashing for change detection
    pub enable_content_hashing: bool,
    /// Cache directory for incremental compilation artifacts
    pub cache_directory: PathBuf,
    /// Enable parallel compilation of independent units
    pub enable_parallel_compilation: bool,
}

impl Default for IncrementalConfig {
    fn default() -> Self {
        Self {
            max_units: 10000,
            enable_dependency_tracking: true,
            enable_content_hashing: true,
            cache_directory: PathBuf::from(".ea_cache"),
            enable_parallel_compilation: true,
        }
    }
}

/// Statistics for incremental compilation
#[derive(Debug, Default, Clone)]
pub struct IncrementalStats {
    /// Total compilation units tracked
    pub total_units: u64,
    /// Units that were recompiled
    pub units_recompiled: u64,
    /// Units that were reused from cache
    pub units_cached: u64,
    /// Total compilation time saved
    pub time_saved: Duration,
    /// Total compilation time spent
    pub time_spent: Duration,
    /// Number of dependency cycles detected
    pub dependency_cycles: u64,
}

impl IncrementalStats {
    /// Calculate cache hit ratio
    pub fn cache_hit_ratio(&self) -> f64 {
        if self.total_units == 0 {
            0.0
        } else {
            (self.units_cached as f64 / self.total_units as f64) * 100.0
        }
    }
}

/// Incremental compilation manager
pub struct IncrementalCompiler {
    /// Configuration
    config: IncrementalConfig,
    /// Compilation units by file path
    units: HashMap<PathBuf, CompilationUnit>,
    /// Dependency graph
    dependencies: HashMap<PathBuf, HashSet<PathBuf>>,
    /// Statistics
    stats: IncrementalStats,
}

impl IncrementalCompiler {
    /// Create a new incremental compiler
    pub fn new() -> Self {
        Self::with_config(IncrementalConfig::default())
    }

    /// Create a new incremental compiler with custom configuration
    pub fn with_config(config: IncrementalConfig) -> Self {
        // Create cache directory if it doesn't exist
        if !config.cache_directory.exists() {
            let _ = fs::create_dir_all(&config.cache_directory);
        }

        Self {
            config,
            units: HashMap::new(),
            dependencies: HashMap::new(),
            stats: IncrementalStats::default(),
        }
    }

    /// Hash file contents for change detection
    fn hash_file_contents(&self, file_path: &Path) -> Result<u64> {
        let contents = fs::read_to_string(file_path).map_err(|e| {
            CompileError::codegen_error(
                format!("Failed to read file {}: {}", file_path.display(), e),
                None,
            )
        })?;

        let mut hasher = DefaultHasher::new();
        contents.hash(&mut hasher);
        Ok(hasher.finish())
    }

    /// Check if a file has changed
    fn has_file_changed(&self, file_path: &Path) -> Result<bool> {
        let metadata = fs::metadata(file_path).map_err(|e| {
            CompileError::codegen_error(
                format!("Failed to get metadata for {}: {}", file_path.display(), e),
                None,
            )
        })?;

        let last_modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);

        if let Some(unit) = self.units.get(file_path) {
            if unit.last_modified != last_modified {
                return Ok(true);
            }

            if self.config.enable_content_hashing {
                let current_hash = self.hash_file_contents(file_path)?;
                return Ok(current_hash != unit.content_hash);
            }
        }

        Ok(true) // File not tracked or content hashing disabled
    }

    /// Add a compilation unit
    pub fn add_unit(
        &mut self,
        file_path: PathBuf,
        ast: Vec<Stmt>,
        type_context: TypeContext,
        compilation_time: Duration,
    ) -> Result<()> {
        let metadata = fs::metadata(&file_path).map_err(|e| {
            CompileError::codegen_error(
                format!("Failed to get metadata for {}: {}", file_path.display(), e),
                None,
            )
        })?;

        let last_modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        let content_hash = if self.config.enable_content_hashing {
            self.hash_file_contents(&file_path)?
        } else {
            0
        };

        let unit = CompilationUnit {
            file_path: file_path.clone(),
            content_hash,
            last_modified,
            dependencies: HashSet::new(),
            ast: Some(ast),
            type_context: Some(type_context),
            compiled_at: SystemTime::now(),
            compilation_time,
            needs_recompilation: false,
        };

        self.units.insert(file_path, unit);
        self.stats.total_units += 1;

        Ok(())
    }

    /// Check if a unit needs recompilation
    pub fn needs_recompilation(&self, file_path: &Path) -> Result<bool> {
        self.needs_recompilation_recursive(file_path, &mut HashSet::new())
    }

    /// Check if a unit needs recompilation (with cycle detection)
    fn needs_recompilation_recursive(
        &self,
        file_path: &Path,
        visited: &mut HashSet<PathBuf>,
    ) -> Result<bool> {
        if visited.contains(file_path) {
            return Ok(false); // Cycle detected, assume no recompilation needed
        }

        if !self.units.contains_key(file_path) {
            return Ok(true); // Not compiled yet
        }

        if self.has_file_changed(file_path)? {
            return Ok(true); // File changed
        }

        // Check if any dependencies have changed
        if let Some(deps) = self.dependencies.get(file_path) {
            visited.insert(file_path.to_path_buf());
            for dep in deps {
                if self.needs_recompilation_recursive(dep, visited)? {
                    return Ok(true); // Dependency changed
                }
            }
            visited.remove(file_path);
        }

        Ok(false)
    }

    /// Get compilation units that need recompilation
    pub fn get_units_needing_recompilation(&self) -> Result<Vec<PathBuf>> {
        let mut units_to_recompile = Vec::new();

        for file_path in self.units.keys() {
            if self.needs_recompilation(file_path)? {
                units_to_recompile.push(file_path.clone());
            }
        }

        Ok(units_to_recompile)
    }

    /// Add dependency relationship
    pub fn add_dependency(&mut self, from: PathBuf, to: PathBuf) {
        self.dependencies
            .entry(from)
            .or_insert_with(HashSet::new)
            .insert(to);
    }

    /// Perform topological sort to determine compilation order using Kahn's algorithm
    /// DEVELOPMENT_PROCESS.md: Replace recursive algorithm to eliminate infinite loop possibility
    pub fn get_compilation_order(&mut self, units: &[PathBuf]) -> Result<Vec<PathBuf>> {
        // Simple case: if no units, return empty
        if units.is_empty() {
            return Ok(Vec::new());
        }
        
        // Build in-degree map for Kahn's algorithm (iterative topological sort)
        let mut in_degree: HashMap<PathBuf, usize> = HashMap::new();
        let mut graph: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
        
        // Initialize all units with 0 in-degree
        for unit in units {
            in_degree.insert(unit.clone(), 0);
            graph.insert(unit.clone(), Vec::new());
        }
        
        // Build the graph and compute in-degrees
        for unit in units {
            if let Some(deps) = self.dependencies.get(unit) {
                for dep in deps {
                    // unit depends on dep, so dep -> unit edge
                    // Only process dependencies that are in our unit set
                    if in_degree.contains_key(dep) {
                        graph.get_mut(dep).unwrap().push(unit.clone());
                        *in_degree.get_mut(unit).unwrap() += 1;
                    }
                }
            }
        }
        
        // Find all nodes with no incoming edges
        let mut queue: std::collections::VecDeque<PathBuf> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(unit, _)| unit.clone())
            .collect();
        
        let mut result = Vec::new();
        
        // Process nodes with no dependencies first
        while let Some(unit) = queue.pop_front() {
            result.push(unit.clone());
            
            // Process all nodes that depend on this unit
            if let Some(dependents) = graph.get(&unit) {
                for dependent in dependents {
                    // Decrease in-degree
                    let current_degree = in_degree.get_mut(dependent).unwrap();
                    *current_degree -= 1;
                    
                    // If no more dependencies, add to queue
                    if *current_degree == 0 {
                        queue.push_back(dependent.clone());
                    }
                }
            }
        }
        
        // Check for cycles - if we didn't process all units, there's a cycle
        if result.len() != units.len() {
            self.stats.dependency_cycles += 1;
            return Err(CompileError::codegen_error(
                "Circular dependency detected in compilation units".to_string(),
                None,
            ));
        }
        
        Ok(result)
    }

    /// Get cached compilation result
    pub fn get_cached_result(&self, file_path: &Path) -> Option<(&Vec<Stmt>, &TypeContext)> {
        self.units.get(file_path).and_then(|unit| {
            if let (Some(ast), Some(type_context)) = (&unit.ast, &unit.type_context) {
                if !unit.needs_recompilation {
                    return Some((ast, type_context));
                }
            }
            None
        })
    }

    /// Mark unit as needing recompilation
    pub fn mark_for_recompilation(&mut self, file_path: &Path) {
        if let Some(unit) = self.units.get_mut(file_path) {
            unit.needs_recompilation = true;
        }
    }

    /// Clear all cached units
    pub fn clear_cache(&mut self) {
        self.units.clear();
        self.dependencies.clear();
        self.stats = IncrementalStats::default();
    }

    /// Get statistics
    pub fn get_stats(&self) -> &IncrementalStats {
        &self.stats
    }

    /// Print compilation statistics
    pub fn print_stats(&self) {
        eprintln!("📊 Incremental Compilation Statistics:");
        eprintln!("   Total units tracked: {}", self.stats.total_units);
        eprintln!("   Units recompiled: {}", self.stats.units_recompiled);
        eprintln!("   Units cached: {}", self.stats.units_cached);
        eprintln!("   Cache hit ratio: {:.1}%", self.stats.cache_hit_ratio());
        eprintln!("   Time saved: {:?}", self.stats.time_saved);
        eprintln!("   Time spent: {:?}", self.stats.time_spent);
        eprintln!("   Dependency cycles: {}", self.stats.dependency_cycles);
    }
}

/// Global incremental compiler instance
static mut GLOBAL_INCREMENTAL_COMPILER: Option<IncrementalCompiler> = None;
static INCREMENTAL_INIT: std::sync::Once = std::sync::Once::new();

/// Initialize the global incremental compiler
pub fn initialize_incremental_compiler(config: IncrementalConfig) {
    INCREMENTAL_INIT.call_once(|| unsafe {
        GLOBAL_INCREMENTAL_COMPILER = Some(IncrementalCompiler::with_config(config));
    });
}

/// Get reference to the global incremental compiler
pub fn get_incremental_compiler() -> &'static mut IncrementalCompiler {
    unsafe {
        GLOBAL_INCREMENTAL_COMPILER
            .as_mut()
            .expect("Incremental compiler not initialized")
    }
}

/// Initialize incremental compiler with default configuration
pub fn initialize_default_incremental_compiler() {
    initialize_incremental_compiler(IncrementalConfig::default());
}

/// Compile a file with incremental compilation
pub fn compile_file_incremental(file_path: &Path) -> Result<(Vec<Stmt>, TypeContext)> {
    let compiler = get_incremental_compiler();

    // Check if we can use cached result
    if let Some((ast, type_context)) = compiler.get_cached_result(file_path) {
        eprintln!(
            "✅ Using cached compilation result for {}",
            file_path.display()
        );
        return Ok((ast.clone(), type_context.clone()));
    }

    // Need to recompile
    eprintln!("🔧 Recompiling {}", file_path.display());
    let start_time = SystemTime::now();

    let source = fs::read_to_string(file_path).map_err(|e| {
        CompileError::codegen_error(
            format!("Failed to read file {}: {}", file_path.display(), e),
            None,
        )
    })?;

    let (ast, type_context) = crate::compile_to_ast(&source)?;

    let compilation_time = start_time.elapsed().unwrap_or(Duration::from_secs(0));

    // Cache the result
    compiler.add_unit(
        file_path.to_path_buf(),
        ast.clone(),
        type_context.clone(),
        compilation_time,
    )?;
    compiler.stats.units_recompiled += 1;
    compiler.stats.time_spent += compilation_time;

    eprintln!(
        "✅ Compiled {} in {:?}",
        file_path.display(),
        compilation_time
    );

    Ok((ast, type_context))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_incremental_compiler_creation() {
        let compiler = IncrementalCompiler::new();
        assert_eq!(compiler.units.len(), 0);
        assert_eq!(compiler.stats.total_units, 0);
    }

    #[test]
    fn test_file_change_detection() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.ea");

        // Create test file
        let mut file = fs::File::create(&file_path).unwrap();
        writeln!(file, "func main() {{ print(42); }}").unwrap();

        let mut compiler = IncrementalCompiler::new();

        // First check - file should need compilation
        assert!(compiler.has_file_changed(&file_path).unwrap());

        // Add unit
        let ast = vec![]; // Empty AST for testing
        let type_context = crate::type_system::TypeContext::new();
        compiler
            .add_unit(
                file_path.clone(),
                ast,
                type_context,
                Duration::from_millis(100),
            )
            .unwrap();

        // Should not need recompilation immediately
        assert!(!compiler.has_file_changed(&file_path).unwrap());

        // Modify file
        std::thread::sleep(Duration::from_millis(10)); // Ensure different timestamp
        let mut file = fs::File::create(&file_path).unwrap();
        writeln!(file, "func main() {{ print(43); }}").unwrap();

        // Should need recompilation now
        assert!(compiler.has_file_changed(&file_path).unwrap());
    }

    #[test]
    fn test_dependency_tracking() {
        let mut compiler = IncrementalCompiler::new();

        let file_a = PathBuf::from("a.ea");
        let file_b = PathBuf::from("b.ea");
        let file_c = PathBuf::from("c.ea");

        compiler.add_dependency(file_a.clone(), file_b.clone());
        compiler.add_dependency(file_b.clone(), file_c.clone());

        let order = compiler
            .get_compilation_order(&[file_a.clone(), file_b.clone(), file_c.clone()])
            .unwrap();

        // The dependencies mean: a depends on b, b depends on c
        // So compilation order should be: c, b, a
        // But let's be more flexible and just verify the dependencies are respected
        assert_eq!(order.len(), 3);

        // c should come before b
        let c_pos = order.iter().position(|x| x == &file_c).unwrap();
        let b_pos = order.iter().position(|x| x == &file_b).unwrap();
        let a_pos = order.iter().position(|x| x == &file_a).unwrap();

        assert!(c_pos < b_pos); // c before b
        assert!(b_pos < a_pos); // b before a
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut compiler = IncrementalCompiler::new();

        let file_a = PathBuf::from("a.ea");
        let file_b = PathBuf::from("b.ea");

        compiler.add_dependency(file_a.clone(), file_b.clone());
        compiler.add_dependency(file_b.clone(), file_a.clone());

        let result = compiler.get_compilation_order(&[file_a.clone(), file_b.clone()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_cache_hit_ratio() {
        let mut stats = IncrementalStats::default();
        stats.total_units = 10;
        stats.units_cached = 7;
        assert_eq!(stats.cache_hit_ratio(), 70.0);
    }
}
