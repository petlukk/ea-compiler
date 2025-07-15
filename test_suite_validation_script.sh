#!/bin/bash
# test_suite_validation_script.sh

set -e

echo "=== TEST SUITE STABILITY VALIDATION ==="

# Step 1: Verify basic SIMD identifier parsing
echo "Step 1: Testing SIMD identifier parsing..."
./target/release/ea --run test_suite_validation.ea > test_output.txt 2>&1
COMPILE_EXIT=$?

# Check if compilation succeeded (should exit with 10, the program result)
if [ "$COMPILE_EXIT" != "10" ]; then
    echo "FAILURE: Compilation failed with exit code $COMPILE_EXIT"
    cat test_output.txt
    exit 1
fi

ACTUAL_OUTPUT=$(cat test_output.txt | grep "exit code:" | grep -o '[0-9]*')
EXPECTED_OUTPUT="10"

if [ "$ACTUAL_OUTPUT" != "$EXPECTED_OUTPUT" ]; then
    echo "FAILURE: SIMD identifier output mismatch"
    echo "Expected: $EXPECTED_OUTPUT"
    echo "Actual: $ACTUAL_OUTPUT"
    exit 1
fi

echo "✅ SIMD identifier parsing test passed (result: $ACTUAL_OUTPUT)"

# Step 2: Run core test suite (lexer and parser only)
echo "Step 2: Running core lexer tests..."
cargo test --features=llvm --lib lexer::tests -- --nocapture --test-threads=1 || {
    echo "FAILURE: Lexer tests failed"
    exit 1
}

echo "Step 3: Running parser tests..."
cargo test --features=llvm --lib parser::tests -- --nocapture --test-threads=1 || {
    echo "FAILURE: Parser tests failed"
    exit 1
}

# Step 4: Code quality check
echo "Step 4: Code quality validation..."
grep -r "TODO\|PLACEHOLDER\|NOT IMPLEMENTED\|FIXME\|STUB" src/lexer/ src/parser/ && {
    echo "FAILURE: Placeholder code detected in core modules"
    exit 1
}

echo "=== ALL TEST SUITE VALIDATION PASSED ==="
echo "Test suite stability is REAL and WORKING"