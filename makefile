# Makefile for Eä Compiler Development
# Provides convenient commands for common development tasks

.PHONY: all build test clean doc fmt lint bench install run-examples help quality-check

# Default target
all: build

# Build the compiler
build:
	@echo "🔨 Building Eä compiler..."
	cargo build --features=llvm

# Build release version
release:
	@echo "🚀 Building Eä compiler (release)..."
	cargo build --release --features=llvm

# Run all tests
test:
	@echo "🧪 Running test suite..."
	cargo test --features=llvm

# Run tests with output
test-verbose:
	@echo "🧪 Running test suite (verbose)..."
	cargo test --features=llvm -- --nocapture

# Run benchmarks
bench:
	@echo "⚡ Running performance benchmarks..."
	cargo bench --features=llvm

# Format code
fmt:
	@echo "🎨 Formatting code..."
	cargo fmt --all

# Run linter
lint:
	@echo "🔍 Running clippy linter..."
	cargo clippy --all-targets --all-features -- -D warnings

# Generate documentation
doc:
	@echo "📚 Generating documentation..."
	cargo doc --features=llvm --no-deps --open

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	rm -f *.ll *.o

# Install dependencies (Ubuntu/Debian)
install-deps:
	@echo "📦 Installing dependencies..."
	sudo apt update
	sudo apt install -y llvm-14-dev clang-14 build-essential

# Run quality checks
quality-check: fmt lint test
	@echo "✅ Quality checks completed!"

# Comprehensive check (recommended before commits)
check-all: quality-check bench doc
	@echo "🎉 All checks completed successfully!"

# Run example programs
run-examples: release
	@echo "📝 Testing example programs..."
	@if [ -f "examples/fibonacci.ea" ]; then \
		echo "Running fibonacci example..."; \
		./target/release/ea examples/fibonacci.ea; \
	fi
	@if [ -f "examples/hello_world.ea" ]; then \
		echo "Running hello world example..."; \
		./target/release/ea examples/hello_world.ea; \
	fi

# Create example programs directory and files
create-examples:
	@echo "📁 Creating example programs..."
	@mkdir -p examples
	@echo 'func main() -> () {\n    print("Hello, World!");\n    return;\n}' > examples/hello_world.ea
	@echo 'func fibonacci(n: i32) -> i32 {\n    if (n <= 1) {\n        return n;\n    }\n    return fibonacci(n - 1) + fibonacci(n - 2);\n}\n\nfunc main() -> () {\n    let result = fibonacci(10);\n    print("Fibonacci calculation complete");\n    return;\n}' > examples/fibonacci.ea
	@echo "✅ Example programs created in examples/"

# Run the CLI with built-in tests
test-cli: release
	@echo "🧪 Testing CLI interface..."
	./target/release/ea --test

# Development setup (one-time setup for new contributors)
setup: install-deps create-examples build test
	@echo "🎉 Development environment setup complete!"
	@echo "📖 Next steps:"
	@echo "   1. Read docs/getting_started.md"
	@echo "   2. Try: make run-examples"
	@echo "   3. Try: make test-cli"
	@echo "   4. Start coding!"

# Performance regression check
perf-check:
	@echo "📊 Checking for performance regressions..."
	@echo "Running quick benchmark..."
	cargo bench --features=llvm 2>&1 | grep -E "(time:|ns/iter)"

# Memory usage check
memory-check:
	@echo "🧠 Checking memory usage..."
	@if command -v valgrind >/dev/null 2>&1; then \
		echo "Running with valgrind..."; \
		valgrind --tool=massif --stacks=yes ./target/release/ea --test; \
	else \
		echo "Install valgrind for memory analysis: sudo apt install valgrind"; \
	fi

# Security audit (requires cargo-audit)
security-audit:
	@echo "🔒 Running security audit..."
	@if command -v cargo-audit >/dev/null 2>&1; then \
		cargo audit; \
	else \
		echo "Install cargo-audit: cargo install cargo-audit"; \
	fi

# Code coverage (requires cargo-tarpaulin)
coverage:
	@echo "📊 Generating code coverage report..."
	@if command -v cargo-tarpaulin >/dev/null 2>&1; then \
		cargo tarpaulin --features=llvm --out Html --output-dir coverage; \
		echo "Coverage report generated in coverage/"; \
	else \
		echo "Install cargo-tarpaulin: cargo install cargo-tarpaulin"; \
	fi

# Sprint 2 preparation
sprint2-prep: check-all
	@echo "🚀 Preparing for Sprint 2..."
	@echo "✅ Current status:"
	@echo "   - All tests passing"
	@echo "   - Code quality excellent" 
	@echo "   - Performance benchmarks established"
	@echo "   - Documentation up to date"
	@echo ""
	@echo "🎯 Sprint 2 targets:"
	@echo "   - SIMD integration"
	@echo "   - Memory regions"
	@echo "   - Adaptive optimization"
	@echo "   - Security features"
	@echo ""
	@echo "Ready to begin Sprint 2 development! 🚀"

# Help target
help:
	@echo "🛠️  Eä Compiler Development Commands"
	@echo "===================================="
	@echo ""
	@echo "Build Commands:"
	@echo "  build           Build the compiler (debug)"
	@echo "  release         Build the compiler (release)"
	@echo "  clean           Clean build artifacts"
	@echo ""
	@echo "Testing Commands:"
	@echo "  test            Run all tests"
	@echo "  test-verbose    Run tests with output"
	@echo "  test-cli        Test CLI interface"
	@echo "  bench           Run performance benchmarks"
	@echo ""
	@echo "Quality Commands:"
	@echo "  fmt             Format code"
	@echo "  lint            Run clippy linter"
	@echo "  quality-check   Run fmt + lint + test"
	@echo "  check-all       Run all quality checks + bench + doc"
	@echo ""
	@echo "Documentation:"
	@echo "  doc             Generate and open documentation"
	@echo ""
	@echo "Examples:"
	@echo "  create-examples Create example programs"
	@echo "  run-examples    Run example programs"
	@echo ""
	@echo "Development:"
	@echo "  setup           One-time development setup"
	@echo "  install-deps    Install system dependencies"
	@echo "  sprint2-prep    Prepare for Sprint 2"
	@echo ""
	@echo "Analysis:"
	@echo "  perf-check      Check performance"
	@echo "  memory-check    Check memory usage"
	@echo "  security-audit  Run security audit"
	@echo "  coverage        Generate code coverage"
	@echo ""
	@echo "Shortcuts:"
	@echo "  make            Build (default)"
	@echo "  make help       Show this help"