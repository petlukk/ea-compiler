use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ea_compiler::*;
use std::time::Duration;

/// Cross-platform validation benchmarks for Week 11-12
/// Tests consistent performance across all target architectures

fn platform_compilation_consistency(c: &mut Criterion) {
    let mut group = c.benchmark_group("cross_platform_compilation");
    
    // Test programs of various sizes for platform consistency
    let small_program = r#"
        func main() -> i32 {
            let x = 42;
            let y = x * 2;
            return y;
        }
    "#;
    
    let medium_program = r#"
        func fibonacci(n: i32) -> i32 {
            if n <= 1 {
                return n;
            }
            return fibonacci(n - 1) + fibonacci(n - 2);
        }
        
        func main() -> i32 {
            let result = fibonacci(10);
            return result;
        }
    "#;
    
    let large_program = r#"
        func matrix_multiply(a: [i32; 16], b: [i32; 16]) -> [i32; 16] {
            let mut result = [0; 16];
            for i in 0..4 {
                for j in 0..4 {
                    let mut sum = 0;
                    for k in 0..4 {
                        sum += a[i * 4 + k] * b[k * 4 + j];
                    }
                    result[i * 4 + j] = sum;
                }
            }
            return result;
        }
        
        func main() -> i32 {
            let a = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
            let b = [16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1];
            let result = matrix_multiply(a, b);
            return result[0];
        }
    "#;
    
    // Benchmark compilation across different program sizes
    group.bench_function("small_program", |b| {
        b.iter(|| {
            compile_to_llvm(black_box(small_program), "small_program").unwrap()
        })
    });
    
    group.bench_function("medium_program", |b| {
        b.iter(|| {
            compile_to_llvm(black_box(medium_program), "medium_program").unwrap()
        })
    });
    
    group.bench_function("large_program", |b| {
        b.iter(|| {
            compile_to_llvm(black_box(large_program), "large_program").unwrap()
        })
    });
    
    group.finish();
}

fn simd_cross_platform_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_cross_platform");
    
    // SIMD program that should work across all platforms
    let simd_program = r#"
        func vector_add(a: f32x4, b: f32x4) -> f32x4 {
            return a + b;
        }
        
        func vector_multiply(a: f32x4, b: f32x4) -> f32x4 {
            return a * b;
        }
        
        func main() -> i32 {
            let a = f32x4(1.0, 2.0, 3.0, 4.0);
            let b = f32x4(5.0, 6.0, 7.0, 8.0);
            let sum = vector_add(a, b);
            let product = vector_multiply(a, b);
            return 0;
        }
    "#;
    
    group.bench_function("simd_compilation", |b| {
        b.iter(|| {
            compile_to_llvm(black_box(simd_program), "simd_program").unwrap()
        })
    });
    
    group.finish();
}

fn memory_management_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_management_validation");
    
    // Test memory-intensive operations
    let memory_program = r#"
        func allocate_large_array() -> [i32; 1000] {
            let mut arr = [0; 1000];
            for i in 0..1000 {
                arr[i] = i;
            }
            return arr;
        }
        
        func main() -> i32 {
            let arr = allocate_large_array();
            let mut sum = 0;
            for i in 0..1000 {
                sum += arr[i];
            }
            return sum;
        }
    "#;
    
    group.bench_function("memory_intensive", |b| {
        b.iter(|| {
            compile_to_llvm(black_box(memory_program), "memory_program").unwrap()
        })
    });
    
    group.finish();
}

fn error_handling_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling_validation");
    
    // Test error handling across platforms
    let error_cases = vec![
        "invalid syntax %^&*",
        "func main() -> i32 { return \"string\"; }",  // type mismatch
        "func main() -> i32 { undefined_var; }",      // undefined variable
        "func main() -> i32 { 1 + ; }",               // incomplete expression
    ];
    
    for (i, case) in error_cases.iter().enumerate() {
        group.bench_function(&format!("error_case_{}", i), |b| {
            b.iter(|| {
                // These should fail gracefully, not crash
                let _ = compile_to_llvm(black_box(case), &format!("error_case_{}", i));
            })
        });
    }
    
    group.finish();
}

fn platform_consistency_metrics(c: &mut Criterion) {
    let mut group = c.benchmark_group("platform_consistency");
    
    // Standard test program for consistency measurement
    let test_program = r#"
        func factorial(n: i32) -> i32 {
            if n <= 1 {
                return 1;
            }
            return n * factorial(n - 1);
        }
        
        func main() -> i32 {
            let result = factorial(5);
            return result;
        }
    "#;
    
    // Measure compilation phases separately for consistency analysis
    group.bench_function("lexer_consistency", |b| {
        b.iter(|| {
            tokenize(black_box(test_program)).unwrap()
        })
    });
    
    group.bench_function("parser_consistency", |b| {
        b.iter(|| {
            parse(black_box(test_program)).unwrap()
        })
    });
    
    group.bench_function("codegen_consistency", |b| {
        b.iter(|| {
            compile_to_llvm(black_box(test_program), "test_program").unwrap()
        })
    });
    
    group.finish();
}

criterion_group!(
    cross_platform_benches,
    platform_compilation_consistency,
    simd_cross_platform_validation,
    memory_management_validation,
    error_handling_validation,
    platform_consistency_metrics
);

criterion_main!(cross_platform_benches);