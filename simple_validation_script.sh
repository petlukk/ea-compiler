#!/bin/bash
# simple_validation_script.sh

set -e

echo "=== SIMPLE VEC VALIDATION ==="

# Step 1: Basic Compilation with LLVM IR generation
echo "Step 1: Compiling simple Vec test..."
./target/release/ea --emit-llvm Vec_validation.ea || {
    echo "FAILURE: Compilation failed"
    exit 1
}

# Step 2: LLVM IR Quality Check
echo "Step 2: Validating LLVM IR..."
llvm-as Vec_validation.ll || {
    echo "FAILURE: Invalid LLVM IR"
    exit 1
}

# Check for required function calls
grep -q "@malloc\|vec_new\|vec_push\|vec_len\|vec_get" Vec_validation.ll || {
    echo "FAILURE: Required Vec functions not found in LLVM IR"
    exit 1
}

# Step 3: Execution Test
echo "Step 3: Running simple test..."
ACTUAL_OUTPUT=$(timeout 10s ./target/release/ea --run Vec_validation.ea 2>/dev/null | grep -E "Starting Vec test|Vec created|Element pushed|Length correct|Value correct|Vec test completed")
EXPECTED_OUTPUT="Starting Vec test
Vec created
Element pushed
Length correct
Value correct
Vec test completed"

if [ "$ACTUAL_OUTPUT" != "$EXPECTED_OUTPUT" ]; then
    echo "FAILURE: Output mismatch"
    echo "Expected: $EXPECTED_OUTPUT"
    echo "Actual: $ACTUAL_OUTPUT"
    exit 1
fi

echo "=== SIMPLE VALIDATION PASSED ==="
echo "Basic Vec functionality is WORKING"

echo ""
echo "=== SIMPLE HASHMAP VALIDATION ==="

# Step 1: Basic Compilation with LLVM IR generation
echo "Step 1: Compiling simple HashMap test..."
./target/release/ea --emit-llvm HashMap_validation.ea || {
    echo "FAILURE: HashMap compilation failed"
    exit 1
}

# Step 2: LLVM IR Quality Check
echo "Step 2: Validating HashMap LLVM IR..."
llvm-as HashMap_validation.ll || {
    echo "FAILURE: Invalid HashMap LLVM IR"
    exit 1
}

# Check for required function calls
grep -q "@malloc\|hashmap_new\|hashmap_insert\|hashmap_get\|hashmap_len" HashMap_validation.ll || {
    echo "FAILURE: Required HashMap functions not found in LLVM IR"
    exit 1
}

# Step 3: Execution Test
echo "Step 3: Running HashMap test..."
ACTUAL_OUTPUT=$(timeout 10s ./target/release/ea --run HashMap_validation.ea 2>/dev/null | grep -E "Starting HashMap test|HashMap created|Elements inserted|Value correct|Length correct|HashMap test completed")
EXPECTED_OUTPUT="Starting HashMap test
HashMap created
Elements inserted
Value correct
Length correct
HashMap test completed"

if [ "$ACTUAL_OUTPUT" != "$EXPECTED_OUTPUT" ]; then
    echo "FAILURE: HashMap output mismatch"
    echo "Expected: $EXPECTED_OUTPUT"
    echo "Actual: $ACTUAL_OUTPUT"
    exit 1
fi

echo "=== HASHMAP VALIDATION PASSED ==="
echo "HashMap functionality is WORKING"

echo ""
echo "=== SIMPLE STRING VALIDATION ==="

# Step 1: Basic Compilation with LLVM IR generation
echo "Step 1: Compiling simple String test..."
./target/release/ea --emit-llvm String_validation.ea || {
    echo "FAILURE: String compilation failed"
    exit 1
}

# Step 2: LLVM IR Quality Check
echo "Step 2: Validating String LLVM IR..."
llvm-as String_validation.ll || {
    echo "FAILURE: Invalid String LLVM IR"
    exit 1
}

# Check for required function calls
grep -q "@malloc\|string_new\|string_len" String_validation.ll || {
    echo "FAILURE: Required String functions not found in LLVM IR"
    exit 1
}

# Step 3: Execution Test
echo "Step 3: Running String test..."
ACTUAL_OUTPUT=$(timeout 10s ./target/release/ea --run String_validation.ea 2>/dev/null | grep -E "String created|String length works|String operations complete|String validation test completed")
EXPECTED_OUTPUT="String created
String length works
String operations complete
String validation test completed"

if [ "$ACTUAL_OUTPUT" != "$EXPECTED_OUTPUT" ]; then
    echo "FAILURE: String output mismatch"
    echo "Expected: $EXPECTED_OUTPUT"
    echo "Actual: $ACTUAL_OUTPUT"
    exit 1
fi

echo "=== STRING VALIDATION PASSED ==="
echo "String functionality is WORKING"

echo ""
echo "=== SIMPLE HASHSET VALIDATION ==="

# Step 1: Basic Compilation with LLVM IR generation
echo "Step 1: Compiling simple HashSet test..."
./target/release/ea --emit-llvm HashSet_validation.ea || {
    echo "FAILURE: HashSet compilation failed"
    exit 1
}

# Step 2: LLVM IR Quality Check
echo "Step 2: Validating HashSet LLVM IR..."
llvm-as HashSet_validation.ll || {
    echo "FAILURE: Invalid HashSet LLVM IR"
    exit 1
}

# Check for required function calls
grep -q "@malloc\|HashSet_new\|HashSet_insert\|HashSet_contains\|HashSet_len" HashSet_validation.ll || {
    echo "FAILURE: Required HashSet functions not found in LLVM IR"
    exit 1
}

# Step 3: Execution Test
echo "Step 3: Running HashSet test..."
ACTUAL_OUTPUT=$(timeout 10s ./target/release/ea --run HashSet_validation.ea 2>/dev/null | grep -E "Starting HashSet test|HashSet created|Elements inserted|Contains test passed|Remove test passed|HashSet test completed")
EXPECTED_OUTPUT="Starting HashSet test
HashSet created
Elements inserted
Contains test passed
Remove test passed
HashSet test completed"

if [ "$ACTUAL_OUTPUT" != "$EXPECTED_OUTPUT" ]; then
    echo "FAILURE: HashSet output mismatch"
    echo "Expected: $EXPECTED_OUTPUT"
    echo "Actual: $ACTUAL_OUTPUT"
    exit 1
fi

echo "=== HASHSET VALIDATION PASSED ==="
echo "HashSet functionality is WORKING"

echo ""
echo "=== ALL VALIDATIONS PASSED ==="
echo "Vec, HashMap, String, and HashSet implementations are WORKING"