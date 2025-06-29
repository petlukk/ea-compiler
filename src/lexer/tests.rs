//! Tests for the Eä lexer.

use super::*;
use pretty_assertions::assert_eq;

/// Helper function to create a token with default span and position.
fn token(kind: TokenKind) -> Token {
    Token {
        kind,
        span: (0, 0),
        line: 1,
        column: 1,
    }
}

/// Helper function to create a token with a specific lexeme.
fn ident(name: &str) -> Token {
    Token {
        kind: TokenKind::Identifier(name.to_string()),
        span: (0, name.len()),
        line: 1,
        column: 1,
    }
}

/// Helper function to create a number token.
fn number(n: f64) -> Token {
    Token {
        kind: TokenKind::Number(n),
        span: (0, n.to_string().len()),
        line: 1,
        column: 1,
    }
}

/// Helper function to create a string literal token.
fn string_lit(s: &str) -> Token {
    let content = format!("\"{}\"", s);
    Token {
        kind: TokenKind::StringLiteral(s.to_string()),
        span: (0, content.len()),
        line: 1,
        column: 1,
    }
}

#[test]
fn test_lexer_basic() {
    let input = "let x = 42;";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.scan_tokens();
    
    assert_eq!(tokens[0].kind, TokenKind::Let);
    assert_eq!(tokens[1].kind, TokenKind::Identifier("x".to_string()));
    assert_eq!(tokens[2].kind, TokenKind::Equal);
    assert_eq!(tokens[3].kind, TokenKind::Number(42.0));
    assert_eq!(tokens[4].kind, TokenKind::Semicolon);
    assert_eq!(tokens[5].kind, TokenKind::Eof);
}

#[test]
fn test_lexer_keywords() {
    let input = "fn let mut const if else return true false for while in";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.scan_tokens();
    
    let expected = vec![
        TokenKind::Fn,
        TokenKind::Let,
        TokenKind::Mut,
        TokenKind::Const,
        TokenKind::If,
        TokenKind::Else,
        TokenKind::Return,
        TokenKind::True,
        TokenKind::False,
        TokenKind::For,
        TokenKind::While,
        TokenKind::In,
        TokenKind::Eof,
    ];
    
    for (i, kind) in expected.into_iter().enumerate() {
        assert_eq!(tokens[i].kind, kind);
    }
}

#[test]
fn test_lexer_operators() {
    let input = "+ - * / = == != < <= > >= -> |";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.scan_tokens();
    
    let expected = vec![
        TokenKind::Plus,
        TokenKind::Minus,
        TokenKind::Star,
        TokenKind::Slash,
        TokenKind::Equal,
        TokenKind::EqualEqual,
        TokenKind::BangEqual,
        TokenKind::Less,
        TokenKind::LessEqual,
        TokenKind::Greater,
        TokenKind::GreaterEqual,
        TokenKind::Arrow,
        TokenKind::Pipe,
        TokenKind::Eof,
    ];
    
    for (i, kind) in expected.into_iter().enumerate() {
        assert_eq!(tokens[i].kind, kind);
    }
}

#[test]
fn test_lexer_comments() {
    let input = r#"
        // This is a comment
        let x = 42; // Another comment
        /* Multi-line
           comment */
        let y = 3.14;
    "#;
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.scan_tokens();
    
    // Should only find the actual code tokens, not comments
    assert_eq!(tokens[0].kind, TokenKind::Let);
    assert_eq!(tokens[1].kind, TokenKind::Identifier("x".to_string()));
    assert_eq!(tokens[2].kind, TokenKind::Equal);
    assert_eq!(tokens[3].kind, TokenKind::Number(42.0));
    assert_eq!(tokens[4].kind, TokenKind::Semicolon);
    assert_eq!(tokens[5].kind, TokenKind::Let);
    assert_eq!(tokens[6].kind, TokenKind::Identifier("y".to_string()));
    assert_eq!(tokens[7].kind, TokenKind::Equal);
    assert_eq!(tokens[8].kind, TokenKind::Number(3.14));
    assert_eq!(tokens[9].kind, TokenKind::Semicolon);
    assert_eq!(tokens[10].kind, TokenKind::Eof);
}

#[test]
fn test_lexer_strings() {
    let input = r#"""hello" "world" "escaped \"quote\"""#;
    let mut lexer = Lexer::new(input);
    let tokens = lexer.scan_tokens();
    
    assert_eq!(tokens[0].kind, TokenKind::StringLiteral("hello".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::StringLiteral("world".to_string()));
    assert_eq!(tokens[2].kind, TokenKind::StringLiteral("escaped \"quote\"".to_string()));
    assert_eq!(tokens[3].kind, TokenKind::Eof);
}

#[test]
fn test_lexer_numbers() {
    let input = "42 3.14 0.5 .5 1e10 1e-5 1.2e+3";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.scan_tokens();
    
    assert_eq!(tokens[0].kind, TokenKind::Number(42.0));
    assert_eq!(tokens[1].kind, TokenKind::Number(3.14));
    assert_eq!(tokens[2].kind, TokenKind::Number(0.5));
    assert_eq!(tokens[3].kind, TokenKind::Number(0.5));
    assert_eq!(tokens[4].kind, TokenKind::Number(1e10));
    assert_eq!(tokens[5].kind, TokenKind::Number(1e-5));
    assert_eq!(tokens[6].kind, TokenKind::Number(1.2e3));
    assert_eq!(tokens[7].kind, TokenKind::Eof);
}

#[test]
fn test_lexer_mem_keywords() {
    let input = "mem_region parallel vectorize async await";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.scan_tokens();
    
    assert_eq!(tokens[0].kind, TokenKind::MemRegion);
    assert_eq!(tokens[1].kind, TokenKind::Parallel);
    assert_eq!(tokens[2].kind, TokenKind::Vectorize);
    assert_eq!(tokens[3].kind, TokenKind::Async);
    assert_eq!(tokens[4].kind, TokenKind::Await);
    assert_eq!(tokens[5].kind, TokenKind::Eof);
}

// Add more tests for error cases, edge cases, etc.
