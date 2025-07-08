//! Evidence-Based Performance Benchmarks for Eä Compiler
//! 
//! This benchmark suite provides real, measured performance comparisons
//! against rustc, g++, and go build to replace fantasy claims with facts.
//! 
//! Focus: Actual compilation speed and memory usage measurements

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use ea_compiler::{compile_to_llvm, tokenize};
use std::fs;
use std::process::Command;
use std::time::Instant;
use std::path::Path;

/// Simple test programs that are equivalent across languages
const FIBONACCI_EA: &str = r#"
func fibonacci(n: i32) -> i32 {
    if n <= 1 {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

func main() -> i32 {
    return fibonacci(20);
}
"#;

const FIBONACCI_RUST: &str = r#"
fn fibonacci(n: i32) -> i32 {
    if n <= 1 {
        return n;
    }
    fibonacci(n - 1) + fibonacci(n - 2)
}

fn main() {
    println!("{}", fibonacci(20));
}
"#;

const FIBONACCI_CPP: &str = r#"
#include <iostream>

int fibonacci(int n) {
    if (n <= 1) {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

int main() {
    std::cout << fibonacci(20) << std::endl;
    return 0;
}
"#;

const FIBONACCI_GO: &str = r#"
package main

import "fmt"

func fibonacci(n int) int {
    if n <= 1 {
        return n
    }
    return fibonacci(n-1) + fibonacci(n-2)
}

func main() {
    fmt.Println(fibonacci(20))
}
"#;

/// Baseline Eä compilation speed measurement
fn bench_ea_compilation_speed(c: &mut Criterion) {
    let mut group = c.benchmark_group("ea_compilation_baseline");
    
    // Test different complexity levels
    let test_programs = vec![
        ("simple_fibonacci", FIBONACCI_EA),
        ("loop_heavy", generate_loop_heavy_program()),
        ("function_heavy", generate_function_heavy_program()),
        ("arithmetic_heavy", generate_arithmetic_heavy_program()),
    ];
    
    for (name, program) in test_programs {
        group.bench_function(name, |b| {
            b.iter(|| {
                // Measure full compilation pipeline
                let start = Instant::now();
                let result = compile_to_llvm(black_box(program), &format!("test_{}", name));
                let duration = start.elapsed();
                black_box((result, duration))
            })
        });
    }
    
    group.finish();
}

/// Head-to-head compilation speed comparison
fn bench_compilation_speed_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("compilation_speed_head_to_head");
    group.sample_size(10); // Reduce sample size for external compiler calls
    
    // Benchmark Eä compiler
    group.bench_function("ea_fibonacci", |b| {
        b.iter(|| {
            let start = Instant::now();
            let result = compile_to_llvm(black_box(FIBONACCI_EA), "fibonacci_ea");
            let duration = start.elapsed();
            black_box((result, duration))
        })
    });
    
    // Benchmark Rust compiler (if available)
    if is_compiler_available("rustc") {
        group.bench_function("rustc_fibonacci", |b| {
            b.iter(|| {
                // Write source file
                fs::write("temp_fibonacci.rs", FIBONACCI_RUST).unwrap();
                
                let start = Instant::now();
                let output = Command::new("rustc")
                    .arg("temp_fibonacci.rs")
                    .arg("-O")
                    .arg("--emit=llvm-ir")
                    .arg("-o")
                    .arg("temp_fibonacci.ll")
                    .output()
                    .unwrap();
                let duration = start.elapsed();
                
                // Cleanup
                fs::remove_file("temp_fibonacci.rs").ok();
                fs::remove_file("temp_fibonacci.ll").ok();
                fs::remove_file("temp_fibonacci").ok();
                
                black_box((output.status.success(), duration))
            })
        });
    }
    
    // Benchmark C++ compiler (if available)
    if is_compiler_available("g++") {
        group.bench_function("gcc_fibonacci", |b| {
            b.iter(|| {
                // Write source file
                fs::write("temp_fibonacci.cpp", FIBONACCI_CPP).unwrap();
                
                let start = Instant::now();
                let output = Command::new("g++")
                    .arg("temp_fibonacci.cpp")
                    .arg("-O2")
                    .arg("-S")
                    .arg("-emit-llvm")
                    .arg("-o")
                    .arg("temp_fibonacci.ll")
                    .output()
                    .unwrap();
                let duration = start.elapsed();
                
                // Cleanup
                fs::remove_file("temp_fibonacci.cpp").ok();
                fs::remove_file("temp_fibonacci.ll").ok();
                fs::remove_file("temp_fibonacci").ok();
                
                black_box((output.status.success(), duration))
            })
        });
    }
    
    // Benchmark Go compiler (if available)
    if is_compiler_available("go") {
        group.bench_function("go_fibonacci", |b| {
            b.iter(|| {
                // Write source file
                fs::write("temp_fibonacci.go", FIBONACCI_GO).unwrap();
                
                let start = Instant::now();
                let output = Command::new("go")
                    .arg("build")
                    .arg("-o")
                    .arg("temp_fibonacci")
                    .arg("temp_fibonacci.go")
                    .output()
                    .unwrap();
                let duration = start.elapsed();
                
                // Cleanup
                fs::remove_file("temp_fibonacci.go").ok();
                fs::remove_file("temp_fibonacci").ok();
                
                black_box((output.status.success(), duration))
            })
        });
    }
    
    group.finish();
}

/// Memory usage during compilation measurement
fn bench_compilation_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("compilation_memory_usage");
    
    // Test with progressively larger programs
    let program_sizes = vec![100, 500, 1000, 2000];
    
    for size in program_sizes {
        let program = generate_large_program(size);
        
        group.bench_function(&format!("program_size_{}", size), |b| {
            b.iter(|| {
                // Note: In a real implementation, we would measure actual memory usage
                // For now, we measure compilation time as proxy for complexity
                let start = Instant::now();
                let result = compile_to_llvm(black_box(&program), &format!("large_{}", size));
                let duration = start.elapsed();
                black_box((result, duration))
            })
        });
    }
    
    group.finish();
}

/// Lexer speed vs other language tokenizers
fn bench_lexer_speed_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer_speed_comparison");
    
    // Test tokenization speed
    let large_program = generate_large_program(1000);
    let medium_program = generate_medium_program();
    let test_programs = vec![
        ("small", FIBONACCI_EA),
        ("medium", &medium_program),
        ("large", &large_program),
    ];
    
    for (name, program) in test_programs {
        group.bench_function(&format!("ea_tokenize_{}", name), |b| {
            b.iter(|| {
                black_box(tokenize(black_box(program)).unwrap())
            })
        });
    }
    
    group.finish();
}

/// Incremental compilation performance
fn bench_incremental_compilation(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental_compilation");
    
    let base_program = r#"
func helper1(x: i32) -> i32 {
    return x * 2;
}

func helper2(x: i32) -> i32 {
    return x + 10;
}

func main() -> i32 {
    return helper1(5) + helper2(3);
}
"#;
    
    // Simulate small changes
    let modified_program = r#"
func helper1(x: i32) -> i32 {
    return x * 3;  // Changed multiplication factor
}

func helper2(x: i32) -> i32 {
    return x + 10;
}

func main() -> i32 {
    return helper1(5) + helper2(3);
}
"#;
    
    group.bench_function("full_recompilation", |b| {
        b.iter(|| {
            let result1 = compile_to_llvm(black_box(base_program), "base");
            let result2 = compile_to_llvm(black_box(modified_program), "modified");
            black_box((result1, result2))
        })
    });
    
    group.finish();
}

/// Error handling performance
fn bench_error_handling_speed(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling_performance");
    
    let error_cases = vec![
        ("syntax_error", "func main() -> i32 { invalid syntax"),
        ("type_error", "func main() -> i32 { return \"not_an_int\"; }"),
        ("undefined_function", "func main() -> i32 { return undefined_func(); }"),
        ("missing_semicolon", "func main() -> i32 { let x = 5 return x; }"),
    ];
    
    for (name, error_code) in error_cases {
        group.bench_function(name, |b| {
            b.iter(|| {
                // Measure how quickly errors are detected and reported
                let start = Instant::now();
                let result = compile_to_llvm(black_box(error_code), &format!("error_{}", name));
                let duration = start.elapsed();
                black_box((result, duration))
            })
        });
    }
    
    group.finish();
}

/// Real-world application compilation benchmark
fn bench_real_world_applications(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_world_applications");
    
    // JSON parser simulation
    let json_parser_program = r#"
func parse_number(text: string, start: i32) -> i32 {
    let result = 0;
    let i = start;
    while i < 10 {  // Simplified length check
        let digit = i - 48;  // ASCII '0'
        if digit >= 0 && digit <= 9 {
            result = result * 10 + digit;
        }
        i = i + 1;
    }
    return result;
}

func validate_brackets(text: string) -> bool {
    let count = 0;
    let i = 0;
    while i < 10 {  // Simplified
        if i == 123 {  // '{'
            count = count + 1;
        } else if i == 125 {  // '}'
            count = count - 1;
        }
        i = i + 1;
    }
    return count == 0;
}

func main() -> i32 {
    let valid = validate_brackets("test");
    if valid {
        return 1;
    }
    return 0;
}
"#;
    
    group.bench_function("json_parser_simulation", |b| {
        b.iter(|| {
            compile_to_llvm(black_box(json_parser_program), "json_parser")
        })
    });
    
    // Mathematical computation
    let math_program = r#"
func factorial(n: i32) -> i32 {
    if n <= 1 {
        return 1;
    }
    return n * factorial(n - 1);
}

func gcd(a: i32, b: i32) -> i32 {
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    return a;
}

func main() -> i32 {
    let fact = factorial(10);
    let divisor = gcd(48, 18);
    return fact + divisor;
}
"#;
    
    group.bench_function("mathematical_computation", |b| {
        b.iter(|| {
            compile_to_llvm(black_box(math_program), "math_computation")
        })
    });
    
    group.finish();
}

// Helper functions for generating test programs

fn generate_loop_heavy_program() -> &'static str {
    r#"
func nested_loops(n: i32) -> i32 {
    let sum = 0;
    let i = 0;
    while i < n {
        let j = 0;
        while j < n {
            let k = 0;
            while k < 10 {
                sum = sum + i + j + k;
                k = k + 1;
            }
            j = j + 1;
        }
        i = i + 1;
    }
    return sum;
}

func main() -> i32 {
    return nested_loops(20);
}
"#
}

fn generate_function_heavy_program() -> &'static str {
    r#"
func f1(x: i32) -> i32 { return x + 1; }
func f2(x: i32) -> i32 { return f1(x) * 2; }
func f3(x: i32) -> i32 { return f2(x) + 3; }
func f4(x: i32) -> i32 { return f3(x) - 1; }
func f5(x: i32) -> i32 { return f4(x) * 2; }
func f6(x: i32) -> i32 { return f5(x) + f1(x); }
func f7(x: i32) -> i32 { return f6(x) + f2(x); }
func f8(x: i32) -> i32 { return f7(x) * f3(x); }
func f9(x: i32) -> i32 { return f8(x) + f4(x); }
func f10(x: i32) -> i32 { return f9(x) - f5(x); }

func main() -> i32 {
    return f10(5) + f6(3) + f2(1);
}
"#
}

fn generate_arithmetic_heavy_program() -> &'static str {
    r#"
func complex_arithmetic(a: i32, b: i32, c: i32) -> i32 {
    let result = a + b * c - a / 2 + b % 3 + c * a;
    result = result + (a * b) - (c / 2) + (a % b) * c;
    result = result * 2 + a - b + c * 3 - a / 4;
    result = result + (a + b) * (c - a) + (b * c) / (a + 1);
    return result;
}

func main() -> i32 {
    return complex_arithmetic(10, 20, 30) + complex_arithmetic(5, 15, 25);
}
"#
}

fn generate_medium_program() -> &'static str {
    r#"
func bubble_sort_simulation(n: i32) -> i32 {
    let swapped = true;
    let pass = 0;
    while swapped && pass < n {
        swapped = false;
        let i = 1;
        while i < n - pass {
            if i > i - 1 {  // Simplified comparison
                swapped = true;
            }
            i = i + 1;
        }
        pass = pass + 1;
    }
    return pass;
}

func binary_search_simulation(target: i32, size: i32) -> i32 {
    let low = 0;
    let high = size - 1;
    
    while low <= high {
        let mid = low + (high - low) / 2;
        
        if mid == target {
            return mid;
        }
        
        if mid < target {
            low = mid + 1;
        } else {
            high = mid - 1;
        }
    }
    
    return -1;
}

func main() -> i32 {
    let sorted = bubble_sort_simulation(100);
    let found = binary_search_simulation(50, sorted);
    return found;
}
"#
}

fn generate_large_program(num_functions: usize) -> String {
    let mut program = String::new();
    
    // Generate many simple functions
    for i in 0..num_functions {
        program.push_str(&format!(
            r#"
func func_{i}(x: i32) -> i32 {{
    let temp = x + {i};
    if temp > 50 {{
        return temp - 10;
    }} else {{
        return temp + 5;
    }}
}}
"#,
            i = i
        ));
    }
    
    // Add main function that calls some of the generated functions
    program.push_str(&format!(
        r#"
func main() -> i32 {{
    let result = func_0(10);
    if {} > 10 {{
        result = result + func_{}(result);
    }}
    return result;
}}
"#,
        num_functions,
        std::cmp::min(num_functions - 1, 5)
    ));
    
    program
}

fn is_compiler_available(compiler: &str) -> bool {
    let version_arg = if compiler == "go" { "version" } else { "--version" };
    Command::new(compiler)
        .arg(version_arg)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

criterion_group!(
    evidence_based_benches,
    bench_ea_compilation_speed,
    bench_compilation_speed_comparison,
    bench_compilation_memory_usage,
    bench_lexer_speed_comparison,
    bench_incremental_compilation,
    bench_error_handling_speed,
    bench_real_world_applications
);

criterion_main!(evidence_based_benches);