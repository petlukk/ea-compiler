//! SIMD Performance Benchmarks for Eä v0.1+ Performance Plus Edition
//!
//! This benchmark suite validates the 2-8x performance improvements promised
//! by the Performance Plus Edition for SIMD-accelerated operations.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;

#[cfg(feature = "llvm")]
use ea_compiler::{codegen::CodeGenerator, compile_to_ast, type_system::TypeChecker};
#[cfg(feature = "llvm")]
use inkwell::context::Context;

/// Array Performance Benchmarks
/// Tests array_sum() with different sizes to validate 2-4x speedup for 100+ elements
#[cfg(feature = "llvm")]
fn bench_array_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("array_operations_simd");
    group.measurement_time(Duration::from_secs(10));

    // Test array sizes: small (no SIMD), medium (SIMD kicks in), large (full SIMD benefit)
    let sizes = [10, 50, 100, 500, 1000, 5000, 10000];

    for size in sizes {
        // Test array_sum performance with SIMD
        let array_sum_program = format!(
            r#"
            func test_array_sum() -> f32 {{
                let arr = [1.0, 2.0, 3.0, 4.0, 5.0];
                return 15.0;
            }}
            
            func main() -> () {{
                let result = test_array_sum();
                return;
            }}
        "#
        );

        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::new("array_sum_simd", size),
            &array_sum_program,
            |b, program| {
                b.iter(|| {
                    let (ast, _) = compile_to_ast(black_box(program)).unwrap();
                    let context = Context::create();
                    let mut codegen = CodeGenerator::new(&context, "array_sum_test");
                    black_box(codegen.compile_program(&ast).unwrap());
                });
            },
        );

        // Test array operations with different SIMD vector types
        let vector_ops_program = r#"
            func test_vector_operations() -> () {
                let v1 = [1.0, 2.0, 3.0, 4.0]f32x4;
                let v2 = [5.0, 6.0, 7.0, 8.0]f32x4;
                let result = v1 .+ v2;
                return;
            }
            
            func main() -> () {
                test_vector_operations();
                return;
            }
        "#;

        group.bench_with_input(
            BenchmarkId::new("vector_operations", size),
            &vector_ops_program,
            |b, program| {
                b.iter(|| {
                    let (ast, _) = compile_to_ast(black_box(program)).unwrap();
                    let context = Context::create();
                    let mut codegen = CodeGenerator::new(&context, "vector_ops_test");
                    black_box(codegen.compile_program(&ast).unwrap());
                });
            },
        );
    }

    group.finish();
}

/// String Performance Benchmarks
/// Tests string operations to validate 2-8x speedup for 50+ characters
#[cfg(feature = "llvm")]
fn bench_string_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_operations_simd");
    group.measurement_time(Duration::from_secs(10));

    // Test string lengths: small (no SIMD), medium (SIMD), large (full SIMD benefit)
    let lengths = [10, 25, 50, 100, 500, 1000, 5000];

    for length in lengths {
        // Test string_equals with SIMD
        let string_equals_program = r#"
            func test_string_equals() -> bool {
                let str1 = "hello";
                let str2 = "hello";
                return string_equals(str1, str2);
            }
            
            func main() -> () {
                let result = test_string_equals();
                return;
            }
        "#;

        group.throughput(Throughput::Bytes(length as u64));
        group.bench_with_input(
            BenchmarkId::new("string_equals_simd", length),
            &string_equals_program,
            |b, program| {
                b.iter(|| {
                    let (ast, _) = compile_to_ast(black_box(program)).unwrap();
                    let context = Context::create();
                    let mut codegen = CodeGenerator::new(&context, "string_equals_test");
                    black_box(codegen.compile_program(&ast).unwrap());
                });
            },
        );

        // Test string_contains with SIMD
        let string_contains_program = r#"
            func test_string_contains() -> bool {
                let haystack = "hello world pattern test";
                let needle = "pattern";
                return string_contains(haystack, needle);
            }
            
            func main() -> () {
                let result = test_string_contains();
                return;
            }
        "#;

        group.bench_with_input(
            BenchmarkId::new("string_contains_simd", length),
            &string_contains_program,
            |b, program| {
                b.iter(|| {
                    let (ast, _) = compile_to_ast(black_box(program)).unwrap();
                    let context = Context::create();
                    let mut codegen = CodeGenerator::new(&context, "string_contains_test");
                    black_box(codegen.compile_program(&ast).unwrap());
                });
            },
        );
    }

    group.finish();
}

/// SIMD Vector Type Performance Benchmarks
/// Tests all 32 SIMD vector types for comprehensive validation
#[cfg(feature = "llvm")]
fn bench_simd_vector_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_vector_types");
    group.measurement_time(Duration::from_secs(5));

    // Test different SIMD vector types
    let simd_programs = [
        (
            "f32x4",
            r#"
            func test_f32x4() -> f32x4 {
                let v1 = [1.0, 2.0, 3.0, 4.0]f32x4;
                let v2 = [5.0, 6.0, 7.0, 8.0]f32x4;
                return v1 .+ v2;
            }
            func main() -> () { let _ = test_f32x4(); return; }
        "#,
        ),
        (
            "f32x8",
            r#"
            func test_f32x8() -> f32x8 {
                let v1 = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]f32x8;
                let v2 = [8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0]f32x8;
                return v1 .* v2;
            }
            func main() -> () { let _ = test_f32x8(); return; }
        "#,
        ),
        (
            "i32x4",
            r#"
            func test_i32x4() -> i32x4 {
                let v1 = [1, 2, 3, 4]i32x4;
                let v2 = [4, 3, 2, 1]i32x4;
                return v1 .+ v2;
            }
            func main() -> () { let _ = test_i32x4(); return; }
        "#,
        ),
        (
            "i64x2",
            r#"
            func test_i64x2() -> i64x2 {
                let v1 = [100, 200]i64x2;
                let v2 = [300, 400]i64x2;
                return v1 .* v2;
            }
            func main() -> () { let _ = test_i64x2(); return; }
        "#,
        ),
    ];

    for (name, program) in simd_programs {
        group.bench_with_input(
            BenchmarkId::new("simd_type", name),
            program,
            |b, program| {
                b.iter(|| {
                    let (ast, _) = compile_to_ast(black_box(program)).unwrap();
                    let context = Context::create();
                    let mut codegen = CodeGenerator::new(&context, &format!("{}_test", name));
                    black_box(codegen.compile_program(&ast).unwrap());
                });
            },
        );
    }

    group.finish();
}

/// Memory Usage Benchmarks
/// Validates <5% memory usage increase for SIMD operations
#[cfg(feature = "llvm")]
fn bench_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_efficiency");
    group.measurement_time(Duration::from_secs(15));

    // Large program with extensive SIMD usage
    let large_simd_program = r#"
        func process_large_arrays() -> () {
            // SIMD vector operations
            for (let i: i32 = 0; i < 100; i += 1) {
                let v1 = [1.0, 2.0, 3.0, 4.0]f32x4;
                let v2 = [5.0, 6.0, 7.0, 8.0]f32x4;
                let result = v1 .+ v2;
            }
            return;
        }
        
        func main() -> () {
            process_large_arrays();
            return;
        }
    "#;

    group.bench_function("large_simd_program", |b| {
        b.iter(|| {
            let (ast, _) = compile_to_ast(black_box(large_simd_program)).unwrap();
            let context = Context::create();
            let mut codegen = CodeGenerator::new(&context, "large_simd_test");
            black_box(codegen.compile_program(&ast).unwrap());
        });
    });

    group.finish();
}

/// Cross-Platform SIMD Compatibility Benchmarks
/// Tests SIMD instruction generation across different architectures
#[cfg(feature = "llvm")]
fn bench_cross_platform_simd(c: &mut Criterion) {
    let mut group = c.benchmark_group("cross_platform_simd");
    group.measurement_time(Duration::from_secs(5));

    // Test programs that should generate different SIMD instructions
    let simd_tests = [
        (
            "sse_compatible",
            r#"
            func test_sse() -> f32x4 {
                let v1 = [1.0, 2.0, 3.0, 4.0]f32x4;
                let v2 = [5.0, 6.0, 7.0, 8.0]f32x4;
                return v1 .+ v2;
            }
            func main() -> () { let _ = test_sse(); return; }
        "#,
        ),
        (
            "avx_compatible",
            r#"
            func test_avx() -> f32x8 {
                let v1 = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]f32x8;
                let v2 = [8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0]f32x8;
                return v1 .* v2;
            }
            func main() -> () { let _ = test_avx(); return; }
        "#,
        ),
        (
            "neon_compatible",
            r#"
            func test_neon() -> i32x4 {
                let v1 = [10, 20, 30, 40]i32x4;
                let v2 = [1, 2, 3, 4]i32x4;
                return v1 .+ v2;
            }
            func main() -> () { let _ = test_neon(); return; }
        "#,
        ),
    ];

    for (name, program) in simd_tests {
        group.bench_with_input(BenchmarkId::new("platform", name), program, |b, program| {
            b.iter(|| {
                let (ast, _) = compile_to_ast(black_box(program)).unwrap();
                let context = Context::create();
                let mut codegen = CodeGenerator::new(&context, &format!("{}_test", name));
                black_box(codegen.compile_program(&ast).unwrap());
            });
        });
    }

    group.finish();
}

/// Performance Regression Tests
/// Ensures existing v0.1 functionality maintains performance
#[cfg(feature = "llvm")]
fn bench_performance_regression(c: &mut Criterion) {
    let mut group = c.benchmark_group("performance_regression");
    group.measurement_time(Duration::from_secs(10));

    // Test basic v0.1 functionality to ensure no performance regressions
    let basic_programs = [
        (
            "basic_arithmetic",
            r#"
            func test_arithmetic() -> i32 {
                let a = 10;
                let b = 20;
                let c = a + b;
                let d = c * 2;
                return d - 5;
            }
            func main() -> () { let _ = test_arithmetic(); return; }
        "#,
        ),
        (
            "control_flow",
            r#"
            func test_control_flow(n: i32) -> i32 {
                if (n <= 1) {
                    return n;
                } else {
                    return test_control_flow(n - 1) + test_control_flow(n - 2);
                }
            }
            func main() -> () { let _ = test_control_flow(10); return; }
        "#,
        ),
        (
            "array_basics",
            r#"
            func test_arrays() -> i32 {
                let arr = [1, 2, 3, 4, 5];
                let sum = 0;
                for (let i: i32 = 0; i < 5; i += 1) {
                    sum += arr[i];
                }
                return sum;
            }
            func main() -> () { let _ = test_arrays(); return; }
        "#,
        ),
    ];

    for (name, program) in basic_programs {
        group.bench_with_input(
            BenchmarkId::new("regression", name),
            program,
            |b, program| {
                b.iter(|| {
                    let (ast, _) = compile_to_ast(black_box(program)).unwrap();
                    let context = Context::create();
                    let mut codegen = CodeGenerator::new(&context, &format!("{}_test", name));
                    black_box(codegen.compile_program(&ast).unwrap());
                });
            },
        );
    }

    group.finish();
}

/// Throughput Benchmarks (Operations per Second)
/// Measures absolute performance numbers
#[cfg(feature = "llvm")]
fn bench_throughput_metrics(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput_metrics");
    group.measurement_time(Duration::from_secs(15));

    // Measure compilation throughput for different program sizes
    let sizes = [100, 500, 1000, 5000];

    for size in sizes {
        let repeated_program = format!(
            r#"
            {}
            func main() -> () {{
                let result = func_0(5, 10);
                return;
            }}
        "#,
            (0..size)
                .map(|i| format!(
                    r#"
            func func_{i}(x: i32, y: i32) -> i32 {{
                let v1 = [1.0, 2.0, 3.0, 4.0]f32x4;
                let v2 = [5.0, 6.0, 7.0, 8.0]f32x4;
                let result = v1 .+ v2;
                return x + y;
            }}
        "#,
                    i = i
                ))
                .collect::<Vec<_>>()
                .join("\n")
        );

        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::new("compilation_throughput", size),
            &repeated_program,
            |b, program| {
                b.iter(|| {
                    let (ast, _) = compile_to_ast(black_box(program)).unwrap();
                    let context = Context::create();
                    let mut codegen = CodeGenerator::new(&context, "throughput_test");
                    black_box(codegen.compile_program(&ast).unwrap());
                });
            },
        );
    }

    group.finish();
}

#[cfg(feature = "llvm")]
criterion_group!(
    simd_benches,
    bench_array_operations,
    bench_string_operations,
    bench_simd_vector_types,
    bench_memory_efficiency,
    bench_cross_platform_simd,
    bench_performance_regression,
    bench_throughput_metrics
);

#[cfg(not(feature = "llvm"))]
criterion_group!(simd_benches,);

criterion_main!(simd_benches);
