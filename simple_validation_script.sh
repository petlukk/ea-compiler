#!/bin/bash
# simple_validation_script.sh

set -e

echo "=== SIMPLE VEC VALIDATION ==="

# Step 1: Basic Compilation with LLVM IR generation
echo "Step 1: Compiling simple Vec test..."
./target/release/ea --emit-llvm simple_vec_test.ea || {
    echo "FAILURE: Compilation failed"
    exit 1
}

# Step 2: LLVM IR Quality Check
echo "Step 2: Validating LLVM IR..."
llvm-as simple_vec_test.ll || {
    echo "FAILURE: Invalid LLVM IR"
    exit 1
}

# Check for required function calls
grep -q "@malloc\|vec_new\|vec_push\|vec_len\|vec_get" simple_vec_test.ll || {
    echo "FAILURE: Required Vec functions not found in LLVM IR"
    exit 1
}

# Step 3: Execution Test
echo "Step 3: Running simple test..."
ACTUAL_OUTPUT=$(timeout 10s ./target/release/ea --run simple_vec_test.ea 2>/dev/null | grep -E "Starting simple Vec test|Vec created|Element pushed|Length correct|Value correct|Simple Vec test completed")
EXPECTED_OUTPUT="Starting simple Vec test
Vec created
Element pushed
Length correct
Value correct
Simple Vec test completed"

if [ "$ACTUAL_OUTPUT" != "$EXPECTED_OUTPUT" ]; then
    echo "FAILURE: Output mismatch"
    echo "Expected: $EXPECTED_OUTPUT"
    echo "Actual: $ACTUAL_OUTPUT"
    exit 1
fi

echo "=== SIMPLE VALIDATION PASSED ==="
echo "Basic Vec functionality is WORKING"