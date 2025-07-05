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

# Alternative: Use makefile
make test           # All tests
make test-verbose   # With output
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
│   └── simd_validator.rs # SIMD type validation
└── codegen/            # LLVM code generation
    └── mod.rs          # Code generator implementation
```

## Development Environment Setup
```bash
# Install dependencies (Ubuntu/WSL)
sudo apt install llvm-14-dev clang-14 build-essential

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
./target/release/ea --emit-tokens test_tokens.ea
./target/release/ea --emit-ast test_simple_expr.ea
./target/release/ea --emit-llvm test_fibonacci.ea

# Test JIT execution
./target/release/ea --run test_minimal.ea
./target/release/ea --run --verbose test_print.ea

# Verify compilation output
./target/release/ea --emit-llvm-only test_simd.ea | lli
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

# Run tests with pattern matching
cargo test --features=llvm fibonacci
cargo test --features=llvm simd
cargo test --features=llvm vector

# Run single test function
cargo test --features=llvm test_basic_tokenization
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