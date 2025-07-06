// tests/simd_lexer_tests.rs
//! Comprehensive integration tests for SIMD lexer functionality

use ea_compiler::{Lexer, Position, TokenKind};

/// Test basic SIMD keyword recognition
#[test]
fn test_simd_basic_keywords() {
    let source = "vectorize simd_width simd_auto";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all().unwrap();

    assert_eq!(tokens.len(), 4); // 3 keywords + EOF
    assert_eq!(tokens[0].kind, TokenKind::Vectorize);
    assert_eq!(tokens[1].kind, TokenKind::SimdWidth);
    assert_eq!(tokens[2].kind, TokenKind::SimdAuto);
    assert_eq!(tokens[3].kind, TokenKind::Eof);
}

/// Test SIMD function keywords
#[test]
fn test_simd_function_keywords() {
    let source =
        "horizontal_sum horizontal_min horizontal_max from_slice to_array splat shuffle lanes";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all().unwrap();

    let expected_tokens = [
        TokenKind::HorizontalSum,
        TokenKind::HorizontalMin,
        TokenKind::HorizontalMax,
        TokenKind::FromSlice,
        TokenKind::ToArray,
        TokenKind::Splat,
        TokenKind::Shuffle,
        TokenKind::Lanes,
        TokenKind::Eof,
    ];

    for (i, expected) in expected_tokens.iter().enumerate() {
        assert_eq!(tokens[i].kind, *expected, "Token {} mismatch", i);
    }
}

/// Test hardware feature detection
#[test]
fn test_hardware_features() {
    let source = "sse sse2 sse3 sse4 avx avx2 avx512 neon altivec";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all().unwrap();

    let expected_features = [
        TokenKind::SSE,
        TokenKind::SSE2,
        TokenKind::SSE3,
        TokenKind::SSE4,
        TokenKind::AVX,
        TokenKind::AVX2,
        TokenKind::AVX512,
        TokenKind::NEON,
        TokenKind::AltiVec,
        TokenKind::Eof,
    ];

    for (i, expected) in expected_features.iter().enumerate() {
        assert_eq!(tokens[i].kind, *expected, "Hardware feature {} mismatch", i);
    }
}

/// Test all SIMD vector types
#[test]
fn test_comprehensive_simd_types() {
    let source = r#"
        f32x2 f32x4 f32x8 f32x16
        f64x2 f64x4 f64x8
        i32x2 i32x4 i32x8 i32x16
        i64x2 i64x4 i64x8
        i16x4 i16x8 i16x16 i16x32
        i8x8 i8x16 i8x32 i8x64
        u32x4 u32x8 u16x8 u16x16
        u8x16 u8x32
        mask8 mask16 mask32 mask64
    "#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all().unwrap();

    // Filter out EOF token
    let simd_tokens: Vec<_> = tokens
        .into_iter()
        .filter(|t| t.kind != TokenKind::Eof)
        .collect();

    // We should have 33 SIMD type tokens
    assert_eq!(simd_tokens.len(), 33);

    // Verify some key types
    let token_kinds: Vec<_> = simd_tokens.iter().map(|t| &t.kind).collect();
    assert!(token_kinds.contains(&&TokenKind::F32x4));
    assert!(token_kinds.contains(&&TokenKind::I32x8));
    assert!(token_kinds.contains(&&TokenKind::U8x32));
    assert!(token_kinds.contains(&&TokenKind::Mask16));
}

/// Test SIMD operators
#[test]
fn test_simd_operators() {
    let source = ".* .+ ./ .& .| .^";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all().unwrap();

    let expected_ops = [
        TokenKind::DotMultiply,
        TokenKind::DotAdd,
        TokenKind::DotDivide,
        TokenKind::DotAnd,
        TokenKind::DotOr,
        TokenKind::DotXor,
        TokenKind::Eof,
    ];

    for (i, expected) in expected_ops.iter().enumerate() {
        assert_eq!(tokens[i].kind, *expected, "SIMD operator {} mismatch", i);
    }
}

/// Test realistic SIMD program tokenization
#[test]
fn test_realistic_simd_program() {
    let source = r#"
        @target_feature(avx2)
        func vector_multiply(a: f32x8, b: f32x8) -> f32x8 {
            vectorize with simd_width(256) {
                let result = a .* b;
                return result.horizontal_sum();
            }
        }
        
        func process_data() -> () {
            let vec_a = f32x8::from_slice(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
            let vec_b = f32x8::splat(2.0);
            let result = vector_multiply(vec_a, vec_b);
            return;
        }
    "#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all().unwrap();

    // Verify key SIMD tokens are present
    let token_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();

    // Attributes and target features
    assert!(token_kinds.contains(&&TokenKind::At));
    assert!(token_kinds.contains(&&TokenKind::TargetFeature));
    assert!(token_kinds.contains(&&TokenKind::AVX2));

    // SIMD keywords
    assert!(token_kinds.contains(&&TokenKind::Vectorize));
    assert!(token_kinds.contains(&&TokenKind::SimdWidth));

    // SIMD types
    assert!(token_kinds.contains(&&TokenKind::F32x8));

    // SIMD operators
    assert!(token_kinds.contains(&&TokenKind::DotMultiply));

    // SIMD functions
    assert!(token_kinds.contains(&&TokenKind::HorizontalSum));
    assert!(token_kinds.contains(&&TokenKind::FromSlice));
    assert!(token_kinds.contains(&&TokenKind::Splat));

    // Standard language constructs should still work
    assert!(token_kinds.contains(&&TokenKind::Func));
    assert!(token_kinds.contains(&&TokenKind::Let));
    assert!(token_kinds.contains(&&TokenKind::Return));
}

/// Test SIMD literal parsing (basic structure)
#[test]
fn test_simd_literal_structure() {
    // Note: Full SIMD literal parsing will be implemented in the parser
    // For now, we test that the components are correctly tokenized
    let source = "[1.0, 2.0, 3.0, 4.0] f32x4";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all().unwrap();

    // Should tokenize as: [ 1.0 , 2.0 , 3.0 , 4.0 ] f32x4 EOF
    let expected_sequence = [
        TokenKind::LeftBracket,
        // Numbers and commas...
        // We'll check for the SIMD type at the end
    ];

    assert_eq!(tokens[0].kind, TokenKind::LeftBracket);

    // Find the SIMD type token
    let simd_type_token = tokens.iter().find(|t| t.kind == TokenKind::F32x4);
    assert!(simd_type_token.is_some(), "Should find f32x4 token");
}

/// Test position tracking with SIMD tokens
#[test]
fn test_simd_position_tracking() {
    let source = "vectorize\nf32x8\n.*";
    let mut lexer = Lexer::new(source);

    let token1 = lexer.next_token().unwrap();
    assert_eq!(token1.kind, TokenKind::Vectorize);
    assert_eq!(token1.position.line, 1);
    assert_eq!(token1.position.column, 1);

    let token2 = lexer.next_token().unwrap();
    assert_eq!(token2.kind, TokenKind::F32x8);
    assert_eq!(token2.position.line, 2);
    assert_eq!(token2.position.column, 1);

    let token3 = lexer.next_token().unwrap();
    assert_eq!(token3.kind, TokenKind::DotMultiply);
    assert_eq!(token3.position.line, 3);
    assert_eq!(token3.position.column, 1);
}

/// Test SIMD keywords don't conflict with identifiers
#[test]
fn test_simd_keyword_identifier_distinction() {
    let source = "vectorize_func vectorize my_simd_width simd_width";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all().unwrap();

    // Should be: Identifier("vectorize_func"), Vectorize, Identifier("my_simd_width"), SimdWidth, EOF
    assert_eq!(
        tokens[0].kind,
        TokenKind::Identifier("vectorize_func".to_string())
    );
    assert_eq!(tokens[1].kind, TokenKind::Vectorize);
    assert_eq!(
        tokens[2].kind,
        TokenKind::Identifier("my_simd_width".to_string())
    );
    assert_eq!(tokens[3].kind, TokenKind::SimdWidth);
}

/// Test mixed scalar and SIMD types
#[test]
fn test_mixed_scalar_simd_types() {
    let source = "i32 f32x4 f64 i16x8 bool u8x16 string mask32";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all().unwrap();

    let expected_types = [
        TokenKind::I32,    // scalar
        TokenKind::F32x4,  // SIMD
        TokenKind::F64,    // scalar
        TokenKind::I16x8,  // SIMD
        TokenKind::Bool,   // scalar
        TokenKind::U8x16,  // SIMD
        TokenKind::String, // scalar
        TokenKind::Mask32, // SIMD mask
        TokenKind::Eof,
    ];

    for (i, expected) in expected_types.iter().enumerate() {
        assert_eq!(tokens[i].kind, *expected, "Mixed type {} mismatch", i);
    }
}

/// Test complex SIMD expressions
#[test]
fn test_complex_simd_expressions() {
    let source = "vec_a .* vec_b .+ f32x8::splat(1.0) ./ other_vec";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all().unwrap();

    // Find SIMD operators in the token stream
    let simd_ops: Vec<_> = tokens
        .iter()
        .filter(|t| {
            matches!(
                t.kind,
                TokenKind::DotMultiply | TokenKind::DotAdd | TokenKind::DotDivide
            )
        })
        .collect();

    assert_eq!(simd_ops.len(), 3);
    assert_eq!(simd_ops[0].kind, TokenKind::DotMultiply);
    assert_eq!(simd_ops[1].kind, TokenKind::DotAdd);
    assert_eq!(simd_ops[2].kind, TokenKind::DotDivide);
}

/// Test error handling with invalid SIMD syntax
#[test]
fn test_simd_error_handling() {
    let source = "f32x4 $ invalid";
    let mut lexer = Lexer::new(source);
    let result = lexer.tokenize_all();

    assert!(result.is_err());
    if let Err(e) = result {
        let error_str = e.to_string();
        assert!(error_str.contains("Unexpected character"));
        assert!(error_str.contains("$"));
    }
}

/// Test SIMD numeric literals with underscores
#[test]
fn test_simd_with_numeric_underscores() {
    let source = "let mask = 0xFF_FF_FF_FF; let hex = 0xDEAD_BEEF_CAFE_BABE;";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all().unwrap();

    // Find integer literals
    let int_tokens: Vec<_> = tokens
        .iter()
        .filter(|t| matches!(t.kind, TokenKind::Integer(_)))
        .collect();

    assert_eq!(int_tokens.len(), 2);

    if let TokenKind::Integer(val1) = int_tokens[0].kind {
        assert_eq!(val1, 0xFFFFFFFF);
    }

    // Note: The second hex literal might overflow i64, so we check it exists
    assert!(matches!(int_tokens[1].kind, TokenKind::Integer(_)));
}

/// Test performance with large SIMD programs
#[test]
fn test_simd_lexer_performance() {
    use std::time::Instant;

    // Generate a large SIMD program
    let mut large_program = String::new();
    for i in 0..1000 {
        large_program.push_str(&format!(
            "func simd_func_{i}(a: f32x8, b: f32x8) -> f32x8 {{\n\
             vectorize with simd_width(256) {{\n\
             let result = a .* b .+ f32x8::splat({i}.0);\n\
             return result.horizontal_sum();\n\
             }}\n\
             }}\n"
        ));
    }

    let start = Instant::now();
    let mut lexer = Lexer::new(&large_program);
    let tokens = lexer.tokenize_all().unwrap();
    let duration = start.elapsed();

    // Should tokenize quickly (less than 1 second for this size)
    assert!(
        duration.as_secs() < 1,
        "Lexing took too long: {:?}",
        duration
    );

    // Should have many tokens
    assert!(
        tokens.len() > 10000,
        "Should have many tokens, got {}",
        tokens.len()
    );

    println!("Tokenized {} tokens in {:?}", tokens.len(), duration);
}

/// Test SIMD function calls and method syntax
#[test]
fn test_simd_method_syntax() {
    let source = "vec.horizontal_sum() vec.to_array() f32x8::from_slice(data) f32x8::splat(value)";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all().unwrap();

    // Find method calls and static calls
    let token_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();

    assert!(token_kinds.contains(&&TokenKind::HorizontalSum));
    assert!(token_kinds.contains(&&TokenKind::ToArray));
    assert!(token_kinds.contains(&&TokenKind::FromSlice));
    assert!(token_kinds.contains(&&TokenKind::Splat));
    assert!(token_kinds.contains(&&TokenKind::F32x8));
    assert!(token_kinds.contains(&&TokenKind::DoubleColon)); // ::
}

/// Test edge cases with SIMD type names
#[test]
fn test_simd_type_edge_cases() {
    // Test that we don't accidentally tokenize invalid SIMD types
    let source = "f32x3 f32x0 i32x99 f64x1000 invalid_x4";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all().unwrap();

    // All of these should be parsed as identifiers, not SIMD types
    let non_eof_tokens: Vec<_> = tokens
        .into_iter()
        .filter(|t| t.kind != TokenKind::Eof)
        .collect();

    for token in non_eof_tokens {
        assert!(
            matches!(token.kind, TokenKind::Identifier(_)),
            "Token should be identifier, got {:?}",
            token.kind
        );
    }
}

/// Test vectorize block syntax components
#[test]
fn test_vectorize_block_syntax() {
    let source = r#"
        vectorize with simd_width(auto) {
            // block content
        }
        
        vectorize for item in collection {
            // iterator content  
        }
    "#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all().unwrap();

    let token_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();

    // Check for vectorize constructs
    assert!(token_kinds.contains(&&TokenKind::Vectorize));
    assert!(token_kinds.contains(&&TokenKind::SimdWidth));
    assert!(token_kinds.contains(&&TokenKind::SimdAuto));
    assert!(token_kinds.contains(&&TokenKind::For));

    // Check for structural tokens
    assert!(token_kinds.contains(&&TokenKind::LeftBrace));
    assert!(token_kinds.contains(&&TokenKind::RightBrace));
    assert!(token_kinds.contains(&&TokenKind::LeftParen));
    assert!(token_kinds.contains(&&TokenKind::RightParen));
}
