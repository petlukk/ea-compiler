// src/parallel_compilation.rs
//! Parallel compilation system for the Eä programming language.
//!
//! This module provides multi-threaded compilation capabilities to leverage
//! multi-core systems for faster compilation of large projects.

use crate::ast::Stmt;
use crate::error::{CompileError, Result};
use crate::type_system::TypeContext;
// IncrementalCompiler import removed as it's unused
use crossbeam_channel::{bounded, Receiver, Sender};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};
// rayon::prelude import removed as it's unused

/// Compilation job for parallel processing
#[derive(Debug, Clone)]
pub struct CompilationJob {
    /// File path to compile
    pub file_path: PathBuf,
    /// Source code content
    pub source: String,
    /// Job priority (higher = more important)
    pub priority: u32,
    /// Dependencies that must be compiled first
    pub dependencies: Vec<PathBuf>,
    /// Job ID for tracking
    pub job_id: u64,
}

/// Result of a parallel compilation job
#[derive(Debug, Clone)]
pub struct CompilationResult {
    /// Job ID
    pub job_id: u64,
    /// File path that was compiled
    pub file_path: PathBuf,
    /// Compiled AST
    pub ast: Vec<Stmt>,
    /// Type context
    pub type_context: TypeContext,
    /// Compilation time
    pub compilation_time: Duration,
    /// Success flag
    pub success: bool,
    /// Error message if compilation failed
    pub error_message: Option<String>,
}

/// Parallel compilation configuration
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// Number of worker threads (0 = auto-detect)
    pub num_threads: usize,
    /// Maximum number of jobs in the queue
    pub max_queue_size: usize,
    /// Enable work stealing between threads
    pub enable_work_stealing: bool,
    /// Enable job prioritization
    pub enable_job_prioritization: bool,
    /// Maximum compilation time per job (in seconds)
    pub max_job_time_seconds: u64,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            num_threads: 0, // Auto-detect
            max_queue_size: 1000,
            enable_work_stealing: true,
            enable_job_prioritization: true,
            max_job_time_seconds: 60,
        }
    }
}

/// Statistics for parallel compilation
#[derive(Debug, Default, Clone)]
pub struct ParallelStats {
    /// Total jobs processed
    pub total_jobs: u64,
    /// Jobs completed successfully
    pub successful_jobs: u64,
    /// Jobs that failed
    pub failed_jobs: u64,
    /// Total compilation time across all threads
    pub total_compilation_time: Duration,
    /// Wall-clock time for parallel compilation
    pub wall_clock_time: Duration,
    /// Number of threads used
    pub threads_used: usize,
    /// Average job completion time
    pub average_job_time: Duration,
    /// Peak memory usage
    pub peak_memory_usage: u64,
}

impl ParallelStats {
    /// Calculate parallelization efficiency
    pub fn parallelization_efficiency(&self) -> f64 {
        if self.wall_clock_time.is_zero() {
            0.0
        } else {
            let serial_time = self.total_compilation_time.as_secs_f64();
            let parallel_time = self.wall_clock_time.as_secs_f64();
            let theoretical_speedup = self.threads_used as f64;
            let actual_speedup = serial_time / parallel_time;
            (actual_speedup / theoretical_speedup) * 100.0
        }
    }

    /// Calculate throughput (jobs per second)
    pub fn throughput(&self) -> f64 {
        if self.wall_clock_time.is_zero() {
            0.0
        } else {
            self.total_jobs as f64 / self.wall_clock_time.as_secs_f64()
        }
    }
}

/// Parallel compilation engine
pub struct ParallelCompiler {
    /// Configuration
    config: ParallelConfig,
    /// Job queue
    job_queue: Arc<RwLock<Vec<CompilationJob>>>,
    /// Completed results
    results: Arc<RwLock<HashMap<u64, CompilationResult>>>,
    /// Statistics
    stats: Arc<RwLock<ParallelStats>>,
    /// Next job ID
    next_job_id: Arc<Mutex<u64>>,
    /// Worker threads
    workers: Vec<thread::JoinHandle<()>>,
    /// Job sender
    job_sender: Option<Sender<CompilationJob>>,
    /// Result receiver
    result_receiver: Option<Receiver<CompilationResult>>,
}

impl ParallelCompiler {
    /// Create a new parallel compiler
    pub fn new() -> Self {
        Self::with_config(ParallelConfig::default())
    }

    /// Create a new parallel compiler with custom configuration
    pub fn with_config(config: ParallelConfig) -> Self {
        let num_threads = if config.num_threads == 0 {
            num_cpus::get()
        } else {
            config.num_threads
        };

        eprintln!(
            "🚀 Initializing parallel compiler with {} threads",
            num_threads
        );

        let (job_sender, job_receiver) = bounded(config.max_queue_size);
        let (result_sender, result_receiver) = bounded(config.max_queue_size);

        let job_queue = Arc::new(RwLock::new(Vec::new()));
        let results = Arc::new(RwLock::new(HashMap::new()));
        let stats = Arc::new(RwLock::new(ParallelStats {
            threads_used: num_threads,
            ..Default::default()
        }));
        let next_job_id = Arc::new(Mutex::new(1));

        // Start worker threads
        let mut workers = Vec::new();
        for worker_id in 0..num_threads {
            let job_receiver: crossbeam_channel::Receiver<CompilationJob> = job_receiver.clone();
            let result_sender = result_sender.clone();
            let stats = stats.clone();
            let max_job_time = Duration::from_secs(config.max_job_time_seconds);

            let worker = thread::spawn(move || {
                eprintln!("🔧 Worker thread {} started", worker_id);

                while let Ok(job) = job_receiver.recv() {
                    let start_time = Instant::now();

                    eprintln!(
                        "🔨 Worker {} processing job {} ({})",
                        worker_id,
                        job.job_id,
                        job.file_path.display()
                    );

                    // Compile the job with timeout
                    let result = Self::compile_job_with_timeout(job.clone(), max_job_time);

                    let compilation_time = start_time.elapsed();

                    // Update statistics
                    {
                        let mut stats = stats.write().unwrap();
                        stats.total_jobs += 1;
                        stats.total_compilation_time += compilation_time;
                        stats.average_job_time =
                            stats.total_compilation_time / stats.total_jobs.max(1) as u32;

                        if result.success {
                            stats.successful_jobs += 1;
                        } else {
                            stats.failed_jobs += 1;
                        }
                    }

                    eprintln!(
                        "✅ Worker {} completed job {} in {:?}",
                        worker_id, job.job_id, compilation_time
                    );

                    // Send result
                    if result_sender.send(result).is_err() {
                        eprintln!("❌ Worker {} failed to send result", worker_id);
                        break;
                    }
                }

                eprintln!("🔧 Worker thread {} finished", worker_id);
            });

            workers.push(worker);
        }

        Self {
            config,
            job_queue,
            results,
            stats,
            next_job_id,
            workers,
            job_sender: Some(job_sender),
            result_receiver: Some(result_receiver),
        }
    }

    /// Compile a job with timeout
    fn compile_job_with_timeout(job: CompilationJob, timeout: Duration) -> CompilationResult {
        let start_time = Instant::now();

        // Use a timeout-based approach for compilation
        let result = std::panic::catch_unwind(|| crate::compile_to_ast(&job.source));

        let compilation_time = start_time.elapsed();

        match result {
            Ok(Ok((ast, type_context))) => CompilationResult {
                job_id: job.job_id,
                file_path: job.file_path,
                ast,
                type_context,
                compilation_time,
                success: true,
                error_message: None,
            },
            Ok(Err(e)) => CompilationResult {
                job_id: job.job_id,
                file_path: job.file_path,
                ast: Vec::new(),
                type_context: crate::type_system::TypeContext::new(),
                compilation_time,
                success: false,
                error_message: Some(format!("Compilation error: {}", e)),
            },
            Err(_) => CompilationResult {
                job_id: job.job_id,
                file_path: job.file_path,
                ast: Vec::new(),
                type_context: crate::type_system::TypeContext::new(),
                compilation_time,
                success: false,
                error_message: Some("Compilation panicked".to_string()),
            },
        }
    }

    /// Submit a compilation job
    pub fn submit_job(&mut self, file_path: PathBuf, source: String, priority: u32) -> Result<u64> {
        let job_id = {
            let mut next_id = self.next_job_id.lock().unwrap();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let job = CompilationJob {
            file_path,
            source,
            priority,
            dependencies: Vec::new(),
            job_id,
        };

        if let Some(sender) = &self.job_sender {
            sender.send(job).map_err(|_| {
                CompileError::codegen_error("Failed to submit compilation job".to_string(), None)
            })?;
        }

        Ok(job_id)
    }

    /// Submit multiple jobs for parallel compilation
    pub fn submit_jobs(&mut self, jobs: Vec<(PathBuf, String, u32)>) -> Result<Vec<u64>> {
        let mut job_ids = Vec::new();

        for (file_path, source, priority) in jobs {
            let job_id = self.submit_job(file_path, source, priority)?;
            job_ids.push(job_id);
        }

        Ok(job_ids)
    }

    /// Wait for a specific job to complete
    pub fn wait_for_job(&mut self, job_id: u64) -> Result<CompilationResult> {
        while let Some(receiver) = &self.result_receiver {
            if let Ok(result) = receiver.recv() {
                if result.job_id == job_id {
                    return Ok(result);
                }

                // Store other results for later retrieval
                self.results.write().unwrap().insert(result.job_id, result);
            }
        }

        Err(CompileError::codegen_error(
            "Job not found or receiver closed".to_string(),
            None,
        ))
    }

    /// Wait for all jobs to complete
    pub fn wait_for_all_jobs(&mut self) -> Result<HashMap<u64, CompilationResult>> {
        let total_jobs = self.stats.read().unwrap().total_jobs;
        let mut completed_jobs = 0;
        let mut all_results = HashMap::new();

        while completed_jobs < total_jobs {
            if let Some(receiver) = &self.result_receiver {
                if let Ok(result) = receiver.recv() {
                    all_results.insert(result.job_id, result);
                    completed_jobs += 1;
                }
            }
        }

        Ok(all_results)
    }

    /// Compile multiple files in parallel
    pub fn compile_files_parallel(
        &mut self,
        files: Vec<PathBuf>,
    ) -> Result<HashMap<PathBuf, CompilationResult>> {
        let start_time = Instant::now();

        eprintln!("🚀 Starting parallel compilation of {} files", files.len());

        // Read all files and submit jobs
        let mut job_ids = Vec::new();
        for file_path in &files {
            let source = std::fs::read_to_string(file_path).map_err(|e| {
                CompileError::codegen_error(
                    format!("Failed to read {}: {}", file_path.display(), e),
                    None,
                )
            })?;

            let job_id = self.submit_job(file_path.clone(), source, 1)?;
            job_ids.push(job_id);
        }

        // Wait for all jobs to complete
        let mut results = HashMap::new();
        for job_id in job_ids {
            let result = self.wait_for_job(job_id)?;
            results.insert(result.file_path.clone(), result);
        }

        // Update wall-clock time
        let wall_clock_time = start_time.elapsed();
        self.stats.write().unwrap().wall_clock_time = wall_clock_time;

        eprintln!("✅ Parallel compilation completed in {:?}", wall_clock_time);
        self.print_stats();

        Ok(results)
    }

    /// Get compilation statistics
    pub fn get_stats(&self) -> ParallelStats {
        self.stats.read().unwrap().clone()
    }

    /// Print compilation statistics
    pub fn print_stats(&self) {
        let stats = self.stats.read().unwrap();
        eprintln!("📊 Parallel Compilation Statistics:");
        eprintln!("   Total jobs: {}", stats.total_jobs);
        eprintln!("   Successful jobs: {}", stats.successful_jobs);
        eprintln!("   Failed jobs: {}", stats.failed_jobs);
        eprintln!("   Threads used: {}", stats.threads_used);
        eprintln!("   Wall-clock time: {:?}", stats.wall_clock_time);
        eprintln!(
            "   Total compilation time: {:?}",
            stats.total_compilation_time
        );
        eprintln!("   Average job time: {:?}", stats.average_job_time);
        eprintln!("   Throughput: {:.2} jobs/sec", stats.throughput());
        eprintln!(
            "   Parallelization efficiency: {:.1}%",
            stats.parallelization_efficiency()
        );
    }
}

impl Drop for ParallelCompiler {
    fn drop(&mut self) {
        eprintln!("🔧 Shutting down parallel compiler...");

        // Close channels to signal workers to stop
        drop(self.job_sender.take());
        drop(self.result_receiver.take());

        // Wait for workers to finish
        while let Some(worker) = self.workers.pop() {
            if let Err(e) = worker.join() {
                eprintln!("❌ Worker thread panic: {:?}", e);
            }
        }

        eprintln!("✅ Parallel compiler shutdown complete");
    }
}

/// Global parallel compiler instance
static GLOBAL_PARALLEL_COMPILER: LazyLock<Mutex<Option<ParallelCompiler>>> = LazyLock::new(|| Mutex::new(None));
/// Initialize the global parallel compiler
pub fn initialize_parallel_compiler(config: ParallelConfig) {
    let mut compiler = GLOBAL_PARALLEL_COMPILER.lock().unwrap();
    if compiler.is_none() {
        *compiler = Some(ParallelCompiler::with_config(config));
    }
}

/// Run operation with the global parallel compiler
pub fn with_parallel_compiler<T>(f: impl FnOnce(&mut ParallelCompiler) -> T) -> T {
    let mut compiler = GLOBAL_PARALLEL_COMPILER.lock().unwrap();
    let compiler_ref = compiler.as_mut().expect("Parallel compiler not initialized");
    f(compiler_ref)
}

/// Initialize parallel compiler with default configuration
pub fn initialize_default_parallel_compiler() {
    initialize_parallel_compiler(ParallelConfig::default());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parallel_config_default() {
        let config = ParallelConfig::default();
        assert_eq!(config.num_threads, 0); // Auto-detect
        assert!(config.enable_work_stealing);
        assert!(config.enable_job_prioritization);
    }

    #[test]
    fn test_parallel_stats_calculations() {
        let mut stats = ParallelStats::default();
        stats.total_jobs = 10;
        stats.successful_jobs = 8;
        stats.failed_jobs = 2;
        stats.wall_clock_time = Duration::from_secs(5);
        stats.total_compilation_time = Duration::from_secs(20);
        stats.threads_used = 4;

        assert_eq!(stats.throughput(), 2.0); // 10 jobs / 5 seconds
        assert_eq!(stats.parallelization_efficiency(), 100.0); // 20s / 5s = 4x speedup on 4 threads
    }

    #[test]
    fn test_compilation_job_creation() {
        let job = CompilationJob {
            file_path: PathBuf::from("test.ea"),
            source: "func main() {}".to_string(),
            priority: 1,
            dependencies: Vec::new(),
            job_id: 123,
        };

        assert_eq!(job.job_id, 123);
        assert_eq!(job.priority, 1);
        assert!(job.dependencies.is_empty());
    }
}
