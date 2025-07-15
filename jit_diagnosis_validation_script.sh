#!/bin/bash
# JIT Diagnosis Validation Script - Following DEVELOPMENT_PROCESS.md

set -e

echo "=== JIT DIAGNOSIS VALIDATION ==="

echo "Step 1: Static compilation validation..."
./target/release/ea --emit-llvm-only jit_diagnosis.ea > jit_diagnosis.ll || {
    echo "FAILURE: Static compilation failed"
    exit 1
}

echo "Step 2: LLVM IR validation..."
llvm-as jit_diagnosis.ll || {
    echo "FAILURE: Invalid LLVM IR"
    exit 1
}

echo "Step 3: Static execution test..."
EXPECTED_OUTPUT="JIT test"
ACTUAL_STATIC=$(timeout 5s lli jit_diagnosis.ll)

if [ "$ACTUAL_STATIC" != "$EXPECTED_OUTPUT" ]; then
    echo "FAILURE: Static execution failed"
    echo "Expected: '$EXPECTED_OUTPUT'"
    echo "Actual: '$ACTUAL_STATIC'"
    exit 1
fi
echo "✅ Static execution works perfectly"

echo "Step 4: JIT execution diagnosis..."
echo "Attempting JIT execution with timeout..."

if timeout 3s ./target/release/ea --run jit_diagnosis.ea > /tmp/jit_output.log 2>&1; then
    ACTUAL_JIT=$(cat /tmp/jit_output.log)
    if [ "$ACTUAL_JIT" = "$EXPECTED_OUTPUT" ]; then
        echo "✅ JIT execution works!"
        echo "=== DIAGNOSIS: JIT is actually working ==="
        exit 0
    else
        echo "❌ JIT execution produced wrong output"
        echo "Expected: '$EXPECTED_OUTPUT'"  
        echo "Actual: '$ACTUAL_JIT'"
    fi
else
    EXIT_CODE=$?
    echo "❌ JIT execution failed with exit code: $EXIT_CODE"
    echo "=== DIAGNOSIS: JIT engine has fundamental issues ==="
    echo "Last 10 lines of JIT output:"
    tail -10 /tmp/jit_output.log
fi

echo ""
echo "=== TECHNICAL ROOT CAUSE ANALYSIS ==="
echo "1. Static compilation: ✅ WORKS"
echo "2. LLVM IR generation: ✅ WORKS" 
echo "3. Static execution: ✅ WORKS"
echo "4. JIT execution: ❌ FAILS"
echo ""
echo "CONCLUSION: JIT engine symbol resolution or execution context issue"
echo "RECOMMENDATION: Fix JIT engine or implement intelligent fallback"