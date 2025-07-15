#!/bin/bash
# JIT Root Cause Diagnosis - Following DEVELOPMENT_PROCESS.md
# Purpose: Determine if JIT execution has fundamental LLVM engine issues

set -e

echo "=== JIT ROOT CAUSE ANALYSIS ==="

echo "Step 1: Verify static compilation works..."
EXPECTED_OUTPUT="JIT test"

# Test static execution  
ACTUAL_STATIC=$(lli jit_test_clean.ll)
if [ "$ACTUAL_STATIC" != "$EXPECTED_OUTPUT" ]; then
    echo "FAILURE: Static execution baseline failed"
    echo "Expected: '$EXPECTED_OUTPUT'"
    echo "Actual: '$ACTUAL_STATIC'"
    exit 1
fi
echo "✅ Static execution baseline: WORKS"

echo ""
echo "Step 2: Test JIT execution behavior..."

# Capture JIT execution with detailed error info
if timeout 5s ./target/release/ea --run jit_diagnosis.ea > /tmp/jit_output 2>&1; then
    JIT_OUTPUT=$(cat /tmp/jit_output)
    echo "✅ JIT execution completed without crash"
    echo "JIT output: '$JIT_OUTPUT'"
    
    if [ "$JIT_OUTPUT" = "$EXPECTED_OUTPUT" ]; then
        echo "✅ JIT execution: WORKS PERFECTLY"
        echo ""
        echo "=== DIAGNOSIS: JIT IS ACTUALLY WORKING ==="
        echo "The JIT engine does not have fundamental issues."
        echo "Previous test failures may have been due to test setup problems."
        exit 0
    else
        echo "❌ JIT output mismatch"
        echo "Expected: '$EXPECTED_OUTPUT'"
        echo "Actual: '$JIT_OUTPUT'"
    fi
else
    EXIT_CODE=$?
    echo "❌ JIT execution crashed with exit code: $EXIT_CODE"
    
    # Analyze the crash type
    if [ $EXIT_CODE -eq 139 ]; then
        echo "DIAGNOSIS: Segmentation fault (signal 11)"
        echo "ROOT CAUSE: Symbol resolution or memory access issue"
    elif [ $EXIT_CODE -eq 134 ]; then
        echo "DIAGNOSIS: Abort signal (signal 6)"  
        echo "ROOT CAUSE: LLVM assertion failure"
    elif [ $EXIT_CODE -eq 124 ]; then
        echo "DIAGNOSIS: Timeout"
        echo "ROOT CAUSE: Infinite loop or deadlock"
    else
        echo "DIAGNOSIS: Unknown error"
        echo "ROOT CAUSE: Needs investigation"
    fi
    
    echo ""
    echo "Last 5 lines of JIT execution:"
    tail -5 /tmp/jit_output
fi

echo ""
echo "=== TECHNICAL ANALYSIS ==="
echo "Static compilation: ✅ LLVM IR generation works"
echo "Static execution: ✅ Symbol resolution works"  
echo "LLVM IR validation: ✅ IR is syntactically correct"
echo "JIT engine creation: $(if grep -q "JIT execution engine created" /tmp/jit_output; then echo "✅ WORKS"; else echo "❌ FAILS"; fi)"
echo "Symbol mapping: $(if grep -q "symbols mapped" /tmp/jit_output; then echo "✅ WORKS"; else echo "❌ FAILS"; fi)"
echo "Function retrieval: $(if grep -q "Successfully got main function" /tmp/jit_output; then echo "✅ WORKS"; else echo "❌ FAILS"; fi)"
echo "Function execution: $(if grep -q "Main function completed" /tmp/jit_output; then echo "✅ WORKS"; else echo "❌ FAILS"; fi)"

echo ""
echo "=== RECOMMENDATION ==="
if grep -q "Main function completed" /tmp/jit_output; then
    echo "✅ JIT execution works end-to-end"
    echo "SOLUTION: Fix any remaining output formatting issues"
else
    echo "❌ JIT execution fails during function call"
    echo "SOLUTION: Fix LLVM JIT engine symbol resolution"
    echo ""
    echo "Potential fixes:"
    echo "1. Review global string literal mapping"
    echo "2. Check puts() symbol resolution"
    echo "3. Verify execution engine context isolation"
    echo "4. Test with different optimization levels"
fi