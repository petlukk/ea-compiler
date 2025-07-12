//! Frontend Performance Benchmarks
//!
//! These benchmarks measure the parsing and AST generation performance
//! of the Eä compiler frontend, without LLVM context issues.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use ea_compiler::{compile_to_ast, tokenize};

/// Benchmark tokenization performance
fn bench_tokenization(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenization");

    let test_programs = vec![
        ("simple", "func main() -> i32 { return 42; }"),
        (
            "arithmetic",
            r#"
func calculate() -> i32 {
    let a = 10;
    let b = 20;
    let c = a + b;
    let d = c * 2;
    return d - 5;
}
"#,
        ),
        (
            "control_flow",
            r#"
func fibonacci(n: i32) -> i32 {
    if (n <= 1) {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}
"#,
        ),
        (
            "simd",
            r#"
func simd_operations() -> f32x4 {
    let v1 = [1.0, 2.0, 3.0, 4.0]f32x4;
    let v2 = [5.0, 6.0, 7.0, 8.0]f32x4;
    let v3 = v1 .+ v2;
    let v4 = v1 .* v2;
    return v3 .+ v4;
}
"#,
        ),
    ];

    for (name, source) in test_programs {
        group.bench_function(name, |b| {
            b.iter(|| {
                let tokens = tokenize(black_box(source));
                black_box(tokens.unwrap().len())
            })
        });
    }

    group.finish();
}

/// Benchmark AST parsing performance
fn bench_ast_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("ast_parsing");

    let test_programs = vec![
        ("simple", "func main() -> i32 { return 42; }"),
        (
            "multiple_functions",
            r#"
func helper(x: i32) -> i32 {
    return x * 2;
}

func main() -> i32 {
    return helper(21);
}
"#,
        ),
        (
            "loops_and_arrays",
            r#"
func array_sum() -> i32 {
    let arr = [1, 2, 3, 4, 5];
    let sum = 0;
    for (let i: i32 = 0; i < 5; i += 1) {
        sum += arr[i];
    }
    return sum;
}
"#,
        ),
        (
            "nested_control_flow",
            r#"
func complex_function(n: i32) -> i32 {
    let result = 0;
    for (let i: i32 = 0; i < n; i += 1) {
        if (i % 2 == 0) {
            result += i;
        } else {
            while (result > 100) {
                result -= 10;
            }
        }
    }
    return result;
}
"#,
        ),
    ];

    for (name, source) in test_programs {
        group.bench_function(name, |b| {
            b.iter(|| {
                let (ast, _) = compile_to_ast(black_box(source)).unwrap();
                black_box(ast.len())
            })
        });
    }

    group.finish();
}

/// Benchmark parsing of different program sizes
fn bench_program_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("program_sizes");

    let sizes = vec![1, 5, 10, 25, 50];

    for size in sizes {
        let program = generate_program(size);

        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::new("functions", size), &program, |b, prog| {
            b.iter(|| {
                let (ast, _) = compile_to_ast(black_box(prog)).unwrap();
                black_box(ast.len())
            })
        });
    }

    group.finish();
}

/// Benchmark SIMD parsing performance
fn bench_simd_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_parsing");

    let simd_types = vec![
        ("f32x4", "[1.0, 2.0, 3.0, 4.0]f32x4"),
        ("f32x8", "[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]f32x8"),
        ("i32x4", "[1, 2, 3, 4]i32x4"),
        ("i64x2", "[100, 200]i64x2"),
    ];

    for (type_name, vector_literal) in simd_types {
        let source = format!(
            r#"
func test_simd() -> {} {{
    let v1 = {};
    let v2 = {};
    return v1 .+ v2;
}}
"#,
            type_name, vector_literal, vector_literal
        );

        group.bench_function(type_name, |b| {
            b.iter(|| {
                let (ast, _) = compile_to_ast(black_box(&source)).unwrap();
                black_box(ast.len())
            })
        });
    }

    group.finish();
}

/// Benchmark error detection speed
fn bench_error_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_detection");

    let error_cases = vec![
        ("syntax_error", "func main() -> i32 { invalid syntax"),
        (
            "type_error",
            r#"func main() -> i32 { return "not_an_int"; }"#,
        ),
        ("missing_return", r#"func test() -> i32 { let x = 5; }"#),
        (
            "undefined_function",
            r#"func main() -> i32 { return undefined(); }"#,
        ),
    ];

    for (name, source) in error_cases {
        group.bench_function(name, |b| {
            b.iter(|| {
                let result = compile_to_ast(black_box(source));
                black_box(result.is_err())
            })
        });
    }

    group.finish();
}

// Helper function to generate programs of different sizes
fn generate_program(num_functions: usize) -> String {
    let mut program = String::new();

    for i in 0..num_functions {
        program.push_str(&format!(
            r#"
func function_{}(x: i32) -> i32 {{
    let temp = x + {};
    if temp > 50 {{
        return temp - 10;
    }} else {{
        return temp + 5;
    }}
}}
"#,
            i, i
        ));
    }

    if num_functions > 0 {
        program.push_str(&format!(
            r#"
func main() -> i32 {{
    return function_0(42);
}}
"#
        ));
    }

    program
}

criterion_group!(
    frontend_benches,
    bench_tokenization,
    bench_ast_parsing,
    bench_program_sizes,
    bench_simd_parsing,
    bench_error_detection
);

criterion_main!(frontend_benches);
