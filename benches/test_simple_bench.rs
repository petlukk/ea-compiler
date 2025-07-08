use criterion::{criterion_group, criterion_main, Criterion};
use ea_compiler::compile_to_llvm;

const SIMPLE_PROGRAM: &str = r#"
func main() -> i32 {
    return 42;
}
"#;

fn bench_simple_compilation(c: &mut Criterion) {
    c.bench_function("simple_compile", |b| {
        b.iter(|| {
            compile_to_llvm(SIMPLE_PROGRAM, "test").unwrap()
        })
    });
}

criterion_group!(benches, bench_simple_compilation);
criterion_main!(benches);