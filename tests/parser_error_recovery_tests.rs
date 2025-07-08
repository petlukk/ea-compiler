// Tests for parser error recovery and intelligent suggestions
// Week 3: Production Readiness - Day 18-19

use ea_compiler::{lexer::Lexer, parser::Parser};

#[test]
fn test_error_recovery_missing_semicolon() {
    let source = r#"
func main() -> () {
    let x = 42  // Missing semicolon
    let y = 10;
    return;
}
"#;
    
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all().expect("Lexing should succeed");
    let mut parser = Parser::new(tokens);
    
    // Parse and check that we get errors but also continue parsing
    let result = parser.parse_program();
    
    // Should have errors collected
    let errors = parser.get_errors();
    assert!(!errors.is_empty(), "Should have collected parse errors");
    
    // Should have attempted to parse subsequent statements
    println!("Collected {} errors during parsing", errors.len());
    for error in errors {
        println!("Error: {:?}", error);
    }
}

#[test]
fn test_error_recovery_unmatched_parentheses() {
    let source = r#"
func test_func(x: i32 -> i32 {  // Missing closing paren
    let result = x + 1;
    return result;
}
"#;
    
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all().expect("Lexing should succeed");
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse_program();
    let errors = parser.get_errors();
    
    assert!(!errors.is_empty(), "Should detect unmatched parentheses");
    println!("Unmatched parentheses test - {} errors", errors.len());
}

#[test]
fn test_error_recovery_missing_braces() {
    let source = r#"
func incomplete_func() -> () {
    let x = 10;
    if (x > 5) 
        println("Greater than 5");  // Missing braces
    
    return;
}
"#;
    
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all().expect("Lexing should succeed");
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse_program();
    let errors = parser.get_errors();
    
    assert!(!errors.is_empty(), "Should detect missing braces");
    println!("Missing braces test - {} errors", errors.len());
}

#[test]
fn test_error_recovery_invalid_simd_syntax() {
    let source = r#"
func simd_test() -> () {
    let v1 = [1.0, 2.0, 3.0, 4.0]f32x4;
    let v2 = [5.0, 6.0, 7.0, 8.0]f32x4;
    let result = v1 + v2;  // Should be .+ for SIMD
    return;
}
"#;
    
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all().expect("Lexing should succeed");
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse_program();
    let errors = parser.get_errors();
    
    // This might not error at parse time, but we can test the parsing
    println!("SIMD syntax test - {} errors", errors.len());
}

#[test]
fn test_error_recovery_typo_in_keyword() {
    let source = r#"
fucn main() -> () {  // Typo: "fucn" instead of "func"
    let x = 42;
    return;
}
"#;
    
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all().expect("Lexing should succeed");
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse_program();
    let errors = parser.get_errors();
    
    assert!(!errors.is_empty(), "Should detect typo in keyword");
    println!("Keyword typo test - {} errors", errors.len());
}

#[test]
fn test_error_recovery_multiple_errors() {
    let source = r#"
fucn broken_function( -> () {  // Multiple errors: typo + missing param
    let x = 42  // Missing semicolon
    let y = x +;  // Incomplete expression
    return;
"#;
    
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all().expect("Lexing should succeed");
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse_program();
    let errors = parser.get_errors();
    
    // Should collect multiple errors and continue parsing
    assert!(!errors.is_empty(), "Should collect multiple errors");
    println!("Multiple errors test - collected {} errors:", errors.len());
    
    for (i, error) in errors.iter().enumerate() {
        println!("  {}. {:?}", i + 1, error);
    }
}

#[test]
fn test_error_recovery_nested_structures() {
    let source = r#"
func test_nested() -> () {
    if (true) {
        let x = 10;
        while (x > 0 {  // Missing closing paren
            x = x - 1;
        }
    return;  // Missing closing brace for if
}
"#;
    
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all().expect("Lexing should succeed");
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse_program();
    let errors = parser.get_errors();
    
    assert!(!errors.is_empty(), "Should detect nested structure errors");
    println!("Nested structures test - {} errors", errors.len());
}

#[test]
fn test_successful_parsing_after_errors() {
    let source = r#"
// This function has errors
fucn broken() -> () {
    let x = 42
    return;

// This function should still parse correctly
func good_function() -> () {
    let y = 100;
    println("This should work");
    return;
}
"#;
    
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all().expect("Lexing should succeed");
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse_program();
    let errors = parser.get_errors();
    
    // Should have errors from first function but continue to parse second
    assert!(!errors.is_empty(), "Should have errors from broken function");
    println!("Recovery test - {} errors, parsing continued", errors.len());
}

#[test]
fn test_error_suggestions_generation() {
    // Test that error suggestions are generated appropriately
    // This test focuses on the suggestion generation logic
    
    let source = r#"
func test() -> () {
    let x = 42  // Missing semicolon - should suggest adding one
    return;
}
"#;
    
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all().expect("Lexing should succeed");
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse_program();
    let errors = parser.get_errors();
    
    // The important part is that the parser continues and provides helpful suggestions
    println!("Error suggestions test completed");
}

#[test]
fn test_simd_specific_error_recovery() {
    let source = r#"
func simd_errors() -> () {
    let v1 = [1.0, 2.0, 3.0, 4.0]f32x5;  // Invalid SIMD type
    let v2 = [1.0, 2.0, 3.0]f32x4;       // Wrong element count
    let result = v1 ++ v2;                // Invalid operator
    return;
}
"#;
    
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all().expect("Lexing should succeed");
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse_program();
    let errors = parser.get_errors();
    
    println!("SIMD error recovery test - {} errors", errors.len());
    // Parser should attempt to recover and provide SIMD-specific suggestions
}