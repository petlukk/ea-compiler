# Getting Started with Eä Language Development

Welcome to the Eä programming language compiler project! This guide will help you get up and running with development.

## 🚀 Quick Start

### Prerequisites

- **Rust 1.70+** - [Install Rust](https://rustup.rs/)
- **LLVM 14** - For code generation (Ubuntu: `sudo apt install llvm-14-dev`)
- **Git** - For version control

### Clone and Build

```bash
git clone <repository-url>
cd ea-compiler
cargo build --features=llvm
```

### Run Tests

```bash
# Run all tests
cargo test --features=llvm

# Run benchmarks
cargo bench

# Run with test output
cargo test -- --nocapture
```

### Try the Compiler

```bash
# Build and run the CLI
cargo run --features=llvm -- examples/fibonacci.ea
```

## 📁 Project Structure

```
ea-compiler/
├── src/
│   ├── lexer/          # Tokenization
│   │   ├── mod.rs      # Main lexer implementation
│   │   └── tokens.rs   # Token utilities
│   ├── parser/         # Syntax analysis
│   │   └── mod.rs      # Parser implementation
│   ├── ast.rs          # Abstract Syntax Tree definitions
│   ├── type_system/    # Type checking
│   │   ├── mod.rs      # Type checker
│   │   └── types.rs    # Type definitions
│   ├── codegen/        # LLVM code generation
│   │   └── mod.rs      # Code generator
│   ├── error.rs        # Error handling
│   ├── utils.rs        # Utilities
│   ├── lib.rs          # Library interface
│   └── main.rs         # CLI entry point
├── tests/              # Integration tests
├── benches/            # Performance benchmarks
├── examples/           # Example Eä programs
└── docs/               # Documentation
```

## 🔧 Development Workflow

### 1. Making Changes

The compiler is organized into clear phases:

1. **Lexer** (`src/lexer/`) - Converts source text to tokens
2. **Parser** (`src/parser/`) - Converts tokens to AST
3. **Type System** (`src/type_system/`) - Validates types and semantics
4. **Code Generator** (`src/codegen/`) - Produces LLVM IR

### 2. Testing Your Changes

```bash
# Test specific component
cargo test lexer_tests
cargo test parser_tests
cargo test type_system_tests

# Test end-to-end compilation
cargo test integration_tests --features=llvm

# Run performance tests
cargo bench
```

### 3. Adding New Features

When adding new language features:

1. **Update the lexer** - Add new tokens in `src/lexer/mod.rs`
2. **Update the parser** - Add parsing logic in `src/parser/mod.rs`
3. **Update the AST** - Add new AST nodes in `src/ast.rs`
4. **Update type checking** - Add type rules in `src/type_system/mod.rs`
5. **Update code generation** - Add LLVM IR generation in `src/codegen/mod.rs`
6. **Add tests** - Create comprehensive tests for the new feature

## 🧪 Testing Guidelines

### Unit Tests

Each module should have comprehensive unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specific_functionality() {
        // Test implementation
        let result = function_under_test(input);
        assert_eq!(result, expected_output);
    }
}
```

### Integration Tests

Test complete compilation pipeline:

```rust
#[test]
fn test_new_feature_end_to_end() {
    let source = r#"
        // Eä source code using new feature
        func test() -> () {
            // ...
            return;
        }
    "#;
    
    let result = compile_to_ast(source);
    assert!(result.is_ok(), "Should compile successfully");
}
```

### Performance Tests

Use criterion for benchmarking:

```rust
fn bench_new_feature(c: &mut Criterion) {
    c.bench_function("new_feature", |b| {
        b.iter(|| {
            // Benchmark implementation
            black_box(function_to_benchmark(black_box(input)))
        })
    });
}
```

## 📊 Current Capabilities

### ✅ Implemented Features

- **Complete lexical analysis** - All tokens, position tracking, error handling
- **Full expression parsing** - Arithmetic, logical, comparison, assignment
- **Statement parsing** - Functions, variables, control flow (if/while/for)
- **Comprehensive type system** - Type checking, inference, compatibility
- **LLVM code generation** - Working compilation to machine code
- **Error handling** - Position-aware errors with helpful messages

### 🚧 Sprint 2 Targets

- **SIMD integration** - Built-in vectorization support
- **Memory regions** - Zero-cost memory management
- **Adaptive optimization** - Compile-time execution and caching
- **Security features** - Taint tracking and capability types

## 🐛 Debugging Tips

### Common Issues

1. **LLVM linking errors**
   ```bash
   # Install LLVM development packages
   sudo apt install llvm-14-dev
   # Set environment variables if needed
   export LLVM_SYS_140_PREFIX=/usr/lib/llvm-14
   ```

2. **Test failures**
   ```bash
   # Run with verbose output
   cargo test -- --nocapture
   # Run specific test
   cargo test test_name -- --exact
   ```

3. **Performance issues**
   ```bash
   # Run benchmarks to identify bottlenecks
   cargo bench
   # Profile with optimizations
   cargo build --release
   ```

### Debugging Compilation

Add debug prints to trace compilation:

```rust
// In lexer
println!("Token: {:?} at {}:{}", token.kind, token.position.line, token.position.column);

// In parser  
println!("Parsing expression: {:?}", expr);

// In type checker
println!("Type checking: {:?} -> {:?}", expr, result_type);

// In code generator
println!("Generating LLVM IR for: {:?}", statement);
```

## 📈 Performance Expectations

### Current Benchmarks

- **Lexer throughput**: >1MB/sec
- **Small programs**: <100ms compilation
- **Medium programs**: <500ms compilation  
- **Large programs**: <2s compilation
- **Memory usage**: Minimal, efficient allocation

### Optimization Guidelines

1. **Avoid allocations** in hot paths
2. **Use string slices** instead of owned strings where possible
3. **Cache frequently accessed data**
4. **Profile before optimizing**

## 🤝 Contributing

### Code Style

- Follow Rust standard formatting: `cargo fmt`
- Run clippy for lints: `cargo clippy`
- Write comprehensive tests for new features
- Document public APIs with `///` comments

### Pull Request Process

1. Create feature branch: `git checkout -b feature/simd-support`
2. Implement feature with tests
3. Run full test suite: `cargo test --features=llvm`
4. Run benchmarks: `cargo bench`
5. Update documentation
6. Submit pull request

### Commit Messages

Use conventional commit format:
```
feat: add SIMD type support to lexer
fix: resolve parser precedence issue  
docs: update getting started guide
test: add comprehensive SIMD tests
perf: optimize type checking performance
```

## 🔗 Useful Resources

- **Rust Language Reference**: https://doc.rust-lang.org/reference/
- **LLVM Documentation**: https://llvm.org/docs/
- **Inkwell LLVM Bindings**: https://thedan64.github.io/inkwell/
- **Logos Lexer Generator**: https://docs.rs/logos/
- **Criterion Benchmarking**: https://docs.rs/criterion/

## 🆘 Getting Help

1. **Check existing tests** - Look at `tests/` for examples
2. **Read the code** - The codebase is well-documented
3. **Run benchmarks** - Use `cargo bench` to understand performance
4. **Create minimal examples** - Isolate issues with small test cases

## 🎯 Next Steps

After getting familiar with the codebase:

1. **Run the full test suite** - Ensure everything works
2. **Try the examples** - Compile the example programs
3. **Explore the benchmarks** - Understand performance characteristics
4. **Read the Sprint 2 planning** - Understand upcoming features
5. **Pick a small task** - Start contributing!

Welcome to the Eä language development team! 🚀