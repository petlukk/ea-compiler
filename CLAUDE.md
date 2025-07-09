# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Reporting Guidelines

**Maintain evidence-based, honest technical reporting.** Focus on:

- Concrete measurements and test results
- Specific limitations and known issues  
- Objective comparisons with quantified data
- Clear distinction between implemented features and aspirational claims
- Acknowledgment of incomplete functionality
- Technical trade-offs and constraints

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
cargo test --features=llvm compilation_tests
cargo test --features=llvm core_functionality_tests

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
```

### Performance and Benchmarks
```bash
# Run performance benchmarks
cargo bench --features=llvm

# Specific benchmarks
cargo bench --features=llvm compilation_performance
cargo bench --features=llvm cross_language_comparison
cargo bench --features=llvm ea_advanced_vs_full_pipeline
cargo bench --features=llvm frontend_performance
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

# CLI output modes
./target/release/ea --emit-tokens hello.ea
./target/release/ea --emit-ast hello.ea
./target/release/ea --emit-llvm hello.ea
./target/release/ea --emit-llvm-only hello.ea | lli
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
- Module exports for all advanced features

#### 2. **Lexical Analysis** (`src/lexer/`)
- **Engine**: Uses `logos` crate for high-performance tokenization
- **SIMD-first design**: Native support for 32 SIMD vector types (f32x4, i64x8, etc.)
- **Hardware features**: SSE, AVX, AVX2, AVX512, NEON, AltiVec tokens
- **SIMD operators**: Element-wise operations (.*, .+, ./, .&, .|, .^)
- **Position tracking**: Line/column information for error reporting
- **Key files**: `mod.rs` (main lexer), `tokens.rs` (token utilities)

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
  - SIMD type validation infrastructure
- **Key files**: `mod.rs` (main type checker), `types.rs` (type definitions), `simd_validator.rs` (SIMD validation), `hardware.rs` (hardware feature detection)

#### 5. **Code Generation** (`src/codegen/`)
- **LLVM Integration**: Uses `inkwell` Rust bindings for LLVM 14
- **Core**: `CodeGenerator` managing LLVM context, module, and builder
- **Capabilities**:
  - Complete control flow compilation (if/else, loops)
  - Function generation with parameters and locals
  - Expression compilation for all arithmetic/logical operations
  - Memory management via LLVM stack allocation
  - SSA form generation
  - SIMD instruction generation

#### 6. **Advanced Features (v0.2)**

##### **Memory Management** (`src/memory/`)
- **Memory regions**: ReadOnly, WorkingSet, Pool, Stack, Static
- **Region analysis**: Compile-time memory analysis with optimization hints
- **Safety checking**: Use-after-free, buffer overflow detection
- **Memory pools**: GlobalAlloc, ThreadLocal, SIMDAlloc strategies
- **Performance optimization**: Cache-friendly allocation patterns

##### **Compile-time Execution** (`src/comptime/`)
- **ComptimeEngine**: Algorithm selection and optimization at compile time
- **Algorithm database**: QuickSort, MergeSort, RadixSort, BinarySearch, FFT, etc.
- **Data-driven optimization**: Selection based on data characteristics
- **Lookup table generation**: Mathematical functions, optimization tables
- **Performance modeling**: Cache behavior and energy consumption estimates

##### **Advanced SIMD** (`src/simd_advanced/`)
- **Hardware detection**: Comprehensive instruction set support (37 sets)
- **Adaptive vectorization**: Hardware-specific optimization
- **Specialized operations**: Matrix multiplication, convolution, FFT
- **Performance modeling**: Cycle count and cache behavior prediction

##### **Package Management** (`src/package/`)
- **Performance-aware dependencies**: Dependencies specify performance requirements
- **Build system**: Multi-target builds with optimization profiles
- **Benchmark integration**: Automated performance testing
- **Dependency resolution**: Semantic versioning with performance constraints

##### **Language Server Protocol** (`src/lsp/`)
- **Real-time analysis**: Performance analysis and error detection
- **SIMD optimization**: Automated optimization suggestions
- **VS Code integration**: Complete IDE support with syntax highlighting
- **Performance dashboard**: Visual performance metrics

##### **Standard Library** (`src/stdlib/`)
- **SIMD-accelerated collections**: Vec, HashMap, HashSet with 2-4x performance improvement
- **Hardware feature detection**: Automatic SSE/AVX/AVX2/AVX512/NEON support
- **Optimized I/O**: High-performance file operations and string processing
- **Mathematical functions**: Vectorized math operations with fallback support
- **Configurable optimization**: Debug, Release, and Aggressive optimization levels

##### **JIT Compilation System** (`src/jit_execution.rs`, `src/jit_cache.rs`)
- **JIT execution engine**: Immediate program execution with symbol mapping
- **Compilation caching**: Intelligent caching of compiled code with hit/miss statistics
- **Symbol resolution**: Comprehensive mapping of system functions (libc integration)
- **Performance tracking**: Detailed execution statistics and memory usage profiling
- **Fallback handling**: Graceful degradation for complex I/O operations

##### **Performance Infrastructure**
- **Memory profiler** (`src/memory_profiler.rs`): Real-time memory usage tracking
- **LLVM optimization** (`src/llvm_optimization.rs`): Advanced optimization passes
- **Parser optimization** (`src/parser_optimization.rs`): High-performance parsing
- **Incremental compilation** (`src/incremental_compilation.rs`): Fast recompilation
- **Parallel compilation** (`src/parallel_compilation.rs`): Multi-threaded compilation
- **Streaming compiler** (`src/streaming_compiler.rs`): Large file processing
- **Resource manager** (`src/resource_manager.rs`): Efficient resource cleanup

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
├── lsp_main.rs         # LSP server binary entry point
├── ast.rs              # AST definitions including SIMD nodes
├── error.rs            # Error types and handling
├── utils.rs            # Shared utilities
├── lexer/              # Tokenization with SIMD support
│   ├── mod.rs          # Main lexer implementation
│   └── tokens.rs       # Token utilities
├── parser/             # Syntax analysis and AST generation
│   └── mod.rs          # Parser implementation
├── type_system/        # Type checking and validation
│   ├── mod.rs          # Type checker implementation
│   ├── types.rs        # Type definitions
│   ├── simd_validator.rs # SIMD type validation
│   └── hardware.rs     # Hardware feature detection
├── codegen/            # LLVM code generation
│   └── mod.rs          # Code generator implementation
├── memory/             # Advanced memory management (v0.2)
│   └── mod.rs          # Memory region analysis and optimization
├── comptime/           # Compile-time execution engine (v0.2)
│   └── mod.rs          # Algorithm selection and comptime execution
├── simd_advanced/      # Advanced SIMD operations (v0.2)
│   └── mod.rs          # Hardware-specific SIMD optimization
├── package/            # Package management system (v0.2)
│   └── mod.rs          # Performance-aware dependency resolution
├── lsp/                # Language Server Protocol (v0.2)
│   └── mod.rs          # LSP server with performance analysis
├── stdlib/             # Standard library with SIMD acceleration
│   ├── mod.rs          # Library initialization and feature detection
│   ├── collections.rs  # Vec, HashMap, HashSet with SIMD optimization
│   ├── io.rs           # I/O operations and file handling
│   ├── math.rs         # Mathematical functions with SIMD support
│   └── string.rs       # String operations with vectorization
├── incremental_compilation.rs # Incremental compilation system
├── jit_cache.rs        # JIT compilation caching
├── jit_cached.rs       # Cached JIT result structures
├── jit_execution.rs    # JIT execution engine with symbol mapping
├── llvm_optimization.rs # LLVM-level optimization passes
├── memory_profiler.rs  # Memory usage profiling and analysis
├── parallel_compilation.rs # Parallel compilation infrastructure
├── parser_optimization.rs # Parser performance optimizations
├── resource_manager.rs # Resource management and cleanup
└── streaming_compiler.rs # Streaming compilation for large files

tests/                  # Integration test suite
├── compilation_tests.rs    # Comprehensive compilation pipeline tests
└── core_functionality_tests.rs # Core language functionality tests

benches/                # Performance benchmarks
├── compilation_performance.rs # Compilation speed benchmarks
├── cross_language_comparison.rs # Cross-language performance tests
├── ea_advanced_vs_full_pipeline.rs # Advanced vs standard pipeline comparison
├── frontend_performance.rs # Frontend-specific performance tests
└── simple_cross_language.rs # Simple cross-language benchmarks

vscode-extension/       # VS Code language support
├── package.json        # Extension configuration
├── src/extension.ts    # Extension implementation
└── syntaxes/ea.tmGrammar.json # Syntax highlighting
```

## Development Environment Setup

### System Requirements
- **Rust**: 1.70+ (install via [rustup.rs](https://rustup.rs/))
- **LLVM**: Version 14 specifically (not 15 or later)
- **Platform**: Linux (Ubuntu/WSL recommended), macOS, Windows (via WSL)
- **Memory**: 4GB+ recommended for compilation

### Ubuntu/WSL Setup
```bash
# Install dependencies
sudo apt update
sudo apt install llvm-14-dev clang-14 build-essential

# Verify LLVM installation
llvm-config-14 --version  # Should show 14.x.x

# One-time setup
make setup

# Verify installation
cargo test --features=llvm
make quality-check
```

### Common Setup Issues
- **LLVM version mismatch**: Ensure exactly LLVM 14, not 15+
- **Path issues**: Verify `llvm-config-14` is in PATH
- **Permission errors**: Use `sudo` for apt installs only

## Performance Characteristics

### Measured Performance (as of current implementation)
- **Compilation time**: 4.21µs (small) to 57.24µs (large programs) for frontend
- **Memory usage**: ~18MB peak during compilation
- **Test suite**: 79/79 tests passing
- **Build time**: ~1-2 minutes for full compiler build

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

## Testing Infrastructure

### Test Organization
- **Unit Tests**: Module-level tests in `src/*/mod.rs` files
- **Integration Tests**: End-to-end tests in `tests/` directory
- **Compilation Tests**: Comprehensive compilation pipeline validation (`tests/compilation_tests.rs`)
- **Core Functionality Tests**: Language feature validation (`tests/core_functionality_tests.rs`)
- **Performance Tests**: Benchmark suite in `benches/`
- **Production Tests**: Stability and regression tests

### Running Specific Test Categories
```bash
# Run specific test files
cargo test --features=llvm compilation_tests
cargo test --features=llvm core_functionality_tests

# Run tests with pattern matching
cargo test --features=llvm fibonacci
cargo test --features=llvm simd
cargo test --features=llvm vector

# Run single test function
cargo test --features=llvm test_basic_tokenization
cargo test --features=llvm test_simd_vector_operations

# Run with specific output
cargo test --features=llvm -- --test-threads=1 --nocapture
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

## Advanced Features Architecture

### Memory Management System
- **Region Types**: ReadOnly, WorkingSet, Pool, Stack, Static
- **Analysis Engine**: Compile-time safety verification with 940+ lines of analysis code
- **Pool System**: GlobalAlloc (lock-free), ThreadLocal (high-frequency), SIMDAlloc (64-byte aligned)
- **Safety Features**: Use-after-free detection, buffer overflow prevention, alignment validation

### Compile-time Execution Engine
- **Algorithm Database**: 14 algorithm implementations with intelligent selection
- **Performance Modeling**: Cache behavior prediction, energy consumption estimates
- **Data Profiling**: Size, distribution, and access pattern analysis
- **Optimization Selection**: Automatic algorithm choice based on data characteristics

### Advanced SIMD Architecture
- **Instruction Sets**: 37 supported (SSE through AVX512, NEON, SVE, AltiVec, RISC-V Vector)
- **Specialized Operations**: Matrix multiplication, convolution, FFT, cryptographic functions
- **Adaptive Vectorization**: Hardware-specific instruction selection
- **Performance Modeling**: Cycle count and cache behavior prediction

### VS Code Extension Features
- **Language Support**: Comprehensive syntax highlighting for all Eä constructs
- **Performance Tools**: Real-time performance analysis and SIMD optimization suggestions
- **Developer Productivity**: Context menus, keyboard shortcuts, status bar integration
- **LSP Integration**: Real-time error detection with performance context

## Current Status

**Version**: v0.1.1 - Production-ready compiler with comprehensive v0.2 advanced features
**Test Status**: 79 tests passing
**Build Status**: Clean compilation with LLVM features enabled
**Features**: Complete compilation pipeline, SIMD support, advanced v0.2 features implemented

### Feature Implementation Status
- ✅ **Core Language**: Complete compilation pipeline with SIMD support
- ✅ **Advanced Memory**: Region-based analysis and optimization (940+ lines)
- ✅ **Compile-time Execution**: Algorithm selection and optimization (1,100+ lines)
- ✅ **Advanced SIMD**: Hardware-specific optimization (779 lines)
- ✅ **Package Management**: Performance-aware dependency resolution
- ✅ **Developer Tools**: LSP server and VS Code extension
- ✅ **Cross-platform**: Multi-architecture validation infrastructure

### Development Priorities
1. **Evidence-based development**: All claims backed by measurements
2. **Performance validation**: Comprehensive benchmarking against competitors
3. **Production stability**: Real-world application testing
4. **Documentation**: Complete API and usage documentation