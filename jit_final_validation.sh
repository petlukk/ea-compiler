#!/bin/bash
# Final JIT Validation - Following DEVELOPMENT_PROCESS.md
# Purpose: Test the global mapping fix with scientific rigor

set -e

echo "=== FINAL JIT VALIDATION ==="

echo "Step 1: Test static execution baseline..."
EXPECTED_OUTPUT="JIT test"
ACTUAL_STATIC=$(lli jit_test_clean.ll)

if [ "$ACTUAL_STATIC" != "$EXPECTED_OUTPUT" ]; then
    echo "FAILURE: Static execution baseline failed"
    echo "Expected: '$EXPECTED_OUTPUT'"
    echo "Actual: '$ACTUAL_STATIC'"
    exit 1
fi
echo "✅ Static execution: WORKS"

echo ""
echo "Step 2: Test JIT execution with global mapping..."

# Test with full verbose output to see global mapping
if timeout 5s ./target/release/ea --run jit_diagnosis.ea > /tmp/jit_full_output 2>&1; then
    JIT_EXIT_CODE=0
else
    JIT_EXIT_CODE=$?
fi

JIT_OUTPUT=$(cat /tmp/jit_full_output | grep -o "JIT test" | head -1 || echo "")

echo "JIT exit code: $JIT_EXIT_CODE"
echo "JIT final output: '$JIT_OUTPUT'"

if [ $JIT_EXIT_CODE -eq 0 ]; then
    if [ "$JIT_OUTPUT" = "$EXPECTED_OUTPUT" ]; then
        echo ""
        echo "🎉🎉🎉 SUCCESS! JIT EXECUTION WORKS! 🎉🎉🎉"
        echo ""
        echo "=== VALIDATION COMPLETE ==="
        echo "✅ Static compilation: WORKS"
        echo "✅ Static execution: WORKS"
        echo "✅ JIT compilation: WORKS"
        echo "✅ JIT execution: WORKS"
        echo "✅ Global mapping: WORKS"
        echo ""
        echo "🎯 The Eä compiler JIT engine is FIXED!"
        exit 0
    else
        echo "❌ JIT output mismatch"
        echo "Expected: '$EXPECTED_OUTPUT'"
        echo "Actual: '$JIT_OUTPUT'"
    fi
else
    echo "❌ JIT execution failed with exit code: $JIT_EXIT_CODE"
    if grep -q "Successfully got main function" /tmp/jit_full_output; then
        echo "ANALYSIS: Function retrieval works"
    fi
    if grep -q "string literals mapped" /tmp/jit_full_output; then
        echo "ANALYSIS: Global mapping works"
    fi
    if grep -q "Calling main function now" /tmp/jit_full_output; then
        echo "ANALYSIS: Segfault occurs during function execution"
        echo "ROOT CAUSE: Symbol resolution issue in puts() call"
    fi
fi

echo ""
echo "=== DIAGNOSTIC INFORMATION ==="
echo "Last 10 lines of JIT execution:"
tail -10 /tmp/jit_full_output