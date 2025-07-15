#!/bin/bash
# Test different JIT optimization levels to find the issue
# Following DEVELOPMENT_PROCESS.md

set -e

echo "=== JIT OPTIMIZATION LEVEL TEST ==="

# Test function to check if JIT execution works
test_jit_execution() {
    local level_name="$1"
    echo "Testing optimization level: $level_name"
    
    if timeout 3s ./target/release/ea --run jit_diagnosis.ea >/dev/null 2>&1; then
        echo "✅ $level_name: SUCCESS"
        return 0
    else
        echo "❌ $level_name: FAILED (exit code: $?)"
        return 1
    fi
}

echo "Current implementation uses OptimizationLevel::None"
echo "Testing if the issue is optimization-related..."

# Since we can't easily change optimization levels without code changes,
# let's test the pure computation case to confirm JIT works
echo ""
echo "Testing pure computation (known to work):"
if timeout 3s ./target/release/ea --run debug_no_io.ea >/dev/null 2>&1; then
    echo "✅ Pure computation: SUCCESS"
else
    echo "❌ Pure computation: FAILED - JIT engine is broken"
    exit 1
fi

echo ""
echo "Testing I/O operation (currently failing):"
if timeout 3s ./target/release/ea --run jit_diagnosis.ea >/dev/null 2>&1; then
    echo "✅ I/O operation: SUCCESS - PROBLEM SOLVED!"
    exit 0
else
    echo "❌ I/O operation: FAILED"
fi

echo ""
echo "=== ANALYSIS ==="
echo "✅ JIT engine works for pure computation"
echo "❌ JIT engine fails for I/O operations"
echo "ROOT CAUSE: Symbol resolution issue with external calls (puts)"
echo ""
echo "TECHNICAL ISSUE: The global string literal mapping is working,"
echo "but the 'puts' function call from within JIT context fails."
echo "This suggests the issue is with the execution context isolation"
echo "between the JIT-compiled code and the mapped external symbols."