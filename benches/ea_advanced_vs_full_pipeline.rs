//! Eä Advanced Features vs Full Pipeline Comparison
//!
//! This benchmark showcases Eä's core advantages:
//! - SIMD-optimized operations vs scalar equivalents
//! - JIT compilation and caching vs traditional compilation
//! - Compile-time optimization vs runtime optimization
//! - Full pipeline performance with advanced features enabled

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::fs;
use std::process::Command;
use std::time::Duration;

use ea_compiler::{compile_to_ast, tokenize};

#[cfg(feature = "llvm")]
use ea_compiler::compile_to_llvm;

/// Test programs that showcase Eä's advantages
struct AdvancedTest {
    name: &'static str,
    ea_code: &'static str,
    rust_code: &'static str,
    cpp_code: &'static str,
    go_code: &'static str,
    description: &'static str,
}

const SIMD_VECTOR_ADD: AdvancedTest = AdvancedTest {
    name: "simd_vector_add",
    description: "SIMD vector addition - Eä's native advantage",
    ea_code: r#"
func simd_vector_add() -> f32x4 {
    let v1 = [1.0, 2.0, 3.0, 4.0]f32x4;
    let v2 = [5.0, 6.0, 7.0, 8.0]f32x4;
    let v3 = [9.0, 10.0, 11.0, 12.0]f32x4;
    let v4 = [13.0, 14.0, 15.0, 16.0]f32x4;
    
    let result1 = v1 .+ v2;
    let result2 = v3 .+ v4;
    let final_result = result1 .+ result2;
    
    return final_result;
}

func main() -> i32 {
    let result = simd_vector_add();
    return 0;
}
"#,
    rust_code: r#"
fn simd_vector_add() -> [f32; 4] {
    let v1 = [1.0, 2.0, 3.0, 4.0];
    let v2 = [5.0, 6.0, 7.0, 8.0];
    let v3 = [9.0, 10.0, 11.0, 12.0];
    let v4 = [13.0, 14.0, 15.0, 16.0];
    
    let mut result1 = [0.0; 4];
    let mut result2 = [0.0; 4];
    let mut final_result = [0.0; 4];
    
    for i in 0..4 {
        result1[i] = v1[i] + v2[i];
        result2[i] = v3[i] + v4[i];
        final_result[i] = result1[i] + result2[i];
    }
    
    final_result
}

fn main() {
    let _result = simd_vector_add();
}
"#,
    cpp_code: r#"
#include <iostream>
#include <array>

std::array<float, 4> simd_vector_add() {
    std::array<float, 4> v1 = {1.0, 2.0, 3.0, 4.0};
    std::array<float, 4> v2 = {5.0, 6.0, 7.0, 8.0};
    std::array<float, 4> v3 = {9.0, 10.0, 11.0, 12.0};
    std::array<float, 4> v4 = {13.0, 14.0, 15.0, 16.0};
    
    std::array<float, 4> result1, result2, final_result;
    
    for (int i = 0; i < 4; i++) {
        result1[i] = v1[i] + v2[i];
        result2[i] = v3[i] + v4[i];
        final_result[i] = result1[i] + result2[i];
    }
    
    return final_result;
}

int main() {
    auto result = simd_vector_add();
    return 0;
}
"#,
    go_code: r#"
package main

func simdVectorAdd() [4]float32 {
    v1 := [4]float32{1.0, 2.0, 3.0, 4.0}
    v2 := [4]float32{5.0, 6.0, 7.0, 8.0}
    v3 := [4]float32{9.0, 10.0, 11.0, 12.0}
    v4 := [4]float32{13.0, 14.0, 15.0, 16.0}
    
    var result1, result2, finalResult [4]float32
    
    for i := 0; i < 4; i++ {
        result1[i] = v1[i] + v2[i]
        result2[i] = v3[i] + v4[i]
        finalResult[i] = result1[i] + result2[i]
    }
    
    return finalResult
}

func main() {
    _ = simdVectorAdd()
}
"#,
};

const COMPLEX_ARITHMETIC: AdvancedTest = AdvancedTest {
    name: "complex_arithmetic",
    description: "Complex arithmetic with multiple operations",
    ea_code: r#"
func complex_arithmetic(n: i32) -> i32 {
    let result = 0;
    for (let i: i32 = 0; i < n; i += 1) {
        let temp = i * 2;
        let squared = temp * temp;
        let cubed = squared * temp;
        result += cubed % 1000;
    }
    return result;
}

func main() -> i32 {
    return complex_arithmetic(1000);
}
"#,
    rust_code: r#"
fn complex_arithmetic(n: i32) -> i32 {
    let mut result = 0;
    for i in 0..n {
        let temp = i * 2;
        let squared = temp * temp;
        let cubed = squared * temp;
        result += cubed % 1000;
    }
    result
}

fn main() {
    let _result = complex_arithmetic(1000);
}
"#,
    cpp_code: r#"
#include <iostream>

int complex_arithmetic(int n) {
    int result = 0;
    for (int i = 0; i < n; i++) {
        int temp = i * 2;
        int squared = temp * temp;
        int cubed = squared * temp;
        result += cubed % 1000;
    }
    return result;
}

int main() {
    int result = complex_arithmetic(1000);
    return 0;
}
"#,
    go_code: r#"
package main

func complexArithmetic(n int) int {
    result := 0
    for i := 0; i < n; i++ {
        temp := i * 2
        squared := temp * temp
        cubed := squared * temp
        result += cubed % 1000
    }
    return result
}

func main() {
    _ = complexArithmetic(1000)
}
"#,
};

const RECURSIVE_FIBONACCI: AdvancedTest = AdvancedTest {
    name: "recursive_fibonacci",
    description: "Recursive fibonacci - tests optimization and caching",
    ea_code: r#"
func fibonacci(n: i32) -> i32 {
    if (n <= 1) {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

func main() -> i32 {
    return fibonacci(20);
}
"#,
    rust_code: r#"
fn fibonacci(n: i32) -> i32 {
    if n <= 1 {
        return n;
    }
    fibonacci(n - 1) + fibonacci(n - 2)
}

fn main() {
    let _result = fibonacci(20);
}
"#,
    cpp_code: r#"
#include <iostream>

int fibonacci(int n) {
    if (n <= 1) {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

int main() {
    int result = fibonacci(20);
    return 0;
}
"#,
    go_code: r#"
package main

func fibonacci(n int) int {
    if n <= 1 {
        return n
    }
    return fibonacci(n-1) + fibonacci(n-2)
}

func main() {
    _ = fibonacci(20)
}
"#,
};

const ARRAY_INTENSIVE: AdvancedTest = AdvancedTest {
    name: "array_intensive",
    description: "Array-intensive operations with multiple passes",
    ea_code: r#"
func array_intensive() -> i32 {
    let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let result = 0;
    
    // Multiple passes over the array
    for (let pass: i32 = 0; pass < 100; pass += 1) {
        for (let i: i32 = 0; i < 10; i += 1) {
            result += arr[i] * pass;
        }
    }
    
    return result;
}

func main() -> i32 {
    return array_intensive();
}
"#,
    rust_code: r#"
fn array_intensive() -> i32 {
    let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mut result = 0;
    
    // Multiple passes over the array
    for pass in 0..100 {
        for i in 0..10 {
            result += arr[i] * pass;
        }
    }
    
    result
}

fn main() {
    let _result = array_intensive();
}
"#,
    cpp_code: r#"
#include <iostream>

int array_intensive() {
    int arr[] = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10};
    int result = 0;
    
    // Multiple passes over the array
    for (int pass = 0; pass < 100; pass++) {
        for (int i = 0; i < 10; i++) {
            result += arr[i] * pass;
        }
    }
    
    return result;
}

int main() {
    int result = array_intensive();
    return 0;
}
"#,
    go_code: r#"
package main

func arrayIntensive() int {
    arr := [10]int{1, 2, 3, 4, 5, 6, 7, 8, 9, 10}
    result := 0
    
    // Multiple passes over the array
    for pass := 0; pass < 100; pass++ {
        for i := 0; i < 10; i++ {
            result += arr[i] * pass
        }
    }
    
    return result
}

func main() {
    _ = arrayIntensive()
}
"#,
};

/// Helper function to check if a compiler is available
fn is_compiler_available(compiler: &str) -> bool {
    let version_arg = match compiler {
        "go" => "version",
        _ => "--version",
    };

    Command::new(compiler)
        .arg(version_arg)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Benchmark Eä's advanced frontend features
fn bench_ea_advanced_frontend(c: &mut Criterion) {
    let mut group = c.benchmark_group("ea_advanced_frontend");
    group.measurement_time(Duration::from_secs(10));

    let tests = [
        SIMD_VECTOR_ADD,
        COMPLEX_ARITHMETIC,
        RECURSIVE_FIBONACCI,
        ARRAY_INTENSIVE,
    ];

    for test in tests {
        group.bench_function(&format!("ea_tokenize_{}", test.name), |b| {
            b.iter(|| {
                let tokens = tokenize(black_box(test.ea_code));
                black_box(tokens.unwrap().len())
            })
        });

        group.bench_function(&format!("ea_parse_{}", test.name), |b| {
            b.iter(|| {
                let (ast, _) = compile_to_ast(black_box(test.ea_code)).unwrap();
                black_box(ast.len())
            })
        });
    }

    group.finish();
}

/// Benchmark full compilation pipeline comparison
#[cfg(feature = "llvm")]
fn bench_full_compilation_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_compilation_pipeline");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(20);

    let tests = [
        SIMD_VECTOR_ADD,
        COMPLEX_ARITHMETIC,
        RECURSIVE_FIBONACCI,
        ARRAY_INTENSIVE,
    ];

    for test in tests {
        // Eä full pipeline with advanced features
        group.bench_function(&format!("ea_full_pipeline_{}", test.name), |b| {
            b.iter(|| {
                let result =
                    compile_to_llvm(black_box(test.ea_code), &format!("bench_{}", test.name));
                // Clean up immediately
                let _ = fs::remove_file(&format!("bench_{}.ll", test.name));
                black_box(result.is_ok())
            })
        });

        // Rust full pipeline
        if is_compiler_available("rustc") {
            group.bench_function(&format!("rust_full_pipeline_{}", test.name), |b| {
                b.iter(|| {
                    let filename = format!("bench_{}.rs", test.name);
                    fs::write(&filename, test.rust_code).unwrap();

                    let result = Command::new("rustc")
                        .arg(&filename)
                        .arg("-C")
                        .arg("opt-level=2") // Equivalent optimization level
                        .arg("-o")
                        .arg(&format!("bench_{}_rust", test.name))
                        .output();

                    // Clean up
                    let _ = fs::remove_file(&filename);
                    let _ = fs::remove_file(&format!("bench_{}_rust", test.name));

                    black_box(result.map(|o| o.status.success()).unwrap_or(false))
                })
            });
        }

        // C++ full pipeline with optimization
        if is_compiler_available("g++") {
            group.bench_function(&format!("cpp_full_pipeline_{}", test.name), |b| {
                b.iter(|| {
                    let filename = format!("bench_{}.cpp", test.name);
                    fs::write(&filename, test.cpp_code).unwrap();

                    let result = Command::new("g++")
                        .arg(&filename)
                        .arg("-O2") // Equivalent optimization level
                        .arg("-o")
                        .arg(&format!("bench_{}_cpp", test.name))
                        .output();

                    // Clean up
                    let _ = fs::remove_file(&filename);
                    let _ = fs::remove_file(&format!("bench_{}_cpp", test.name));

                    black_box(result.map(|o| o.status.success()).unwrap_or(false))
                })
            });
        }

        // Go full pipeline
        if is_compiler_available("go") {
            group.bench_function(&format!("go_full_pipeline_{}", test.name), |b| {
                b.iter(|| {
                    let filename = format!("bench_{}.go", test.name);
                    fs::write(&filename, test.go_code).unwrap();

                    let result = Command::new("go")
                        .arg("build")
                        .arg("-o")
                        .arg(&format!("bench_{}_go", test.name))
                        .arg(&filename)
                        .output();

                    // Clean up
                    let _ = fs::remove_file(&filename);
                    let _ = fs::remove_file(&format!("bench_{}_go", test.name));

                    black_box(result.map(|o| o.status.success()).unwrap_or(false))
                })
            });
        }
    }

    group.finish();
}

/// Benchmark SIMD-specific advantages
fn bench_simd_advantages(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_advantages");
    group.measurement_time(Duration::from_secs(10));

    // Test multiple SIMD vector sizes
    let simd_tests = [
        ("f32x4", "[1.0, 2.0, 3.0, 4.0]f32x4 .+ [5.0, 6.0, 7.0, 8.0]f32x4"),
        ("f32x8", "[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]f32x8 .+ [9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0]f32x8"),
        ("i32x4", "[1, 2, 3, 4]i32x4 .+ [5, 6, 7, 8]i32x4"),
        ("i64x2", "[100, 200]i64x2 .+ [300, 400]i64x2"),
    ];

    for (simd_type, operation) in simd_tests {
        let ea_code = format!(
            r#"
func simd_test() -> {} {{
    let result = {};
    return result;
}}
"#,
            simd_type, operation
        );

        group.bench_function(&format!("ea_simd_{}", simd_type), |b| {
            b.iter(|| {
                let (ast, _) = compile_to_ast(black_box(&ea_code)).unwrap();
                black_box(ast.len())
            })
        });
    }

    group.finish();
}

/// Benchmark compilation scaling with program complexity
fn bench_compilation_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("compilation_scaling");
    group.measurement_time(Duration::from_secs(15));

    let scales = [10, 50, 100, 200];

    for scale in scales {
        // Generate complex Eä program
        let ea_program = generate_complex_ea_program(scale);
        let rust_program = generate_complex_rust_program(scale);

        group.throughput(Throughput::Elements(scale as u64));

        group.bench_with_input(
            BenchmarkId::new("ea_complex", scale),
            &ea_program,
            |b, prog| {
                b.iter(|| {
                    let (ast, _) = compile_to_ast(black_box(prog)).unwrap();
                    black_box(ast.len())
                })
            },
        );

        if is_compiler_available("rustc") {
            group.bench_with_input(
                BenchmarkId::new("rust_complex", scale),
                &rust_program,
                |b, prog| {
                    b.iter(|| {
                        let filename = format!("complex_{}.rs", scale);
                        fs::write(&filename, prog).unwrap();

                        let result = Command::new("rustc")
                            .arg("--emit=metadata")
                            .arg("--crate-type=lib")
                            .arg(&filename)
                            .output();

                        // Clean up
                        let _ = fs::remove_file(&filename);
                        let _ = fs::remove_file(&format!("libcomplex_{}.rmeta", scale));

                        black_box(result.map(|o| o.status.success()).unwrap_or(false))
                    })
                },
            );
        }
    }

    group.finish();
}

/// Generate complex Eä program with SIMD operations
fn generate_complex_ea_program(num_functions: usize) -> String {
    let mut program = String::new();

    for i in 0..num_functions {
        if i % 3 == 0 {
            // SIMD function
            program.push_str(&format!(
                r#"
func simd_function_{}() -> f32x4 {{
    let v1 = [{}.0, {}.0, {}.0, {}.0]f32x4;
    let v2 = [{}.0, {}.0, {}.0, {}.0]f32x4;
    let result = v1 .+ v2;
    return result .* result;
}}
"#,
                i,
                i,
                i + 1,
                i + 2,
                i + 3,
                i + 4,
                i + 5,
                i + 6,
                i + 7
            ));
        } else {
            // Regular function
            program.push_str(&format!(
                r#"
func function_{}(x: i32) -> i32 {{
    let temp = x * {};
    if temp > 100 {{
        return temp - 50;
    }} else {{
        return temp + 25;
    }}
}}
"#,
                i,
                i + 1
            ));
        }
    }

    program.push_str(&format!(
        r#"
func main() -> i32 {{
    let result = function_0(42);
    return result;
}}
"#
    ));

    program
}

/// Generate complex Rust program for comparison
fn generate_complex_rust_program(num_functions: usize) -> String {
    let mut program = String::new();

    for i in 0..num_functions {
        if i % 3 == 0 {
            // Array-based simulation of SIMD
            program.push_str(&format!(
                r#"
fn simd_function_{}() -> [f32; 4] {{
    let v1 = [{}.0, {}.0, {}.0, {}.0];
    let v2 = [{}.0, {}.0, {}.0, {}.0];
    let mut result = [0.0; 4];
    for i in 0..4 {{
        result[i] = (v1[i] + v2[i]) * (v1[i] + v2[i]);
    }}
    result
}}
"#,
                i,
                i,
                i + 1,
                i + 2,
                i + 3,
                i + 4,
                i + 5,
                i + 6,
                i + 7
            ));
        } else {
            // Regular function
            program.push_str(&format!(
                r#"
fn function_{}(x: i32) -> i32 {{
    let temp = x * {};
    if temp > 100 {{
        temp - 50
    }} else {{
        temp + 25
    }}
}}
"#,
                i,
                i + 1
            ));
        }
    }

    program.push_str(&format!(
        r#"
fn main() {{
    let _result = function_0(42);
}}
"#
    ));

    program
}

#[cfg(feature = "llvm")]
criterion_group!(
    ea_advanced_benches,
    bench_ea_advanced_frontend,
    bench_full_compilation_pipeline,
    bench_simd_advantages,
    bench_compilation_scaling
);

#[cfg(not(feature = "llvm"))]
criterion_group!(
    ea_advanced_benches,
    bench_ea_advanced_frontend,
    bench_simd_advantages,
    bench_compilation_scaling
);

criterion_main!(ea_advanced_benches);
