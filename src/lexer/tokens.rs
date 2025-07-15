// src/lexer/tokens.rs - UPDATED with SIMD utilities
//! Token definitions and utilities for the Eä lexer with SIMD support
//!
//! This module contains helper functions and constants related to tokenization,
//! with extensive support for SIMD types and operations.

use super::TokenKind;

/// SIMD vector width information
#[derive(Debug, Clone, PartialEq)]
pub struct SimdInfo {
    pub element_type: String,
    pub width: usize,
    pub total_bits: usize,
}

impl SimdInfo {
    pub fn new(element_type: String, width: usize) -> Self {
        let element_bits = match element_type.as_str() {
            "f32" | "i32" | "u32" => 32,
            "f64" | "i64" | "u64" => 64,
            "i16" | "u16" => 16,
            "i8" | "u8" => 8,
            _ => 32, // default
        };

        Self {
            element_type,
            width,
            total_bits: element_bits * width,
        }
    }
}

/// Check if a token kind represents a keyword
pub fn is_keyword(token: &TokenKind) -> bool {
    matches!(
        token,
        TokenKind::Func
            | TokenKind::Let
            | TokenKind::Mut
            | TokenKind::Const
            | TokenKind::If
            | TokenKind::Else
            | TokenKind::Return
            | TokenKind::MemRegion
            | TokenKind::Parallel
            | TokenKind::Vectorize
            | TokenKind::Async
            | TokenKind::Await
            | TokenKind::True
            | TokenKind::False
            | TokenKind::While
            | TokenKind::For
    )
}

/// Check if a token kind represents a SIMD keyword
pub fn is_simd_keyword(token: &TokenKind) -> bool {
    matches!(
        token,
        TokenKind::Vectorize
            | TokenKind::SimdWidth
            | TokenKind::SimdAuto
            | TokenKind::TargetFeature
            | TokenKind::Lanes
    )
}

/// Check if a token kind represents a hardware feature
pub fn is_hardware_feature(token: &TokenKind) -> bool {
    matches!(
        token,
        TokenKind::SSE
            | TokenKind::SSE2
            | TokenKind::SSE3
            | TokenKind::SSE4
            | TokenKind::AVX
            | TokenKind::AVX2
            | TokenKind::AVX512
            | TokenKind::NEON
            | TokenKind::AltiVec
    )
}

/// Check if a token kind represents a scalar type
pub fn is_scalar_type(token: &TokenKind) -> bool {
    matches!(
        token,
        TokenKind::I8
            | TokenKind::I16
            | TokenKind::I32
            | TokenKind::I64
            | TokenKind::U8
            | TokenKind::U16
            | TokenKind::U32
            | TokenKind::U64
            | TokenKind::F32
            | TokenKind::F64
            | TokenKind::Bool
            | TokenKind::String
    )
}

/// Check if a token kind represents a SIMD vector type
pub fn is_simd_type(token: &TokenKind) -> bool {
    matches!(
        token,
        // Float SIMD types
        TokenKind::F32x2 | TokenKind::F32x4 | TokenKind::F32x8 | TokenKind::F32x16 |
        TokenKind::F64x2 | TokenKind::F64x4 | TokenKind::F64x8 |
        // Integer SIMD types
        TokenKind::I32x2 | TokenKind::I32x4 | TokenKind::I32x8 | TokenKind::I32x16 |
        TokenKind::I64x2 | TokenKind::I64x4 | TokenKind::I64x8 |
        TokenKind::I16x4 | TokenKind::I16x8 | TokenKind::I16x16 | TokenKind::I16x32 |
        TokenKind::I8x8 | TokenKind::I8x16 | TokenKind::I8x32 | TokenKind::I8x64 |
        // Unsigned SIMD types
        TokenKind::U32x4 | TokenKind::U32x8 | TokenKind::U16x8 | TokenKind::U16x16 |
        TokenKind::U8x16 | TokenKind::U8x32 |
        // Mask types
        TokenKind::Mask8 | TokenKind::Mask16 | TokenKind::Mask32 | TokenKind::Mask64
    )
}

/// Check if a token kind represents any type (scalar or SIMD)
pub fn is_type(token: &TokenKind) -> bool {
    is_scalar_type(token) || is_simd_type(token)
}

/// Check if a token kind represents a standard operator
pub fn is_operator(token: &TokenKind) -> bool {
    matches!(
        token,
        TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Star
            | TokenKind::Slash
            | TokenKind::Percent
            | TokenKind::Assign
            | TokenKind::Equal
            | TokenKind::NotEqual
            | TokenKind::Less
            | TokenKind::LessEqual
            | TokenKind::Greater
            | TokenKind::GreaterEqual
            | TokenKind::And
            | TokenKind::Or
            | TokenKind::Not
            | TokenKind::Ampersand
            | TokenKind::PlusAssign
            | TokenKind::MinusAssign
            | TokenKind::StarAssign
            | TokenKind::SlashAssign
    )
}

/// Check if a token kind represents a SIMD-specific operator
pub fn is_simd_operator(token: &TokenKind) -> bool {
    matches!(
        token,
        TokenKind::DotMultiply
            | TokenKind::DotAdd
            | TokenKind::DotDivide
            | TokenKind::DotOr
            | TokenKind::DotAnd
            | TokenKind::DotXor
    )
}

/// Check if a token kind represents any operator (standard or SIMD)
pub fn is_any_operator(token: &TokenKind) -> bool {
    is_operator(token) || is_simd_operator(token)
}

/// Check if a token kind represents a literal
pub fn is_literal(token: &TokenKind) -> bool {
    matches!(
        token,
        TokenKind::Integer(_)
            | TokenKind::Float(_)
            | TokenKind::StringLiteral(_)
            | TokenKind::SimdLiteral(_)
            | TokenKind::True
            | TokenKind::False
    )
}

/// Get the precedence of a binary operator (higher number = higher precedence)
pub fn operator_precedence(token: &TokenKind) -> Option<u8> {
    match token {
        TokenKind::Or => Some(1),
        TokenKind::And => Some(2),
        TokenKind::Equal | TokenKind::NotEqual => Some(3),
        TokenKind::Less | TokenKind::LessEqual | TokenKind::Greater | TokenKind::GreaterEqual => {
            Some(4)
        }
        TokenKind::Plus | TokenKind::Minus => Some(5),
        TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Some(6),

        // SIMD operators have same precedence as their scalar counterparts
        TokenKind::DotAdd => Some(5), // Same as Plus
        TokenKind::DotMultiply | TokenKind::DotDivide => Some(6), // Same as Star/Slash
        TokenKind::DotOr => Some(1),  // Same as Or
        TokenKind::DotAnd => Some(2), // Same as And
        TokenKind::DotXor => Some(2), // Same as And

        _ => None,
    }
}

/// Check if an operator is right-associative
pub fn is_right_associative(token: &TokenKind) -> bool {
    matches!(
        token,
        TokenKind::Assign
            | TokenKind::PlusAssign
            | TokenKind::MinusAssign
            | TokenKind::StarAssign
            | TokenKind::SlashAssign
    )
}

/// Extract SIMD information from a SIMD type token
pub fn get_simd_info(token: &TokenKind) -> Option<SimdInfo> {
    match token {
        // Float SIMD types
        TokenKind::F32x2 => Some(SimdInfo::new("f32".to_string(), 2)),
        TokenKind::F32x4 => Some(SimdInfo::new("f32".to_string(), 4)),
        TokenKind::F32x8 => Some(SimdInfo::new("f32".to_string(), 8)),
        TokenKind::F32x16 => Some(SimdInfo::new("f32".to_string(), 16)),
        TokenKind::F64x2 => Some(SimdInfo::new("f64".to_string(), 2)),
        TokenKind::F64x4 => Some(SimdInfo::new("f64".to_string(), 4)),
        TokenKind::F64x8 => Some(SimdInfo::new("f64".to_string(), 8)),

        // Integer SIMD types
        TokenKind::I32x2 => Some(SimdInfo::new("i32".to_string(), 2)),
        TokenKind::I32x4 => Some(SimdInfo::new("i32".to_string(), 4)),
        TokenKind::I32x8 => Some(SimdInfo::new("i32".to_string(), 8)),
        TokenKind::I32x16 => Some(SimdInfo::new("i32".to_string(), 16)),
        TokenKind::I64x2 => Some(SimdInfo::new("i64".to_string(), 2)),
        TokenKind::I64x4 => Some(SimdInfo::new("i64".to_string(), 4)),
        TokenKind::I64x8 => Some(SimdInfo::new("i64".to_string(), 8)),

        // 16-bit integers
        TokenKind::I16x4 => Some(SimdInfo::new("i16".to_string(), 4)),
        TokenKind::I16x8 => Some(SimdInfo::new("i16".to_string(), 8)),
        TokenKind::I16x16 => Some(SimdInfo::new("i16".to_string(), 16)),
        TokenKind::I16x32 => Some(SimdInfo::new("i16".to_string(), 32)),

        // 8-bit integers
        TokenKind::I8x8 => Some(SimdInfo::new("i8".to_string(), 8)),
        TokenKind::I8x16 => Some(SimdInfo::new("i8".to_string(), 16)),
        TokenKind::I8x32 => Some(SimdInfo::new("i8".to_string(), 32)),
        TokenKind::I8x64 => Some(SimdInfo::new("i8".to_string(), 64)),

        // Unsigned types
        TokenKind::U32x4 => Some(SimdInfo::new("u32".to_string(), 4)),
        TokenKind::U32x8 => Some(SimdInfo::new("u32".to_string(), 8)),
        TokenKind::U16x8 => Some(SimdInfo::new("u16".to_string(), 8)),
        TokenKind::U16x16 => Some(SimdInfo::new("u16".to_string(), 16)),
        TokenKind::U8x16 => Some(SimdInfo::new("u8".to_string(), 16)),
        TokenKind::U8x32 => Some(SimdInfo::new("u8".to_string(), 32)),

        // Mask types (treated as boolean vectors)
        TokenKind::Mask8 => Some(SimdInfo::new("bool".to_string(), 8)),
        TokenKind::Mask16 => Some(SimdInfo::new("bool".to_string(), 16)),
        TokenKind::Mask32 => Some(SimdInfo::new("bool".to_string(), 32)),
        TokenKind::Mask64 => Some(SimdInfo::new("bool".to_string(), 64)),

        _ => None,
    }
}

/// Get the SIMD width (number of elements) for hardware features
pub fn get_hardware_simd_width(feature: &TokenKind) -> Option<Vec<usize>> {
    match feature {
        TokenKind::SSE => Some(vec![4]),     // 128-bit: 4 x f32 or 4 x i32
        TokenKind::SSE2 => Some(vec![4, 2]), // 128-bit: 4 x f32, 2 x f64
        TokenKind::SSE3 => Some(vec![4, 2]), // Same as SSE2
        TokenKind::SSE4 => Some(vec![4, 2, 8, 16]), // + 8 x i16, 16 x i8
        TokenKind::AVX => Some(vec![8, 4]),  // 256-bit: 8 x f32, 4 x f64
        TokenKind::AVX2 => Some(vec![8, 4, 16, 32]), // + integer vectors
        TokenKind::AVX512 => Some(vec![16, 8, 32, 64]), // 512-bit vectors
        TokenKind::NEON => Some(vec![4, 2, 8, 16]), // ARM NEON 128-bit
        TokenKind::AltiVec => Some(vec![4, 8, 16]), // PowerPC AltiVec 128-bit
        _ => None,
    }
}

/// Check if a SIMD type is compatible with a hardware feature
pub fn is_simd_compatible_with_hardware(simd_type: &TokenKind, feature: &TokenKind) -> bool {
    if let (Some(simd_info), Some(hw_widths)) =
        (get_simd_info(simd_type), get_hardware_simd_width(feature))
    {
        // Check if the SIMD width is supported by the hardware
        hw_widths.contains(&simd_info.width) &&
        // Check if the total bit width is reasonable for the hardware
        match feature {
            TokenKind::SSE | TokenKind::SSE2 | TokenKind::SSE3 | TokenKind::SSE4 | TokenKind::NEON | TokenKind::AltiVec => {
                simd_info.total_bits <= 128
            }
            TokenKind::AVX | TokenKind::AVX2 => {
                simd_info.total_bits <= 256
            }
            TokenKind::AVX512 => {
                simd_info.total_bits <= 512
            }
            _ => false,
        }
    } else {
        false
    }
}

/// Convert a SIMD operator to its scalar equivalent
pub fn simd_to_scalar_operator(simd_op: &TokenKind) -> Option<TokenKind> {
    match simd_op {
        TokenKind::DotAdd => Some(TokenKind::Plus),
        TokenKind::DotMultiply => Some(TokenKind::Star),
        TokenKind::DotDivide => Some(TokenKind::Slash),
        TokenKind::DotAnd => Some(TokenKind::Ampersand),
        TokenKind::DotOr => Some(TokenKind::Or),
        _ => None,
    }
}

/// Get optimal SIMD width for a given element type and target
pub fn get_optimal_simd_width(element_type: &str, target_features: &[TokenKind]) -> usize {
    let element_bits = match element_type {
        "f32" | "i32" | "u32" => 32,
        "f64" | "i64" | "u64" => 64,
        "i16" | "u16" => 16,
        "i8" | "u8" => 8,
        _ => 32,
    };

    // Find the most advanced feature available
    let max_register_bits = if target_features.contains(&TokenKind::AVX512) {
        512
    } else if target_features.contains(&TokenKind::AVX2)
        || target_features.contains(&TokenKind::AVX)
    {
        256
    } else if target_features.iter().any(|f| {
        matches!(
            f,
            TokenKind::SSE | TokenKind::SSE2 | TokenKind::SSE3 | TokenKind::SSE4
        )
    }) {
        128
    } else {
        128 // Default to 128-bit for compatibility
    };

    // Calculate optimal width
    max_register_bits / element_bits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_detection() {
        assert!(is_keyword(&TokenKind::Func));
        assert!(is_keyword(&TokenKind::Let));
        assert!(!is_keyword(&TokenKind::Plus));
        assert!(!is_keyword(&TokenKind::Identifier("test".to_string())));
    }

    #[test]
    fn test_simd_keyword_detection() {
        assert!(is_simd_keyword(&TokenKind::Vectorize));
        assert!(is_simd_keyword(&TokenKind::SimdWidth));
        assert!(!is_simd_keyword(&TokenKind::Func));
    }

    #[test]
    fn test_simd_type_detection() {
        assert!(is_simd_type(&TokenKind::F32x4));
        assert!(is_simd_type(&TokenKind::I32x8));
        assert!(is_simd_type(&TokenKind::Mask16));
        assert!(!is_simd_type(&TokenKind::F32));
        assert!(!is_simd_type(&TokenKind::I32));
    }

    #[test]
    fn test_scalar_type_detection() {
        assert!(is_scalar_type(&TokenKind::F32));
        assert!(is_scalar_type(&TokenKind::I32));
        assert!(is_scalar_type(&TokenKind::Bool));
        assert!(!is_scalar_type(&TokenKind::F32x4));
    }

    #[test]
    fn test_hardware_feature_detection() {
        assert!(is_hardware_feature(&TokenKind::AVX2));
        assert!(is_hardware_feature(&TokenKind::SSE4));
        assert!(is_hardware_feature(&TokenKind::NEON));
        assert!(!is_hardware_feature(&TokenKind::F32x4));
    }

    #[test]
    fn test_operator_precedence() {
        assert_eq!(operator_precedence(&TokenKind::Plus), Some(5));
        assert_eq!(operator_precedence(&TokenKind::Star), Some(6));
        assert_eq!(operator_precedence(&TokenKind::DotAdd), Some(5));
        assert_eq!(operator_precedence(&TokenKind::DotMultiply), Some(6));
        assert_eq!(operator_precedence(&TokenKind::Func), None);
    }

    #[test]
    fn test_simd_info_extraction() {
        let info = get_simd_info(&TokenKind::F32x4).unwrap();
        assert_eq!(info.element_type, "f32");
        assert_eq!(info.width, 4);
        assert_eq!(info.total_bits, 128);

        let info = get_simd_info(&TokenKind::I64x8).unwrap();
        assert_eq!(info.element_type, "i64");
        assert_eq!(info.width, 8);
        assert_eq!(info.total_bits, 512);
    }

    #[test]
    fn test_hardware_compatibility() {
        assert!(is_simd_compatible_with_hardware(
            &TokenKind::F32x4,
            &TokenKind::SSE
        ));
        assert!(is_simd_compatible_with_hardware(
            &TokenKind::F32x8,
            &TokenKind::AVX
        ));
        assert!(is_simd_compatible_with_hardware(
            &TokenKind::F32x16,
            &TokenKind::AVX512
        ));

        // Should not be compatible - too wide for hardware
        assert!(!is_simd_compatible_with_hardware(
            &TokenKind::F32x16,
            &TokenKind::SSE
        ));
    }

    #[test]
    fn test_optimal_simd_width() {
        let avx2_features = vec![TokenKind::AVX2];
        assert_eq!(get_optimal_simd_width("f32", &avx2_features), 8); // 256/32 = 8
        assert_eq!(get_optimal_simd_width("f64", &avx2_features), 4); // 256/64 = 4

        let sse_features = vec![TokenKind::SSE4];
        assert_eq!(get_optimal_simd_width("f32", &sse_features), 4); // 128/32 = 4
        assert_eq!(get_optimal_simd_width("i16", &sse_features), 8); // 128/16 = 8
    }

    #[test]
    fn test_simd_to_scalar_conversion() {
        assert_eq!(
            simd_to_scalar_operator(&TokenKind::DotAdd),
            Some(TokenKind::Plus)
        );
        assert_eq!(
            simd_to_scalar_operator(&TokenKind::DotMultiply),
            Some(TokenKind::Star)
        );
        assert_eq!(
            simd_to_scalar_operator(&TokenKind::DotDivide),
            Some(TokenKind::Slash)
        );
        assert_eq!(simd_to_scalar_operator(&TokenKind::Plus), None);
    }

    #[test]
    fn test_associativity() {
        assert!(is_right_associative(&TokenKind::Assign));
        assert!(!is_right_associative(&TokenKind::Plus));
        assert!(!is_right_associative(&TokenKind::DotAdd));
    }
}
