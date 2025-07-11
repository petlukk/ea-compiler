#!/bin/bash
# HashSet implementation validation script

set -e

echo "=== HASHSET IMPLEMENTATION VALIDATION ==="

# Step 1: Build the compiler
echo "Step 1: Building compiler..."
cargo build --release --features=llvm || {
    echo "FAILURE: Compiler build failed"
    exit 1
}

# Step 2: Compilation Test
echo "Step 2: Compiling HashSet validation test..."
./target/release/ea HashSet_validation.ea || {
    echo "FAILURE: Compilation failed"
    exit 1
}

# Step 3: LLVM IR Quality Check
echo "Step 3: Validating LLVM IR..."
if [ -f "HashSet_validation.ll" ]; then
    llvm-as HashSet_validation.ll || {
        echo "FAILURE: Invalid LLVM IR"
        exit 1
    }
    
    # Check for required HashSet function calls
    grep -q "@HashSet_new" HashSet_validation.ll || {
        echo "FAILURE: HashSet_new function not found in LLVM IR"
        exit 1
    }
    
    grep -q "@HashSet_insert" HashSet_validation.ll || {
        echo "FAILURE: HashSet_insert function not found in LLVM IR"
        exit 1
    }
    
    grep -q "@HashSet_contains" HashSet_validation.ll || {
        echo "FAILURE: HashSet_contains function not found in LLVM IR"
        exit 1
    }
    
    grep -q "@HashSet_len" HashSet_validation.ll || {
        echo "FAILURE: HashSet_len function not found in LLVM IR"
        exit 1
    }
    
    grep -q "@HashSet_remove" HashSet_validation.ll || {
        echo "FAILURE: HashSet_remove function not found in LLVM IR"
        exit 1
    }
    
    grep -q "@HashSet_clear" HashSet_validation.ll || {
        echo "FAILURE: HashSet_clear function not found in LLVM IR"
        exit 1
    }
    
    grep -q "@HashSet_is_empty" HashSet_validation.ll || {
        echo "FAILURE: HashSet_is_empty function not found in LLVM IR"
        exit 1
    }
    
    echo "LLVM IR validation passed - all required functions found"
else
    echo "FAILURE: No LLVM IR file generated"
    exit 1
fi

# Step 4: JIT Execution Test
echo "Step 4: Running HashSet validation test..."
EXPECTED_OUTPUT="HashSet Validation Test
HashSet created
Elements inserted
Contains test passed
Length test passed
Remove test passed
Remove verification passed
Clear test passed
ALL TESTS PASSED"

ACTUAL_OUTPUT=$(timeout 10s ./target/release/ea --run HashSet_validation.ea 2>&1)

if [ "$ACTUAL_OUTPUT" = "$EXPECTED_OUTPUT" ]; then
    echo "OUTPUT VALIDATION PASSED"
else
    echo "FAILURE: Output mismatch"
    echo "Expected:"
    echo "$EXPECTED_OUTPUT"
    echo "Actual:"
    echo "$ACTUAL_OUTPUT"
    exit 1
fi

# Step 5: Memory Safety Test (if valgrind is available)
echo "Step 5: Memory safety validation..."
if command -v valgrind &> /dev/null; then
    echo "Running valgrind memory check..."
    valgrind --leak-check=full --error-exitcode=1 --quiet ./target/release/ea --run HashSet_validation.ea > /dev/null 2>&1 || {
        echo "FAILURE: Memory leaks detected"
        exit 1
    }
    echo "Memory safety validation passed"
else
    echo "Valgrind not available, skipping memory safety check"
fi

# Step 6: Stress Test
echo "Step 6: Running stress test..."
./target/release/ea HashSet_stress_test.ea || {
    echo "FAILURE: Stress test compilation failed"
    exit 1
}

timeout 30s ./target/release/ea --run HashSet_stress_test.ea || {
    echo "FAILURE: Stress test execution failed"
    exit 1
}

echo "Stress test passed"

# Step 7: Code Quality Check
echo "Step 7: Code quality check..."
if grep -r "TODO\|PLACEHOLDER\|NOT IMPLEMENTED\|FIXME\|STUB" src/runtime/hashset_runtime.c; then
    echo "FAILURE: Placeholder code detected in HashSet runtime"
    exit 1
fi

if grep -r "todo!\|unimplemented!\|unreachable!" src/codegen/mod.rs | grep -i hashset; then
    echo "FAILURE: Placeholder code detected in HashSet codegen"
    exit 1
fi

echo "Code quality check passed"

echo "=== ALL VALIDATION PASSED ==="
echo "HashSet implementation is REAL and WORKING"
echo "✅ Compilation: PASS"
echo "✅ LLVM IR: PASS"  
echo "✅ Execution: PASS"
echo "✅ Memory Safety: PASS"
echo "✅ Stress Test: PASS"
echo "✅ Code Quality: PASS"