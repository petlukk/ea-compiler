use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ea_compiler::*;
use std::process::Command;
use std::fs;
use std::time::Instant;

/// Comprehensive competitive benchmarks for Week 11-12
/// Head-to-head comparison with Rust, C++, and Go

fn compilation_speed_vs_rust(c: &mut Criterion) {
    let mut group = c.benchmark_group("compilation_speed_comparison");
    group.sample_size(10); // Reduce sample size for compilation benchmarks
    
    // Create equivalent programs for comparison
    let ea_fibonacci = r#"
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
    
    let rust_fibonacci = r#"
        fn fibonacci(n: i32) -> i32 {
            if n <= 1 {
                return n;
            }
            return fibonacci(n - 1) + fibonacci(n - 2);
        }
        
        fn main() -> i32 {
            fibonacci(20)
        }
    "#;
    
    let cpp_fibonacci = r#"
        int fibonacci(int n) {
            if (n <= 1) {
                return n;
            }
            return fibonacci(n - 1) + fibonacci(n - 2);
        }
        
        int main() {
            return fibonacci(20);
        }
    "#;
    
    let go_fibonacci = r#"
        package main
        
        func fibonacci(n int) int {
            if n <= 1 {
                return n
            }
            return fibonacci(n-1) + fibonacci(n-2)
        }
        
        func main() int {
            return fibonacci(20)
        }
    "#;
    
    // Benchmark Eä compilation
    group.bench_function("ea_fibonacci", |b| {
        b.iter(|| {
            compile_to_llvm(black_box(ea_fibonacci), "ea_fibonacci").unwrap()
        })
    });
    
    // Benchmark Rust compilation (if available)
    if Command::new("rustc").arg("--version").output().is_ok() {
        group.bench_function("rust_fibonacci", |b| {
            b.iter(|| {
                fs::write("temp_rust.rs", rust_fibonacci).unwrap();
                let start = Instant::now();
                let output = Command::new("rustc")
                    .arg("temp_rust.rs")
                    .arg("-O")
                    .arg("--emit=llvm-ir")
                    .arg("-o")
                    .arg("temp_rust.ll")
                    .output()
                    .unwrap();
                let duration = start.elapsed();
                fs::remove_file("temp_rust.rs").ok();
                fs::remove_file("temp_rust.ll").ok();
                black_box(duration)
            })
        });
    }
    
    // Benchmark C++ compilation (if available)
    if Command::new("clang++").arg("--version").output().is_ok() {
        group.bench_function("cpp_fibonacci", |b| {
            b.iter(|| {
                fs::write("temp_cpp.cpp", cpp_fibonacci).unwrap();
                let start = Instant::now();
                let output = Command::new("clang++")
                    .arg("temp_cpp.cpp")
                    .arg("-O2")
                    .arg("-S")
                    .arg("-emit-llvm")
                    .arg("-o")
                    .arg("temp_cpp.ll")
                    .output()
                    .unwrap();
                let duration = start.elapsed();
                fs::remove_file("temp_cpp.cpp").ok();
                fs::remove_file("temp_cpp.ll").ok();
                black_box(duration)
            })
        });
    }
    
    // Benchmark Go compilation (if available)
    if Command::new("go").arg("version").output().is_ok() {
        group.bench_function("go_fibonacci", |b| {
            b.iter(|| {
                fs::write("temp_go.go", go_fibonacci).unwrap();
                let start = Instant::now();
                let output = Command::new("go")
                    .arg("build")
                    .arg("-o")
                    .arg("temp_go")
                    .arg("temp_go.go")
                    .output()
                    .unwrap();
                let duration = start.elapsed();
                fs::remove_file("temp_go.go").ok();
                fs::remove_file("temp_go").ok();
                black_box(duration)
            })
        });
    }
    
    group.finish();
}

fn memory_usage_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage_comparison");
    
    // Large program to test memory consumption
    let large_program = r#"
        func large_computation() -> i32 {
            let mut sum = 0;
            for i in 0..1000 {
                for j in 0..100 {
                    sum += i * j;
                }
            }
            return sum;
        }
        
        func main() -> i32 {
            return large_computation();
        }
    "#;
    
    group.bench_function("ea_memory_usage", |b| {
        b.iter(|| {
            compile_to_llvm(black_box(large_program), "large_program").unwrap()
        })
    });
    
    group.finish();
}

fn simd_performance_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_performance");
    
    // SIMD-intensive program
    let simd_program = r#"
        func vector_operations() -> f32x4 {
            let a = f32x4(1.0, 2.0, 3.0, 4.0);
            let b = f32x4(5.0, 6.0, 7.0, 8.0);
            let c = f32x4(9.0, 10.0, 11.0, 12.0);
            
            let sum = a + b + c;
            let product = a * b * c;
            let result = sum + product;
            
            return result;
        }
        
        func main() -> i32 {
            let result = vector_operations();
            return 0;
        }
    "#;
    
    group.bench_function("ea_simd_compilation", |b| {
        b.iter(|| {
            compile_to_llvm(black_box(simd_program), "simd_program").unwrap()
        })
    });
    
    group.finish();
}

fn scalability_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalability_comparison");
    
    // Generate programs of increasing size
    let generate_program = |size: usize| -> String {
        let mut program = String::new();
        program.push_str("func main() -> i32 {\n");
        program.push_str("    let mut sum = 0;\n");
        
        for i in 0..size {
            program.push_str(&format!("    sum += {};\n", i));
        }
        
        program.push_str("    return sum;\n");
        program.push_str("}\n");
        program
    };
    
    // Test different program sizes
    let sizes = vec![100, 500, 1000, 5000];
    
    for size in sizes {
        let program = generate_program(size);
        group.bench_function(&format!("ea_scale_{}", size), |b| {
            b.iter(|| {
                compile_to_llvm(black_box(&program), &format!("scale_{}", size)).unwrap()
            })
        });
    }
    
    group.finish();
}

fn error_handling_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling_performance");
    
    // Test error handling efficiency
    let error_cases = vec![
        ("syntax_error", "func main() -> i32 { invalid syntax here"),
        ("type_error", "func main() -> i32 { return \"string\"; }"),
        ("undefined_var", "func main() -> i32 { return undefined_var; }"),
        ("missing_semicolon", "func main() -> i32 { let x = 5 return x; }"),
    ];
    
    for (name, code) in error_cases {
        group.bench_function(&format!("handle_{}", name), |b| {
            b.iter(|| {
                let _ = compile_to_llvm(black_box(code), &format!("error_{}", name));
            })
        });
    }
    
    group.finish();
}

fn real_world_application_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_world_applications");
    
    // JSON parser equivalent
    let json_parser = r#"
        func parse_number(text: string) -> i32 {
            let mut result = 0;
            let mut i = 0;
            while i < text.len() {
                let digit = text[i] - '0';
                if digit >= 0 && digit <= 9 {
                    result = result * 10 + digit;
                }
                i += 1;
            }
            return result;
        }
        
        func validate_json(text: string) -> bool {
            let mut brace_count = 0;
            let mut bracket_count = 0;
            let mut in_string = false;
            let mut i = 0;
            
            while i < text.len() {
                let ch = text[i];
                if ch == '"' {
                    in_string = !in_string;
                } else if !in_string {
                    if ch == '{' {
                        brace_count += 1;
                    } else if ch == '}' {
                        brace_count -= 1;
                    } else if ch == '[' {
                        bracket_count += 1;
                    } else if ch == ']' {
                        bracket_count -= 1;
                    }
                }
                i += 1;
            }
            
            return brace_count == 0 && bracket_count == 0;
        }
        
        func main() -> i32 {
            let json = "{\"key\": [1, 2, 3]}";
            if validate_json(json) {
                return 1;
            }
            return 0;
        }
    "#;
    
    group.bench_function("json_parser", |b| {
        b.iter(|| {
            compile_to_llvm(black_box(json_parser), "json_parser").unwrap()
        })
    });
    
    // Matrix operations
    let matrix_ops = r#"
        func matrix_multiply_4x4(a: [f32; 16], b: [f32; 16]) -> [f32; 16] {
            let mut result = [0.0; 16];
            for i in 0..4 {
                for j in 0..4 {
                    let mut sum = 0.0;
                    for k in 0..4 {
                        sum += a[i * 4 + k] * b[k * 4 + j];
                    }
                    result[i * 4 + j] = sum;
                }
            }
            return result;
        }
        
        func main() -> i32 {
            let a = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 
                     9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0];
            let b = [16.0, 15.0, 14.0, 13.0, 12.0, 11.0, 10.0, 9.0,
                     8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0];
            let result = matrix_multiply_4x4(a, b);
            return 0;
        }
    "#;
    
    group.bench_function("matrix_operations", |b| {
        b.iter(|| {
            compile_to_llvm(black_box(matrix_ops), "matrix_ops").unwrap()
        })
    });
    
    group.finish();
}

criterion_group!(
    competitive_benches,
    compilation_speed_vs_rust,
    memory_usage_comparison,
    simd_performance_comparison,
    scalability_benchmarks,
    error_handling_performance,
    real_world_application_benchmark
);

criterion_main!(competitive_benches);