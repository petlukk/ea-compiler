# Eä Programming Language Compiler

<div align="center">

**A systems programming language with native SIMD support and efficient compilation**

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![LLVM](https://img.shields.io/badge/LLVM-14-blue.svg?style=flat-square)](https://llvm.org/)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green.svg?style=flat-square)](#license)
[![Tests](https://img.shields.io/badge/tests-102%20passing-brightgreen.svg?style=flat-square)](#testing)

</div>

## Overview

Eä is a systems programming language compiler that generates LLVM IR from source code. It features comprehensive SIMD support, strong typing, and an efficient compilation pipeline.

**Current Status**: v0.1.1 - Working static compilation and JIT execution engine with comprehensive test coverage.

### Core Features (v0.1.1)

**Compilation Pipeline**
- **LLVM-based compilation** - Lexer → Parser → Type Checker → LLVM Code Generator
- **JIT execution engine** - Immediate program execution for supported operations
- **Static compilation** - Generates LLVM IR that can be executed with `lli`
- **Error handling** - Position-aware errors with source location information

**Language Features**
- **Type system** - Type checking, inference, and compatibility validation
- **Control flow** - if/else statements, while/for loops, pattern matching
- **Functions** - Parameters, return values, recursion support
- **Data structures** - Arrays, structs, enums with data variants
- **Memory management** - Stack-based allocation with LLVM-managed cleanup

**SIMD Support**
- **32 SIMD vector types** - f32x4, i32x8, u8x16, etc. covering standard widths
- **Element-wise operations** - .+, .-, .*, ./, .&, .|, .^ for vectors
- **LLVM code generation** - Generates optimal vector instructions (AVX2, SSE4.2)
- **Target features** - Compiler enables appropriate instruction sets

**Standard Library**
- **I/O functions** - print(), println(), read_file(), write_file()
- **Math functions** - sqrt(), sin(), cos(), abs(), min(), max()
- **String functions** - length, concatenation, equality
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

### Validated Performance Characteristics (2025)

**Compilation Speed** (validated with equivalent programs):
- **vs C++**: 30% faster compilation (0.743s vs 1.079s)
- **vs Rust**: 36% faster compilation (0.743s vs 1.156s)  
- **vs Go**: 3.4x slower compilation (0.754s vs 0.222s)
- **Memory usage**: 18MB during compilation

**Code Generation**:
- **SIMD instructions**: Generates AVX2/SSE4.2 vector operations
- **LLVM optimization**: Target features +avx2,+sse4.2,+fma enabled
- **Test suite**: 102/102 tests passing

### Execution Modes

**JIT Execution** (`ea --run program.ea`)
- **Use case**: Immediate execution for testing and development
- **Supports**: Language features, arithmetic, control flow, functions
- **I/O**: Basic operations supported (println, print)
- **Performance**: ~0.9s execution time including compilation

**Static Compilation** (`ea program.ea`)
- **Use case**: Generates LLVM IR for deployment
- **Supports**: Complete language feature set including I/O operations
- **Deployment**: Execute with `lli program.ll` or process with LLVM tools
- **Performance**: ~0.7s compilation time for typical programs

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
- **Compilation pipeline** - Source → Tokens → AST → Type-checked → LLVM IR
- **Arrays** - Literals `[1, 2, 3]`, indexing `arr[i]`, slicing `arr[1:3]`, iteration
- **Structs** - Declarations, instantiation `Point { x: 1.0, y: 2.0 }`, field access
- **Enums** - Variants with data, pattern matching with `match` expressions
- **Functions** - Parameters, return values, recursion
- **Control flow** - if/else, while loops, for loops, for-in iteration
- **Type system** - Type checking, type inference, compatibility checking

#### **SIMD Support**
- **32 vector types** - f32x4, i32x8, u8x16, etc. covering standard SIMD widths
- **Element-wise operations** - `.+`, `.-`, `.*`, `./`, `.&`, `.|`, `.^`
- **Vector literals** - `[1.0, 2.0, 3.0, 4.0]f32x4`
- **Code generation** - Produces AVX2/SSE4.2 LLVM vector instructions
- **Target features** - Enables appropriate SIMD instruction sets

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
- **Parser**: Recursive descent parser with error recovery
- **Type System**: Type checking with inference capabilities
- **Code Generator**: LLVM IR generation with SIMD support
- **Error System**: Position-aware errors with source location information

## Competitive Position

### Performance Comparison (Validated 2025)

| Language | Compilation Speed | Memory Usage | SIMD Support | Use Case |
|----------|------------------|--------------|--------------|----------|
| **Eä** | 0.74s | 18MB | Native syntax | Performance computing |
| **Go** | 0.22s ⭐ | 26MB | None | Web services |
| **C++** | 1.08s | 142MB | Intrinsics | Systems programming |
| **Rust** | 1.16s | 131MB | Experimental | Systems programming |

### Eä's Unique Advantages

- **Native SIMD syntax**: Only systems language with intuitive vector operations
- **Memory efficiency**: 31% less memory than Go, 8x less than C++/Rust during compilation
- **Fast iteration**: 30-50% faster compilation than C++/Rust for development cycles
- **LLVM integration**: Direct access to modern optimization infrastructure

### Target Applications

- **Scientific computing**: Native SIMD for mathematical operations
- **Game development**: Vector operations for graphics and physics
- **Signal processing**: Audio, video, and data stream processing
- **Performance libraries**: Building high-performance computational components

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