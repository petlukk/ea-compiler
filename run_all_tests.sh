#!/bin/bash

# Comprehensive test runner for Eä compiler
# Captures all test results and errors to files

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results directory
RESULTS_DIR="test_results"
mkdir -p "$RESULTS_DIR"

# Timestamp for this test run
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
MAIN_LOG="$RESULTS_DIR/test_run_${TIMESTAMP}.log"

# Function to log with timestamp
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$MAIN_LOG"
}

# Function to run a test and capture output
run_test() {
    local test_name="$1"
    local test_command="$2"
    local output_file="$RESULTS_DIR/${test_name}_${TIMESTAMP}.log"
    
    log "🧪 Starting: $test_name"
    echo "Command: $test_command" > "$output_file"
    echo "Started: $(date)" >> "$output_file"
    echo "=================================" >> "$output_file"
    
    # Run the test and capture both stdout and stderr
    if timeout 600 bash -c "$test_command" >> "$output_file" 2>&1; then
        log "✅ PASSED: $test_name"
        echo "RESULT: PASSED" >> "$output_file"
        return 0
    else
        local exit_code=$?
        if [ $exit_code -eq 124 ]; then
            log "⏰ TIMEOUT: $test_name (600s)"
            echo "RESULT: TIMEOUT (600s)" >> "$output_file"
        else
            log "❌ FAILED: $test_name (exit code: $exit_code)"
            echo "RESULT: FAILED (exit code: $exit_code)" >> "$output_file"
        fi
        return $exit_code
    fi
}

# Function to run cargo test with specific filters
run_cargo_test() {
    local test_name="$1"
    local test_filter="$2"
    local extra_args="$3"
    
    run_test "$test_name" "cargo test --features=llvm $extra_args $test_filter"
}

# Start main log
log "🚀 Starting comprehensive test suite for Eä compiler"
log "Results will be saved in: $RESULTS_DIR"
log "Main log: $MAIN_LOG"

# Build the project first
log "🔧 Building project..."
if ! run_test "build_debug" "cargo build --features=llvm"; then
    log "❌ Build failed, aborting tests"
    exit 1
fi

if ! run_test "build_release" "cargo build --release --features=llvm"; then
    log "⚠️  Release build failed, continuing with debug tests"
fi

# Run built-in CLI tests
log "🧪 Running built-in CLI tests..."
run_test "cli_builtin_tests" "./target/debug/ea --test"

# Test CLI with different modes
log "🧪 Testing CLI modes..."
run_test "cli_version" "./target/debug/ea --version"
run_test "cli_help" "./target/debug/ea --help"

# Test with example files (if they exist)
if [ -f "tests/fibonacci.ea" ]; then
    run_test "cli_fibonacci" "./target/debug/ea tests/fibonacci.ea"
    run_test "cli_fibonacci_run" "./target/debug/ea --run tests/fibonacci.ea"
fi

if [ -f "tests/hello.ea" ]; then
    run_test "cli_hello_tokens" "./target/debug/ea --emit-tokens tests/hello.ea"
    run_test "cli_hello_ast" "./target/debug/ea --emit-ast tests/hello.ea"
    run_test "cli_hello_llvm" "./target/debug/ea --emit-llvm tests/hello.ea"
fi

# Run all unit tests
log "🧪 Running unit tests..."
run_cargo_test "unit_tests" "" "--lib"

# Run integration tests by category
log "🧪 Running integration tests..."

# Core functionality tests
run_cargo_test "core_functionality_tests" "core_functionality_tests" ""
run_cargo_test "fibonacci_test" "fibonacci_test" ""
run_cargo_test "lexer_tests" "lexer_tests" ""
run_cargo_test "parser_tests" "parser_error_recovery_tests" ""
run_cargo_test "type_system_tests" "type_system_tests" ""
run_cargo_test "integration_tests" "integration_tests" ""

# SIMD-specific tests
run_cargo_test "simd_lexer_tests" "simd_lexer_tests" ""
run_cargo_test "simd_codegen_tests" "simd_codegen_tests" ""
run_cargo_test "simd_integration_tests" "simd_integration_tests" ""

# Standard Library Integration tests
run_cargo_test "stdlib_integration_tests" "stdlib_integration_tests" ""
run_cargo_test "stdlib_tokenization_test" "test_stdlib_tokenization" ""
run_cargo_test "stdlib_parsing_test" "test_stdlib_parsing" ""
run_cargo_test "stdlib_type_checking_test" "test_stdlib_type_checking" ""
run_cargo_test "println_tokenization_test" "test_println_tokenization" ""
run_cargo_test "basic_stdlib_program_test" "test_basic_stdlib_program" ""

# Validation tests
run_cargo_test "final_validation_test" "final_validation_test" ""
run_cargo_test "implementation_validation_tests" "implementation_validation_tests" ""

# Production tests
run_cargo_test "production_stress_tests" "production_stress_tests" ""
run_cargo_test "production_stability_tests" "production_stability_tests" ""

# Performance tests  
run_cargo_test "performance_validation_test" "performance_validation_test" ""

# Simple error test
run_cargo_test "simple_error_test" "simple_error_test" ""

# Run all tests together (comprehensive)
log "🧪 Running comprehensive test suite..."
run_cargo_test "all_tests_comprehensive" "" "--all-targets"

# Run tests with verbose output
log "🧪 Running tests with verbose output..."
run_cargo_test "all_tests_verbose" "" "--all-targets -- --nocapture"

# Run specific test functions that exist
log "🧪 Running specific test functions..."
run_cargo_test "test_basic_tokenization" "test_basic_tokenization" ""
run_cargo_test "test_jit_cache_basic" "test_jit_cache_basic_operations" ""

# Quality checks
log "🧪 Running quality checks..."
run_test "format_check" "cargo fmt --check"
run_test "clippy_check" "cargo clippy --all-targets --all-features -- -D warnings"

# Documentation tests
log "🧪 Running documentation tests..."
run_test "doc_tests" "cargo test --doc --features=llvm"

# Benchmark tests (if available)
log "🧪 Running benchmark tests..."
run_test "benchmark_tests" "cargo bench --features=llvm --no-run"

# Generate summary report
SUMMARY_FILE="$RESULTS_DIR/test_summary_${TIMESTAMP}.txt"
log "📊 Generating test summary..."

echo "Eä Compiler Test Summary" > "$SUMMARY_FILE"
echo "========================" >> "$SUMMARY_FILE"
echo "Test run: $(date)" >> "$SUMMARY_FILE"
echo "Timestamp: $TIMESTAMP" >> "$SUMMARY_FILE"
echo "" >> "$SUMMARY_FILE"

# Count results
passed_tests=$(grep -l "RESULT: PASSED" "$RESULTS_DIR"/*_${TIMESTAMP}.log | wc -l)
failed_tests=$(grep -l "RESULT: FAILED" "$RESULTS_DIR"/*_${TIMESTAMP}.log | wc -l)
timeout_tests=$(grep -l "RESULT: TIMEOUT" "$RESULTS_DIR"/*_${TIMESTAMP}.log | wc -l)

echo "Results Summary:" >> "$SUMMARY_FILE"
echo "✅ Passed: $passed_tests" >> "$SUMMARY_FILE"
echo "❌ Failed: $failed_tests" >> "$SUMMARY_FILE"
echo "⏰ Timeout: $timeout_tests" >> "$SUMMARY_FILE"
echo "" >> "$SUMMARY_FILE"

# List failed tests
if [ $failed_tests -gt 0 ]; then
    echo "Failed Tests:" >> "$SUMMARY_FILE"
    grep -l "RESULT: FAILED" "$RESULTS_DIR"/*_${TIMESTAMP}.log | while read -r file; do
        test_name=$(basename "$file" "_${TIMESTAMP}.log")
        echo "  - $test_name" >> "$SUMMARY_FILE"
    done
    echo "" >> "$SUMMARY_FILE"
fi

# List timeout tests
if [ $timeout_tests -gt 0 ]; then
    echo "Timeout Tests:" >> "$SUMMARY_FILE"
    grep -l "RESULT: TIMEOUT" "$RESULTS_DIR"/*_${TIMESTAMP}.log | while read -r file; do
        test_name=$(basename "$file" "_${TIMESTAMP}.log")
        echo "  - $test_name" >> "$SUMMARY_FILE"
    done
    echo "" >> "$SUMMARY_FILE"
fi

# Add detailed results
echo "Detailed Results:" >> "$SUMMARY_FILE"
echo "=================" >> "$SUMMARY_FILE"
for log_file in "$RESULTS_DIR"/*_${TIMESTAMP}.log; do
    if [ -f "$log_file" ]; then
        test_name=$(basename "$log_file" "_${TIMESTAMP}.log")
        result=$(grep "RESULT:" "$log_file" | tail -1)
        echo "$test_name: $result" >> "$SUMMARY_FILE"
    fi
done

# Final summary
log "📊 Test run completed!"
log "Summary written to: $SUMMARY_FILE"
log "Individual test logs in: $RESULTS_DIR"

# Display summary
echo ""
echo -e "${BLUE}Test Summary:${NC}"
echo -e "${GREEN}✅ Passed: $passed_tests${NC}"
if [ $failed_tests -gt 0 ]; then
    echo -e "${RED}❌ Failed: $failed_tests${NC}"
fi
if [ $timeout_tests -gt 0 ]; then
    echo -e "${YELLOW}⏰ Timeout: $timeout_tests${NC}"
fi

# Exit with appropriate code
if [ $failed_tests -gt 0 ] || [ $timeout_tests -gt 0 ]; then
    exit 1
else
    exit 0
fi