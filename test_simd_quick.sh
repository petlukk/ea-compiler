#!/bin/bash
# Quick SIMD validation test

echo "=== QUICK SIMD VALIDATION ==="

# Test 1: SIMD Compilation
echo "Testing SIMD compilation..."
if ./target/release/ea --emit-llvm simd_runtime_test.ea >/dev/null 2>&1; then
    echo "✅ SIMD compilation successful"
else
    echo "❌ SIMD compilation failed"
    exit 1
fi

# Test 2: Vector operations in unoptimized IR
echo "Testing vector operations..."
if ./target/debug/ea --emit-llvm-only simd_validation_unoptimized.ea 2>/dev/null | grep -q "<4 x float>"; then
    echo "✅ SIMD vector operations found"
else
    echo "❌ SIMD vector operations not found"
    exit 1
fi

# Test 3: SIMD Runtime execution
echo "Testing SIMD runtime..."
if timeout 10 ./target/release/ea --run simd_runtime_test.ea 2>&1 | grep -q "Test successful - SIMD is working"; then
    echo "✅ SIMD runtime execution successful"
else
    echo "❌ SIMD runtime execution failed"
    exit 1
fi

echo "=== ALL SIMD TESTS PASSED ==="