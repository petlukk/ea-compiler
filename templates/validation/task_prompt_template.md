TASK: Implement HashMap with complete runtime functionality

CRITICAL REQUIREMENTS:

1. The validation program HashMap_validation.ea MUST compile and run successfully
2. Memory management MUST pass valgrind with zero leaks
3. LLVM IR MUST contain actual function calls (not placeholders)
4. Implementation MUST NOT contain placeholder code

DELIVERABLES:

1. Modified source files with actual implementation
2. Runtime library if needed (e.g., C code for system integration)
3. Updated build system to link dependencies
4. All validation tests MUST pass

VALIDATION CRITERIA:

- HashMap_validation.ea compiles without errors
- Execution produces exact expected output
- valgrind shows zero memory leaks
- Implementation contains no TODO/PLACEHOLDER/FIXME comments
- LLVM IR contains expected function calls
- Stress test with large data passes

ANTI-CHEATING MEASURES:

- Output verified character-by-character
- LLVM IR inspected for actual function calls
- Memory safety validated with external tools
- Code searched for placeholder patterns
- Performance tested under load

FAILURE CONDITIONS:

- Any compilation error
- Any runtime crash or incorrect output
- Any memory leaks detected
- Any placeholder code remaining
- Missing required function calls in LLVM IR

You MUST implement actual working functionality, not just pass tests.
