#!/bin/bash

# Cross-Platform Validation Script for Eä Compiler v0.2
# Tests compilation and performance across multiple platforms
# Week 3: Production Readiness - Day 15-17

set -e

echo "🌐 Cross-Platform Validation for Eä Compiler v0.2"
echo "=================================================="
echo ""

# Configuration
TARGETS=(
    "x86_64-unknown-linux-gnu"
    "x86_64-pc-windows-gnu"
    "x86_64-apple-darwin"
    "aarch64-unknown-linux-gnu"
)

CRITICAL_TESTS=(
    "test_basic_tokenization"
    "test_basic_parsing"
    "test_basic_type_checking"
    "test_llvm_compilation"
    "test_simd_vector_literal_codegen"
    "test_fibonacci_compiles_to_llvm"
)

PERFORMANCE_THRESHOLD=5 # 5% variance allowed
RESULTS_DIR="cross_platform_results"
mkdir -p "$RESULTS_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

success_count=0
failure_count=0

log_result() {
    local level=$1
    local message=$2
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    case $level in
        "SUCCESS") echo -e "${GREEN}✅ [$timestamp] $message${NC}" ;;
        "FAILURE") echo -e "${RED}❌ [$timestamp] $message${NC}" ;;
        "WARNING") echo -e "${YELLOW}⚠️  [$timestamp] $message${NC}" ;;
        "INFO")    echo -e "${BLUE}ℹ️  [$timestamp] $message${NC}" ;;
    esac
}

# Function to test basic compilation for a target
test_target_compilation() {
    local target=$1
    local result_file="$RESULTS_DIR/compilation_${target}.log"
    
    log_result "INFO" "Testing compilation for target: $target"
    
    # Check if target is installed
    if ! rustup target list --installed | grep -q "$target"; then
        log_result "INFO" "Installing target: $target"
        if ! rustup target add "$target" 2>"$result_file"; then
            log_result "FAILURE" "Failed to install target: $target"
            return 1
        fi
    fi
    
    # Test compilation
    local start_time=$(date +%s%3N)
    if cargo build --features=llvm --target="$target" &>"$result_file"; then
        local end_time=$(date +%s%3N)
        local duration=$((end_time - start_time))
        log_result "SUCCESS" "Compilation successful for $target (${duration}ms)"
        echo "$duration" > "$RESULTS_DIR/build_time_${target}.txt"
        return 0
    else
        log_result "FAILURE" "Compilation failed for $target"
        return 1
    fi
}

# Function to run tests for a target
test_target_tests() {
    local target=$1
    local result_file="$RESULTS_DIR/tests_${target}.log"
    
    log_result "INFO" "Running tests for target: $target"
    
    # Test each critical test individually
    local passed=0
    local failed=0
    
    for test in "${CRITICAL_TESTS[@]}"; do
        if timeout 30 cargo test --features=llvm --target="$target" "$test" &>"$result_file"; then
            log_result "SUCCESS" "Test '$test' passed for $target"
            ((passed++))
        else
            log_result "FAILURE" "Test '$test' failed for $target"
            ((failed++))
        fi
    done
    
    echo "PASSED: $passed, FAILED: $failed" > "$RESULTS_DIR/test_summary_${target}.txt"
    
    if [ $failed -eq 0 ]; then
        log_result "SUCCESS" "All critical tests passed for $target"
        return 0
    else
        log_result "FAILURE" "$failed tests failed for $target"
        return 1
    fi
}

# Function to test performance consistency
test_performance_consistency() {
    log_result "INFO" "Testing performance consistency across platforms"
    
    local baseline_time=""
    local variance_results=()
    
    # Use Linux as baseline if available
    if [ -f "$RESULTS_DIR/build_time_x86_64-unknown-linux-gnu.txt" ]; then
        baseline_time=$(cat "$RESULTS_DIR/build_time_x86_64-unknown-linux-gnu.txt")
        log_result "INFO" "Using Linux build time as baseline: ${baseline_time}ms"
    else
        log_result "WARNING" "No Linux baseline available"
        return 1
    fi
    
    # Compare all targets against baseline
    for target in "${TARGETS[@]}"; do
        local time_file="$RESULTS_DIR/build_time_${target}.txt"
        if [ -f "$time_file" ]; then
            local target_time=$(cat "$time_file")
            local variance=$(echo "scale=2; (($target_time - $baseline_time) / $baseline_time) * 100" | bc -l)
            local abs_variance=$(echo "$variance" | tr -d '-')
            
            if [ $(echo "$abs_variance > $PERFORMANCE_THRESHOLD" | bc) -eq 1 ]; then
                log_result "FAILURE" "Performance variance too high for $target: ${variance}%"
                variance_results+=("FAIL")
            else
                log_result "SUCCESS" "Performance within threshold for $target: ${variance}%"
                variance_results+=("PASS")
            fi
        else
            log_result "WARNING" "No build time data for $target"
            variance_results+=("SKIP")
        fi
    done
    
    # Summary
    local variance_passes=0
    for result in "${variance_results[@]}"; do
        if [ "$result" = "PASS" ]; then
            ((variance_passes++))
        fi
    done
    
    if [ $variance_passes -eq ${#TARGETS[@]} ]; then
        log_result "SUCCESS" "All targets meet performance consistency requirements"
        return 0
    else
        log_result "FAILURE" "Performance consistency check failed"
        return 1
    fi
}

# Function to generate final report
generate_report() {
    local report_file="$RESULTS_DIR/cross_platform_report.md"
    
    cat > "$report_file" << 'EOF'
# Cross-Platform Validation Report

## Summary
This report details the cross-platform validation results for the Eä Compiler v0.2.

## Test Results by Target

EOF

    for target in "${TARGETS[@]}"; do
        echo "### $target" >> "$report_file"
        
        # Compilation results
        if [ -f "$RESULTS_DIR/build_time_${target}.txt" ]; then
            local build_time=$(cat "$RESULTS_DIR/build_time_${target}.txt")
            echo "- **Build Time**: ${build_time}ms" >> "$report_file"
            echo "- **Compilation Status**: ✅ SUCCESS" >> "$report_file"
        else
            echo "- **Compilation Status**: ❌ FAILED" >> "$report_file"
        fi
        
        # Test results
        if [ -f "$RESULTS_DIR/test_summary_${target}.txt" ]; then
            local test_summary=$(cat "$RESULTS_DIR/test_summary_${target}.txt")
            echo "- **Test Results**: $test_summary" >> "$report_file"
        else
            echo "- **Test Results**: Not available" >> "$report_file"
        fi
        
        echo "" >> "$report_file"
    done
    
    echo "## Performance Consistency Analysis" >> "$report_file"
    echo "- **Threshold**: ±${PERFORMANCE_THRESHOLD}% variance allowed" >> "$report_file"
    echo "- **Baseline**: Linux x86_64 build time" >> "$report_file"
    echo "" >> "$report_file"
    
    # Performance data
    for target in "${TARGETS[@]}"; do
        if [ -f "$RESULTS_DIR/build_time_${target}.txt" ]; then
            local build_time=$(cat "$RESULTS_DIR/build_time_${target}.txt")
            echo "- **$target**: ${build_time}ms" >> "$report_file"
        fi
    done
    
    echo "" >> "$report_file"
    echo "## Recommendations" >> "$report_file"
    echo "- All targets should maintain <5% performance variance" >> "$report_file"
    echo "- Failed targets require investigation and fixes" >> "$report_file"
    echo "- SIMD instruction generation should be verified on all platforms" >> "$report_file"
    
    log_result "SUCCESS" "Cross-platform validation report generated: $report_file"
}

# Main execution
main() {
    log_result "INFO" "Starting cross-platform validation"
    
    # Test each target
    for target in "${TARGETS[@]}"; do
        echo ""
        log_result "INFO" "====== Testing target: $target ======"
        
        # Skip Windows and macOS if not available (WSL environment)
        if [[ "$target" == *"windows"* ]] || [[ "$target" == *"darwin"* ]]; then
            # Check if we're in WSL or limited environment
            if [[ ! -f /proc/version ]] || ! grep -q "Microsoft" /proc/version 2>/dev/null; then
                log_result "INFO" "Testing $target compilation (build only)"
                if test_target_compilation "$target"; then
                    ((success_count++))
                else
                    ((failure_count++))
                fi
            else
                log_result "WARNING" "Skipping $target in WSL environment"
            fi
        else
            # Test compilation
            if test_target_compilation "$target"; then
                ((success_count++))
                
                # Test critical functionality
                if test_target_tests "$target"; then
                    log_result "SUCCESS" "Target $target fully validated"
                else
                    ((failure_count++))
                fi
            else
                ((failure_count++))
            fi
        fi
    done
    
    echo ""
    log_result "INFO" "====== Performance Consistency Check ======"
    if test_performance_consistency; then
        log_result "SUCCESS" "Performance consistency validated"
    else
        log_result "FAILURE" "Performance consistency issues detected"
        ((failure_count++))
    fi
    
    echo ""
    log_result "INFO" "====== Final Results ======"
    log_result "SUCCESS" "Successful validations: $success_count"
    log_result "FAILURE" "Failed validations: $failure_count"
    
    # Generate report
    generate_report
    
    if [ $failure_count -eq 0 ]; then
        log_result "SUCCESS" "Cross-platform validation PASSED"
        exit 0
    else
        log_result "FAILURE" "Cross-platform validation FAILED"
        exit 1
    fi
}

# Run main function
main "$@"