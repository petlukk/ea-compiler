// tests/type_system_tests.rs
//! Basic tests for the Eä type system implementation.

use ea_compiler::{
    compile_to_ast,
    lexer::Lexer,
    parser::Parser,
    type_system::{EaType, FunctionType, TypeChecker},
};

/// Helper function to type check a single expression.
fn type_check_expression(source: &str) -> Result<EaType, ea_compiler::error::CompileError> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all()?;
    let mut parser = Parser::new(tokens);
    let expr = parser.parse()?;

    let mut type_checker = TypeChecker::new();
    type_checker.check_expression(&expr)
}

#[test]
fn test_debug_tokenization() {
    // Debug the tokenization issue
    let source = "i32";
    let tokens = ea_compiler::tokenize(source).unwrap();

    println!("Tokens for 'i32': {:?}", tokens[0].kind);

    // Should be TokenKind::I32, not Identifier("i32")
    match &tokens[0].kind {
        ea_compiler::lexer::TokenKind::I32 => {
            println!("✅ Correctly tokenized as I32");
        }
        ea_compiler::lexer::TokenKind::Identifier(name) => {
            println!("❌ Incorrectly tokenized as Identifier: {}", name);
        }
        other => {
            println!("❌ Unexpected token: {:?}", other);
        }
    }
}

#[test]
fn test_literal_types() {
    // Integer literals
    assert_eq!(type_check_expression("42").unwrap(), EaType::I64);
    assert_eq!(type_check_expression("-42").unwrap(), EaType::I64);

    // Float literals
    assert_eq!(type_check_expression("3.14").unwrap(), EaType::F64);
    assert_eq!(type_check_expression("-3.14").unwrap(), EaType::F64);

    // Boolean literals
    assert_eq!(type_check_expression("true").unwrap(), EaType::Bool);
    assert_eq!(type_check_expression("false").unwrap(), EaType::Bool);

    // String literals
    assert_eq!(type_check_expression("\"hello\"").unwrap(), EaType::String);
}

#[test]
fn test_arithmetic_expressions() {
    // Integer arithmetic
    assert_eq!(type_check_expression("1 + 2").unwrap(), EaType::I64);
    assert_eq!(type_check_expression("10 - 5").unwrap(), EaType::I64);
    assert_eq!(type_check_expression("3 * 4").unwrap(), EaType::I64);
    assert_eq!(type_check_expression("8 / 2").unwrap(), EaType::I64);
    assert_eq!(type_check_expression("9 % 4").unwrap(), EaType::I64);

    // Float arithmetic
    assert_eq!(type_check_expression("1.5 + 2.5").unwrap(), EaType::F64);
    assert_eq!(type_check_expression("10.0 - 3.14").unwrap(), EaType::F64);
    assert_eq!(type_check_expression("2.0 * 3.0").unwrap(), EaType::F64);
    assert_eq!(type_check_expression("7.5 / 2.5").unwrap(), EaType::F64);

    // Complex expressions
    assert_eq!(type_check_expression("(1 + 2) * 3").unwrap(), EaType::I64);
    assert_eq!(type_check_expression("1 + 2 * 3").unwrap(), EaType::I64);
}

#[test]
fn test_comparison_expressions() {
    // Integer comparisons
    assert_eq!(type_check_expression("1 < 2").unwrap(), EaType::Bool);
    assert_eq!(type_check_expression("5 > 3").unwrap(), EaType::Bool);
    assert_eq!(type_check_expression("4 <= 4").unwrap(), EaType::Bool);
    assert_eq!(type_check_expression("7 >= 6").unwrap(), EaType::Bool);
    assert_eq!(type_check_expression("1 == 1").unwrap(), EaType::Bool);
    assert_eq!(type_check_expression("2 != 3").unwrap(), EaType::Bool);

    // Float comparisons
    assert_eq!(type_check_expression("1.5 < 2.5").unwrap(), EaType::Bool);
    assert_eq!(type_check_expression("3.14 == 3.14").unwrap(), EaType::Bool);

    // String comparisons
    assert_eq!(
        type_check_expression("\"a\" == \"b\"").unwrap(),
        EaType::Bool
    );
    assert_eq!(
        type_check_expression("\"hello\" != \"world\"").unwrap(),
        EaType::Bool
    );
}

#[test]
fn test_logical_expressions() {
    // Boolean logic
    assert_eq!(
        type_check_expression("true && false").unwrap(),
        EaType::Bool
    );
    assert_eq!(
        type_check_expression("true || false").unwrap(),
        EaType::Bool
    );
    assert_eq!(type_check_expression("!true").unwrap(), EaType::Bool);
    assert_eq!(type_check_expression("!false").unwrap(), EaType::Bool);

    // Complex logical expressions
    assert_eq!(
        type_check_expression("(1 < 2) && (3 > 2)").unwrap(),
        EaType::Bool
    );
    assert_eq!(
        type_check_expression("true || (5 == 5)").unwrap(),
        EaType::Bool
    );
}

#[test]
fn test_variable_declarations() {
    // Variable with explicit type and initializer
    let result = compile_to_ast("let x: i32 = 42;");
    assert!(result.is_ok());

    // Variable with type inference
    let result = compile_to_ast("let y = 3.14;");
    assert!(result.is_ok());

    // Variable with explicit type, no initializer
    let result = compile_to_ast("let z: bool;");
    assert!(result.is_ok());

    // Mutable variable
    let result = compile_to_ast("let mut a: i64 = 100;");
    assert!(result.is_ok());
}

#[test]
fn test_function_declarations() {
    // Simple function with parameters and return type
    let source = r#"
        func add(a: i32, b: i32) -> i32 {
            return a + b;
        }
    "#;
    let result = compile_to_ast(source);
    assert!(result.is_ok());

    // Void function
    let source = r#"
        func greet(name: string) -> () {
            return;
        }
    "#;
    let result = compile_to_ast(source);
    assert!(result.is_ok());
}

#[test]
fn test_function_calls() {
    // Test with built-in print function
    let source = r#"
        func test() -> () {
            print("hello");
            return;
        }
    "#;
    let result = compile_to_ast(source);
    assert!(result.is_ok());
}

#[test]
fn test_control_flow_statements() {
    // If statement
    let source = r#"
        func test_if() -> () {
            if (true) {
                let x = 1;
            } else {
                let y = 2;
            }
            return;
        }
    "#;
    let result = compile_to_ast(source);
    assert!(result.is_ok());

    // While loop
    let source = r#"
        func test_while() -> () {
            while (false) {
                let i = 0;
            }
            return;
        }
    "#;
    let result = compile_to_ast(source);
    assert!(result.is_ok());

    // For loop
    let source = r#"
        func test_for() -> () {
            for (let i: i32 = 0; i < 10; i += 1) {
                let x = i * 2;
            }
            return;
        }
    "#;
    let result = compile_to_ast(source);
    assert!(result.is_ok());
}

#[test]
fn test_type_errors() {
    // Type mismatch in variable declaration
    let source = "let x: i32 = \"hello\";";
    let result = compile_to_ast(source);
    assert!(result.is_err());

    // Type mismatch in arithmetic
    let result = type_check_expression("1 + \"hello\"");
    assert!(result.is_err());

    // Invalid condition in if statement
    let source = r#"
        func test() -> () {
            if (42) {
                return;
            }
            return;
        }
    "#;
    let result = compile_to_ast(source);
    assert!(result.is_err());

    // Wrong return type
    let source = r#"
        func test() -> i32 {
            return "hello";
        }
    "#;
    let result = compile_to_ast(source);
    assert!(result.is_err());
}

#[test]
fn test_complex_program() {
    let source = r#"
        func fibonacci(n: i32) -> i32 {
            if (n <= 1) {
                return n;
            }
            return fibonacci(n - 1) + fibonacci(n - 2);
        }

        func main() -> () {
            let result: i32 = fibonacci(10);
            let flag: bool = result > 50;
            
            if (flag) {
                let message: string = "Large result";
            } else {
                let count: i32 = 0;
                while (count < result) {
                    count += 1;
                }
            }
            
            return;
        }
    "#;

    let result = compile_to_ast(source);
    assert!(
        result.is_ok(),
        "Complex program should type check successfully"
    );
}

#[test]
fn test_scoping() {
    let source = r#"
        func test_scoping() -> () {
            let x: i32 = 1;
            {
                let x: string = "shadowed";
                let y: bool = true;
            }
            // x should be i32 again here, y should not be accessible
            let z: i32 = x + 1;
            return;
        }
    "#;

    let result = compile_to_ast(source);
    assert!(result.is_ok(), "Scoping should work correctly");
}

#[test]
fn test_type_basic_properties() {
    // Test basic type equality
    assert_eq!(EaType::I32, EaType::I32);
    assert_ne!(EaType::I32, EaType::I64);
    assert_ne!(EaType::I32, EaType::String);

    // Test built-in type properties (methods we actually implemented)
    assert!(EaType::I32.is_integer());
    assert!(EaType::I32.is_numeric());
    assert!(!EaType::I32.is_float());
    assert!(EaType::I32.is_comparable());

    assert!(EaType::F64.is_float());
    assert!(EaType::F64.is_numeric());
    assert!(!EaType::F64.is_integer());
    assert!(EaType::F64.is_comparable());

    assert!(!EaType::Bool.is_numeric());
    assert!(!EaType::Bool.is_comparable());

    assert!(EaType::String.is_comparable());
    assert!(!EaType::String.is_numeric());
}

#[test]
fn test_expressions_with_variables() {
    let source = r#"
        func test() -> () {
            let x: i32 = 10;
            let y: i32 = 20;
            let sum: i32 = x + y;
            let is_equal: bool = x == y;
            return;
        }
    "#;

    let result = compile_to_ast(source);
    assert!(
        result.is_ok(),
        "Variable expressions should type check correctly"
    );
}

#[test]
fn test_nested_expressions() {
    // Test complex nested expressions
    assert_eq!(
        type_check_expression("((1 + 2) * 3) + (4 - 5)").unwrap(),
        EaType::I64
    );
    assert_eq!(
        type_check_expression("(1 < 2) && (3 > 2) || (4 == 4)").unwrap(),
        EaType::Bool
    );
}

// Tests from basic_type_test.rs (consolidated for optimization)

#[test]
fn test_basic_compilation() {
    let source = r#"
        func add(a: i32, b: i32) -> i32 {
            return a + b;
        }
        
        func main() -> () {
            let result: i32 = add(5, 10);
            return;
        }
    "#;

    // Test full compilation pipeline
    let result = compile_to_ast(source);
    assert!(result.is_ok(), "Basic program should compile successfully");
}

#[test]
fn test_type_inference() {
    let source = r#"
        func test() -> () {
            let x = 42;        // Should infer i64
            let y = 3.14;      // Should infer f64  
            let z = true;      // Should infer bool
            let w = "hello";   // Should infer string
            return;
        }
    "#;

    let result = compile_to_ast(source);
    assert!(result.is_ok(), "Type inference should work correctly");
}

#[test]
fn test_basic_type_errors() {
    let source = r#"
        func bad_func() -> i32 {
            return "hello";  // Type error: string instead of i32
        }
    "#;

    let result = compile_to_ast(source);
    assert!(result.is_err(), "Type errors should be caught");
}

#[test]
fn test_expression_types() {
    let mut type_checker = TypeChecker::new();

    // Test arithmetic expression
    let tokens = ea_compiler::tokenize("1 + 2").unwrap();
    let mut parser = ea_compiler::parser::Parser::new(tokens);
    let expr = parser.parse().unwrap();

    let expr_type = type_checker.check_expression(&expr).unwrap();
    assert_eq!(expr_type, EaType::I64);

    // Test boolean expression
    let tokens = ea_compiler::tokenize("true && false").unwrap();
    let mut parser = ea_compiler::parser::Parser::new(tokens);
    let expr = parser.parse().unwrap();

    let expr_type = type_checker.check_expression(&expr).unwrap();
    assert_eq!(expr_type, EaType::Bool);
}

#[test]
fn test_basic_function_calls() {
    let source = r#"
        func multiply(a: i32, b: i32) -> i32 {
            return a * b;
        }
        
        func test() -> () {
            let result: i32 = multiply(3, 4);
            return;
        }
    "#;

    let result = compile_to_ast(source);
    assert!(result.is_ok(), "Function calls should type check correctly");
}
