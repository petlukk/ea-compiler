#!/bin/bash

# Comprehensive benchmark runner for Eä compiler
# Captures all benchmark results and performance data to files

# set -e  # Temporarily disabled to allow individual benchmark failures

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Benchmark results directory
RESULTS_DIR="benchmark_results"
mkdir -p "$RESULTS_DIR"

# Timestamp for this benchmark run
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
MAIN_LOG="$RESULTS_DIR/benchmark_run_${TIMESTAMP}.log"

# Function to log with timestamp
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$MAIN_LOG"
}

# Function to run a benchmark and capture output
run_benchmark() {
    local bench_name="$1"
    local bench_command="$2"
    local output_file="$RESULTS_DIR/${bench_name}_${TIMESTAMP}.log"
    
    log "🚀 Starting benchmark: $bench_name"
    echo "Command: $bench_command" > "$output_file"
    echo "Started: $(date)" >> "$output_file"
    echo "=================================" >> "$output_file"
    
    # Run the benchmark with extended timeout (benchmarks can take longer)
    if timeout 1200 bash -c "$bench_command" >> "$output_file" 2>&1; then
        log "✅ COMPLETED: $bench_name"
        echo "RESULT: COMPLETED" >> "$output_file"
        return 0
    else
        local exit_code=$?
        if [ $exit_code -eq 124 ]; then
            log "⏰ TIMEOUT: $bench_name (1200s)"
            echo "RESULT: TIMEOUT (1200s)" >> "$output_file"
        else
            log "❌ FAILED: $bench_name (exit code: $exit_code)"
            echo "RESULT: FAILED (exit code: $exit_code)" >> "$output_file"
        fi
        return $exit_code
    fi
}

# Start main log
log "🚀 Starting comprehensive benchmark suite for Eä compiler"
log "Results will be saved in: $RESULTS_DIR"
log "Main log: $MAIN_LOG"

# Build the project first (release mode for accurate benchmarks)
log "🔧 Building project in release mode..."
if ! run_benchmark "build_release" "cargo build --release --features=llvm"; then
    log "❌ Release build failed, aborting benchmarks"
    exit 1
fi

# Check if benchmark compilation works
log "🔧 Checking benchmark compilation..."
if ! run_benchmark "bench_compile_check" "cargo bench --features=llvm --no-run"; then
    log "⚠️  Some benchmarks failed to compile, continuing with available ones"
fi

# Run individual benchmark suites
log "📊 Running individual benchmark suites..."

# Core performance benchmarks
log "🧪 Running core performance benchmarks..."
run_benchmark "frontend_performance" "cargo bench --features=llvm --bench frontend_performance"
run_benchmark "compilation_performance" "cargo bench --features=llvm --bench compilation_performance"

# Advanced feature benchmarks - Eä's competitive advantages
log "🚀 Running advanced feature benchmarks (Eä's strengths)..."
run_benchmark "ea_advanced_vs_full_pipeline" "cargo bench --features=llvm --bench ea_advanced_vs_full_pipeline"
run_benchmark "simd_performance" "cargo bench --features=llvm --bench simd_performance_benchmarks"

# Cross-language comparisons
log "🏁 Running cross-language comparisons..."
run_benchmark "cross_language_advanced" "cargo bench --features=llvm --bench simple_cross_language"
run_benchmark "cross_language_full_pipeline" "cargo bench --features=llvm --bench cross_language_comparison"

# Legacy benchmarks (if they exist)
log "📊 Running legacy benchmarks..."
run_benchmark "honest_full_pipeline" "cargo bench --features=llvm --bench honest_full_pipeline_benchmarks" || true
run_benchmark "competitive_benchmarks" "cargo bench --features=llvm --bench competitive_benchmarks" || true
run_benchmark "competitive_performance_validation" "cargo bench --features=llvm --bench competitive_performance_validation" || true

# Run all benchmarks together (comprehensive)
log "🎯 Running comprehensive benchmark suite..."
run_benchmark "all_benchmarks_comprehensive" "cargo bench --features=llvm"

# CLI performance tests with built-in examples
log "⚡ Testing CLI performance..."
if [ -f "tests/fibonacci.ea" ]; then
    run_benchmark "cli_fibonacci_jit" "./target/release/ea --run tests/fibonacci.ea"
    run_benchmark "cli_fibonacci_compile" "./target/release/ea tests/fibonacci.ea"
fi

if [ -f "tests/hello.ea" ]; then
    run_benchmark "cli_hello_compile" "./target/release/ea tests/hello.ea"
    run_benchmark "cli_hello_ast" "./target/release/ea --emit-ast tests/hello.ea"
    run_benchmark "cli_hello_llvm" "./target/release/ea --emit-llvm tests/hello.ea"
fi

# Memory usage benchmarks
log "💾 Running memory usage analysis..."
run_benchmark "memory_usage_large_program" "timeout 300 valgrind --tool=massif --time-unit=ms ./target/release/ea tests/fibonacci.ea 2>/dev/null || echo 'Valgrind not available'"

# Compilation speed benchmarks
log "⚡ Running compilation speed tests..."
run_benchmark "compilation_speed_small" "time ./target/release/ea --emit-llvm-only tests/hello.ea"
run_benchmark "compilation_speed_medium" "time ./target/release/ea --emit-llvm-only tests/fibonacci.ea"

# SIMD-specific performance tests
log "🔢 Running SIMD performance tests..."
if [ -f "tests/simd_test.ea" ]; then
    run_benchmark "simd_compilation" "./target/release/ea tests/simd_test.ea"
    run_benchmark "simd_jit_execution" "./target/release/ea --run tests/simd_test.ea"
fi

# Development cycle benchmarks (edit-compile-run)
log "🔄 Running development cycle benchmarks..."
run_benchmark "dev_cycle_benchmark" "bash -c 'for i in {1..5}; do time ./target/release/ea tests/fibonacci.ea > /dev/null; done'"

# Generate performance summary report
SUMMARY_FILE="$RESULTS_DIR/benchmark_summary_${TIMESTAMP}.txt"
log "📊 Generating benchmark summary..."

echo "Eä Compiler Benchmark Summary" > "$SUMMARY_FILE"
echo "=============================" >> "$SUMMARY_FILE"
echo "Benchmark run: $(date)" >> "$SUMMARY_FILE"
echo "Timestamp: $TIMESTAMP" >> "$SUMMARY_FILE"
echo "" >> "$SUMMARY_FILE"

# Count results
completed=$(grep -l "RESULT: COMPLETED" "$RESULTS_DIR"/*_${TIMESTAMP}.log 2>/dev/null | wc -l)
failed=$(grep -l "RESULT: FAILED" "$RESULTS_DIR"/*_${TIMESTAMP}.log 2>/dev/null | wc -l)
timeout=$(grep -l "RESULT: TIMEOUT" "$RESULTS_DIR"/*_${TIMESTAMP}.log 2>/dev/null | wc -l)

echo "Results Summary:" >> "$SUMMARY_FILE"
echo "✅ Completed: $completed" >> "$SUMMARY_FILE"
echo "❌ Failed: $failed" >> "$SUMMARY_FILE"
echo "⏰ Timeout: $timeout" >> "$SUMMARY_FILE"
echo "" >> "$SUMMARY_FILE"

# List failed benchmarks
if [ $failed -gt 0 ]; then
    echo "Failed Benchmarks:" >> "$SUMMARY_FILE"
    grep -l "RESULT: FAILED" "$RESULTS_DIR"/*_${TIMESTAMP}.log 2>/dev/null | while read -r file; do
        bench_name=$(basename "$file" "_${TIMESTAMP}.log")
        echo "  - $bench_name" >> "$SUMMARY_FILE"
    done
    echo "" >> "$SUMMARY_FILE"
fi

# List timeout benchmarks
if [ $timeout -gt 0 ]; then
    echo "Timeout Benchmarks:" >> "$SUMMARY_FILE"
    grep -l "RESULT: TIMEOUT" "$RESULTS_DIR"/*_${TIMESTAMP}.log 2>/dev/null | while read -r file; do
        bench_name=$(basename "$file" "_${TIMESTAMP}.log")
        echo "  - $bench_name" >> "$SUMMARY_FILE"
    done
    echo "" >> "$SUMMARY_FILE"
fi

# Extract key performance metrics from completed benchmarks
echo "Performance Highlights:" >> "$SUMMARY_FILE"
echo "======================" >> "$SUMMARY_FILE"

# Look for specific performance indicators in the logs
for log_file in "$RESULTS_DIR"/*_${TIMESTAMP}.log; do
    if [ -f "$log_file" ] && grep -q "RESULT: COMPLETED" "$log_file"; then
        bench_name=$(basename "$log_file" "_${TIMESTAMP}.log")
        
        # Extract timing information
        if grep -q "time:" "$log_file"; then
            timing=$(grep "time:" "$log_file" | head -1)
            echo "$bench_name: $timing" >> "$SUMMARY_FILE"
        fi
        
        # Extract throughput information
        if grep -q "throughput:" "$log_file"; then
            throughput=$(grep "throughput:" "$log_file" | head -1)
            echo "$bench_name: $throughput" >> "$SUMMARY_FILE"
        fi
        
        # Extract memory usage
        if grep -q "memory:" "$log_file"; then
            memory=$(grep "memory:" "$log_file" | head -1)
            echo "$bench_name: $memory" >> "$SUMMARY_FILE"
        fi
    fi
done

# Add detailed results
echo "" >> "$SUMMARY_FILE"
echo "Detailed Results:" >> "$SUMMARY_FILE"
echo "=================" >> "$SUMMARY_FILE"
for log_file in "$RESULTS_DIR"/*_${TIMESTAMP}.log; do
    if [ -f "$log_file" ]; then
        bench_name=$(basename "$log_file" "_${TIMESTAMP}.log")
        result=$(grep "RESULT:" "$log_file" | tail -1)
        echo "$bench_name: $result" >> "$SUMMARY_FILE"
    fi
done

# Performance analysis recommendations
echo "" >> "$SUMMARY_FILE"
echo "Analysis & Recommendations:" >> "$SUMMARY_FILE"
echo "===========================" >> "$SUMMARY_FILE"
echo "1. Review failed benchmarks for compilation or runtime issues" >> "$SUMMARY_FILE"
echo "2. Compare performance metrics against baseline measurements" >> "$SUMMARY_FILE"
echo "3. Investigate timeout benchmarks for optimization opportunities" >> "$SUMMARY_FILE"
echo "4. Use detailed logs for performance regression analysis" >> "$SUMMARY_FILE"

# Final summary
log "📊 Benchmark run completed!"
log "Summary written to: $SUMMARY_FILE"
log "Individual benchmark logs in: $RESULTS_DIR"

# Display summary
echo ""
echo -e "${BLUE}Benchmark Summary:${NC}"
echo -e "${GREEN}✅ Completed: $completed${NC}"
if [ $failed -gt 0 ]; then
    echo -e "${RED}❌ Failed: $failed${NC}"
fi
if [ $timeout -gt 0 ]; then
    echo -e "${YELLOW}⏰ Timeout: $timeout${NC}"
fi

echo ""
echo -e "${PURPLE}📊 Summary: $SUMMARY_FILE${NC}"
echo -e "${PURPLE}📁 Individual logs: $RESULTS_DIR/${NC}"

# Return appropriate exit code
if [ $failed -gt 0 ] || [ $timeout -gt 0 ]; then
    exit 1
else
    exit 0
fi