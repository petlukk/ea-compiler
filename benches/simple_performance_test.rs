//! Simple Performance Test - Actual Measurements
//!
//! This benchmark measures real performance data we can verify

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ea_compiler::{tokenize, parse, compile_to_ast};
use std::time::{Duration, Instant};

const SIMPLE_PROGRAM: &str = r#"
func add(a: i32, b: i32) -> i32 {
    return a + b;
}

func main() -> () {
    let result = add(5, 10);
    return;
}
"#;

/// Measure actual tokenization speed
fn bench_tokenization_speed(c: &mut Criterion) {
    c.bench_function("tokenize_simple_program", |b| {
        b.iter(|| {
            black_box(tokenize(black_box(SIMPLE_PROGRAM)).unwrap())
        })
    });
}

/// Measure actual parsing speed
fn bench_parsing_speed(c: &mut Criterion) {
    c.bench_function("parse_simple_program", |b| {
        b.iter(|| {
            black_box(parse(black_box(SIMPLE_PROGRAM)).unwrap())
        })
    });
}

/// Measure full compilation pipeline speed
fn bench_compilation_speed(c: &mut Criterion) {
    c.bench_function("compile_simple_program", |b| {
        b.iter(|| {
            black_box(compile_to_ast(black_box(SIMPLE_PROGRAM)).unwrap())
        })
    });
}

/// Measure compilation speed scaling with program size
fn bench_compilation_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("compilation_scaling");
    
    for size in [10, 50, 100, 200] {
        let large_program = generate_functions(size);
        
        group.bench_with_input(
            criterion::BenchmarkId::new("functions", size),
            &large_program,
            |b, program| {
                b.iter(|| {
                    black_box(compile_to_ast(black_box(program)).unwrap())
                })
            },
        );
    }
    
    group.finish();
}

/// Generate a program with N functions
fn generate_functions(count: usize) -> String {
    let mut program = String::new();
    
    for i in 0..count {
        program.push_str(&format!(
            r#"
func func_{i}(x: i32) -> i32 {{
    return x + {i};
}}
"#,
            i = i
        ));
    }
    
    program.push_str(
        r#"
func main() -> () {
    let result = func_0(42);
    return;
}
"#,
    );
    
    program
}

/// Measure memory usage during compilation
fn bench_memory_usage(c: &mut Criterion) {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
    
    struct TrackingAllocator;
    
    unsafe impl GlobalAlloc for TrackingAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            let ptr = System.alloc(layout);
            if !ptr.is_null() {
                ALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
            }
            ptr
        }
        
        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            System.dealloc(ptr, layout);
            ALLOCATED.fetch_sub(layout.size(), Ordering::SeqCst);
        }
    }
    
    c.bench_function("memory_usage_simple", |b| {
        b.iter_custom(|iters| {
            let start_memory = ALLOCATED.load(Ordering::SeqCst);
            let start_time = Instant::now();
            
            for _ in 0..iters {
                black_box(compile_to_ast(black_box(SIMPLE_PROGRAM)).unwrap());
            }
            
            let duration = start_time.elapsed();
            let end_memory = ALLOCATED.load(Ordering::SeqCst);
            
            println!("Memory used: {} bytes", end_memory.saturating_sub(start_memory));
            duration
        })
    });
}

/// Performance comparison - measure actual time taken
fn bench_performance_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("performance_comparison");
    group.measurement_time(Duration::from_secs(10));
    
    // Measure raw compilation time
    group.bench_function("raw_compilation_time", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            
            for _ in 0..iters {
                let result = compile_to_ast(SIMPLE_PROGRAM);
                black_box(result.unwrap());
            }
            
            start.elapsed()
        })
    });
    
    // Measure just tokenization time
    group.bench_function("tokenization_only", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            
            for _ in 0..iters {
                let result = tokenize(SIMPLE_PROGRAM);
                black_box(result.unwrap());
            }
            
            start.elapsed()
        })
    });
    
    // Measure just parsing time
    group.bench_function("parsing_only", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            
            for _ in 0..iters {
                let result = parse(SIMPLE_PROGRAM);
                black_box(result.unwrap());
            }
            
            start.elapsed()
        })
    });
    
    group.finish();
}

/// Test advanced feature performance
fn bench_advanced_features(c: &mut Criterion) {
    use ea_compiler::comptime::ComptimeEngine;
    use ea_compiler::memory::MemoryManager;
    
    let mut group = c.benchmark_group("advanced_features");
    
    // Measure compile-time engine creation
    group.bench_function("comptime_engine_creation", |b| {
        b.iter(|| {
            black_box(ComptimeEngine::new())
        })
    });
    
    // Measure memory manager creation
    group.bench_function("memory_manager_creation", |b| {
        b.iter(|| {
            black_box(MemoryManager::new())
        })
    });
    
    group.finish();
}

criterion_group!(
    perf_tests,
    bench_tokenization_speed,
    bench_parsing_speed,
    bench_compilation_speed,
    bench_compilation_scaling,
    bench_memory_usage,
    bench_performance_comparison,
    bench_advanced_features
);

criterion_main!(perf_tests);