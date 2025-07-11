//! Core Functionality Tests
//! 
//! These tests validate the essential features of the Eä compiler
//! without complex LLVM operations that might cause hangs.

use ea_compiler::{tokenize, compile_to_ast};

#[test]
fn test_tokenization_works() {
    let source = "func main() -> i32 { return 42; }";
    
    let result = tokenize(source);
    assert!(result.is_ok(), "Basic tokenization should work");
    
    let tokens = result.unwrap();
    assert!(!tokens.is_empty(), "Should produce tokens");
}

#[test]
fn test_parse_simple_function() {
    let source = r#"
func add(a: i32, b: i32) -> i32 {
    return a + b;
}
"#;

    let result = compile_to_ast(source);
    assert!(result.is_ok(), "Simple function should parse");
    
    let (ast, _) = result.unwrap();
    assert_eq!(ast.len(), 1, "Should have one function");
}

#[test]
fn test_parse_main_function() {
    let source = r#"
func main() -> () {
    let x = 42;
    return;
}
"#;

    let result = compile_to_ast(source);
    assert!(result.is_ok(), "Main function should parse");
    
    let (ast, _) = result.unwrap();
    assert_eq!(ast.len(), 1, "Should have one function");
}

#[test]
fn test_parse_arithmetic() {
    let source = r#"
func calculate() -> i32 {
    let a = 10;
    let b = 20;
    let c = a + b;
    let d = c * 2;
    return d - 5;
}
"#;

    let result = compile_to_ast(source);
    assert!(result.is_ok(), "Arithmetic should parse");
}

#[test]
fn test_parse_if_statement() {
    let source = r#"
func abs_value(x: i32) -> i32 {
    if (x >= 0) {
        return x;
    } else {
        return -x;
    }
}
"#;

    let result = compile_to_ast(source);
    assert!(result.is_ok(), "If statement should parse");
}

#[test]
fn test_parse_for_loop() {
    let source = r#"
func sum_to_n(n: i32) -> i32 {
    let sum = 0;
    for (let i: i32 = 1; i <= n; i += 1) {
        sum += i;
    }
    return sum;
}
"#;

    let result = compile_to_ast(source);
    assert!(result.is_ok(), "For loop should parse");
}

#[test]
fn test_parse_while_loop() {
    let source = r#"
func countdown(n: i32) -> i32 {
    while (n > 0) {
        n -= 1;
    }
    return n;
}
"#;

    let result = compile_to_ast(source);
    assert!(result.is_ok(), "While loop should parse");
}

#[test]
fn test_parse_arrays() {
    let source = r#"
func array_test() -> i32 {
    let arr = [1, 2, 3, 4, 5];
    return arr[0];
}
"#;

    let result = compile_to_ast(source);
    assert!(result.is_ok(), "Arrays should parse");
}

#[test]
fn test_parse_simd_vectors() {
    let source = r#"
func simd_test() -> f32x4 {
    let v1 = [1.0, 2.0, 3.0, 4.0]f32x4;
    let v2 = [5.0, 6.0, 7.0, 8.0]f32x4;
    return v1 .+ v2;
}
"#;

    let result = compile_to_ast(source);
    assert!(result.is_ok(), "SIMD vectors should parse");
}

#[test]
fn test_parse_recursive_function() {
    let source = r#"
func factorial(n: i32) -> i32 {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}
"#;

    let result = compile_to_ast(source);
    assert!(result.is_ok(), "Recursive function should parse");
}

#[test]
fn test_error_detection_syntax() {
    let source = "func main() -> i32 { invalid syntax here }";
    
    let result = compile_to_ast(source);
    assert!(result.is_err(), "Should detect syntax errors");
}

#[test]
fn test_error_detection_missing_return() {
    let source = r#"
func missing_return() -> i32 {
    let x = 42;
    // Missing return statement
}
"#;
    
    let result = compile_to_ast(source);
    assert!(result.is_err(), "Should detect missing return");
}

#[test]
fn test_multiple_functions() {
    let source = r#"
func helper(x: i32) -> i32 {
    return x * 2;
}

func main() -> i32 {
    return helper(21);
}
"#;

    let result = compile_to_ast(source);
    assert!(result.is_ok(), "Multiple functions should parse");
    
    let (ast, context) = result.unwrap();
    assert_eq!(ast.len(), 2, "Should have two functions");
    assert!(context.functions.contains_key("helper"), "Should have helper function");
    assert!(context.functions.contains_key("main"), "Should have main function");
}

#[test]
fn test_different_simd_types() {
    // Test only baseline SIMD types that are available on all target architectures
    let test_cases = vec![
        ("f32x4", "[1.0, 2.0, 3.0, 4.0]f32x4"),  // SSE baseline
        ("f32x2", "[1.0, 2.0]f32x2"),             // Basic SIMD
        ("i32x4", "[1, 2, 3, 4]i32x4"),          // SSE2 baseline
        ("i64x2", "[100, 200]i64x2"),             // SSE2 baseline
    ];

    for (type_name, vector_literal) in test_cases {
        let source = format!(
            r#"
func test_{}() -> {} {{
    let v = {};
    return v;
}}
"#,
            type_name, type_name, vector_literal
        );

        let result = compile_to_ast(&source);
        assert!(result.is_ok(), "SIMD type {} should parse", type_name);
    }
}