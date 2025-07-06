# Eä Programming Language Compiler

<div align="center">

**A systems programming language with comprehensive SIMD support**

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![LLVM](https://img.shields.io/badge/LLVM-14-blue.svg?style=flat-square)](https://llvm.org/)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green.svg?style=flat-square)](#license)
[![Tests](https://img.shields.io/badge/tests-102%20passing-brightgreen.svg?style=flat-square)](#testing)

</div>

## Overview

Eä is a systems programming language compiler focused on extreme performance. It generates highly optimized LLVM IR from source code and features comprehensive SIMD support, strong typing, and zero-cost abstractions.

**Current Status**: v0.1.1 - Production-ready static compilation with working JIT execution engine.

### Core Features (v0.1.1)

**Compilation Pipeline**
- **Complete LLVM-based compilation** - Lexer → Parser → Type Checker → LLVM Code Generator
- **JIT execution engine** - Immediate program execution with comprehensive symbol resolution
- **Static compilation** - Generates valid LLVM IR for deployment scenarios
- **Advanced error handling** - Position-aware errors with helpful diagnostics

**Language Features**
- **Strong type system** - Type checking, inference, and compatibility validation
- **Control flow** - if/else statements, while/for loops, pattern matching
- **Functions** - Parameters, return values, recursion support
- **Data structures** - Arrays, structs, enums with data variants
- **Memory safety** - Stack-based allocation with automatic cleanup

**SIMD & Performance**
- **32 SIMD vector types** - f32x4, i32x8, u8x16, etc. covering all major widths
- **Element-wise operations** - .+, .-, .*, ./, .&, .|, .^ for vectors
- **Memory operations** - Vector load/store with alignment support
- **Hardware detection** - SSE, AVX, NEON target feature support

**Standard Library**
- **I/O functions** - print(), println(), read_file(), write_file()
- **Math functions** - sqrt(), sin(), cos(), abs(), min(), max()
- **String functions** - length, concatenation, equality, SIMD-accelerated operations
- **Array functions** - length, indexing, iteration support

## Quick Start

### Prerequisites

- **Rust 1.70+** - [Install Rust](https://rustup.rs/)
- **LLVM 14** - Ubuntu: `sudo apt install llvm-14-dev`

### Installation

```bash
git clone <repository-url>
cd ea-compiler
cargo build --features=llvm --release
```

### Hello, World!

Create `hello.ea`:
```eä
func main() -> () {
    println("Hello, World!");
    return;
}
```

Compile to LLVM IR:
```bash
./target/release/ea hello.ea
```

Or compile and run immediately:
```bash
./target/release/ea --run hello.ea
```

## Language Examples

### Basic Function

```eä
func fibonacci(n: i32) -> i32 {
    if (n <= 1) {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

func main() -> () {
    let result = fibonacci(10);
    print_i32(result);
    return;
}
```

### Type Inference

```eä
func demonstrate_types() -> () {
    // Type inference
    let integer = 42;          // i64
    let float = 3.14;          // f64
    let boolean = true;        // bool
    let text = "hello";        // string
    
    // Explicit types
    let x: i32 = 100;
    let mut counter: i32 = 0;
    counter += 1;
    
    return;
}
```

### Control Flow and Boolean Logic

```eä
func control_flow_demo() -> () {
    // If statements
    let x = 10;
    if (x > 5) {
        print("x is greater than 5");
    } else {
        print("x is not greater than 5");
    }
    
    // Boolean logic with logical operators
    let a = true;
    let b = false;
    let result1 = a && b;  // false
    let result2 = a || b;  // true
    
    // Complex boolean expressions
    if (x > 5 && x < 20) {
        print("x is between 5 and 20");
    }
    
    // For loops
    for (let i: i32 = 0; i < 10; i += 1) {
        // Loop body
    }
    
    // While loops
    let mut count = 0;
    while (count < 5) {
        count += 1;
    }
    
    return;
}
```

### Arrays and Structs

```eä
struct Point {
    x: f32,
    y: f32,
}

func array_and_struct_demo() -> () {
    // Arrays
    let numbers = [1, 2, 3, 4, 5];
    let slice = numbers[1:4];  // [2, 3, 4]
    let element = numbers[2];  // 3
    
    // Array iteration
    for num in numbers {
        print_i32(num);
    }
    
    // Structs
    let point = Point { x: 10.0, y: 20.0 };
    print_f32(point.x);
    
    return;
}
```

### Enums and Pattern Matching

```eä
enum Option {
    Some(i32),
    None,
}

func pattern_matching_demo() -> () {
    let value = Option::Some(42);
    
    let result = match value {
        Option::Some(x) => x,
        Option::None => 0,
    };
    
    print_i32(result);
    return;
}
```

### SIMD Vector Operations

```eä
func simd_examples() -> () {
    // Vector literals with type annotations
    let vec1 = [1.0, 2.0, 3.0, 4.0]f32x4;
    let vec2 = [5.0, 6.0, 7.0, 8.0]f32x4;
    
    // Element-wise operations
    let sum = vec1 .+ vec2;        // Vector addition
    let product = vec1 .* vec2;    // Vector multiplication
    
    // Access individual elements
    print_f32(sum[0]);
    
    return;
}
```

## Available Built-in Functions

### I/O Functions
- `print(string)` - Print string without newline
- `println(string)` - Print string with newline
- `print_i32(i32)` - Print integer
- `print_f32(f32)` - Print float
- `read_line() -> string` - Read line from stdin

### File Operations
- `read_file(string) -> string` - Read file content
- `write_file(string, string) -> ()` - Write to file
- `file_exists(string) -> bool` - Check if file exists

### String Functions
- `string_length(string) -> i32`
- `string_concat(string, string) -> string`
- `string_equals(string, string) -> bool`

### Math Functions
- `sqrt(f32) -> f32`, `sin(f32) -> f32`, `cos(f32) -> f32`
- `abs(f32) -> f32`, `min(f32, f32) -> f32`, `max(f32, f32) -> f32`

### Array Functions
- `array_length([T]) -> i32`
- `array_get([T], i32) -> T`

## Project Structure

```
ea-compiler/
├── src/
│   ├── lexer/          # Tokenization
│   ├── parser/         # Syntax analysis
│   ├── ast.rs          # Abstract Syntax Tree
│   ├── type_system/    # Type checking
│   ├── codegen/        # LLVM code generation
│   ├── error.rs        # Error handling
│   └── main.rs         # CLI interface
├── tests/              # Integration tests
├── benches/            # Performance benchmarks  
├── examples/           # Example programs
└── docs/               # Documentation
```

## Testing

The compiler has comprehensive test coverage with 102 passing tests:

```bash
# Run all tests
cargo test --features=llvm

# Run with output
cargo test --features=llvm -- --nocapture

# Run benchmarks
cargo bench

# Test CLI
./target/release/ea --test
```

### Test Coverage

- **Lexer Tests** - Token recognition, position tracking, error handling
- **Parser Tests** - Expression parsing, statement parsing, AST generation
- **Type System Tests** - Type checking, inference, compatibility rules
- **Code Generation Tests** - LLVM IR generation, optimization
- **Integration Tests** - End-to-end compilation pipeline
- **Performance Tests** - Benchmarking and regression detection

## Performance

### Compilation Performance (2025)

- **Small Programs**: <100ms compilation
- **Large Programs**: 100k parameter function compiled in 1.2 seconds
- **Generated Code**: LLVM-optimized IR with comprehensive optimization passes
- **Test Suite**: 102/102 tests passing
- **JIT Execution**: Production-ready for compute workloads

### Execution Modes

**JIT Execution** (`ea --run program.ea`)
- **Strengths**: Immediate execution, perfect for compute workloads
- **Supports**: All language features, arithmetic, control flow, functions
- **Limitation**: I/O operations have compatibility constraints in some environments
- **Performance**: Excellent for mathematical and algorithmic code

**Static Compilation** (`ea program.ea`)
- **Strengths**: Generates deployable LLVM IR, full system compatibility
- **Supports**: Complete language feature set including all I/O operations
- **Deployment**: Use with `lli` or compile to native code via LLVM toolchain
- **Performance**: Optimized for production deployment scenarios

### Benchmarks

Run benchmarks with:
```bash
cargo bench --features=llvm
```

## CLI Usage

```bash
# Basic compilation
ea program.ea

# Verbose output
ea --verbose program.ea

# Show AST
ea --emit-ast program.ea

# Show tokens
ea --emit-tokens program.ea

# Show LLVM IR  
ea --emit-llvm program.ea

# JIT execution mode (immediate run)
ea --run program.ea

# Clean output for piping
ea --emit-llvm-only program.ea | lli

# Quiet mode (suppress diagnostics)
ea --quiet program.ea

# Run built-in tests
ea --test

# Help
ea --help
```

## Implementation Status

### ✅ **Completed Features**

#### **Core Language**
- **Complete compilation pipeline** - Source → Tokens → AST → Type-checked → LLVM IR
- **Arrays** - Literals `[1, 2, 3]`, indexing `arr[i]`, slicing `arr[1:3]`, iteration
- **Structs** - Declarations, instantiation `Point { x: 1.0, y: 2.0 }`, field access
- **Enums** - Variants with data, pattern matching with `match` expressions
- **Functions** - Parameters, return values, recursion
- **Control flow** - if/else, while loops, for loops, for-in iteration
- **Type system** - Strong typing, type inference, compatibility checking

#### **SIMD Support**
- **32 vector types** - f32x4, i32x8, u8x16, etc. covering all major SIMD widths
- **Element-wise operations** - `.+`, `.-`, `.*`, `./`, `.&`, `.|`, `.^`
- **Vector literals** - `[1.0, 2.0, 3.0, 4.0]f32x4`
- **Memory operations** - `load_vector()`, `store_vector()` with alignment
- **Hardware detection** - SSE, AVX, NEON target features

#### **Standard Library**
- **I/O functions** - `print()`, `println()`, `print_i32()`, `print_f32()`, `read_line()`
- **File operations** - `read_file()`, `write_file()`, `file_exists()`
- **String functions** - `string_length()`, `string_concat()`, `string_equals()`
- **Math functions** - `sqrt()`, `sin()`, `cos()`, `abs()`, `min()`, `max()`
- **Array functions** - `array_length()`, `array_get()`

#### **Developer Tools**
- **CLI interface** - Multiple output formats, JIT execution mode
- **Error handling** - Position-aware errors with helpful messages
- **Testing** - Comprehensive test suite with 102 passing tests

## Architecture

### Compilation Pipeline

```
Source Code
    ↓
Lexer (logos) → Tokens
    ↓  
Parser (recursive descent) → AST
    ↓
Type Checker → Typed AST
    ↓
Code Generator (inkwell) → LLVM IR
    ↓
LLVM → Machine Code
```

### Key Components

- **Lexer**: High-performance tokenization using `logos` crate
- **Parser**: Recursive descent parser with proper error recovery
- **Type System**: Comprehensive type checking with inference
- **Code Generator**: LLVM IR generation with optimization
- **Error System**: Position-aware errors with helpful diagnostics

## Contributing

We welcome contributions! Please see our [Getting Started Guide](docs/getting_started.md) for development setup.

### Development Workflow

1. **Clone and setup**:
   ```bash
   git clone <repo>
   cd ea-compiler
   cargo build --features=llvm
   ```

2. **Make changes and test**:
   ```bash
   cargo test --features=llvm
   cargo fmt
   cargo clippy
   ```

3. **Submit pull request**

### Areas for Contribution

- **Language features** - Additional control flow, more built-in functions
- **Testing** - More test cases, edge case coverage, performance benchmarks
- **Documentation** - Examples, tutorials, language specification
- **Standard library** - Additional math, string, and utility functions

## Documentation

- [Getting Started Guide](docs/getting-started.md)
- [Language Specification](specification-v01.md) - Full language reference
- [Examples](examples/) - Sample programs demonstrating features

## Test Results

All 102 tests passing. Run tests with:
```bash
cargo test --features=llvm
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Acknowledgments

- **Rust Language Team** - For an excellent systems programming foundation
- **LLVM Project** - For world-class optimization infrastructure  
- **Inkwell** - For excellent Rust LLVM bindings
- **Logos** - For high-performance lexer generation

---

<div align="center">

**Built with Rust**

[Examples](examples/) • [Documentation](docs/)

</div>