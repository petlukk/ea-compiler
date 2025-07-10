// tests/stdlib_integration_tests.rs
//! Standard library integration tests

use ea_compiler::{tokenize, parse, compile_to_ast};

#[test]
fn test_stdlib_tokenization() {
    let source = r#"
        func test() -> () {
            let vec: Vec = Vec::new();
            let map: HashMap = HashMap::new();
            let set: HashSet = HashSet::new();
            let text: String = String::new();
            println("Hello, world!");
            return;
        }
    "#;

    let tokens = tokenize(source).expect("Tokenization should succeed");
    
    // Check for standard library type tokens
    let token_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    
    // Should contain Vec, HashMap, HashSet, String tokens
    assert!(token_kinds.iter().any(|kind| matches!(kind, ea_compiler::lexer::TokenKind::VecType)));
    assert!(token_kinds.iter().any(|kind| matches!(kind, ea_compiler::lexer::TokenKind::HashMapType)));
    assert!(token_kinds.iter().any(|kind| matches!(kind, ea_compiler::lexer::TokenKind::HashSetType)));
    assert!(token_kinds.iter().any(|kind| matches!(kind, ea_compiler::lexer::TokenKind::StringType)));
    assert!(token_kinds.iter().any(|kind| matches!(kind, ea_compiler::lexer::TokenKind::Println)));
}

#[test] 
fn test_stdlib_parsing() {
    let source = r#"
        func test() -> () {
            let vec: Vec = Vec::new();
            println("Hello!");
            return;
        }
    "#;

    // This should parse without errors
    let result = parse(source);
    assert!(result.is_ok(), "Parsing should succeed: {:?}", result.err());
}

#[test]
fn test_stdlib_type_checking() {
    let source = r#"
        func test() -> () {
            let numbers: Vec = Vec::new();
            let mapping: HashMap = HashMap::new();
            let values: HashSet = HashSet::new();
            let text: String = String::new();
            return;
        }
    "#;

    // This should compile (parse + type check) without errors
    let result = compile_to_ast(source);
    assert!(result.is_ok(), "Type checking should succeed: {:?}", result.err());
}

#[test]
fn test_println_tokenization() {
    let source = r#"
        func main() -> () {
            println("Test message");
            return;
        }
    "#;

    let tokens = tokenize(source).expect("Tokenization should succeed");
    let token_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    
    // Should contain println token
    assert!(token_kinds.iter().any(|kind| matches!(kind, ea_compiler::lexer::TokenKind::Println)));
}

#[test]
fn test_basic_stdlib_program() {
    let source = r#"
        func main() -> () {
            println("Standard library test!");
            return;
        }
    "#;

    // Full compilation pipeline should work
    let result = compile_to_ast(source);
    assert!(result.is_ok(), "Full compilation should succeed: {:?}", result.err());
    
    let (ast, _type_context) = result.unwrap();
    assert_eq!(ast.len(), 1); // Should have one function declaration
}

#[test]
fn test_vec_new_parsing() {
    let source = "fn main() { let vec = Vec::new(); }";
    
    // Test tokenization first
    let tokens = tokenize(source).expect("Tokenization should succeed");
    println!("Tokens: {:?}", tokens);
    
    // Test parsing
    let ast = parse(source).expect("Parsing should succeed");
    println!("AST: {:?}", ast);
    
    // Should have parsed successfully
    assert!(!ast.is_empty());
}