//! Compilation Tests
//!
//! These tests validate LLVM compilation in a safe, controlled manner
//! without creating multiple contexts or hanging.

#[cfg(feature = "llvm")]
use ea_compiler::compile_to_llvm;

#[cfg(feature = "llvm")]
#[test]
fn test_compile_simple_function() {
    let source = r#"
func main() -> i32 {
    return 42;
}
"#;

    let result = compile_to_llvm(source, "test_simple");
    assert!(result.is_ok(), "Simple function should compile to LLVM");

    // Clean up the generated file
    let _ = std::fs::remove_file("test_simple.ll");
}

#[cfg(feature = "llvm")]
#[test]
fn test_compile_arithmetic() {
    let source = r#"
func calculate() -> i32 {
    let a = 10;
    let b = 20;
    return a + b;
}
"#;

    let result = compile_to_llvm(source, "test_arithmetic");
    assert!(result.is_ok(), "Arithmetic should compile to LLVM");

    // Clean up
    let _ = std::fs::remove_file("test_arithmetic.ll");
}

#[cfg(feature = "llvm")]
#[test]
fn test_compile_control_flow() {
    let source = r#"
func abs_value(x: i32) -> i32 {
    if (x >= 0) {
        return x;
    } else {
        return -x;
    }
}
"#;

    let result = compile_to_llvm(source, "test_control_flow");
    assert!(result.is_ok(), "Control flow should compile to LLVM");

    // Clean up
    let _ = std::fs::remove_file("test_control_flow.ll");
}

#[cfg(feature = "llvm")]
#[test]
fn test_compile_simd_basic() {
    let source = r#"
func simd_add() -> f32x4 {
    let v1 = [1.0, 2.0, 3.0, 4.0]f32x4;
    let v2 = [5.0, 6.0, 7.0, 8.0]f32x4;
    return v1 .+ v2;
}
"#;

    let result = compile_to_llvm(source, "test_simd");
    assert!(result.is_ok(), "Basic SIMD should compile to LLVM");

    // Clean up
    let _ = std::fs::remove_file("test_simd.ll");
}

#[cfg(feature = "llvm")]
#[test]
fn test_compile_loop() {
    let source = r#"
func sum_loop() -> i32 {
    let sum = 0;
    for (let i: i32 = 1; i <= 10; i += 1) {
        sum += i;
    }
    return sum;
}
"#;

    let result = compile_to_llvm(source, "test_loop");
    assert!(result.is_ok(), "Loop should compile to LLVM");

    // Clean up
    let _ = std::fs::remove_file("test_loop.ll");
}

#[cfg(feature = "llvm")]
#[test]
fn test_compile_multiple_functions() {
    let source = r#"
func helper(x: i32) -> i32 {
    return x * 2;
}

func main() -> i32 {
    return helper(21);
}
"#;

    let result = compile_to_llvm(source, "test_multiple");
    assert!(result.is_ok(), "Multiple functions should compile to LLVM");

    // Clean up
    let _ = std::fs::remove_file("test_multiple.ll");
}

#[cfg(feature = "llvm")]
#[test]
fn test_compilation_error_detection() {
    let source = r#"
func bad_function() -> i32 {
    return "not an integer";
}
"#;

    let result = compile_to_llvm(source, "test_error");
    assert!(
        result.is_err(),
        "Should detect type errors during compilation"
    );

    // Clean up (might not exist due to error)
    let _ = std::fs::remove_file("test_error.ll");
}

#[cfg(not(feature = "llvm"))]
#[test]
fn test_llvm_feature_disabled() {
    // This test ensures the test suite works even without LLVM feature
    assert!(true, "LLVM feature is disabled, compilation tests skipped");
}
