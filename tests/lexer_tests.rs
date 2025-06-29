// tests/lexer_tests.rs
//! Integration tests for the Eä lexer

use ea_compiler::{Lexer, TokenKind};

#[test]
fn test_simple_function_tokenization() {
    let source = r#"
func fibonacci(n: u64) -> u64 {
    if n <= 1 {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}
"#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all().expect("Failed to tokenize");

    // Check that we got the expected tokens
    assert_eq!(tokens[0].kind, TokenKind::Func);
    assert_eq!(tokens[1].kind, TokenKind::Identifier("fibonacci".to_string()));
    assert_eq!(tokens[2].kind, TokenKind::LeftParen);
    assert_eq!(tokens[3].kind, TokenKind::Identifier("n".to_string()));
    assert_eq!(tokens[4].kind, TokenKind::Colon);
    assert_eq!(tokens[5].kind, TokenKind::U64);
    assert_eq!(tokens[6].kind, TokenKind::RightParen);
    assert_eq!(tokens[7].kind, TokenKind::Arrow);
    assert_eq!(tokens[8].kind, TokenKind::U64);
}

#[test]
fn test_numeric_literals() {
    let source = "42 3.14 1000000 0.001";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all().expect("Failed to tokenize");

    assert_eq!(tokens[0].kind, TokenKind::Integer(42));
    assert_eq!(tokens[1].kind, TokenKind::Float(3.14));
    assert_eq!(tokens[2].kind, TokenKind::Integer(1000000));
    assert_eq!(tokens[3].kind, TokenKind::Float(0.001));
}

#[test]
fn test_hex_binary_literals() {
    let source = "0xFF 0xDEADBEEF 0b1010 0b11110000 0x0 0b0";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all().expect("Failed to tokenize");

    assert_eq!(tokens[0].kind, TokenKind::Integer(255));
    assert_eq!(tokens[1].kind, TokenKind::Integer(0xDEADBEEF));
    assert_eq!(tokens[2].kind, TokenKind::Integer(10));
    assert_eq!(tokens[3].kind, TokenKind::Integer(240));
    assert_eq!(tokens[4].kind, TokenKind::Integer(0));
    assert_eq!(tokens[5].kind, TokenKind::Integer(0));
}

#[test]
fn test_numeric_underscores() {
    let source = "1_000_000 3.141_592_653 0xFF_FF_FF_FF 0b1010_1010_1010_1010";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all().expect("Failed to tokenize");

    assert_eq!(tokens[0].kind, TokenKind::Integer(1_000_000));
    assert_eq!(tokens[1].kind, TokenKind::Float(3.141_592_653));
    assert_eq!(tokens[2].kind, TokenKind::Integer(0xFFFFFFFF_u32 as i64));
    assert_eq!(tokens[3].kind, TokenKind::Integer(0b1010101010101010));
}

#[test]
fn test_string_literals() {
    let source = r#""hello" "world" "escaped \"quotes\"""#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all().expect("Failed to tokenize");

    assert_eq!(tokens[0].kind, TokenKind::StringLiteral("hello".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::StringLiteral("world".to_string()));
    assert_eq!(tokens[2].kind, TokenKind::StringLiteral(r#"escaped \"quotes\""#.to_string()));
}

#[test]
fn test_eä_specific_keywords() {
    let source = "mem_region parallel vectorize async await";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all().expect("Failed to tokenize");

    assert_eq!(tokens[0].kind, TokenKind::MemRegion);
    assert_eq!(tokens[1].kind, TokenKind::Parallel);
    assert_eq!(tokens[2].kind, TokenKind::Vectorize);
    assert_eq!(tokens[3].kind, TokenKind::Async);
    assert_eq!(tokens[4].kind, TokenKind::Await);
}

#[test]
fn test_simd_types() {
    let source = "f32 f64 i32 i64 u8 u16 u32 u64";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all().expect("Failed to tokenize");

    assert_eq!(tokens[0].kind, TokenKind::F32);
    assert_eq!(tokens[1].kind, TokenKind::F64);
    assert_eq!(tokens[2].kind, TokenKind::I32);
    assert_eq!(tokens[3].kind, TokenKind::I64);
    assert_eq!(tokens[4].kind, TokenKind::U8);
    assert_eq!(tokens[5].kind, TokenKind::U16);
    assert_eq!(tokens[6].kind, TokenKind::U32);
    assert_eq!(tokens[7].kind, TokenKind::U64);
}

#[test]
fn test_optimization_attributes() {
    let source = "@optimize @tainted @untainted";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all().expect("Failed to tokenize");

    assert_eq!(tokens[0].kind, TokenKind::At);
    assert_eq!(tokens[1].kind, TokenKind::Identifier("optimize".to_string()));
    assert_eq!(tokens[2].kind, TokenKind::At);
    assert_eq!(tokens[3].kind, TokenKind::Identifier("tainted".to_string()));
    assert_eq!(tokens[4].kind, TokenKind::At);
    assert_eq!(tokens[5].kind, TokenKind::Identifier("untainted".to_string()));
}

#[test]
fn test_assignment_operators() {
    let source = "= += -= *= /=";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all().expect("Failed to tokenize");

    assert_eq!(tokens[0].kind, TokenKind::Assign);
    assert_eq!(tokens[1].kind, TokenKind::PlusAssign);
    assert_eq!(tokens[2].kind, TokenKind::MinusAssign);
    assert_eq!(tokens[3].kind, TokenKind::StarAssign);
    assert_eq!(tokens[4].kind, TokenKind::SlashAssign);
}

#[test]
fn test_complex_expression() {
    let source = "(a + b) * (c - d) / e % f";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all().expect("Failed to tokenize");

    assert_eq!(tokens[0].kind, TokenKind::LeftParen);
    assert_eq!(tokens[1].kind, TokenKind::Identifier("a".to_string()));
    assert_eq!(tokens[2].kind, TokenKind::Plus);
    assert_eq!(tokens[3].kind, TokenKind::Identifier("b".to_string()));
    assert_eq!(tokens[4].kind, TokenKind::RightParen);
    assert_eq!(tokens[5].kind, TokenKind::Star);
}

#[test]
fn test_comments_are_skipped() {
    let source = r#"
// This is a comment
let x = 42; // Another comment
/* Multi-line
   comment */
let y = 3.14;
"#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all().expect("Failed to tokenize");

    // Should only find the actual code tokens, not comments
    let non_eof_tokens: Vec<_> = tokens.iter()
        .filter(|t| t.kind != TokenKind::Eof)
        .collect();

    assert_eq!(non_eof_tokens.len(), 10); // let x = 42 ; let y = 3.14 ;
    assert_eq!(non_eof_tokens[0].kind, TokenKind::Let);
    assert_eq!(non_eof_tokens[1].kind, TokenKind::Identifier("x".to_string()));
    assert_eq!(non_eof_tokens[2].kind, TokenKind::Assign);
    assert_eq!(non_eof_tokens[3].kind, TokenKind::Integer(42));
    assert_eq!(non_eof_tokens[4].kind, TokenKind::Semicolon);
}

#[test]
fn test_position_tracking() {
    let source = "func\nmain";
    let mut lexer = Lexer::new(source);
    
    let token1 = lexer.next_token().unwrap();
    assert_eq!(token1.position.line, 1);
    assert_eq!(token1.position.column, 1);
    
    let token2 = lexer.next_token().unwrap();
    assert_eq!(token2.position.line, 2);
    assert_eq!(token2.position.column, 1);
}

#[test]
fn test_error_handling() {
    // Test that invalid characters produce errors
    let source = "func $ main";
    let mut lexer = Lexer::new(source);
    let result = lexer.tokenize_all();
    
    assert!(result.is_err());
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(error_msg.contains("Unexpected character"));
        assert!(error_msg.contains("$"));
    }
}