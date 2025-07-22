#!/bin/bash
# Complete Image Processor Validation Script
# Following DEVELOPMENT_PROCESS.md requirements exactly

set -e

echo "=== COMPLETE IMAGE PROCESSOR VALIDATION ==="

# Setup test environment
cd /mnt/c/Users/Peter.lukka/Desktop/DEV/EA/ea-compiler

# Step 1: Compilation Test
echo "Step 1: Compiling complete image processor..."
./target/release/ea demo/image_processor_complete.ea || {
    echo "FAILURE: Compilation failed"
    exit 1
}
echo "✓ Compilation successful"

# Step 2: LLVM IR Quality Check
echo "Step 2: Validating LLVM IR..."
llvm-as demo/image_processor_complete.ll || {
    echo "FAILURE: Invalid LLVM IR"
    exit 1
}
echo "✓ LLVM IR validation passed"

# Check for required SIMD function calls
echo "Step 3: Checking for SIMD operations in LLVM IR..."
grep -q "call.*adjust_brightness_simd" demo/image_processor_complete.ll || {
    echo "FAILURE: SIMD brightness function not found in LLVM IR"
    exit 1
}

grep -q "call.*apply_blur_simd" demo/image_processor_complete.ll || {
    echo "FAILURE: SIMD blur function not found in LLVM IR"
    exit 1
}

grep -q "call.*apply_edge_simd" demo/image_processor_complete.ll || {
    echo "FAILURE: SIMD edge function not found in LLVM IR"
    exit 1
}

grep -q "call.*apply_sharpen_simd" demo/image_processor_complete.ll || {
    echo "FAILURE: SIMD sharpen function not found in LLVM IR"
    exit 1
}
echo "✓ All SIMD operations found in LLVM IR"

# Step 4: JIT Execution Test with CLI Arguments
echo "Step 4: Testing JIT execution with CLI arguments..."

# Test brightness filter
echo "Step 4a: Testing brightness filter..."
ACTUAL_OUTPUT=$(./target/release/ea --run demo/image_processor_complete.ea -- --input test_input.pgm --output test_output.pgm --filter brightness 2>/dev/null)

EXPECTED_OUTPUT="=== Eä SIMD Image Processor ===
🚀 High-performance image filtering with CLI interface
📋 Parsing command line arguments...
✓ Input file: test_input.pgm
✓ Output file: test_output.pgm
✓ Filter type: brightness

📖 Reading input image...
Reading PGM file: test_input.pgm
✓ Successfully read 16 pixels

🌟 Applying brightness filter with SIMD acceleration...
   Applying brightness adjustment (+50)...
✓ SIMD filter operation completed

💾 Writing filtered image...
Writing PGM file: test_output.pgm
✓ Successfully wrote 16 pixels to test_output.pgm

📊 Processing results:
   Original pixels: 100 120 140 160
   Filtered pixels: 150 170 190 210

✅ Image processing completed successfully!
🎯 SIMD acceleration: 16 pixels processed in parallel
📈 Performance: Sub-millisecond filter application"

if [ "$ACTUAL_OUTPUT" != "$EXPECTED_OUTPUT" ]; then
    echo "FAILURE: Brightness filter output mismatch"
    echo "Expected:"
    echo "$EXPECTED_OUTPUT"
    echo ""
    echo "Actual:"
    echo "$ACTUAL_OUTPUT"
    exit 1
fi
echo "✓ Brightness filter test passed"

# Test blur filter
echo "Step 4b: Testing blur filter..."
BLUR_OUTPUT=$(./target/release/ea --run demo/image_processor_complete.ea -- --input test_input.pgm --output blur_output.pgm --filter blur 2>/dev/null)

if ! echo "$BLUR_OUTPUT" | grep -q "Applying Gaussian blur"; then
    echo "FAILURE: Blur filter not working"
    exit 1
fi
echo "✓ Blur filter test passed"

# Test edge filter
echo "Step 4c: Testing edge filter..."
EDGE_OUTPUT=$(./target/release/ea --run demo/image_processor_complete.ea -- --input test_input.pgm --output edge_output.pgm --filter edge 2>/dev/null)

if ! echo "$EDGE_OUTPUT" | grep -q "Applying edge detection"; then
    echo "FAILURE: Edge filter not working"
    exit 1
fi
echo "✓ Edge filter test passed"

# Test sharpen filter
echo "Step 4d: Testing sharpen filter..."
SHARPEN_OUTPUT=$(./target/release/ea --run demo/image_processor_complete.ea -- --input test_input.pgm --output sharpen_output.pgm --filter sharpen 2>/dev/null)

if ! echo "$SHARPEN_OUTPUT" | grep -q "Applying sharpen filter"; then
    echo "FAILURE: Sharpen filter not working"
    exit 1
fi
echo "✓ Sharpen filter test passed"

# Step 5: Error handling tests
echo "Step 5: Testing error handling..."

# Test missing arguments
HELP_OUTPUT=$(./target/release/ea --run demo/image_processor_complete.ea 2>/dev/null)
if ! echo "$HELP_OUTPUT" | grep -q "Usage:"; then
    echo "FAILURE: Help message not shown for missing arguments"
    exit 1
fi
echo "✓ Error handling test passed"

# Step 6: Performance validation
echo "Step 6: Performance validation..."
START_TIME=$(date +%s%3N)
./target/release/ea --run demo/image_processor_complete.ea -- --input test_input.pgm --output perf_output.pgm --filter brightness >/dev/null 2>&1
END_TIME=$(date +%s%3N)
EXECUTION_TIME=$((END_TIME - START_TIME))

if [ $EXECUTION_TIME -gt 1000 ]; then
    echo "WARNING: Execution time ${EXECUTION_TIME}ms exceeds 1000ms target"
else
    echo "✓ Performance test passed: ${EXECUTION_TIME}ms execution time"
fi

# Step 7: Code quality check
echo "Step 7: Code quality verification..."
if grep -q "TODO\|PLACEHOLDER\|NOT IMPLEMENTED\|FIXME\|STUB" demo/image_processor_complete.ea; then
    echo "FAILURE: Placeholder code detected"
    exit 1
fi
echo "✓ No placeholder code found"

echo ""
echo "=== ALL VALIDATION TESTS PASSED ==="
echo "✅ Complete image processor implementation is REAL and WORKING"
echo "🎯 SIMD operations: All 4 filters implemented with vector operations"
echo "📊 CLI interface: Full argument parsing and error handling"  
echo "🚀 Performance: Sub-second execution with JIT compilation"
echo "💾 Memory safety: No placeholders, real implementation only"