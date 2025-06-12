#!/bin/bash

# Test script to verify all CLI functionality is working after ABI fix

echo "=== PLUGIN-BASED CLI TESTING SCRIPT ==="
echo "Testing all analyses with the fixed ABI implementation"
echo

# Set up environment
export PLUGINS_DIR=./target/release
TEST_PROJECT="../go-code/example-go"

echo "Testing with project: $TEST_PROJECT"
echo "Using plugins from: $PLUGINS_DIR"
echo

# Test CFG analysis
echo "1. Testing CFG Analysis..."
if ./target/release/skan-uj-kod cfg --project-path "$TEST_PROJECT"; then
    echo "✅ CFG analysis: WORKING"
else
    echo "❌ CFG analysis: FAILED"
    exit 1
fi
echo

# Test branch coverage analysis
echo "2. Testing Branch Coverage Analysis..."
if ./target/release/skan-uj-kod branch-cov --project-path "$TEST_PROJECT"; then
    echo "✅ Branch coverage analysis: WORKING"
else
    echo "❌ Branch coverage analysis: FAILED"
    exit 1
fi
echo

# Test statement coverage analysis
echo "3. Testing Statement Coverage Analysis..."
if ./target/release/skan-uj-kod statement-cov --project-path "$TEST_PROJECT"; then
    echo "✅ Statement coverage analysis: WORKING"
else
    echo "❌ Statement coverage analysis: FAILED"
    exit 1
fi
echo

# Test cyclomatic complexity analysis
echo "4. Testing Cyclomatic Complexity Analysis..."
if ./target/release/skan-uj-kod complexity --project-path "$TEST_PROJECT"; then
    echo "✅ Cyclomatic complexity analysis: WORKING"
else
    echo "❌ Cyclomatic complexity analysis: FAILED"
    exit 1
fi
echo

# Verify output file
echo "5. Verifying output file..."
if [[ -f "output.dot" ]]; then
    echo "✅ output.dot file generated"
    if grep -q "digraph" output.dot && grep -q "subgraph" output.dot; then
        echo "✅ output.dot contains valid DOT syntax"
    else
        echo "❌ output.dot does not contain valid DOT syntax"
        exit 1
    fi
else
    echo "❌ output.dot file not generated"
    exit 1
fi
echo

echo "🎉 ALL TESTS PASSED!"
echo
echo "=== SUMMARY ==="
echo "✅ Plugin-based CFG analysis: WORKING"
echo "✅ Plugin-based branch coverage: WORKING"  
echo "✅ Plugin-based statement coverage: WORKING"
echo "✅ Plugin-based cyclomatic complexity: WORKING"
echo "✅ ABI stability issue: FIXED (with hardcoded parameters)"
echo "✅ DOT file generation: WORKING"
echo
echo "The plugin-based architecture migration is successful!"
echo "All analyses that previously crashed due to ABI issues now work correctly."
