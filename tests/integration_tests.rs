//! tests/integration_tests.rs
//! End-to-end integration tests for the Eä compiler

use ea_compiler::{compile_to_ast, parse, tokenize};
use std::fs;
use std::path::Path;

#[cfg(feature = "llvm")]
use ea_compiler::compile_to_llvm;

/// Test helper to create temporary Eä source files
fn create_test_file(content: &str, filename: &str) -> std::io::Result<()> {
    fs::write(filename, content)
}

/// Test helper to clean up test files
fn cleanup_test_file(filename: &str) {
    let _ = fs::remove_file(filename);
}

#[test]
fn test_hello_world_end_to_end() {
    let source = r#"
func main() -> () {
    print("Hello, World!");
    return;
}
"#;

    let result = compile_to_ast(source);
    assert!(result.is_ok(), "Hello World should compile successfully");

    let (program, context) = result.unwrap();
    assert_eq!(program.len(), 1, "Should have one function declaration");
    assert!(
        context.functions.contains_key("main"),
        "Should have main function"
    );
    assert!(
        context.functions.contains_key("print"),
        "Should have print function"
    );
}

#[test]
fn test_fibonacci_recursive() {
    let source = r#"
func fibonacci(n: i32) -> i32 {
    if (n <= 1) {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

func main() -> () {
    let result = fibonacci(10);
    print("Fibonacci result calculated");
    return;
}
"#;

    let result = compile_to_ast(source);
    assert!(result.is_ok(), "Fibonacci should compile successfully");

    let (program, context) = result.unwrap();
    assert_eq!(program.len(), 2, "Should have two function declarations");
    assert!(
        context.functions.contains_key("fibonacci"),
        "Should have fibonacci function"
    );
    assert!(
        context.functions.contains_key("main"),
        "Should have main function"
    );
}

#[test]
fn test_complex_control_flow() {
    let source = r#"
func test_control_flow() -> i32 {
    let sum = 0;
    
    // Test for loop
    for (let i: i32 = 1; i <= 10; i += 1) {
        // Test nested if
        if (i % 2 == 0) {
            sum += i;
        } else {
            sum += i * 2;
        }
    }
    
    // Test while loop
    let counter = 0;
    while (counter < 5) {
        sum += counter;
        counter += 1;
    }
    
    return sum;
}

func main() -> () {
    let result = test_control_flow();
    return;
}
"#;

    let result = compile_to_ast(source);
    assert!(
        result.is_ok(),
        "Complex control flow should compile successfully"
    );
}

#[test]
fn test_multiple_function_calls() {
    let source = r#"
func add(a: i32, b: i32) -> i32 {
    return a + b;
}

func multiply(a: i32, b: i32) -> i32 {
    return a * b;
}

func calculate(x: i32, y: i32, z: i32) -> i32 {
    let sum = add(x, y);
    let product = multiply(sum, z);
    return product;
}

func main() -> () {
    let result1 = calculate(1, 2, 3);
    let result2 = calculate(4, 5, 6);
    let final_result = add(result1, result2);
    return;
}
"#;

    let result = compile_to_ast(source);
    assert!(
        result.is_ok(),
        "Multiple function calls should compile successfully"
    );

    let (program, context) = result.unwrap();
    assert_eq!(program.len(), 4, "Should have four function declarations");

    // Verify all functions are registered
    assert!(context.functions.contains_key("add"));
    assert!(context.functions.contains_key("multiply"));
    assert!(context.functions.contains_key("calculate"));
    assert!(context.functions.contains_key("main"));
}

#[test]
fn test_type_inference_comprehensive() {
    let source = r#"
func test_inference() -> () {
    // Integer inference
    let int_var = 42;
    let int_calculation = int_var + 10;
    
    // Float inference
    let float_var = 3.14;
    let float_calculation = float_var * 2.0;
    
    // Boolean inference
    let bool_var = true;
    let bool_calculation = bool_var && false;
    
    // String inference
    let string_var = "hello";
    
    // Mixed type operations
    let comparison = int_var > 30;
    let equality = float_var == 3.14;
    
    return;
}
"#;

    let result = compile_to_ast(source);
    assert!(result.is_ok(), "Type inference should work correctly");
}

#[test]
fn test_scoping_rules_comprehensive() {
    let source = r#"
func test_scoping() -> () {
    let outer_var = 1;
    
    {
        let inner_var = 2;
        let outer_var = "shadowed"; // Shadow outer variable
        
        {
            let deeply_nested = 3;
            let inner_var = true; // Shadow inner variable
            // All three variables accessible with correct types
        }
        
        // inner_var is string here, deeply_nested is not accessible
    }
    
    // Only outer_var (integer) is accessible here
    let final_calc = outer_var + 10;
    return;
}
"#;

    let result = compile_to_ast(source);
    assert!(result.is_ok(), "Scoping rules should be enforced correctly");
}

#[test]
fn test_error_detection_comprehensive() {
    // Test various error conditions

    // Type mismatch in return
    let source1 = r#"
func bad_return() -> i32 {
    return "hello";
}
"#;
    assert!(
        compile_to_ast(source1).is_err(),
        "Should detect return type mismatch"
    );

    // Type mismatch in assignment
    let source2 = r#"
func bad_assignment() -> () {
    let x: i32 = "hello";
    return;
}
"#;
    assert!(
        compile_to_ast(source2).is_err(),
        "Should detect assignment type mismatch"
    );

    // Function call with wrong argument types
    let source3 = r#"
func test_func(x: i32) -> () {
    return;
}

func main() -> () {
    test_func("wrong type");
    return;
}
"#;
    assert!(
        compile_to_ast(source3).is_err(),
        "Should detect argument type mismatch"
    );

    // Undefined variable
    let source4 = r#"
func undefined_var() -> () {
    let x = undefined_variable + 1;
    return;
}
"#;
    assert!(
        compile_to_ast(source4).is_err(),
        "Should detect undefined variable"
    );

    // Invalid condition type
    let source5 = r#"
func invalid_condition() -> () {
    if (42) {
        return;
    }
    return;
}
"#;
    assert!(
        compile_to_ast(source5).is_err(),
        "Should detect invalid condition type"
    );
}

#[test]
fn test_large_program_compilation() {
    // Generate a moderately large program
    let mut source = String::new();

    // Add many helper functions
    for i in 0..50 {
        source.push_str(&format!(
            r#"
func helper_{i}(x: i32) -> i32 {{
    if (x > {threshold}) {{
        return x * 2;
    }} else {{
        return x + {increment};
    }}
}}
"#,
            i = i,
            threshold = i * 10,
            increment = i + 1
        ));
    }

    // Add main function that uses many helpers
    source.push_str(
        r#"
func main() -> () {
    let sum = 0;
    for (let i: i32 = 0; i < 50; i += 1) {
        sum += helper_0(i);
        sum += helper_25(i * 2);
        sum += helper_49(i + 10);
    }
    return;
}
"#,
    );

    let result = compile_to_ast(&source);
    assert!(result.is_ok(), "Large program should compile successfully");

    let (program, context) = result.unwrap();
    assert_eq!(program.len(), 51, "Should have 50 helper functions + main");
    assert_eq!(
        context.functions.len(),
        56,
        "Should have all functions + built-ins"
    ); // Updated after memory fix
}

#[test]
fn test_expression_complexity() {
    let source = r#"
func complex_expressions() -> () {
    // Deeply nested arithmetic
    let result1 = ((1 + 2) * (3 - 4)) / ((5 + 6) - (7 * 8));
    
    // Complex boolean logic
    let result2 = (true && false) || ((1 < 2) && (3 >= 3)) || (!(4 == 5));
    
    // Mixed arithmetic and comparison
    let result3 = (1 + 2 * 3) > (4 - 5 / 2) && (6.0 + 7.0) <= (8.0 * 9.0);
    
    // Function calls in expressions
    // let result4 = add(multiply(1, 2), subtract(3, 4)) + calculate(5, 6, 7);
    
    return;
}

func add(a: i32, b: i32) -> i32 { return a + b; }
func multiply(a: i32, b: i32) -> i32 { return a * b; }
func subtract(a: i32, b: i32) -> i32 { return a - b; }
"#;

    let result = compile_to_ast(source);
    assert!(
        result.is_ok(),
        "Complex expressions should compile successfully"
    );
}

#[cfg(feature = "llvm")]
#[test]
fn test_llvm_integration_basic() {
    let source = r#"
func add(a: i32, b: i32) -> i32 {
    return a + b;
}

func main() -> () {
    let result = add(5, 10);
    return;
}
"#;

    // Test LLVM compilation
    let result = compile_to_llvm(source, "test_basic");
    assert!(result.is_ok(), "LLVM compilation should succeed");

    // Clean up generated files
    cleanup_test_file("test_basic.ll");
}

#[cfg(feature = "llvm")]
#[test]
fn test_llvm_integration_control_flow() {
    let source = r#"
func test_control() -> i32 {
    let x = 10;
    
    if (x > 5) {
        return x * 2;
    } else {
        return x + 1;
    }
}

func main() -> () {
    let result = test_control();
    return;
}
"#;

    let result = compile_to_llvm(source, "test_control_flow");
    if let Err(e) = &result {
        eprintln!("Control flow test error: {:?}", e);
    }
    assert!(
        result.is_ok(),
        "LLVM control flow compilation should succeed"
    );

    cleanup_test_file("test_control_flow.ll");
}

#[test]
fn test_compilation_stages_separately() {
    let source = r#"
func test() -> i32 {
    let x = 42;
    return x + 1;
}
"#;

    // Test tokenization
    let tokens = tokenize(source).expect("Tokenization should succeed");
    assert!(!tokens.is_empty(), "Should produce tokens");

    // Test parsing
    let program = parse(source).expect("Parsing should succeed");
    assert_eq!(program.len(), 1, "Should have one function");

    // Test full compilation
    let result = compile_to_ast(source).expect("Full compilation should succeed");
    assert!(result.0.len() == 1, "Should have one statement");
}

#[test]
fn test_error_message_quality() {
    let source = r#"
func test() -> i32 {
    return "hello"; // Type error
}
"#;

    match compile_to_ast(source) {
        Err(error) => {
            let error_message = error.to_string();
            assert!(error_message.contains("Type"), "Error should mention type");
            assert!(
                error_message.contains("mismatch") || error_message.contains("error"),
                "Error should indicate mismatch"
            );
        }
        Ok(_) => panic!("Should produce a type error"),
    }
}

#[test]
fn test_performance_regression() {
    use std::time::Instant;

    let source = r#"
func fibonacci(n: i32) -> i32 {
    if (n <= 1) {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

func main() -> () {
    let result = fibonacci(10);
    return;
}
"#;

    let start = Instant::now();
    let result = compile_to_ast(source);
    let duration = start.elapsed();

    assert!(result.is_ok(), "Compilation should succeed");
    assert!(
        duration.as_millis() < 100,
        "Compilation should be fast (< 100ms)"
    );
}

/// Test that demonstrates the full capability of the Eä compiler
#[test]
fn test_showcase_program() {
    let source = r#"
// Comprehensive Eä language showcase
func factorial(n: i32) -> i32 {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}

func fibonacci(n: i32) -> i32 {
    if (n <= 1) {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

func is_prime(n: i32) -> bool {
    if (n <= 1) {
        return false;
    }
    
    for (let i: i32 = 2; i * i <= n; i += 1) {
        if (n % i == 0) {
            return false;
        }
    }
    
    return true;
}

func main() -> () {
    // Test recursive functions
    let fact_10 = factorial(10);
    let fib_10 = fibonacci(10);
    
    // Test type inference
    let sum = fact_10 + fib_10;
    let is_large = sum > 1000;
    
    // Test control flow
    if (is_large) {
        let message = "Large result";
        print(message);
    }
    
    // Test loops and function calls
    let prime_count = 0;
    for (let i: i32 = 2; i <= 100; i += 1) {
        if (is_prime(i)) {
            prime_count += 1;
        }
    }
    
    // Test complex expressions
    let final_result = (fact_10 + fib_10) * prime_count / 2;
    
    return;
}
"#;

    let result = compile_to_ast(source);
    assert!(
        result.is_ok(),
        "Showcase program should compile successfully"
    );

    let (program, context) = result.unwrap();

    // Verify all functions are present
    assert_eq!(program.len(), 4, "Should have 4 function declarations");
    assert!(context.functions.contains_key("factorial"));
    assert!(context.functions.contains_key("fibonacci"));
    assert!(context.functions.contains_key("is_prime"));
    assert!(context.functions.contains_key("main"));

    println!("✅ Showcase program compiled successfully!");
    println!("📊 Functions defined: {}", context.functions.len());
    println!("📊 Program statements: {}", program.len());
}
