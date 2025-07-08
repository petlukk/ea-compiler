//! SIMD-Accelerated Mathematical Operations
//!
//! High-performance mathematical functions with automatic SIMD vectorization
//! for common operations like trigonometry, exponentials, and linear algebra.

use std::f32::consts::{E, PI};

/// SIMD-accelerated mathematical functions
pub mod simd_math {
    use super::*;
    use crate::stdlib::collections::Vec as EaVec;

    /// SIMD-accelerated sine function for vectors
    pub fn vec_sin(input: &EaVec<f32>) -> EaVec<f32> {
        input.simd_map(|x| x.sin())
    }

    /// SIMD-accelerated cosine function for vectors
    pub fn vec_cos(input: &EaVec<f32>) -> EaVec<f32> {
        input.simd_map(|x| x.cos())
    }

    /// SIMD-accelerated tangent function for vectors
    pub fn vec_tan(input: &EaVec<f32>) -> EaVec<f32> {
        input.simd_map(|x| x.tan())
    }

    /// SIMD-accelerated square root for vectors
    pub fn vec_sqrt(input: &EaVec<f32>) -> EaVec<f32> {
        input.simd_map(|x| x.sqrt())
    }

    /// SIMD-accelerated exponential function for vectors
    pub fn vec_exp(input: &EaVec<f32>) -> EaVec<f32> {
        input.simd_map(|x| x.exp())
    }

    /// SIMD-accelerated natural logarithm for vectors
    pub fn vec_ln(input: &EaVec<f32>) -> EaVec<f32> {
        input.simd_map(|x| x.ln())
    }

    /// SIMD-accelerated power function for vectors
    pub fn vec_pow(base: &EaVec<f32>, exponent: f32) -> EaVec<f32> {
        base.simd_map(|x| x.powf(exponent))
    }

    /// SIMD-accelerated absolute value for vectors
    pub fn vec_abs(input: &EaVec<f32>) -> EaVec<f32> {
        input.simd_map(|x| x.abs())
    }

    /// SIMD-accelerated element-wise minimum
    pub fn vec_min(a: &EaVec<f32>, b: &EaVec<f32>) -> Result<EaVec<f32>, MathError> {
        if a.len() != b.len() {
            return Err(MathError::DimensionMismatch(a.len(), b.len()));
        }

        let mut result = EaVec::with_capacity(a.len());
        for i in 0..a.len() {
            let val_a = a.get(i).unwrap();
            let val_b = b.get(i).unwrap();
            result.push(val_a.min(*val_b));
        }
        Ok(result)
    }

    /// SIMD-accelerated element-wise maximum
    pub fn vec_max(a: &EaVec<f32>, b: &EaVec<f32>) -> Result<EaVec<f32>, MathError> {
        if a.len() != b.len() {
            return Err(MathError::DimensionMismatch(a.len(), b.len()));
        }

        let mut result = EaVec::with_capacity(a.len());
        for i in 0..a.len() {
            let val_a = a.get(i).unwrap();
            let val_b = b.get(i).unwrap();
            result.push(val_a.max(*val_b));
        }
        Ok(result)
    }

    /// SIMD-accelerated vector normalization
    pub fn vec_normalize(input: &EaVec<f32>) -> Result<EaVec<f32>, MathError> {
        if input.is_empty() {
            return Err(MathError::EmptyVector);
        }

        // Calculate magnitude using SIMD dot product
        let magnitude_squared = input.simd_dot(input)?;
        if magnitude_squared == 0.0 {
            return Err(MathError::ZeroMagnitude);
        }

        let magnitude = magnitude_squared.sqrt();
        let inv_magnitude = 1.0 / magnitude;
        
        Ok(input.simd_map(|x| x * inv_magnitude))
    }

    /// SIMD-accelerated matrix-vector multiplication (simplified)
    pub fn matrix_vector_multiply(
        matrix: &[EaVec<f32>], 
        vector: &EaVec<f32>
    ) -> Result<EaVec<f32>, MathError> {
        if matrix.is_empty() {
            return Err(MathError::EmptyMatrix);
        }

        let rows = matrix.len();
        let cols = matrix[0].len();

        if vector.len() != cols {
            return Err(MathError::DimensionMismatch(vector.len(), cols));
        }

        let mut result = EaVec::with_capacity(rows);
        
        for row in matrix {
            if row.len() != cols {
                return Err(MathError::InconsistentMatrixDimensions);
            }
            
            let dot_product = row.simd_dot(vector)?;
            result.push(dot_product);
        }

        Ok(result)
    }

    /// SIMD-accelerated polynomial evaluation using Horner's method
    pub fn vec_poly_eval(coefficients: &EaVec<f32>, x_values: &EaVec<f32>) -> EaVec<f32> {
        if coefficients.is_empty() {
            return EaVec::new();
        }

        x_values.simd_map(|x| {
            let mut result = *coefficients.get(0).unwrap_or(&0.0);
            for i in 1..coefficients.len() {
                result = result * x + coefficients.get(i).unwrap();
            }
            result
        })
    }

    /// Fast SIMD-accelerated approximate sine using polynomial approximation
    pub fn vec_sin_fast(input: &EaVec<f32>) -> EaVec<f32> {
        // Polynomial approximation for sin(x) on [-π, π]
        // sin(x) ≈ x - x³/6 + x⁵/120 - x⁷/5040 (Taylor series)
        input.simd_map(|mut x| {
            // Normalize to [-π, π]
            x = x % (2.0 * PI);
            if x > PI {
                x -= 2.0 * PI;
            }
            
            let x2 = x * x;
            let x3 = x2 * x;
            let x5 = x3 * x2;
            let x7 = x5 * x2;
            
            x - x3 / 6.0 + x5 / 120.0 - x7 / 5040.0
        })
    }

    /// SIMD-accelerated linear interpolation
    pub fn vec_lerp(a: &EaVec<f32>, b: &EaVec<f32>, t: f32) -> Result<EaVec<f32>, MathError> {
        if a.len() != b.len() {
            return Err(MathError::DimensionMismatch(a.len(), b.len()));
        }

        let mut result = EaVec::with_capacity(a.len());
        for i in 0..a.len() {
            let val_a = a.get(i).unwrap();
            let val_b = b.get(i).unwrap();
            result.push(val_a + t * (val_b - val_a));
        }
        Ok(result)
    }
}

/// Mathematical constants
pub mod constants {
    pub const PI: f32 = std::f32::consts::PI;
    pub const E: f32 = std::f32::consts::E;
    pub const TAU: f32 = 2.0 * PI;
    pub const SQRT_2: f32 = std::f32::consts::SQRT_2;
    pub const SQRT_3: f32 = 1.7320508075688772;
    pub const GOLDEN_RATIO: f32 = 1.618033988749895;
}

/// Scalar mathematical functions
pub mod scalar {
    use super::*;

    /// Fast integer square root using Newton's method
    pub fn isqrt(n: u32) -> u32 {
        if n == 0 {
            return 0;
        }
        
        let mut x = n;
        let mut y = (x + 1) / 2;
        
        while y < x {
            x = y;
            y = (x + n / x) / 2;
        }
        
        x
    }

    /// Fast modular exponentiation
    pub fn mod_pow(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
        if modulus == 1 {
            return 0;
        }
        
        let mut result = 1;
        base %= modulus;
        
        while exp > 0 {
            if exp & 1 == 1 {
                result = (result * base) % modulus;
            }
            exp >>= 1;
            base = (base * base) % modulus;
        }
        
        result
    }

    /// Greatest common divisor using Euclidean algorithm
    pub fn gcd(mut a: u64, mut b: u64) -> u64 {
        while b != 0 {
            let temp = b;
            b = a % b;
            a = temp;
        }
        a
    }

    /// Least common multiple
    pub fn lcm(a: u64, b: u64) -> u64 {
        if a == 0 || b == 0 {
            0
        } else {
            (a / gcd(a, b)) * b
        }
    }

    /// Check if number is prime using trial division
    pub fn is_prime(n: u64) -> bool {
        if n < 2 {
            return false;
        }
        if n == 2 {
            return true;
        }
        if n % 2 == 0 {
            return false;
        }
        
        let limit = (n as f64).sqrt() as u64 + 1;
        for i in (3..=limit).step_by(2) {
            if n % i == 0 {
                return false;
            }
        }
        true
    }

    /// Factorial calculation with overflow protection
    pub fn factorial(n: u32) -> Result<u64, MathError> {
        if n > 20 {
            return Err(MathError::Overflow("Factorial too large for u64".to_string()));
        }
        
        let mut result = 1u64;
        for i in 2..=n {
            result = result.checked_mul(i as u64)
                .ok_or_else(|| MathError::Overflow("Factorial overflow".to_string()))?;
        }
        Ok(result)
    }
}

#[derive(Debug, PartialEq)]
pub enum MathError {
    DimensionMismatch(usize, usize),
    EmptyVector,
    EmptyMatrix,
    ZeroMagnitude,
    InconsistentMatrixDimensions,
    Overflow(String),
    InvalidInput(String),
}

impl std::fmt::Display for MathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MathError::DimensionMismatch(a, b) => {
                write!(f, "Dimension mismatch: {} vs {}", a, b)
            }
            MathError::EmptyVector => write!(f, "Empty vector provided"),
            MathError::EmptyMatrix => write!(f, "Empty matrix provided"),
            MathError::ZeroMagnitude => write!(f, "Vector has zero magnitude"),
            MathError::InconsistentMatrixDimensions => write!(f, "Matrix has inconsistent dimensions"),
            MathError::Overflow(msg) => write!(f, "Mathematical overflow: {}", msg),
            MathError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

impl std::error::Error for MathError {}

// Convert SIMD errors to math errors for compatibility
impl From<crate::stdlib::collections::SIMDError> for MathError {
    fn from(error: crate::stdlib::collections::SIMDError) -> Self {
        match error {
            crate::stdlib::collections::SIMDError::LengthMismatch(a, b) => {
                MathError::DimensionMismatch(a, b)
            }
            crate::stdlib::collections::SIMDError::InvalidVectorWidth(w) => {
                MathError::InvalidInput(format!("Invalid vector width: {}", w))
            }
            crate::stdlib::collections::SIMDError::UnsupportedOperation(op) => {
                MathError::InvalidInput(format!("Unsupported operation: {}", op))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stdlib::collections::Vec as EaVec;

    #[test]
    fn test_simd_trigonometric_functions() {
        let angles: EaVec<f32> = vec![0.0, PI/6.0, PI/4.0, PI/3.0, PI/2.0].into();
        
        let sin_results = simd_math::vec_sin(&angles);
        assert_eq!(sin_results.len(), 5);
        
        // Check some known values (with small tolerance for floating point)
        assert!((sin_results.get(0).unwrap() - 0.0).abs() < 1e-6);
        assert!((sin_results.get(4).unwrap() - 1.0).abs() < 1e-6);
        
        let cos_results = simd_math::vec_cos(&angles);
        assert_eq!(cos_results.len(), 5);
        assert!((cos_results.get(0).unwrap() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_simd_mathematical_operations() {
        let vec1: EaVec<f32> = vec![1.0, 4.0, 9.0, 16.0].into();
        
        let sqrt_results = simd_math::vec_sqrt(&vec1);
        assert_eq!(sqrt_results.len(), 4);
        assert_eq!(sqrt_results.get(0), Some(&1.0));
        assert_eq!(sqrt_results.get(1), Some(&2.0));
        assert_eq!(sqrt_results.get(2), Some(&3.0));
        assert_eq!(sqrt_results.get(3), Some(&4.0));
        
        let exp_results = simd_math::vec_exp(&vec1);
        assert_eq!(exp_results.len(), 4);
        
        let abs_results = simd_math::vec_abs(&vec![-1.0, 2.0, -3.0, 4.0].into());
        assert_eq!(abs_results.get(0), Some(&1.0));
        assert_eq!(abs_results.get(2), Some(&3.0));
    }

    #[test]
    fn test_vector_normalization() {
        let vec: EaVec<f32> = vec![3.0, 4.0].into(); // 3-4-5 triangle
        let normalized = simd_math::vec_normalize(&vec).unwrap();
        
        assert_eq!(normalized.len(), 2);
        assert!((normalized.get(0).unwrap() - 0.6).abs() < 1e-6);
        assert!((normalized.get(1).unwrap() - 0.8).abs() < 1e-6);
        
        // Test magnitude is 1
        let magnitude_squared = normalized.simd_dot(&normalized).unwrap();
        assert!((magnitude_squared - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_matrix_vector_multiplication() {
        let matrix = vec![
            vec![1.0, 2.0].into(),
            vec![3.0, 4.0].into(),
        ];
        let vector: EaVec<f32> = vec![5.0, 6.0].into();
        
        let result = simd_math::matrix_vector_multiply(&matrix, &vector).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result.get(0), Some(&17.0)); // 1*5 + 2*6
        assert_eq!(result.get(1), Some(&39.0)); // 3*5 + 4*6
    }

    #[test]
    fn test_polynomial_evaluation() {
        // Test polynomial: 2x² + 3x + 1
        let coefficients: EaVec<f32> = vec![2.0, 3.0, 1.0].into();
        let x_values: EaVec<f32> = vec![0.0, 1.0, 2.0].into();
        
        let results = simd_math::vec_poly_eval(&coefficients, &x_values);
        assert_eq!(results.len(), 3);
        assert_eq!(results.get(0), Some(&1.0));  // 2*0² + 3*0 + 1 = 1
        assert_eq!(results.get(1), Some(&6.0));  // 2*1² + 3*1 + 1 = 6
        assert_eq!(results.get(2), Some(&15.0)); // 2*2² + 3*2 + 1 = 15
    }

    #[test]
    fn test_scalar_mathematical_functions() {
        assert_eq!(scalar::isqrt(16), 4);
        assert_eq!(scalar::isqrt(15), 3);
        assert_eq!(scalar::isqrt(0), 0);
        
        assert_eq!(scalar::gcd(48, 18), 6);
        assert_eq!(scalar::lcm(12, 8), 24);
        
        assert!(scalar::is_prime(17));
        assert!(!scalar::is_prime(16));
        assert!(scalar::is_prime(2));
        
        assert_eq!(scalar::factorial(5).unwrap(), 120);
        assert_eq!(scalar::factorial(0).unwrap(), 1);
        assert!(scalar::factorial(25).is_err()); // Too large
    }

    #[test]
    fn test_fast_sine_approximation() {
        let angles: EaVec<f32> = vec![0.0, PI/6.0, PI/4.0, PI/2.0].into();
        let fast_sin = simd_math::vec_sin_fast(&angles);
        let exact_sin = simd_math::vec_sin(&angles);
        
        // Fast approximation should be reasonably close to exact values
        for i in 0..angles.len() {
            let fast = fast_sin.get(i).unwrap();
            let exact = exact_sin.get(i).unwrap();
            let error = (fast - exact).abs();
            assert!(error < 0.01, "Fast sin approximation error too large: {}", error);
        }
    }

    #[test]
    fn test_linear_interpolation() {
        let a: EaVec<f32> = vec![0.0, 10.0, 20.0].into();
        let b: EaVec<f32> = vec![100.0, 200.0, 300.0].into();
        
        let result = simd_math::vec_lerp(&a, &b, 0.5).unwrap();
        assert_eq!(result.get(0), Some(&50.0));   // (0 + 100) / 2
        assert_eq!(result.get(1), Some(&105.0));  // (10 + 200) / 2
        assert_eq!(result.get(2), Some(&160.0));  // (20 + 300) / 2
    }

    #[test]
    fn test_math_error_handling() {
        let empty_vec = EaVec::new();
        let result = simd_math::vec_normalize(&empty_vec);
        assert_eq!(result, Err(MathError::EmptyVector));
        
        let zero_vec: EaVec<f32> = vec![0.0, 0.0].into();
        let result = simd_math::vec_normalize(&zero_vec);
        assert_eq!(result, Err(MathError::ZeroMagnitude));
        
        let vec1: EaVec<f32> = vec![1.0, 2.0].into();
        let vec2: EaVec<f32> = vec![1.0, 2.0, 3.0].into();
        let result = simd_math::vec_min(&vec1, &vec2);
        assert_eq!(result, Err(MathError::DimensionMismatch(2, 3)));
    }
}