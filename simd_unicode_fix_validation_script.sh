#!/bin/bash
# simd_unicode_fix_validation_script.sh
# Validation script for Unicode character fix in simd_example.ea

set -e

echo "=== SIMD UNICODE FIX VALIDATION ==="

# Change to the correct directory
cd /mnt/c/Users/Peter.lukka/Desktop/DEV/EA/ea-compiler

# Step 1: Compilation
echo "Step 1: Compiling validation test..."
./target/release/ea simd_unicode_fix_validation.ea || {
    echo "FAILURE: Compilation failed"
    exit 1
}

# Step 2: LLVM IR Quality Check
echo "Step 2: Validating LLVM IR..."
if [ -f "simd_unicode_fix_validation.ll" ]; then
    llvm-as-14 simd_unicode_fix_validation.ll || {
        echo "FAILURE: Invalid LLVM IR"
        exit 1
    }
    
    # Check for SIMD function calls
    grep -q "fadd.*<4 x float>" simd_unicode_fix_validation.ll || {
        echo "FAILURE: SIMD vector operations not found in LLVM IR"
        exit 1
    }
else
    echo "WARNING: LLVM IR file not generated, continuing with execution test..."
fi

# Step 3: Execution Test
echo "Step 3: Running validation test..."
ACTUAL_OUTPUT=$(timeout 10s ./target/release/ea --run simd_unicode_fix_validation.ea 2>&1 | grep -o "SIMD Unicode fix validation.*")
EXPECTED_OUTPUT=$(cat simd_unicode_fix_expected_output.txt)

if [ "$ACTUAL_OUTPUT" != "$EXPECTED_OUTPUT" ]; then
    echo "FAILURE: Output mismatch"
    echo "Expected: '$EXPECTED_OUTPUT'"
    echo "Actual: '$ACTUAL_OUTPUT'"
    exit 1
fi

# Step 4: Test that the Unicode character was fixed
echo "Step 4: Testing that Unicode character was fixed..."
if grep -q "eä" examples/simd_example.ea; then
    echo "FAILURE: Unicode character still present in simd_example.ea"
    exit 1
fi

# Test that we can tokenize the imports (the specific fix)
echo "Step 4b: Testing that import statements can be tokenized..."
head -10 examples/simd_example.ea > test_imports.ea
./target/release/ea test_imports.ea 2>&1 | grep -q "LexError.*Unexpected character" && {
    echo "FAILURE: Import statements still contain untokenizable characters"
    exit 1
}
rm -f test_imports.ea

# Step 5: Code Quality Check
echo "Step 5: Code quality check..."

# Check that we replaced with correct ASCII
if ! grep -q "use ea::" examples/simd_example.ea; then
    echo "FAILURE: 'use ea::' not found in fixed simd_example.ea"
    exit 1
fi

echo "=== ALL VALIDATION PASSED ==="
echo "SIMD Unicode fix implementation is REAL and WORKING"