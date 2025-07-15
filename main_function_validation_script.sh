#!/bin/bash
# main_function_validation_script.sh - Following DEVELOPMENT_PROCESS.md

set -e

echo "=== MAIN FUNCTION IMPLEMENTATION VALIDATION ==="

# Expected output (character-by-character match required)
EXPECTED_OUTPUT="Testing main function return type
Program should exit with code 0
Native execution validation test"

# Step 1: Compilation
echo "Step 1: Compiling validation test..."
./target/release/ea --emit-llvm main_function_validation.ea || {
    echo "FAILURE: Compilation failed"
    exit 1
}

# Step 2: LLVM IR Quality Check
echo "Step 2: Validating LLVM IR..."
llvm-as main_function_validation.ll || {
    echo "FAILURE: Invalid LLVM IR"
    exit 1
}

# Check for correct main function signature
grep -q "define i32 @main()" main_function_validation.ll || {
    echo "FAILURE: Main function must return i32, not void"
    exit 1
}

# Check for proper return statement
grep -q "ret i32 0" main_function_validation.ll || {
    echo "FAILURE: Main function must return 0"
    exit 1
}

# Step 3: LLVM Interpreter Test
echo "Step 3: Running with lli..."
ACTUAL_OUTPUT=$(timeout 10s lli main_function_validation.ll)
EXIT_CODE=$?

if [ $EXIT_CODE -ne 0 ]; then
    echo "FAILURE: lli execution failed with exit code $EXIT_CODE"
    exit 1
fi

if [ "$ACTUAL_OUTPUT" != "$EXPECTED_OUTPUT" ]; then
    echo "FAILURE: Output mismatch"
    echo "Expected: $EXPECTED_OUTPUT"
    echo "Actual: $ACTUAL_OUTPUT"
    exit 1
fi

# Step 4: Native Binary Test
echo "Step 4: Testing native binary..."
llc main_function_validation.ll -o main_function_validation.s
gcc -no-pie main_function_validation.s -o main_function_validation_native

NATIVE_OUTPUT=$(timeout 10s ./main_function_validation_native)
NATIVE_EXIT_CODE=$?

if [ $NATIVE_EXIT_CODE -ne 0 ]; then
    echo "FAILURE: Native execution failed with exit code $NATIVE_EXIT_CODE"
    exit 1
fi

if [ "$NATIVE_OUTPUT" != "$EXPECTED_OUTPUT" ]; then
    echo "FAILURE: Native output mismatch"
    echo "Expected: $EXPECTED_OUTPUT"
    echo "Actual: $NATIVE_OUTPUT"
    exit 1
fi

# Step 5: Memory Safety
echo "Step 5: Memory safety validation..."
valgrind --leak-check=full --error-exitcode=1 ./main_function_validation_native > /dev/null 2>&1 || {
    echo "FAILURE: Memory leaks detected"
    exit 1
}

# Step 6: Code Quality Check
echo "Step 6: Code quality check..."
grep -r "TODO\|PLACEHOLDER\|NOT IMPLEMENTED\|FIXME\|STUB" src/ && {
    echo "FAILURE: Placeholder code detected"
    exit 1
}

echo "=== ALL VALIDATION PASSED ==="
echo "Main function implementation is REAL and WORKING"
echo "✅ Correct return type (i32)"
echo "✅ Proper exit code (0)"
echo "✅ Native execution works"
echo "✅ Memory safe"