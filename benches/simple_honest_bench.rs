//! Simple Honest Benchmark - Just Eä compiler performance
//! This tests the core functionality without external compiler dependencies

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ea_compiler::compile_to_llvm;

const FIBONACCI_TEST: &str = r#"
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

fn bench_ea_compilation_only(c: &mut Criterion) {
    let mut group = c.benchmark_group("ea_compilation_honest");
    group.sample_size(100);
    
    group.bench_function("ea_fibonacci_compilation", |b| {
        b.iter(|| {
            compile_to_llvm(black_box(FIBONACCI_TEST), "fibonacci").unwrap()
        })
    });
    
    group.finish();
}

criterion_group!(honest_benches, bench_ea_compilation_only);
criterion_main!(honest_benches);