//! Realistic Performance Benchmarks for Eä Compiler
//! 
//! This benchmark suite provides actual performance measurements
//! instead of theoretical claims.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use ea_compiler::{compile_to_ast, tokenize, parse, type_check};
use std::time::{Duration, Instant};

// Simple working programs that we know compile correctly
const SIMPLE_ARITHMETIC: &str = r#"
func add(a: i32, b: i32) -> i32 {
    return a + b;
}

func main() -> () {
    let result = add(5, 10);
    return;
}
"#;

const SIMPLE_FIBONACCI: &str = r#"
func fibonacci(n: i32) -> i32 {
    if (n <= 1) {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

func main() -> () {
    let result = fibonacci(10);
    return;
}
"#;

const SIMPLE_LOOP: &str = r#"
func sum_to_n(n: i32) -> i32 {
    let sum = 0;
    let i = 1;
    while (i <= n) {
        sum = sum + i;
        i = i + 1;
    }
    return sum;
}

func main() -> () {
    let result = sum_to_n(100);
    return;
}
"#;

/// Baseline lexer performance - how fast can we tokenize?
fn bench_lexer_speed(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer_baseline");
    group.measurement_time(Duration::from_secs(5));

    for (name, program) in [
        ("simple", SIMPLE_ARITHMETIC),
        ("fibonacci", SIMPLE_FIBONACCI),
        ("loop", SIMPLE_LOOP),
    ] {
        group.bench_with_input(
            BenchmarkId::new("tokenize", name),
            program,
            |b, program| {
                b.iter(|| {
                    black_box(tokenize(black_box(program)).unwrap())
                })
            },
        );
    }

    group.finish();
}

/// Baseline parser performance - how fast can we parse?
fn bench_parser_speed(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_baseline");
    group.measurement_time(Duration::from_secs(5));

    for (name, program) in [
        ("simple", SIMPLE_ARITHMETIC),
        ("fibonacci", SIMPLE_FIBONACCI),
        ("loop", SIMPLE_LOOP),
    ] {
        group.bench_with_input(
            BenchmarkId::new("parse", name),
            program,
            |b, program| {
                b.iter(|| {
                    black_box(parse(black_box(program)).unwrap())
                })
            },
        );
    }

    group.finish();
}

/// Baseline type checker performance - how fast can we type check?
fn bench_type_checker_speed(c: &mut Criterion) {
    let mut group = c.benchmark_group("type_checker_baseline");
    group.measurement_time(Duration::from_secs(5));

    for (name, program) in [
        ("simple", SIMPLE_ARITHMETIC),
        ("fibonacci", SIMPLE_FIBONACCI),
        ("loop", SIMPLE_LOOP),
    ] {
        let ast = parse(program).unwrap();
        
        group.bench_with_input(
            BenchmarkId::new("type_check", name),
            &ast,
            |b, ast| {
                b.iter(|| {
                    black_box(type_check(black_box(ast)).unwrap())
                })
            },
        );
    }

    group.finish();
}

/// Full compilation pipeline performance
fn bench_full_compilation_speed(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_compilation_baseline");
    group.measurement_time(Duration::from_secs(10));

    for (name, program) in [
        ("simple", SIMPLE_ARITHMETIC),
        ("fibonacci", SIMPLE_FIBONACCI),
        ("loop", SIMPLE_LOOP),
    ] {
        group.bench_with_input(
            BenchmarkId::new("compile_to_ast", name),
            program,
            |b, program| {
                b.iter(|| {
                    black_box(compile_to_ast(black_box(program)).unwrap())
                })
            },
        );
    }

    group.finish();
}

/// Memory allocation speed benchmark
fn bench_memory_allocation_speed(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");
    group.measurement_time(Duration::from_secs(5));

    // Generate programs of different sizes to measure memory allocation behavior
    let sizes = [10, 50, 100, 500];

    for size in sizes {
        let large_program = generate_program_with_functions(size);
        
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::new("parse_large_program", size),
            &large_program,
            |b, program| {
                b.iter(|| {
                    black_box(parse(black_box(program)).unwrap())
                })
            },
        );
    }

    group.finish();
}

/// Compilation throughput measurement (lines of code per second)
fn bench_compilation_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("compilation_throughput");
    group.measurement_time(Duration::from_secs(10));

    let sizes = [100, 500, 1000];

    for size in sizes {
        let program = generate_program_with_functions(size);
        let line_count = program.lines().count() as u64;
        
        group.throughput(Throughput::Elements(line_count));
        group.bench_with_input(
            BenchmarkId::new("lines_per_second", size),
            &program,
            |b, program| {
                b.iter(|| {
                    black_box(compile_to_ast(black_box(program)).unwrap())
                })
            },
        );
    }

    group.finish();
}

/// Measure compile-time execution engine performance
fn bench_comptime_engine_speed(c: &mut Criterion) {
    use ea_compiler::comptime::{ComptimeEngine, ComptimeValue, AlgorithmType, DataCharacteristics, DataSize, DataDistribution, DataAccessPattern};
    
    let mut group = c.benchmark_group("comptime_engine");
    group.measurement_time(Duration::from_secs(5));

    group.bench_function("algorithm_selection", |b| {
        let mut engine = ComptimeEngine::new();
        let characteristics = DataCharacteristics {
            size: DataSize::Medium,
            distribution: DataDistribution::Random,
            access_pattern: DataAccessPattern::Sequential,
            data_type: "i32".to_string(),
            constraints: vec![],
        };
        
        b.iter(|| {
            black_box(engine.select_optimal_algorithm(
                black_box(AlgorithmType::Sort),
                black_box(characteristics.clone())
            ).unwrap())
        })
    });

    group.finish();
}

/// Measure memory management analysis speed
fn bench_memory_analysis_speed(c: &mut Criterion) {
    use ea_compiler::memory::{MemoryManager, MemoryAttributes, FunctionBodyAnalysis};
    
    let mut group = c.benchmark_group("memory_analysis");
    group.measurement_time(Duration::from_secs(5));

    group.bench_function("function_memory_analysis", |b| {
        let mut manager = MemoryManager::new();
        let attributes = MemoryAttributes {
            pool: None,
            working_set_size: Some(1024),
            max_allocations: Some(10),
            zero_allocation: false,
            regions: vec![],
            lifetime_constraints: vec![],
        };
        let body_analysis = FunctionBodyAnalysis::default();
        
        b.iter(|| {
            black_box(manager.analyze_function_memory(
                black_box("test_function"),
                black_box(&attributes),
                black_box(&body_analysis)
            ).unwrap())
        })
    });

    group.finish();
}

/// Generate a program with many functions for testing large-scale compilation
fn generate_program_with_functions(num_functions: usize) -> String {
    let mut program = String::new();

    for i in 0..num_functions {
        program.push_str(&format!(
            r#"
func function_{i}(x: i32, y: i32) -> i32 {{
    let result = x + y;
    if (result > 50) {{
        return result - 10;
    }} else {{
        return result + 5;
    }}
}}
"#,
            i = i
        ));
    }

    program.push_str(
        r#"
func main() -> () {
    let sum = function_0(10, 20);
    return;
}
"#,
    );

    program
}

/// Compare compilation speed with different optimization levels
fn bench_optimization_levels(c: &mut Criterion) {
    let mut group = c.benchmark_group("optimization_comparison");
    group.measurement_time(Duration::from_secs(5));

    let complex_program = r#"
func complex_calculation(n: i32) -> i32 {
    let result = 0;
    let i = 0;
    while (i < n) {
        let temp = i * i;
        if (temp > 100) {
            result = result + temp;
        } else {
            result = result + (temp * 2);
        }
        i = i + 1;
    }
    return result;
}

func main() -> () {
    let result = complex_calculation(1000);
    return;
}
"#;

    group.bench_function("complex_program", |b| {
        b.iter(|| {
            black_box(compile_to_ast(black_box(complex_program)).unwrap())
        })
    });

    group.finish();
}

/// Actual performance measurement - how long does real compilation take?
fn bench_real_world_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_world_performance");
    group.measurement_time(Duration::from_secs(15));

    // A more realistic program
    let realistic_program = r#"
func bubble_sort(size: i32) -> i32 {
    let swapped = true;
    let i = 0;
    while (swapped) {
        swapped = false;
        i = 1;
        while (i < size) {
            if (i > (i - 1)) {
                swapped = true;
            }
            i = i + 1;
        }
        size = size - 1;
    }
    return size;
}

func binary_search(target: i32, size: i32) -> i32 {
    let low = 0;
    let high = size - 1;
    
    while (low <= high) {
        let mid = low + ((high - low) / 2);
        
        if (mid == target) {
            return mid;
        }
        
        if (mid < target) {
            low = mid + 1;
        } else {
            high = mid - 1;
        }
    }
    
    return -1;
}

func main() -> () {
    let sorted_size = bubble_sort(100);
    let found_index = binary_search(50, sorted_size);
    return;
}
"#;

    // Measure compilation time
    group.bench_function("realistic_compilation", |b| {
        b.iter(|| {
            let start = Instant::now();
            let result = black_box(compile_to_ast(black_box(realistic_program)));
            let duration = start.elapsed();
            black_box((result, duration))
        })
    });

    group.finish();
}

criterion_group!(
    performance_benches,
    bench_lexer_speed,
    bench_parser_speed,
    bench_type_checker_speed,
    bench_full_compilation_speed,
    bench_memory_allocation_speed,
    bench_compilation_throughput,
    bench_comptime_engine_speed,
    bench_memory_analysis_speed,
    bench_optimization_levels,
    bench_real_world_performance
);

criterion_main!(performance_benches);