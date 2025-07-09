//! Cross-Language Performance Comparison
//! 
//! This benchmark provides FAIR comparisons between Eä and other languages:
//! - Rust, C++, Go vs Eä
//! - Equivalent algorithms and data structures
//! - Full compilation pipeline comparisons
//! - Honest reporting of strengths and weaknesses

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs;
use std::process::Command;
use std::time::Duration;

#[cfg(feature = "llvm")]
use ea_compiler::compile_to_llvm;

/// Test program definitions - identical algorithms across languages
struct CrossLanguageTest {
    name: &'static str,
    ea_code: &'static str,
    rust_code: &'static str,
    cpp_code: &'static str,
    go_code: &'static str,
}

const FIBONACCI_TEST: CrossLanguageTest = CrossLanguageTest {
    name: "fibonacci",
    ea_code: r#"
func fibonacci(n: i32) -> i32 {
    if (n <= 1) {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

func main() -> i32 {
    return fibonacci(15);
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
    println!("{}", fibonacci(15));
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
    std::cout << fibonacci(15) << std::endl;
    return 0;
}
"#,
    go_code: r#"
package main

import "fmt"

func fibonacci(n int) int {
    if n <= 1 {
        return n
    }
    return fibonacci(n-1) + fibonacci(n-2)
}

func main() {
    fmt.Println(fibonacci(15))
}
"#,
};

const FACTORIAL_TEST: CrossLanguageTest = CrossLanguageTest {
    name: "factorial",
    ea_code: r#"
func factorial(n: i32) -> i32 {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}

func main() -> i32 {
    return factorial(10);
}
"#,
    rust_code: r#"
fn factorial(n: i32) -> i32 {
    if n <= 1 {
        return 1;
    }
    n * factorial(n - 1)
}

fn main() {
    println!("{}", factorial(10));
}
"#,
    cpp_code: r#"
#include <iostream>

int factorial(int n) {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}

int main() {
    std::cout << factorial(10) << std::endl;
    return 0;
}
"#,
    go_code: r#"
package main

import "fmt"

func factorial(n int) int {
    if n <= 1 {
        return 1
    }
    return n * factorial(n-1)
}

func main() {
    fmt.Println(factorial(10))
}
"#,
};

const ARRAY_SUM_TEST: CrossLanguageTest = CrossLanguageTest {
    name: "array_sum",
    ea_code: r#"
func array_sum() -> i32 {
    let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let sum = 0;
    for (let i: i32 = 0; i < 10; i += 1) {
        sum += arr[i];
    }
    return sum;
}

func main() -> i32 {
    return array_sum();
}
"#,
    rust_code: r#"
fn array_sum() -> i32 {
    let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mut sum = 0;
    for i in 0..10 {
        sum += arr[i];
    }
    sum
}

fn main() {
    println!("{}", array_sum());
}
"#,
    cpp_code: r#"
#include <iostream>

int array_sum() {
    int arr[] = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10};
    int sum = 0;
    for (int i = 0; i < 10; i++) {
        sum += arr[i];
    }
    return sum;
}

int main() {
    std::cout << array_sum() << std::endl;
    return 0;
}
"#,
    go_code: r#"
package main

import "fmt"

func arraySum() int {
    arr := []int{1, 2, 3, 4, 5, 6, 7, 8, 9, 10}
    sum := 0
    for i := 0; i < 10; i++ {
        sum += arr[i]
    }
    return sum
}

func main() {
    fmt.Println(arraySum())
}
"#,
};

const ITERATIVE_LOOP_TEST: CrossLanguageTest = CrossLanguageTest {
    name: "iterative_loop",
    ea_code: r#"
func sum_to_n(n: i32) -> i32 {
    let sum = 0;
    for (let i: i32 = 1; i <= n; i += 1) {
        sum += i;
    }
    return sum;
}

func main() -> i32 {
    return sum_to_n(1000);
}
"#,
    rust_code: r#"
fn sum_to_n(n: i32) -> i32 {
    let mut sum = 0;
    for i in 1..=n {
        sum += i;
    }
    sum
}

fn main() {
    println!("{}", sum_to_n(1000));
}
"#,
    cpp_code: r#"
#include <iostream>

int sum_to_n(int n) {
    int sum = 0;
    for (int i = 1; i <= n; i++) {
        sum += i;
    }
    return sum;
}

int main() {
    std::cout << sum_to_n(1000) << std::endl;
    return 0;
}
"#,
    go_code: r#"
package main

import "fmt"

func sumToN(n int) int {
    sum := 0
    for i := 1; i <= n; i++ {
        sum += i
    }
    return sum
}

func main() {
    fmt.Println(sumToN(1000))
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

/// Benchmark compilation speed comparison
fn bench_compilation_speed(c: &mut Criterion) {
    let mut group = c.benchmark_group("compilation_speed");
    group.measurement_time(Duration::from_secs(20));
    
    let tests = [FIBONACCI_TEST, FACTORIAL_TEST, ARRAY_SUM_TEST, ITERATIVE_LOOP_TEST];
    
    for test in tests {
        // Eä compilation
        #[cfg(feature = "llvm")]
        group.bench_function(&format!("ea_{}", test.name), |b| {
            b.iter(|| {
                let result = compile_to_llvm(black_box(test.ea_code), &format!("bench_{}", test.name));
                // Clean up immediately
                let _ = fs::remove_file(&format!("bench_{}.ll", test.name));
                black_box(result.is_ok())
            })
        });
        
        // Rust compilation
        if is_compiler_available("rustc") {
            group.bench_function(&format!("rust_{}", test.name), |b| {
                b.iter(|| {
                    let filename = format!("bench_{}.rs", test.name);
                    fs::write(&filename, test.rust_code).unwrap();
                    
                    let result = Command::new("rustc")
                        .arg(&filename)
                        .arg("--emit=llvm-ir")
                        .arg("-C").arg("opt-level=0") // Fair comparison - no optimization
                        .arg("-o").arg(&format!("bench_{}.ll", test.name))
                        .output();
                    
                    // Clean up
                    let _ = fs::remove_file(&filename);
                    let _ = fs::remove_file(&format!("bench_{}.ll", test.name));
                    
                    black_box(result.map(|o| o.status.success()).unwrap_or(false))
                })
            });
        }
        
        // C++ compilation
        if is_compiler_available("clang++") {
            group.bench_function(&format!("cpp_{}", test.name), |b| {
                b.iter(|| {
                    let filename = format!("bench_{}.cpp", test.name);
                    fs::write(&filename, test.cpp_code).unwrap();
                    
                    let result = Command::new("clang++")
                        .arg(&filename)
                        .arg("-S")
                        .arg("-emit-llvm")
                        .arg("-O0") // Fair comparison - no optimization
                        .arg("-o").arg(&format!("bench_{}.ll", test.name))
                        .output();
                    
                    // Clean up
                    let _ = fs::remove_file(&filename);
                    let _ = fs::remove_file(&format!("bench_{}.ll", test.name));
                    
                    black_box(result.map(|o| o.status.success()).unwrap_or(false))
                })
            });
        }
        
        // Go compilation
        if is_compiler_available("go") {
            group.bench_function(&format!("go_{}", test.name), |b| {
                b.iter(|| {
                    let filename = format!("bench_{}.go", test.name);
                    fs::write(&filename, test.go_code).unwrap();
                    
                    let result = Command::new("go")
                        .arg("tool")
                        .arg("compile")
                        .arg("-N") // No optimization
                        .arg("-l") // No inlining
                        .arg(&filename)
                        .output();
                    
                    // Clean up
                    let _ = fs::remove_file(&filename);
                    let _ = fs::remove_file(&format!("bench_{}.o", test.name));
                    
                    black_box(result.map(|o| o.status.success()).unwrap_or(false))
                })
            });
        }
    }
    
    group.finish();
}

/// Benchmark full compilation pipeline (source -> executable)
fn bench_full_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_compilation_pipeline");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(10);
    
    let tests = [FIBONACCI_TEST, FACTORIAL_TEST, ARRAY_SUM_TEST];
    
    for test in tests {
        // Eä full pipeline
        #[cfg(feature = "llvm")]
        group.bench_function(&format!("ea_full_{}", test.name), |b| {
            b.iter(|| {
                // Step 1: Eä compilation
                let ir_result = compile_to_llvm(black_box(test.ea_code), &format!("full_{}", test.name));
                
                let success = if ir_result.is_ok() {
                    // Step 2: LLVM backend
                    let llc_result = Command::new("llc")
                        .arg(&format!("full_{}.ll", test.name))
                        .arg("-o").arg(&format!("full_{}.s", test.name))
                        .output();
                    
                    if llc_result.is_ok() {
                        // Step 3: Linking
                        let link_result = Command::new("gcc")
                            .arg(&format!("full_{}.s", test.name))
                            .arg("-o").arg(&format!("full_{}_ea", test.name))
                            .output();
                        
                        link_result.is_ok()
                    } else {
                        false
                    }
                } else {
                    false
                };
                
                // Clean up
                let _ = fs::remove_file(&format!("full_{}.ll", test.name));
                let _ = fs::remove_file(&format!("full_{}.s", test.name));
                let _ = fs::remove_file(&format!("full_{}_ea", test.name));
                
                black_box(success)
            })
        });
        
        // Rust full pipeline
        if is_compiler_available("rustc") {
            group.bench_function(&format!("rust_full_{}", test.name), |b| {
                b.iter(|| {
                    let filename = format!("full_{}.rs", test.name);
                    fs::write(&filename, test.rust_code).unwrap();
                    
                    let result = Command::new("rustc")
                        .arg(&filename)
                        .arg("-C").arg("opt-level=1") // Minimal optimization
                        .arg("-o").arg(&format!("full_{}_rust", test.name))
                        .output();
                    
                    // Clean up
                    let _ = fs::remove_file(&filename);
                    let _ = fs::remove_file(&format!("full_{}_rust", test.name));
                    
                    black_box(result.map(|o| o.status.success()).unwrap_or(false))
                })
            });
        }
        
        // C++ full pipeline
        if is_compiler_available("g++") {
            group.bench_function(&format!("cpp_full_{}", test.name), |b| {
                b.iter(|| {
                    let filename = format!("full_{}.cpp", test.name);
                    fs::write(&filename, test.cpp_code).unwrap();
                    
                    let result = Command::new("g++")
                        .arg(&filename)
                        .arg("-O1") // Minimal optimization
                        .arg("-o").arg(&format!("full_{}_cpp", test.name))
                        .output();
                    
                    // Clean up
                    let _ = fs::remove_file(&filename);
                    let _ = fs::remove_file(&format!("full_{}_cpp", test.name));
                    
                    black_box(result.map(|o| o.status.success()).unwrap_or(false))
                })
            });
        }
        
        // Go full pipeline
        if is_compiler_available("go") {
            group.bench_function(&format!("go_full_{}", test.name), |b| {
                b.iter(|| {
                    let filename = format!("full_{}.go", test.name);
                    fs::write(&filename, test.go_code).unwrap();
                    
                    let result = Command::new("go")
                        .arg("build")
                        .arg("-o").arg(&format!("full_{}_go", test.name))
                        .arg(&filename)
                        .output();
                    
                    // Clean up
                    let _ = fs::remove_file(&filename);
                    let _ = fs::remove_file(&format!("full_{}_go", test.name));
                    
                    black_box(result.map(|o| o.status.success()).unwrap_or(false))
                })
            });
        }
    }
    
    group.finish();
}

/// Benchmark runtime performance of compiled executables
fn bench_runtime_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("runtime_performance");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);
    
    // Pre-compile all programs
    let tests = [FIBONACCI_TEST, FACTORIAL_TEST, ARRAY_SUM_TEST];
    
    for test in tests {
        // Compile all versions first
        let ea_compiled = compile_ea_executable(&test);
        let rust_compiled = compile_rust_executable(&test);
        let cpp_compiled = compile_cpp_executable(&test);
        let go_compiled = compile_go_executable(&test);
        
        // Benchmark execution times
        if ea_compiled {
            group.bench_function(&format!("ea_runtime_{}", test.name), |b| {
                b.iter(|| {
                    let result = Command::new(&format!("./runtime_{}_ea", test.name))
                        .output();
                    black_box(result.map(|o| o.status.success()).unwrap_or(false))
                })
            });
        }
        
        if rust_compiled {
            group.bench_function(&format!("rust_runtime_{}", test.name), |b| {
                b.iter(|| {
                    let result = Command::new(&format!("./runtime_{}_rust", test.name))
                        .output();
                    black_box(result.map(|o| o.status.success()).unwrap_or(false))
                })
            });
        }
        
        if cpp_compiled {
            group.bench_function(&format!("cpp_runtime_{}", test.name), |b| {
                b.iter(|| {
                    let result = Command::new(&format!("./runtime_{}_cpp", test.name))
                        .output();
                    black_box(result.map(|o| o.status.success()).unwrap_or(false))
                })
            });
        }
        
        if go_compiled {
            group.bench_function(&format!("go_runtime_{}", test.name), |b| {
                b.iter(|| {
                    let result = Command::new(&format!("./runtime_{}_go", test.name))
                        .output();
                    black_box(result.map(|o| o.status.success()).unwrap_or(false))
                })
            });
        }
        
        // Clean up executables
        let _ = fs::remove_file(&format!("runtime_{}_ea", test.name));
        let _ = fs::remove_file(&format!("runtime_{}_rust", test.name));
        let _ = fs::remove_file(&format!("runtime_{}_cpp", test.name));
        let _ = fs::remove_file(&format!("runtime_{}_go", test.name));
    }
    
    group.finish();
}

// Helper functions for compilation

#[cfg(feature = "llvm")]
fn compile_ea_executable(test: &CrossLanguageTest) -> bool {
    let ir_result = compile_to_llvm(test.ea_code, &format!("runtime_{}", test.name));
    
    if ir_result.is_ok() {
        let llc_result = Command::new("llc")
            .arg(&format!("runtime_{}.ll", test.name))
            .arg("-o").arg(&format!("runtime_{}.s", test.name))
            .output();
        
        if llc_result.is_ok() {
            let link_result = Command::new("gcc")
                .arg(&format!("runtime_{}.s", test.name))
                .arg("-o").arg(&format!("runtime_{}_ea", test.name))
                .output();
            
            let _ = fs::remove_file(&format!("runtime_{}.ll", test.name));
            let _ = fs::remove_file(&format!("runtime_{}.s", test.name));
            
            link_result.is_ok()
        } else {
            let _ = fs::remove_file(&format!("runtime_{}.ll", test.name));
            false
        }
    } else {
        false
    }
}

#[cfg(not(feature = "llvm"))]
fn compile_ea_executable(_test: &CrossLanguageTest) -> bool {
    false
}

fn compile_rust_executable(test: &CrossLanguageTest) -> bool {
    if !is_compiler_available("rustc") {
        return false;
    }
    
    let filename = format!("runtime_{}.rs", test.name);
    fs::write(&filename, test.rust_code).unwrap();
    
    let result = Command::new("rustc")
        .arg(&filename)
        .arg("-C").arg("opt-level=2") // Standard optimization
        .arg("-o").arg(&format!("runtime_{}_rust", test.name))
        .output();
    
    let _ = fs::remove_file(&filename);
    result.map(|o| o.status.success()).unwrap_or(false)
}

fn compile_cpp_executable(test: &CrossLanguageTest) -> bool {
    if !is_compiler_available("g++") {
        return false;
    }
    
    let filename = format!("runtime_{}.cpp", test.name);
    fs::write(&filename, test.cpp_code).unwrap();
    
    let result = Command::new("g++")
        .arg(&filename)
        .arg("-O2") // Standard optimization
        .arg("-o").arg(&format!("runtime_{}_cpp", test.name))
        .output();
    
    let _ = fs::remove_file(&filename);
    result.map(|o| o.status.success()).unwrap_or(false)
}

fn compile_go_executable(test: &CrossLanguageTest) -> bool {
    if !is_compiler_available("go") {
        return false;
    }
    
    let filename = format!("runtime_{}.go", test.name);
    fs::write(&filename, test.go_code).unwrap();
    
    let result = Command::new("go")
        .arg("build")
        .arg("-o").arg(&format!("runtime_{}_go", test.name))
        .arg(&filename)
        .output();
    
    let _ = fs::remove_file(&filename);
    result.map(|o| o.status.success()).unwrap_or(false)
}

criterion_group!(
    cross_language_benches,
    bench_compilation_speed,
    bench_full_pipeline,
    bench_runtime_performance
);

criterion_main!(cross_language_benches);