#!/bin/bash
# nested_loops_validation_script.sh

set -e

echo "=== NESTED LOOPS LLVM IR GENERATION VALIDATION ==="

# Step 1: Compilation
echo "Step 1: Compiling validation test..."
./target/release/ea --emit-llvm nested_loops_validation.ea || {
    echo "FAILURE: Compilation failed"
    exit 1
}

# Step 2: LLVM IR Quality Check
echo "Step 2: Validating LLVM IR..."
llvm-as nested_loops_validation.ll || {
    echo "FAILURE: Invalid LLVM IR"
    exit 1
}

# Check for required function calls
grep -q "br label" nested_loops_validation.ll || {
    echo "FAILURE: No branch instructions found in LLVM IR"
    exit 1
}

# Check that all basic blocks have terminators
echo "Step 2b: Checking for unterminated basic blocks..."
awk '/^[a-zA-Z_][a-zA-Z0-9_]*:/ { 
    if (block_name != "") {
        if (!has_terminator) {
            print "FAILURE: Block " block_name " has no terminator"
            exit 1
        }
    }
    block_name = $1; 
    has_terminator = 0; 
} 
/^  (br|ret|switch|invoke|resume|unreachable)/ { 
    has_terminator = 1; 
}
END {
    if (block_name != "" && !has_terminator) {
        print "FAILURE: Block " block_name " has no terminator"
        exit 1
    }
}' nested_loops_validation.ll || {
    echo "FAILURE: Found basic blocks without terminators"
    exit 1
}

# Step 3: Execution Test
echo "Step 3: Running validation test..."
ACTUAL_OUTPUT=$(timeout 10s lli nested_loops_validation.ll; echo $?)
EXPECTED_OUTPUT="9"

if [ "$ACTUAL_OUTPUT" != "$EXPECTED_OUTPUT" ]; then
    echo "FAILURE: Output mismatch"
    echo "Expected: $EXPECTED_OUTPUT"
    echo "Actual: $ACTUAL_OUTPUT"
    exit 1
fi

# Step 4: Memory Safety
echo "Step 4: Memory safety validation..."
timeout 15s valgrind --leak-check=full --error-exitcode=1 lli nested_loops_validation.ll > /dev/null 2>&1 || {
    echo "FAILURE: Memory leaks detected"
    exit 1
}

# Step 5: Stress Test
echo "Step 5: Stress testing..."
./target/release/ea --emit-llvm nested_loops_stress_test.ea || {
    echo "FAILURE: Stress test compilation failed"
    exit 1
}

llvm-as nested_loops_stress_test.ll || {
    echo "FAILURE: Stress test LLVM IR invalid"
    exit 1
}

STRESS_OUTPUT=$(timeout 30s lli nested_loops_stress_test.ll; echo $?)
EXPECTED_STRESS="90"

if [ "$STRESS_OUTPUT" != "$EXPECTED_STRESS" ]; then
    echo "FAILURE: Stress test output mismatch"
    echo "Expected: $EXPECTED_STRESS"  
    echo "Actual: $STRESS_OUTPUT"
    exit 1
fi

# Step 6: Code Quality
echo "Step 6: Code quality check..."
grep -r "TODO\|PLACEHOLDER\|NOT IMPLEMENTED\|FIXME\|STUB" src/codegen/ && {
    echo "FAILURE: Placeholder code detected in codegen"
    exit 1
}

echo "=== ALL VALIDATION PASSED ==="
echo "NESTED LOOPS implementation is REAL and WORKING"