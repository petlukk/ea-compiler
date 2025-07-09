//! Compilation Performance Benchmarks
//! 
//! These benchmarks measure LLVM compilation performance safely,
//! with proper resource management and no context leaks.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::fs;

#[cfg(feature = "llvm")]
use ea_compiler::compile_to_llvm;

/// Benchmark safe LLVM compilation
#[cfg(feature = "llvm")]
fn bench_llvm_compilation(c: &mut Criterion) {
    let mut group = c.benchmark_group("llvm_compilation");
    
    let test_programs = vec![
        ("simple", "simple_test", r#"
func main() -> i32 {
    return 42;
}
"#),
        ("arithmetic", "arithmetic_test", r#"
func calculate() -> i32 {
    let a = 10;
    let b = 20;
    let c = a + b;
    let d = c * 2;
    return d - 5;
}
"#),
        ("control_flow", "control_test", r#"
func abs_value(x: i32) -> i32 {
    if (x >= 0) {
        return x;
    } else {
        return -x;
    }
}
"#),
        ("simple_loop", "loop_test", r#"
func sum_numbers() -> i32 {
    let sum = 0;
    for (let i: i32 = 1; i <= 10; i += 1) {
        sum += i;
    }
    return sum;
}
"#),
    ];

    for (name, filename, source) in test_programs {
        group.bench_function(name, |b| {
            b.iter(|| {
                let result = compile_to_llvm(black_box(source), filename);
                // Clean up immediately
                let _ = fs::remove_file(&format!("{}.ll", filename));
                black_box(result.is_ok())
            })
        });
    }
    
    group.finish();
}

/// Benchmark SIMD compilation (safe, single operations)
#[cfg(feature = "llvm")]
fn bench_simd_compilation(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_compilation");
    
    let simd_programs = vec![
        ("f32x4_add", "f32x4_test", r#"
func simd_add() -> f32x4 {
    let v1 = [1.0, 2.0, 3.0, 4.0]f32x4;
    let v2 = [5.0, 6.0, 7.0, 8.0]f32x4;
    return v1 .+ v2;
}
"#),
        ("i32x4_multiply", "i32x4_test", r#"
func simd_multiply() -> i32x4 {
    let v1 = [1, 2, 3, 4]i32x4;
    let v2 = [2, 3, 4, 5]i32x4;
    return v1 .* v2;
}
"#),
    ];

    for (name, filename, source) in simd_programs {
        group.bench_function(name, |b| {
            b.iter(|| {
                let result = compile_to_llvm(black_box(source), filename);
                // Clean up immediately
                let _ = fs::remove_file(&format!("{}.ll", filename));
                black_box(result.is_ok())
            })
        });
    }
    
    group.finish();
}

/// Benchmark compilation of different program complexities
#[cfg(feature = "llvm")]
fn bench_compilation_complexity(c: &mut Criterion) {
    let mut group = c.benchmark_group("compilation_complexity");
    
    let complexities = vec![1, 3, 5];
    
    for complexity in complexities {
        let program = generate_complex_program(complexity);
        let filename = format!("complex_{}", complexity);
        
        group.bench_with_input(
            BenchmarkId::new("functions", complexity),
            &(program, filename),
            |b, (prog, fname)| {
                b.iter(|| {
                    let result = compile_to_llvm(black_box(prog), fname);
                    // Clean up immediately
                    let _ = fs::remove_file(&format!("{}.ll", fname));
                    black_box(result.is_ok())
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark error detection during compilation
#[cfg(feature = "llvm")]
fn bench_compilation_errors(c: &mut Criterion) {
    let mut group = c.benchmark_group("compilation_errors");
    
    let error_cases = vec![
        ("type_error", "type_error_test", r#"
func bad_function() -> i32 {
    return "not an integer";
}
"#),
        ("undefined_variable", "undefined_test", r#"
func bad_function() -> i32 {
    return undefined_variable;
}
"#),
    ];

    for (name, filename, source) in error_cases {
        group.bench_function(name, |b| {
            b.iter(|| {
                let result = compile_to_llvm(black_box(source), filename);
                // Clean up (might not exist due to error)
                let _ = fs::remove_file(&format!("{}.ll", filename));
                black_box(result.is_err())
            })
        });
    }
    
    group.finish();
}

// Helper function to generate programs of different complexities
fn generate_complex_program(complexity: usize) -> String {
    let mut program = String::new();
    
    for i in 0..complexity {
        program.push_str(&format!(
            r#"
func helper_{}(x: i32) -> i32 {{
    let temp = x + {};
    if temp > 25 {{
        return temp * 2;
    }} else {{
        let sum = 0;
        for (let j: i32 = 0; j < temp; j += 1) {{
            sum += j;
        }}
        return sum;
    }}
}}
"#,
            i, i * 10
        ));
    }
    
    program.push_str(r#"
func main() -> i32 {
    return helper_0(5);
}
"#);
    
    program
}

#[cfg(feature = "llvm")]
criterion_group!(
    compilation_benches,
    bench_llvm_compilation,
    bench_simd_compilation,
    bench_compilation_complexity,
    bench_compilation_errors
);

#[cfg(not(feature = "llvm"))]
criterion_group!(compilation_benches,);

criterion_main!(compilation_benches);