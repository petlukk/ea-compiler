//! Simple Performance Validation for Current Eä Implementation
//! 
//! This benchmark tests the actual working parts of the Eä compiler
//! to provide realistic performance measurements.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use ea_compiler::{tokenize, parse, compile_to_ast};
use std::time::Instant;

/// Test programs that we know work with current implementation
const SIMPLE_PROGRAM: &str = r#"
func add(a: i32, b: i32) -> i32 {
    return a + b;
}

func main() -> () {
    let result = add(5, 10);
    return;
}
"#;

const FIBONACCI_PROGRAM: &str = r#"
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

const LOOP_PROGRAM: &str = r#"
func sum_numbers(n: i32) -> i32 {
    let sum = 0;
    let i = 1;
    while (i <= n) {
        sum = sum + i;
        i = i + 1;
    }
    return sum;
}

func main() -> () {
    let result = sum_numbers(100);
    return;
}
"#;

/// Baseline performance of the lexer
fn bench_lexer_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer_performance");
    
    let test_cases = vec![
        ("simple", SIMPLE_PROGRAM),
        ("fibonacci", FIBONACCI_PROGRAM),
        ("loop", LOOP_PROGRAM),
    ];
    
    for (name, program) in test_cases {
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

/// Baseline performance of the parser
fn bench_parser_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_performance");
    
    let test_cases = vec![
        ("simple", SIMPLE_PROGRAM),
        ("fibonacci", FIBONACCI_PROGRAM),
        ("loop", LOOP_PROGRAM),
    ];
    
    for (name, program) in test_cases {
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

/// Full compilation pipeline performance (what actually works)
fn bench_full_compilation_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_compilation_performance");
    
    let test_cases = vec![
        ("simple", SIMPLE_PROGRAM),
        ("fibonacci", FIBONACCI_PROGRAM),
        ("loop", LOOP_PROGRAM),
    ];
    
    for (name, program) in test_cases {
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

/// Test compilation of larger programs to measure scalability
fn bench_scalability_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalability_performance");
    
    // Generate programs of different sizes
    let sizes = vec![10, 50, 100];
    
    for size in sizes {
        let program = generate_large_program(size);
        
        group.bench_with_input(
            BenchmarkId::new("large_program", size),
            &program,
            |b, program| {
                b.iter(|| {
                    black_box(parse(black_box(program)).unwrap())
                })
            },
        );
    }
    
    group.finish();
}

/// Test error handling performance
fn bench_error_handling_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling_performance");
    
    let error_cases = vec![
        ("syntax_error", "func main() -> i32 { invalid syntax"),
        ("type_mismatch", "func main() -> i32 { return \"string\"; }"),
        ("undefined_var", "func main() -> i32 { return undefined_var; }"),
    ];
    
    for (name, error_code) in error_cases {
        group.bench_function(name, |b| {
            b.iter(|| {
                // Measure how quickly we detect and report errors
                let _ = black_box(compile_to_ast(black_box(error_code)));
            })
        });
    }
    
    group.finish();
}

/// Memory and compilation efficiency test
fn bench_compilation_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("compilation_efficiency");
    
    // Test repeated compilation of the same program
    group.bench_function("repeated_compilation", |b| {
        b.iter(|| {
            for _ in 0..10 {
                let _ = black_box(compile_to_ast(black_box(FIBONACCI_PROGRAM)));
            }
        })
    });
    
    // Test compilation with many functions
    let many_functions_program = generate_many_functions_program(20);
    group.bench_function("many_functions", |b| {
        b.iter(|| {
            black_box(compile_to_ast(black_box(&many_functions_program)).unwrap())
        })
    });
    
    group.finish();
}

/// Measure actual compilation times for comparison
fn bench_real_compilation_times(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_compilation_times");
    
    let programs = vec![
        ("simple_add", SIMPLE_PROGRAM),
        ("recursive_fibonacci", FIBONACCI_PROGRAM),
        ("iterative_sum", LOOP_PROGRAM),
        ("complex_program", generate_complex_program()),
    ];
    
    for (name, program) in programs {
        group.bench_function(name, |b| {
            b.iter(|| {
                let start = Instant::now();
                let result = compile_to_ast(black_box(program));
                let duration = start.elapsed();
                black_box((result, duration))
            })
        });
    }
    
    group.finish();
}

// Helper functions to generate test programs

fn generate_large_program(num_functions: usize) -> String {
    let mut program = String::new();
    
    for i in 0..num_functions {
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

fn generate_many_functions_program(count: usize) -> String {
    let mut program = String::new();
    
    for i in 0..count {
        program.push_str(&format!(
            r#"
func process_{i}(input: i32) -> i32 {{
    let temp = input * 2;
    if (temp > 100) {{
        return temp - 50;
    }} else {{
        return temp + 10;
    }}
}}
"#,
            i = i
        ));
    }
    
    // Add main function that uses some of the generated functions
    program.push_str(
        r#"
func main() -> () {
    let result1 = process_0(10);
    let result2 = process_1(20);
    return;
}
"#,
    );
    
    program
}

fn generate_complex_program() -> &'static str {
    r#"
func calculate_factorial(n: i32) -> i32 {
    if (n <= 1) {
        return 1;
    }
    return n * calculate_factorial(n - 1);
}

func find_maximum(a: i32, b: i32, c: i32) -> i32 {
    let max_ab = a;
    if (b > max_ab) {
        max_ab = b;
    }
    
    let result = max_ab;
    if (c > result) {
        result = c;
    }
    
    return result;
}

func perform_calculations(n: i32) -> i32 {
    let factorial_result = calculate_factorial(n);
    let max_result = find_maximum(factorial_result, n * 2, n + 100);
    
    let sum = 0;
    let i = 1;
    while (i <= n) {
        sum = sum + i;
        i = i + 1;
    }
    
    return max_result + sum;
}

func main() -> () {
    let final_result = perform_calculations(10);
    return;
}
"#
}

criterion_group!(
    performance_validation_benches,
    bench_lexer_performance,
    bench_parser_performance,
    bench_full_compilation_performance,
    bench_scalability_performance,
    bench_error_handling_performance,
    bench_compilation_efficiency,
    bench_real_compilation_times
);

criterion_main!(performance_validation_benches);