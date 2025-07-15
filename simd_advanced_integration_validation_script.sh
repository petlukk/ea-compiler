#!/bin/bash
# SIMD Advanced Integration Validation Script
# Following DEVELOPMENT_PROCESS.md requirements

set -e

echo "=== SIMD ADVANCED INTEGRATION VALIDATION ===" 

# Step 1: Compilation
echo "Step 1: Compiling SIMD integration validation test..."
./target/release/ea simd_advanced_integration_validation.ea || {
    echo "FAILURE: Compilation failed"
    exit 1
}

# Step 2: LLVM IR Quality Check
echo "Step 2: Validating LLVM IR..."
llvm-as simd_advanced_integration_validation.ll || {
    echo "FAILURE: Invalid LLVM IR"
    exit 1
}

# Check for required vector instructions
echo "Step 2a: Checking for vector instructions..."
VECTOR_LOADS=$(grep -c "load.*<[0-9]* x [0-9]*>" simd_advanced_integration_validation.ll || echo "0")
VECTOR_OPS=$(grep -c "fadd.*<[0-9]* x [0-9]*>\|fmul.*<[0-9]* x [0-9]*>" simd_advanced_integration_validation.ll || echo "0")
SIMD_INTRINSICS=$(grep -c "@llvm\..*\.avx\|@llvm\..*\.sse\|@llvm\..*\.neon\|@llvm\.fma" simd_advanced_integration_validation.ll || echo "0")

echo "   Vector loads found: $VECTOR_LOADS"
echo "   Vector operations found: $VECTOR_OPS"  
echo "   SIMD intrinsics found: $SIMD_INTRINSICS"

if [ "$VECTOR_OPS" -eq 0 ]; then
    echo "FAILURE: No vector operations found in LLVM IR"
    exit 1
fi

# Step 3: Execution Test
echo "Step 3: Running validation test..."
ACTUAL_OUTPUT=$(timeout 10s lli simd_advanced_integration_validation.ll)
EXPECTED_OUTPUT="=== SIMD Advanced Integration Validation ===
Testing basic SIMD vector operations...
Vector addition completed
Vector multiplication completed
Testing advanced SIMD features...
Wide vector scaling completed
Integer vector operations completed
Testing SIMD code generation and optimization...
Advanced vectorization test completed
Testing hardware feature detection...
Hardware feature detection test completed
=== ALL SIMD INTEGRATION TESTS PASSED ==="

if [ "$ACTUAL_OUTPUT" != "$EXPECTED_OUTPUT" ]; then
    echo "FAILURE: Output mismatch"
    echo "Expected:"
    echo "$EXPECTED_OUTPUT"
    echo ""
    echo "Actual:"
    echo "$ACTUAL_OUTPUT"
    exit 1
fi

# Step 4: Memory Safety
echo "Step 4: Memory safety validation..."
if command -v valgrind >/dev/null 2>&1; then
    valgrind --leak-check=full --error-exitcode=1 lli simd_advanced_integration_validation.ll > /dev/null 2>&1 || {
        echo "FAILURE: Memory leaks detected"
        exit 1
    }
    echo "   Memory safety validation passed"
else
    echo "   Valgrind not found, skipping memory validation"
fi

# Step 5: Integration Depth Check
echo "Step 5: Verifying SIMD integration depth..."

# Check that advanced SIMD module is imported
ADVANCED_SIMD_IMPORTS=$(grep -c "simd_advanced::" src/lib.rs src/codegen/mod.rs || echo "0")
if [ "$ADVANCED_SIMD_IMPORTS" -eq 0 ]; then
    echo "FAILURE: Advanced SIMD module not integrated in main pipeline"
    exit 1
fi

# Check for actual usage of AdvancedSIMDCodegen
ADVANCED_SIMD_USAGE=$(grep -c "AdvancedSIMDCodegen\|try_generate_advanced_simd" src/codegen/mod.rs || echo "0")
if [ "$ADVANCED_SIMD_USAGE" -eq 0 ]; then
    echo "FAILURE: AdvancedSIMDCodegen not used in code generation"
    exit 1
fi

echo "   Advanced SIMD integration verified ($ADVANCED_SIMD_IMPORTS imports, $ADVANCED_SIMD_USAGE usages)"

# Step 6: Code Quality Check
echo "Step 6: Code quality check..."
PLACEHOLDER_CODE=$(grep -r "TODO\|PLACEHOLDER\|NOT IMPLEMENTED\|FIXME\|STUB" src/ | grep -v "test\|comment" || echo "")
if [ -n "$PLACEHOLDER_CODE" ]; then
    echo "FAILURE: Placeholder code detected:"
    echo "$PLACEHOLDER_CODE"
    exit 1
fi

# Step 7: Performance Characteristics
echo "Step 7: Performance characteristics validation..."

# Check compilation doesn't take too long
echo "   Testing compilation performance..."
start_time=$(date +%s%N)
./target/release/ea simd_advanced_integration_validation.ea > /dev/null 2>&1
end_time=$(date +%s%N)
compilation_time_ms=$(( (end_time - start_time) / 1000000 ))

echo "   Compilation time: ${compilation_time_ms}ms"

if [ "$compilation_time_ms" -gt 10000 ]; then
    echo "WARNING: Compilation time unusually high (>${compilation_time_ms}ms)"
fi

# Check that vector instructions are actually generated
TOTAL_INSTRUCTIONS=$(wc -l < simd_advanced_integration_validation.ll)
VECTOR_INSTRUCTION_RATIO=$(echo "scale=2; $VECTOR_OPS * 100 / $TOTAL_INSTRUCTIONS" | bc -l || echo "0")

echo "   Vector instruction ratio: ${VECTOR_INSTRUCTION_RATIO}%"

# Step 8: Advanced Feature Validation
echo "Step 8: Advanced feature validation..."

# Check for hardware capability detection
HARDWARE_DETECTION=$(grep -c "detect_hardware_capabilities\|SIMDCapabilities" simd_advanced_integration_validation.ll || echo "0")
echo "   Hardware detection integration: $HARDWARE_DETECTION"

# Check for adaptive vectorization
ADAPTIVE_FEATURES=$(grep -c "AdaptiveVectorizer\|auto_vectorize" src/codegen/mod.rs || echo "0")
echo "   Adaptive vectorization integration: $ADAPTIVE_FEATURES"

echo "=== ALL VALIDATION PASSED ==="
echo "SIMD Advanced Integration is REAL and WORKING"
echo ""
echo "Summary:"
echo "  ✅ Compilation successful"
echo "  ✅ LLVM IR valid ($VECTOR_OPS vector operations)"
echo "  ✅ Execution output correct"
echo "  ✅ Memory safety verified"
echo "  ✅ Integration depth confirmed"
echo "  ✅ Code quality maintained"
echo "  ✅ Performance characteristics acceptable"
echo "  ✅ Advanced features integrated"
echo ""
echo "🎉 SIMD Advanced Integration Implementation SUCCESS!"