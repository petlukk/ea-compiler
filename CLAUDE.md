# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Development Commands

### Core Build Commands
```bash
# Build the compiler (debug mode)
cargo build --features=llvm

# Build release version
cargo build --release --features=llvm

# Alternative: Use makefile
make build        # Debug build
make release      # Release build
```

### Testing Commands
```bash
# Run all tests
cargo test --features=llvm

# Run with verbose output
cargo test --features=llvm -- --nocapture

# Run specific test suites
cargo test lexer_tests
cargo test parser_tests  
cargo test type_system_tests
cargo test integration_tests
cargo test simd_codegen_tests
cargo test simd_integration_tests
cargo test simd_lexer_tests
cargo test fibonacci_test

# Alternative: Use makefile
make test           # All tests
make test-verbose   # With output

# Run tests for specific functionality
cargo test --features=llvm -- --test-threads=1 --nocapture test_simd_vector_operations
cargo test --features=llvm -- fibonacci
cargo test --features=llvm -- vector_add
```

### Code Quality Commands
```bash
# Format code
cargo fmt

# Run linter
cargo clippy --all-targets --all-features -- -D warnings

# Complete quality check
make quality-check  # Runs fmt + lint + test

# Comprehensive check (recommended before commits)
make check-all     # Runs quality-check + bench + doc

# Alternative individual commands
make fmt
make lint
```

### Performance and Benchmarks
```bash
# Run performance benchmarks
cargo bench --features=llvm

# Alternative
make bench
```

### CLI Testing
```bash
# Build and test CLI interface
./target/release/ea --test

# Test with example files
./target/release/ea examples/fibonacci.ea
./target/release/ea --verbose --emit-ast examples/fibonacci.ea

# JIT execution (immediate run)
./target/release/ea --run examples/fibonacci.ea

# Test SIMD examples
./target/release/ea --run examples/simd_example.ea
./target/release/ea --emit-llvm examples/vector_add.ea
./target/release/ea --run examples/simd_memory_demo.ea

# Test with stress tests
./target/release/ea --run stress_test_1000.ea
./target/release/ea --emit-llvm-only stress_test_100k.ea | head -20

# Alternative
make test-cli
make run-examples
```

## High-Level Architecture

### Compilation Pipeline
The Eä compiler follows a traditional multi-phase compilation pipeline:

**Source Code → Lexer → Parser → Type Checker → Code Generator → LLVM IR → Native Code**

### Key Modules and Responsibilities

#### 1. **Library Entry Point** (`src/lib.rs`)
- Central orchestration of compilation phases
- Public API functions: `tokenize()`, `parse()`, `compile_to_ast()`, `compile_to_llvm()`
- Feature-gated LLVM integration
- Version constants and re-exports

#### 2. **Lexical Analysis** (`src/lexer/`)
- **Engine**: Uses `logos` crate for high-performance tokenization
- **SIMD-first design**: Native support for 32 SIMD vector types (f32x4, i64x8, etc.)
- **Hardware features**: SSE, AVX, AVX2, AVX512, NEON, AltiVec tokens
- **SIMD operators**: Element-wise operations (.*, .+, ./, .&, .|, .^)
- **Position tracking**: Line/column information for error reporting
- **Error resilience**: Continues after lexical errors
- **Key files**: `mod.rs` (main lexer), `tokens.rs` (token utilities)
- **Token types**: 80+ token types including SIMD vector literals and operations

#### 3. **Syntax Analysis** (`src/parser/`, `src/ast.rs`)
- **Pattern**: Recursive descent parser with operator precedence
- **AST generation**: Strongly-typed Abstract Syntax Tree
- **Core AST types**:
  - `Expr`: Expression nodes (literals, variables, operations, function calls)
  - `Stmt`: Statement nodes (declarations, control flow, blocks)
  - `SIMDExpr`: Specialized nodes for SIMD operations
  - `SIMDVectorType`: 32 SIMD vector types with width/element tracking
- **SIMD integration**: Native parsing of vector literals and element-wise operations

#### 4. **Type System** (`src/type_system/`)
- **Core**: `TypeChecker` with `TypeContext` for symbol table management
- **Type representation**: `EaType` enum covering primitives, arrays, functions, references
- **Capabilities**:
  - Strong type checking with implicit conversions
  - Function signature validation
  - Control flow analysis (return statement validation)
  - Scoped variable and function management
  - SIMD type validation infrastructure (foundation for future expansion)
- **Key files**: `mod.rs` (main type checker), `types.rs` (type definitions), `simd_validator.rs` (SIMD validation), `hardware.rs` (hardware feature detection)
- **Type categories**: 14 primitive types, SIMD vectors, arrays, functions, structs, enums

#### 5. **Code Generation** (`src/codegen/`)
- **LLVM Integration**: Uses `inkwell` Rust bindings for LLVM 14
- **Core**: `CodeGenerator` managing LLVM context, module, and builder
- **Capabilities**:
  - Complete control flow compilation (if/else, loops)
  - Function generation with parameters and locals
  - Expression compilation for all arithmetic/logical operations
  - Memory management via LLVM stack allocation
  - SSA form generation
  - Configurable optimization levels

### SIMD Architecture Highlights
The compiler is uniquely designed with SIMD as a first-class citizen:
- **Token level**: SIMD constructs are native tokens, not extensions
- **AST level**: Dedicated `SIMDExpr` nodes preserve semantics
- **Type level**: Comprehensive SIMD vector type system
- **Future ready**: Foundation for LLVM SIMD instruction generation

### Error Handling Strategy
- **Positioned errors**: All errors include source location
- **Phase-specific**: `LexError`, `ParseError`, `TypeError`, `CodeGenError`
- **Graceful degradation**: Parser continues after recoverable errors
- **User-friendly**: Clear error messages with context

## Key Development Patterns

### Feature Implementation Workflow
When adding new language features:
1. **Lexer**: Add new tokens in `src/lexer/mod.rs`
2. **Parser**: Add parsing logic in `src/parser/mod.rs`
3. **AST**: Add new AST nodes in `src/ast.rs`
4. **Type System**: Add type rules in `src/type_system/mod.rs`
5. **Code Gen**: Add LLVM IR generation in `src/codegen/mod.rs`
6. **Tests**: Create comprehensive unit and integration tests

### Testing Strategy
- **Unit tests**: Each module has comprehensive `#[cfg(test)]` sections
- **Integration tests**: End-to-end compilation pipeline testing in `tests/`
- **Performance tests**: Criterion-based benchmarks in `benches/`
- **CLI tests**: Built-in self-tests via `./target/release/ea --test`

### LLVM Integration Notes
- **Version**: LLVM 14 (specified in Cargo.toml)
- **Feature flag**: All LLVM code is behind `--features=llvm`
- **Dependencies**: Requires `llvm-14-dev` system package
- **Output**: Generates `.ll` LLVM IR files for inspection

## Project Structure Context
```
src/
├── lib.rs              # Main library API and compilation orchestration
├── main.rs             # CLI interface with argument parsing
├── ast.rs              # AST definitions including SIMD nodes
├── error.rs            # Error types and handling
├── utils.rs            # Shared utilities
├── lexer/              # Tokenization with SIMD support
│   ├── mod.rs          # Main lexer implementation
│   ├── tokens.rs       # Token utilities
│   └── tests.rs        # Lexer unit tests
├── parser/             # Syntax analysis and AST generation
│   ├── mod.rs          # Parser implementation
│   └── parser_fix.rs   # Parser fixes and enhancements
├── type_system/        # Type checking and validation
│   ├── mod.rs          # Type checker implementation
│   ├── types.rs        # Type definitions
│   ├── simd_validator.rs # SIMD type validation
│   └── hardware.rs     # Hardware feature detection
└── codegen/            # LLVM code generation
    └── mod.rs          # Code generator implementation

tests/                  # Integration test suite
├── integration_tests.rs    # End-to-end compilation tests
├── lexer_tests.rs         # Lexer functionality tests
├── type_system_tests.rs   # Type system validation tests
├── simd_codegen_tests.rs  # SIMD code generation tests
├── simd_integration_tests.rs # SIMD end-to-end tests
├── simd_lexer_tests.rs    # SIMD lexer tests
└── fibonacci_test.rs      # Fibonacci algorithm tests

benches/                # Performance benchmarks
├── benchmark.rs           # General compiler benchmarks
└── simd_performance_benchmarks.rs # SIMD-specific benchmarks

examples/               # Example Eä programs
├── simd_example.ea       # Basic SIMD operations
├── vector_add.ea         # Vector addition example
├── simd_memory_demo.ea   # Memory load/store operations
└── advanced_memory_simd.ea # Advanced SIMD memory patterns
```

## Development Environment Setup

### System Requirements
- **Rust**: 1.70+ (install via [rustup.rs](https://rustup.rs/))
- **LLVM**: Version 14 specifically (not 15 or later)
- **Platform**: Linux (Ubuntu/WSL recommended), macOS, Windows (via WSL)
- **Memory**: 4GB+ recommended for compilation
- **Disk**: 2GB+ for full build including LLVM

### Ubuntu/WSL Setup
```bash
# Install dependencies (Ubuntu/WSL)
sudo apt update
sudo apt install llvm-14-dev clang-14 build-essential

# Verify LLVM installation
llvm-config-14 --version  # Should show 14.x.x

# One-time setup (includes dep install, examples, build, test)
make setup

# Alternative: Install deps only
make install-deps

# Create example programs
make create-examples

# Verify installation
cargo test --features=llvm
make quality-check
```

### macOS Setup
```bash
# Install LLVM 14 via Homebrew
brew install llvm@14
export PATH="/opt/homebrew/opt/llvm@14/bin:$PATH"

# Set environment variables
export LLVM_SYS_140_PREFIX="/opt/homebrew/opt/llvm@14"
```

### Common Setup Issues
- **LLVM version mismatch**: Ensure exactly LLVM 14, not 15+
- **Path issues**: Verify `llvm-config-14` is in PATH
- **Permission errors**: Use `sudo` for apt installs only
- **Disk space**: Ensure 2GB+ free for complete build

## Performance Characteristics
- **Lexer throughput**: >1MB/sec
- **Small programs**: <100ms compilation
- **Medium programs**: <500ms compilation
- **Memory usage**: Efficient allocation patterns
- **Test suite**: 102/102 tests passing

## CLI Interface Details

### Available Commands
```bash
# Basic compilation modes
ea program.ea                    # Compile to LLVM IR
ea --emit-ast program.ea        # Show Abstract Syntax Tree
ea --emit-tokens program.ea     # Show tokenization output
ea --emit-llvm program.ea       # Show LLVM IR with diagnostics
ea --emit-llvm-only program.ea  # Clean LLVM IR output (for piping)

# Execution modes
ea --run program.ea             # JIT compile and execute immediately
ea --verbose program.ea         # Detailed compilation diagnostics
ea --quiet program.ea           # Suppress non-error output

# Testing and development
ea --test                       # Run built-in compiler tests
ea --version                    # Show version information
ea --help                       # Show usage help
```

### CLI Testing Examples
```bash
# Test individual components
./target/release/ea --emit-tokens hello.ea
./target/release/ea --emit-ast test_fibonacci.ea
./target/release/ea --emit-llvm examples/simd_example.ea

# Test JIT execution
./target/release/ea --run hello.ea
./target/release/ea --run --verbose test_print.ea
./target/release/ea --run examples/vector_add.ea

# Verify compilation output
./target/release/ea --emit-llvm-only examples/simd_example.ea | lli
./target/release/ea --emit-llvm-only stress_test_1000.ea > output.ll

# Debug compilation issues
./target/release/ea --diagnose-jit problematic_file.ea
./target/release/ea --verbose --emit-ast failing_program.ea
```

## Testing Infrastructure

### Test Organization
- **Unit Tests**: Module-level tests in `src/*/mod.rs` files
- **Integration Tests**: End-to-end tests in `tests/` directory
- **SIMD Tests**: Specialized SIMD functionality tests
- **CLI Tests**: Command-line interface validation
- **Performance Tests**: Benchmark suite in `benches/`

### Running Specific Test Categories
```bash
# Run specific test files
cargo test --features=llvm integration_tests
cargo test --features=llvm simd_codegen_tests
cargo test --features=llvm simd_integration_tests
cargo test --features=llvm lexer_tests
cargo test --features=llvm type_system_tests
cargo test --features=llvm fibonacci_test

# Run tests with pattern matching
cargo test --features=llvm fibonacci
cargo test --features=llvm simd
cargo test --features=llvm vector
cargo test --features=llvm stress_test
cargo test --features=llvm memory_operations

# Run single test function
cargo test --features=llvm test_basic_tokenization
cargo test --features=llvm test_simd_vector_operations
cargo test --features=llvm test_fibonacci_compilation
cargo test --features=llvm test_vector_load_store

# Run tests with specific output
cargo test --features=llvm -- --test-threads=1 test_jit_execution
cargo test --features=llvm -- --nocapture test_error_handling
```

## Development Workflow

### Before Making Changes
1. **Check current status**: `make quality-check`
2. **Run full test suite**: `cargo test --features=llvm`
3. **Create feature branch**: `git checkout -b feature-name`

### During Development
1. **Continuous testing**: `cargo test --features=llvm` (specific modules)
2. **Format code**: `cargo fmt`
3. **Check linting**: `cargo clippy --all-targets --all-features -- -D warnings`

### Before Committing
1. **Full quality check**: `make check-all`
2. **Verify CLI works**: `./target/release/ea --test`
3. **Test example files**: `make run-examples`

## Error Handling and Debugging

### Common Issues and Solutions
- **LLVM not found**: Install `llvm-14-dev` package
- **Compilation failures**: Check feature flags (`--features=llvm`)
- **Test failures**: Ensure clean build with `cargo clean && cargo build --features=llvm`
- **CLI issues**: Rebuild with `cargo build --release --features=llvm`

### Debug Mode Features
```bash
# Enable debug lexer (requires feature)
cargo build --features=llvm,debug-lexer

# Verbose compilation output
ea --verbose program.ea
```

## Advanced Features

### SIMD Vector Operations
The compiler supports 32 built-in SIMD vector types:
- Integer vectors: `i8x16`, `i16x8`, `i32x4`, `i64x2`, etc.
- Float vectors: `f32x4`, `f64x2`, etc.
- Element-wise operations: `.+`, `.-`, `.*`, `./`
- Memory operations: `load_vector()`, `store_vector()`

### JIT Compilation
- Immediate program execution via `--run` flag
- Uses LLVM ExecutionEngine for in-memory compilation
- Supports both void and i32 return types for main function

### Memory Management
- Stack-based allocation via LLVM
- Automatic memory management for local variables
- Reference types for parameter passing

## Common Development Tasks

### Adding New Language Features
When implementing new language constructs, follow this order:

1. **Lexer** (`src/lexer/mod.rs`): Add token definitions around line 50-200
2. **AST** (`src/ast.rs`): Add new AST node types around line 100-300
3. **Parser** (`src/parser/mod.rs`): Add parsing logic, typically in `expression()` or `declaration()` methods
4. **Type System** (`src/type_system/mod.rs`): Add type checking in `check_expression()` or `check_statement()`
5. **Codegen** (`src/codegen/mod.rs`): Add LLVM IR generation in `compile_expression()` or `compile_statement()`
6. **Tests**: Add comprehensive tests in appropriate `tests/*.rs` files

### Adding New SIMD Operations
SIMD features require special attention across all phases:

1. **Lexer**: Add SIMD tokens in `TokenKind` enum (src/lexer/mod.rs:50-150)
2. **AST**: Extend `SIMDExpr` enum (src/ast.rs:92-200)
3. **Parser**: Add parsing in `parse_simd_expression()` method
4. **Type System**: Update `simd_validator.rs` for new operation validation
5. **Codegen**: Implement LLVM vector intrinsics in codegen module

### Debugging Compilation Issues
```bash
# Step-by-step debugging
./target/release/ea --emit-tokens problematic.ea    # Check tokenization
./target/release/ea --emit-ast problematic.ea       # Check parsing
./target/release/ea --verbose problematic.ea        # Check type checking
./target/release/ea --emit-llvm problematic.ea      # Check code generation
./target/release/ea --diagnose-jit problematic.ea   # Check JIT issues

# Common file locations for errors
grep -n "error" src/lexer/mod.rs     # Lexical errors
grep -n "ParseError" src/parser/mod.rs # Parse errors  
grep -n "TypeError" src/type_system/mod.rs # Type errors
grep -n "CodeGenError" src/codegen/mod.rs # Codegen errors
```

### Performance Testing and Benchmarking
```bash
# Run all benchmarks
cargo bench --features=llvm

# Run specific benchmarks
cargo bench --features=llvm lexer
cargo bench --features=llvm simd
cargo bench --features=llvm fibonacci

# Profile compilation performance
time ./target/release/ea large_program.ea
time ./target/release/ea stress_test_100k.ea

# Memory usage analysis (requires valgrind)
valgrind --tool=massif ./target/release/ea --run examples/simd_example.ea
```

## File Location Quick Reference

### Core Implementation Files
- **Main API**: `src/lib.rs` (compilation pipeline orchestration)
- **CLI Interface**: `src/main.rs` (argument parsing, user interface)
- **Error Handling**: `src/error.rs` (all error types and formatting)
- **AST Definitions**: `src/ast.rs` (all syntax tree node types)

### Module-Specific Files
- **Lexer Core**: `src/lexer/mod.rs` (tokenization engine)
- **Parser Core**: `src/parser/mod.rs` (recursive descent parser)
- **Type Checker**: `src/type_system/mod.rs` (type validation and inference)
- **Code Generator**: `src/codegen/mod.rs` (LLVM IR generation)
- **SIMD Validator**: `src/type_system/simd_validator.rs` (SIMD type checking)

### Test Files by Category
- **Integration**: `tests/integration_tests.rs` (end-to-end pipeline tests)
- **Lexer**: `tests/lexer_tests.rs` (tokenization tests)
- **Type System**: `tests/type_system_tests.rs` (type checking tests)
- **SIMD**: `tests/simd_*_tests.rs` (SIMD-specific functionality)
- **Performance**: `benches/benchmark.rs` (performance regression tests)

### Configuration Files
- **Dependencies**: `Cargo.toml` (crate configuration and dependencies)
- **Build Automation**: `Makefile` (development workflow commands)
- **Examples**: `examples/*.ea` (demonstration programs)
- **Documentation**: `README.md`, `docs/getting-started.md`