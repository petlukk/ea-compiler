#!/bin/bash
# vec_validation_script.sh

set -e

echo "=== VEC IMPLEMENTATION VALIDATION ==="

# Step 1: Compilation
echo "Step 1: Compiling validation test..."
./target/release/ea vec_validation.ea || {
    echo "FAILURE: Compilation failed"
    exit 1
}

# Step 2: LLVM IR Quality Check
echo "Step 2: Validating LLVM IR..."
llvm-as vec_validation.ll || {
    echo "FAILURE: Invalid LLVM IR"
    exit 1
}

# Check for required function calls
grep -q "@malloc\|@realloc\|@free\|@memcpy" vec_validation.ll || {
    echo "FAILURE: Required memory management functions not found in LLVM IR"
    exit 1
}

# Check for Vec method calls
grep -q "vec_new\|vec_push\|vec_get\|vec_len\|vec_pop\|vec_clear" vec_validation.ll || {
    echo "FAILURE: Required Vec functions not found in LLVM IR"
    exit 1
}

# Step 3: Execution Test
echo "Step 3: Running validation test..."
ACTUAL_OUTPUT=$(timeout 10s lli vec_validation.ll)
EXPECTED_OUTPUT="Test 1: Basic Vec operations
✓ Test 1 passed
Test 2: Pop operations
✓ Test 2 passed
Test 3: Capacity management
✓ Test 3 passed
Test 4: Clear operations
✓ Test 4 passed
Test 5: SIMD operations
✓ Test 5 passed
Test 6: Memory stress test
✓ Test 6 passed
Vec created
Vec tested
ALL TESTS PASSED"

if [ "$ACTUAL_OUTPUT" != "$EXPECTED_OUTPUT" ]; then
    echo "FAILURE: Output mismatch"
    echo "Expected: $EXPECTED_OUTPUT"
    echo "Actual: $ACTUAL_OUTPUT"
    exit 1
fi

# Step 4: Memory Safety
echo "Step 4: Memory safety validation..."
valgrind --leak-check=full --error-exitcode=1 lli vec_validation.ll > /dev/null 2>&1 || {
    echo "FAILURE: Memory leaks detected"
    exit 1
}

# Step 5: Stress Test
echo "Step 5: Stress testing..."
./target/release/ea vec_stress_test.ea
timeout 30s lli vec_stress_test.ll || {
    echo "FAILURE: Stress test failed"
    exit 1
}

# Step 6: Code Quality
echo "Step 6: Code quality check..."
grep -r "TODO\|PLACEHOLDER\|NOT IMPLEMENTED\|FIXME\|STUB" src/ && {
    echo "FAILURE: Placeholder code detected"
    exit 1
}

# Step 7: SIMD Performance Check
echo "Step 7: SIMD performance validation..."
# Check that SIMD operations are actually implemented
grep -q "simd_add\|simd_sum\|simd_dot" vec_validation.ll || {
    echo "FAILURE: SIMD operations not found in LLVM IR"
    exit 1
}

# Step 8: Memory Management Pattern Check
echo "Step 8: Memory management pattern validation..."
# Check for proper memory allocation patterns
grep -c "malloc" vec_validation.ll > /dev/null || {
    echo "FAILURE: No malloc calls found - memory management not implemented"
    exit 1
}

grep -c "free" vec_validation.ll > /dev/null || {
    echo "FAILURE: No free calls found - memory cleanup not implemented"
    exit 1
}

echo "=== ALL VALIDATION PASSED ==="
echo "VEC implementation is REAL and WORKING"