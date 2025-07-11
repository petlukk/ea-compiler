#!/bin/bash
# string_validation_script.sh

set -e

echo "=== STRING IMPLEMENTATION VALIDATION ==="

# Step 1: Compilation
echo "Step 1: Compiling validation test..."
./target/release/ea String_validation.ea || {
    echo "FAILURE: Compilation failed"
    exit 1
}

# Step 2: LLVM IR Quality Check
echo "Step 2: Validating LLVM IR..."
llvm-as String_validation.ll || {
    echo "FAILURE: Invalid LLVM IR"
    exit 1
}

# Check for required function calls
grep -q "@string_" String_validation.ll || {
    echo "FAILURE: Required string functions not found in LLVM IR"
    exit 1
}

# Step 3: Execution Test
echo "Step 3: Running validation test..."
ACTUAL_OUTPUT=$(timeout 10s lli String_validation.ll)
EXPECTED_OUTPUT="String created
String from literal
String length: 13
String push result: Hello World
String concat result: Hello World
String clone result: Hello World
String comparison works
Substring result: Hello
Find result: 6
Replace result: Hello Universe
Uppercase result: HELLO
Lowercase result: hello
Trim result: hello world
ALL TESTS PASSED"

if [ "$ACTUAL_OUTPUT" != "$EXPECTED_OUTPUT" ]; then
    echo "FAILURE: Output mismatch"
    echo "Expected: $EXPECTED_OUTPUT"
    echo "Actual: $ACTUAL_OUTPUT"
    exit 1
fi

# Step 4: Memory Safety
echo "Step 4: Memory safety validation..."
valgrind --leak-check=full --error-exitcode=1 lli String_validation.ll > /dev/null 2>&1 || {
    echo "FAILURE: Memory leaks detected"
    exit 1
}

# Step 5: Stress Test
echo "Step 5: Stress testing..."
./target/release/ea String_stress_test.ea
timeout 30s lli String_stress_test.ll || {
    echo "FAILURE: Stress test failed"
    exit 1
}

# Step 6: Code Quality
echo "Step 6: Code quality check..."
grep -r "TODO\|PLACEHOLDER\|NOT IMPLEMENTED\|FIXME\|STUB" src/ && {
    echo "FAILURE: Placeholder code detected"
    exit 1
}

echo "=== ALL VALIDATION PASSED ==="
echo "STRING implementation is REAL and WORKING"