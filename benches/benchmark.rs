//! benches/benchmarks.rs
//! Performance benchmarks for the Eä compiler

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use ea_compiler::{Lexer, Parser, TypeChecker, compile_to_ast};
use std::time::Duration;

// Sample Eä programs for benchmarking
const SMALL_PROGRAM: &str = r#"
func add(a: i32, b: i32) -> i32 {
    return a + b;
}

func main() -> () {
    let result = add(5, 10);
    return;
}
"#;

const MEDIUM_PROGRAM: &str = r#"
func fibonacci(n: i32) -> i32 {
    if (n <= 1) {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

func factorial(n: i32) -> i32 {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}

func main() -> () {
    let fib_result = fibonacci(10);
    let fact_result = factorial(10);
    let sum = fib_result + fact_result;
    return;
}
"#;

const LARGE_PROGRAM: &str = r#"
func bubble_sort(arr: []i32, size: i32) -> () {
    for (let i: i32 = 0; i < size - 1; i += 1) {
        for (let j: i32 = 0; j < size - i - 1; j += 1) {
            if (arr[j] > arr[j + 1]) {
                let temp = arr[j];
                arr[j] = arr[j + 1];
                arr[j + 1] = temp;
            }
        }
    }
    return;
}

func binary_search(arr: []i32, target: i32, low: i32, high: i32) -> i32 {
    if (high >= low) {
        let mid = low + (high - low) / 2;
        
        if (arr[mid] == target) {
            return mid;
        }
        
        if (arr[mid] > target) {
            return binary_search(arr, target, low, mid - 1);
        }
        
        return binary_search(arr, target, mid + 1, high);
    }
    
    return -1;
}

func matrix_multiply(a: [][]f64, b: [][]f64, result: [][]f64, n: i32) -> () {
    for (let i: i32 = 0; i < n; i += 1) {
        for (let j: i32 = 0; j < n; j += 1) {
            result[i][j] = 0.0;
            for (let k: i32 = 0; k < n; k += 1) {
                result[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    return;
}

func main() -> () {
    let numbers = [64, 34, 25, 12, 22, 11, 90];
    bubble_sort(numbers, 7);
    
    let target = 22;
    let index = binary_search(numbers, target, 0, 6);
    
    return;
}
"#;

// Lexer benchmarks
fn bench_lexer(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer");
    
    for (name, program) in [
        ("small", SMALL_PROGRAM),
        ("medium", MEDIUM_PROGRAM), 
        ("large", LARGE_PROGRAM),
    ] {
        group.bench_with_input(
            BenchmarkId::new("tokenize", name),
            program,
            |b, program| {
                b.iter(|| {
                    let mut lexer = Lexer::new(black_box(program));
                    black_box(lexer.tokenize_all().unwrap())
                })
            },
        );
    }
    
    group.finish();
}

// Parser benchmarks
fn bench_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser");
    
    for (name, program) in [
        ("small", SMALL_PROGRAM),
        ("medium", MEDIUM_PROGRAM),
        ("large", LARGE_PROGRAM),
    ] {
        let mut lexer = Lexer::new(program);
        let tokens = lexer.tokenize_all().unwrap();
        
        group.bench_with_input(
            BenchmarkId::new("parse", name),
            &tokens,
            |b, tokens| {
                b.iter(|| {
                    let mut parser = Parser::new(black_box(tokens.clone()));
                    black_box(parser.parse_program().unwrap())
                })
            },
        );
    }
    
    group.finish();
}

// Type checker benchmarks
fn bench_type_checker(c: &mut Criterion) {
    let mut group = c.benchmark_group("type_checker");
    
    for (name, program) in [
        ("small", SMALL_PROGRAM),
        ("medium", MEDIUM_PROGRAM),
        ("large", LARGE_PROGRAM),
    ] {
        let mut lexer = Lexer::new(program);
        let tokens = lexer.tokenize_all().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse_program().unwrap();
        
        group.bench_with_input(
            BenchmarkId::new("check_types", name),
            &ast,
            |b, ast| {
                b.iter(|| {
                    let mut type_checker = TypeChecker::new();
                    black_box(type_checker.check_program(black_box(ast)).unwrap())
                })
            },
        );
    }
    
    group.finish();
}

// End-to-end compilation benchmarks
fn bench_full_compilation(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_compilation");
    group.measurement_time(Duration::from_secs(10));
    
    for (name, program) in [
        ("small", SMALL_PROGRAM),
        ("medium", MEDIUM_PROGRAM),
        ("large", LARGE_PROGRAM),
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

// Memory usage benchmarks
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory");
    
    // Generate a very large program for memory testing
    let large_program = generate_large_program(1000); // 1000 functions
    
    group.bench_function("large_program_memory", |b| {
        b.iter(|| {
            black_box(compile_to_ast(black_box(&large_program)).unwrap())
        })
    });
    
    group.finish();
}

// Helper function to generate large programs for testing
fn generate_large_program(num_functions: usize) -> String {
    let mut program = String::new();
    
    for i in 0..num_functions {
        program.push_str(&format!(
            r#"
func function_{i}(x: i32, y: i32) -> i32 {{
    let result = x + y;
    let multiplied = result * 2;
    if (multiplied > 100) {{
        return multiplied - 50;
    }} else {{
        return multiplied + 25;
    }}
}}
"#,
            i = i
        ));
    }
    
    program.push_str(r#"
func main() -> () {
    let sum = 0;
    for (let i: i32 = 0; i < 100; i += 1) {
        sum += function_0(i, i * 2);
    }
    return;
}
"#);
    
    program
}

// Lexer throughput benchmark (MB/s)
fn bench_lexer_throughput(c: &mut Criterion) {
    let large_input = LARGE_PROGRAM.repeat(100); // ~100KB+ of source
    let input_size = large_input.len();
    
    c.bench_function("lexer_throughput", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(&large_input));
            black_box(lexer.tokenize_all().unwrap())
        });
    });
    
    println!("Input size for throughput test: {} bytes", input_size);
}

criterion_group!(
    benches,
    bench_lexer,
    bench_parser,
    bench_type_checker,
    bench_full_compilation,
    bench_memory_usage,
    bench_lexer_throughput
);

criterion_main!(benches);