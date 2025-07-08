//! Minimal working honest benchmark

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ea_compiler::compile_to_llvm;

const PROGRAM: &str = r#"
func main() -> i32 {
    return 42;
}
"#;

fn bench_ea_only(c: &mut Criterion) {
    c.bench_function("ea_compile", |b| {
        b.iter(|| {
            compile_to_llvm(black_box(PROGRAM), "test")
        })
    });
}

criterion_group!(benches, bench_ea_only);
criterion_main!(benches);