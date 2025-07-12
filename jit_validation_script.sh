#!/bin/bash
# JIT Execution Validation Script - DEVELOPMENT_PROCESS.md compliance

set -e

echo "=== JIT EXECUTION IMPLEMENTATION VALIDATION ==="

# Step 1: Compilation
echo "Step 1: Compiling JIT validation test..."
./target/release/ea jit_validation.ea || {
    echo "FAILURE: Static compilation failed"
    exit 1
}

# Step 2: LLVM IR Quality Check  
echo "Step 2: Validating LLVM IR..."
if [ -f "jit_validation.ll" ]; then
    llvm-as jit_validation.ll || {
        echo "FAILURE: Invalid LLVM IR"
        exit 1
    }
else
    echo "WARNING: No LLVM IR file generated"
fi

# Step 3: JIT Execution Test (THE CRITICAL TEST)
echo "Step 3: Testing JIT execution..."
EXPECTED_OUTPUT="JIT execution working"

# This MUST work without segfault
ACTUAL_OUTPUT=$(timeout 10s ./target/release/ea --run jit_validation.ea 2>/dev/null | tail -1) || {
    echo "FAILURE: JIT execution crashed or timed out"
    echo "This violates DEVELOPMENT_PROCESS.md - the full pipeline must work"
    exit 1
}

if [[ "$ACTUAL_OUTPUT" != *"$EXPECTED_OUTPUT"* ]]; then
    echo "FAILURE: JIT output mismatch"
    echo "Expected: $EXPECTED_OUTPUT"
    echo "Actual: $ACTUAL_OUTPUT" 
    exit 1
fi

echo "=== JIT EXECUTION VALIDATION PASSED ==="
echo "JIT implementation is REAL and WORKING"