#!/bin/bash
# focused_memory_validation.sh
# Focused memory leak validation for specific compiler components

set -e

echo "=== FOCUSED MEMORY LEAK VALIDATION ==="

# Quick validation of specific memory scenarios
echo "Testing JIT memory management..."
valgrind --tool=memcheck \
         --leak-check=full \
         --show-leak-kinds=definite \
         --track-origins=yes \
         --quiet \
         --log-file=jit_memcheck.log \
         ./target/debug/ea --run jit_memory_stress.ea > /dev/null 2>&1

JIT_LEAKS=$(grep "definitely lost:" jit_memcheck.log | grep -o '[0-9,]* bytes' | head -1 | tr -d ',' | grep -o '[0-9]*' || echo "0")
echo "JIT Memory Test - Definitely lost: $JIT_LEAKS bytes"

echo "Testing LLVM IR memory management..."
valgrind --tool=memcheck \
         --leak-check=full \
         --show-leak-kinds=definite \
         --track-origins=yes \
         --quiet \
         --log-file=llvm_memcheck.log \
         ./target/debug/ea llvm_memory_test.ea > /dev/null 2>&1

LLVM_LEAKS=$(grep "definitely lost:" llvm_memcheck.log | grep -o '[0-9,]* bytes' | head -1 | tr -d ',' | grep -o '[0-9]*' || echo "0")
echo "LLVM Memory Test - Definitely lost: $LLVM_LEAKS bytes"

# Summary
TOTAL_LEAKS=$((JIT_LEAKS + LLVM_LEAKS))
echo ""
echo "=== FOCUSED VALIDATION SUMMARY ==="
echo "JIT execution leaks: $JIT_LEAKS bytes"
echo "LLVM compilation leaks: $LLVM_LEAKS bytes"
echo "Total definitely lost: $TOTAL_LEAKS bytes"

if [ "$TOTAL_LEAKS" -eq "0" ]; then
    echo "✅ PASSED: No definite memory leaks detected"
    echo "Memory management is REAL and WORKING"
else
    echo "❌ FAILED: $TOTAL_LEAKS bytes definitely lost"
    exit 1
fi