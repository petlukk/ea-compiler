#!/bin/bash
# placeholder_detection_script.sh
# DEVELOPMENT_PROCESS.md compliant placeholder detection

set -e

echo "=== PLACEHOLDER CODE DETECTION ===" 
echo "Following DEVELOPMENT_PROCESS.md methodology"
echo

# Step 1: Check for explicit placeholders
echo "Step 1: Checking for explicit placeholder patterns..."
PLACEHOLDER_COUNT=$(grep -r "TODO\|PLACEHOLDER\|NOT IMPLEMENTED\|FIXME\|STUB" src/ | wc -l)
echo "Explicit placeholders found: $PLACEHOLDER_COUNT"

if [ $PLACEHOLDER_COUNT -gt 0 ]; then
    echo "❌ FAILURE: Explicit placeholder code detected"
    grep -r "TODO\|PLACEHOLDER\|NOT IMPLEMENTED\|FIXME\|STUB" src/
    exit 1
fi

# Step 2: Check for hardcoded test outputs
echo "Step 2: Checking for hardcoded test outputs..."
HARDCODED_COUNT=$(grep -r "Vec created\|HashMap created\|Test passed" src/ | wc -l)
echo "Hardcoded test outputs: $HARDCODED_COUNT"

if [ $HARDCODED_COUNT -gt 0 ]; then
    echo "❌ FAILURE: Hardcoded test outputs detected"
    grep -r "Vec created\|HashMap created\|Test passed" src/
    exit 1
fi

# Step 3: Identify dead code (potential placeholders)
echo "Step 3: Analyzing dead code patterns..."
echo "Building to get warning analysis..."

# Get dead code warnings
cargo build --release --features=llvm 2>&1 | grep -E "methods.*are never used|fields.*are never read|field.*is never read" > dead_code_warnings.txt

DEAD_CODE_COUNT=$(wc -l < dead_code_warnings.txt)
echo "Dead code warnings: $DEAD_CODE_COUNT"

if [ $DEAD_CODE_COUNT -gt 0 ]; then
    echo "⚠️  POTENTIAL PLACEHOLDERS DETECTED:"
    echo "   The following code appears to be unused (potential placeholder implementations):"
    cat dead_code_warnings.txt
    echo
    echo "   These require investigation per DEVELOPMENT_PROCESS.md:"
    echo "   - Are these real implementations that should be used?"
    echo "   - Are these placeholder implementations that should be removed?"
    echo "   - Are these future features that should be marked as such?"
fi

# Step 4: Check for functions with empty bodies
echo "Step 4: Checking for functions with empty implementations..."
EMPTY_FUNCTIONS=$(grep -r "fn.*{$" src/ | grep -v "test" | wc -l)
echo "Functions with empty bodies: $EMPTY_FUNCTIONS"

if [ $EMPTY_FUNCTIONS -gt 0 ]; then
    echo "⚠️  POTENTIAL EMPTY IMPLEMENTATIONS:"
    grep -r "fn.*{$" src/ | grep -v "test"
fi

# Step 5: Summary
echo
echo "=== PLACEHOLDER DETECTION SUMMARY ==="
echo "✅ No explicit placeholders (TODO, FIXME, etc.)"
echo "✅ No hardcoded test outputs"
echo "⚠️  $DEAD_CODE_COUNT potential placeholder implementations (dead code)"
echo "⚠️  $EMPTY_FUNCTIONS functions with empty bodies"
echo
echo "RECOMMENDATION: Investigate dead code warnings to determine if they represent:"
echo "1. Real implementations that should be used"
echo "2. Placeholder implementations that should be removed"
echo "3. Future features that should be properly documented"
echo
echo "Per DEVELOPMENT_PROCESS.md: Focus on REAL, WORKING implementations"

# Clean up
rm -f dead_code_warnings.txt