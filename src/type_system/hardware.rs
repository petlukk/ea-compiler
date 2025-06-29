//! Hardware capability detection and validation for SIMD types.
//! 
//! This module provides functionality to detect available SIMD instruction sets
//! and validate that SIMD vector types are supported on the target platform.

use crate::ast::SIMDVectorType;
use std::collections::HashSet;

/// Represents available SIMD instruction sets on the target hardware.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SIMDFeature {
    // CPU features
    SSE,
    SSE2,
    SSE3,
    SSSE3,
    SSE41,
    SSE42,
    AVX,
    AVX2,
    AVX512F,
    AVX512DQ,
    AVX512CD,
    AVX512BW,
    AVX512VL,
    
    // ARM features
    NEON,
    ArmSve,
    ArmSve2,
    
    // PowerPC features
    ALTIVEC,
    VSX,
    
    // Generic features
    FMA,
    F16C,
}

/// Hardware capability detector for SIMD features.
#[derive(Debug, Clone)]
pub struct HardwareDetector {
    available_features: HashSet<SIMDFeature>,
    target_arch: String,
}

impl HardwareDetector {
    /// Create a new hardware detector with auto-detected capabilities.
    pub fn new() -> Self {
        let mut detector = Self {
            available_features: HashSet::new(),
            target_arch: std::env::consts::ARCH.to_string(),
        };
        
        detector.detect_features();
        detector
    }
    
    /// Create a hardware detector for a specific target architecture.
    pub fn for_target(target_arch: &str) -> Self {
        let mut detector = Self {
            available_features: HashSet::new(),
            target_arch: target_arch.to_string(),
        };
        
        detector.detect_features_for_target();
        detector
    }
    
    /// Check if a SIMD vector type is supported on the current hardware.
    pub fn is_supported(&self, vector_type: &SIMDVectorType) -> bool {
        match vector_type {
            // F32 vectors
            SIMDVectorType::F32x2 => self.has_basic_simd(),
            SIMDVectorType::F32x4 => self.has_feature(&SIMDFeature::SSE) || self.has_feature(&SIMDFeature::NEON),
            SIMDVectorType::F32x8 => self.has_feature(&SIMDFeature::AVX),
            SIMDVectorType::F32x16 => self.has_feature(&SIMDFeature::AVX512F),
            
            // F64 vectors
            SIMDVectorType::F64x2 => self.has_feature(&SIMDFeature::SSE2) || self.has_feature(&SIMDFeature::NEON),
            SIMDVectorType::F64x4 => self.has_feature(&SIMDFeature::AVX),
            SIMDVectorType::F64x8 => self.has_feature(&SIMDFeature::AVX512F),
            
            // I32 vectors
            SIMDVectorType::I32x2 => self.has_basic_simd(),
            SIMDVectorType::I32x4 => self.has_feature(&SIMDFeature::SSE2) || self.has_feature(&SIMDFeature::NEON),
            SIMDVectorType::I32x8 => self.has_feature(&SIMDFeature::AVX2),
            SIMDVectorType::I32x16 => self.has_feature(&SIMDFeature::AVX512F),
            
            // I64 vectors
            SIMDVectorType::I64x2 => self.has_feature(&SIMDFeature::SSE2) || self.has_feature(&SIMDFeature::NEON),
            SIMDVectorType::I64x4 => self.has_feature(&SIMDFeature::AVX2),
            SIMDVectorType::I64x8 => self.has_feature(&SIMDFeature::AVX512F),
            
            // I16 vectors
            SIMDVectorType::I16x4 => self.has_basic_simd(),
            SIMDVectorType::I16x8 => self.has_feature(&SIMDFeature::SSE2) || self.has_feature(&SIMDFeature::NEON),
            SIMDVectorType::I16x16 => self.has_feature(&SIMDFeature::AVX2),
            SIMDVectorType::I16x32 => self.has_feature(&SIMDFeature::AVX512BW),
            
            // I8 vectors
            SIMDVectorType::I8x8 => self.has_basic_simd(),
            SIMDVectorType::I8x16 => self.has_feature(&SIMDFeature::SSE2) || self.has_feature(&SIMDFeature::NEON),
            SIMDVectorType::I8x32 => self.has_feature(&SIMDFeature::AVX2),
            SIMDVectorType::I8x64 => self.has_feature(&SIMDFeature::AVX512BW),
            
            // Unsigned vectors follow same pattern as signed
            SIMDVectorType::U32x4 => self.has_feature(&SIMDFeature::SSE2) || self.has_feature(&SIMDFeature::NEON),
            SIMDVectorType::U32x8 => self.has_feature(&SIMDFeature::AVX2),
            SIMDVectorType::U16x8 => self.has_feature(&SIMDFeature::SSE2) || self.has_feature(&SIMDFeature::NEON),
            SIMDVectorType::U16x16 => self.has_feature(&SIMDFeature::AVX2),
            SIMDVectorType::U8x16 => self.has_feature(&SIMDFeature::SSE2) || self.has_feature(&SIMDFeature::NEON),
            SIMDVectorType::U8x32 => self.has_feature(&SIMDFeature::AVX2),
            
            // Mask types
            SIMDVectorType::Mask8 => self.has_feature(&SIMDFeature::AVX512BW),
            SIMDVectorType::Mask16 => self.has_feature(&SIMDFeature::AVX512BW),
            SIMDVectorType::Mask32 => self.has_feature(&SIMDFeature::AVX512F),
            SIMDVectorType::Mask64 => self.has_feature(&SIMDFeature::AVX512F),
        }
    }
    
    /// Get the required SIMD features for a vector type.
    pub fn required_features(&self, vector_type: &SIMDVectorType) -> Vec<SIMDFeature> {
        match vector_type {
            SIMDVectorType::F32x4 | SIMDVectorType::F64x2 => {
                if self.target_arch == "x86_64" {
                    vec![SIMDFeature::SSE, SIMDFeature::SSE2]
                } else {
                    vec![SIMDFeature::NEON]
                }
            }
            SIMDVectorType::F32x8 | SIMDVectorType::F64x4 => vec![SIMDFeature::AVX],
            SIMDVectorType::F32x16 | SIMDVectorType::F64x8 => vec![SIMDFeature::AVX512F],
            SIMDVectorType::I32x8 | SIMDVectorType::I64x4 => vec![SIMDFeature::AVX2],
            SIMDVectorType::I32x16 | SIMDVectorType::I64x8 => vec![SIMDFeature::AVX512F],
            SIMDVectorType::I16x32 | SIMDVectorType::I8x64 => vec![SIMDFeature::AVX512BW],
            _ => vec![], // Basic types don't require specific features
        }
    }
    
    /// Get a list of all available SIMD features.
    pub fn available_features(&self) -> &HashSet<SIMDFeature> {
        &self.available_features
    }
    
    /// Get the target architecture.
    pub fn target_arch(&self) -> &str {
        &self.target_arch
    }
    
    /// Get hardware-specific optimization recommendations.
    pub fn optimization_recommendations(&self, vector_type: &SIMDVectorType) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // Alignment recommendations
        if matches!(vector_type, 
            SIMDVectorType::F32x4 | SIMDVectorType::F64x2 | 
            SIMDVectorType::I32x4 | SIMDVectorType::I64x2
        ) {
            recommendations.push("Use 16-byte aligned memory access for optimal performance".to_string());
        } else if matches!(vector_type,
            SIMDVectorType::F32x8 | SIMDVectorType::F64x4 | 
            SIMDVectorType::I32x8 | SIMDVectorType::I64x4
        ) {
            recommendations.push("Use 32-byte aligned memory access for optimal performance".to_string());
        } else if vector_type.width() >= 16 {
            recommendations.push("Use 64-byte aligned memory access for optimal performance".to_string());
        }
        
        // Architecture-specific recommendations
        if self.target_arch == "x86_64" && self.has_feature(&SIMDFeature::AVX512F) {
            recommendations.push("Consider using AVX-512 masked operations for conditional processing".to_string());
        }
        
        if self.target_arch.starts_with("arm") && self.has_feature(&SIMDFeature::NEON) {
            recommendations.push("NEON operations are most efficient with interleaved data layouts".to_string());
        }
        
        recommendations
    }
    
    // Private methods
    
    fn detect_features(&mut self) {
        match self.target_arch.as_str() {
            "x86_64" | "x86" => self.detect_x86_features(),
            arch if arch.starts_with("arm") => self.detect_arm_features(),
            "powerpc64" | "powerpc" => self.detect_powerpc_features(),
            _ => self.detect_generic_features(),
        }
    }
    
    fn detect_features_for_target(&mut self) {
        // For cross-compilation, we use conservative feature detection
        match self.target_arch.as_str() {
            "x86_64" => {
                // Assume baseline x86_64 features
                self.available_features.insert(SIMDFeature::SSE);
                self.available_features.insert(SIMDFeature::SSE2);
                self.available_features.insert(SIMDFeature::SSE3);
            }
            arch if arch.starts_with("arm") => {
                // Assume NEON is available on modern ARM
                self.available_features.insert(SIMDFeature::NEON);
            }
            "powerpc64" => {
                // Assume AltiVec is available
                self.available_features.insert(SIMDFeature::ALTIVEC);
            }
            _ => {}
        }
    }
    
    fn detect_x86_features(&mut self) {
        // In a real implementation, this would use CPUID detection
        // For now, we'll use compile-time feature detection
        
        #[cfg(target_feature = "sse")]
        self.available_features.insert(SIMDFeature::SSE);
        
        #[cfg(target_feature = "sse2")]
        self.available_features.insert(SIMDFeature::SSE2);
        
        #[cfg(target_feature = "sse3")]
        self.available_features.insert(SIMDFeature::SSE3);
        
        #[cfg(target_feature = "ssse3")]
        self.available_features.insert(SIMDFeature::SSSE3);
        
        #[cfg(target_feature = "sse4.1")]
        self.available_features.insert(SIMDFeature::SSE41);
        
        #[cfg(target_feature = "sse4.2")]
        self.available_features.insert(SIMDFeature::SSE42);
        
        #[cfg(target_feature = "avx")]
        self.available_features.insert(SIMDFeature::AVX);
        
        #[cfg(target_feature = "avx2")]
        self.available_features.insert(SIMDFeature::AVX2);
        
        #[cfg(target_feature = "avx512f")]
        self.available_features.insert(SIMDFeature::AVX512F);
        
        #[cfg(target_feature = "fma")]
        self.available_features.insert(SIMDFeature::FMA);
        
        #[cfg(target_feature = "f16c")]
        self.available_features.insert(SIMDFeature::F16C);
    }
    
    fn detect_arm_features(&mut self) {
        #[cfg(target_feature = "neon")]
        self.available_features.insert(SIMDFeature::NEON);
        
        // SVE detection would require runtime checks
        // For now, assume it's not available unless explicitly enabled
    }
    
    fn detect_powerpc_features(&mut self) {
        #[cfg(target_feature = "altivec")]
        self.available_features.insert(SIMDFeature::ALTIVEC);
        
        #[cfg(target_feature = "vsx")]
        self.available_features.insert(SIMDFeature::VSX);
    }
    
    fn detect_generic_features(&mut self) {
        // Generic targets don't have specific SIMD features
        // We can only provide software fallbacks
    }
    
    fn has_feature(&self, feature: &SIMDFeature) -> bool {
        self.available_features.contains(feature)
    }
    
    fn has_basic_simd(&self) -> bool {
        match self.target_arch.as_str() {
            "x86_64" | "x86" => self.has_feature(&SIMDFeature::SSE),
            arch if arch.starts_with("arm") => self.has_feature(&SIMDFeature::NEON),
            "powerpc64" | "powerpc" => self.has_feature(&SIMDFeature::ALTIVEC),
            _ => false,
        }
    }
}

impl Default for HardwareDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// SIMD target feature attributes for functions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TargetFeatures {
    pub required: Vec<SIMDFeature>,
    pub optional: Vec<SIMDFeature>,
}

impl TargetFeatures {
    /// Create empty target features.
    pub fn new() -> Self {
        Self {
            required: Vec::new(),
            optional: Vec::new(),
        }
    }
    
    /// Add a required feature.
    pub fn require(mut self, feature: SIMDFeature) -> Self {
        self.required.push(feature);
        self
    }
    
    /// Add an optional feature.
    pub fn prefer(mut self, feature: SIMDFeature) -> Self {
        self.optional.push(feature);
        self
    }
    
    /// Check if all required features are available.
    pub fn is_supported_by(&self, detector: &HardwareDetector) -> bool {
        self.required.iter().all(|feature| detector.has_feature(feature))
    }
    
    /// Get features that are missing from the hardware.
    pub fn missing_features(&self, detector: &HardwareDetector) -> Vec<&SIMDFeature> {
        self.required.iter()
            .filter(|feature| !detector.has_feature(feature))
            .collect()
    }
}

impl Default for TargetFeatures {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hardware_detector_creation() {
        let detector = HardwareDetector::new();
        assert!(!detector.target_arch().is_empty());
    }
    
    #[test]
    fn test_basic_vector_support() {
        let detector = HardwareDetector::for_target("x86_64");
        
        // F32x4 should be supported on x86_64 (SSE baseline)
        assert!(detector.is_supported(&SIMDVectorType::F32x4));
        
        // F32x16 requires AVX512, which we don't assume
        assert!(!detector.is_supported(&SIMDVectorType::F32x16));
    }
    
    #[test]
    fn test_target_features() {
        let features = TargetFeatures::new()
            .require(SIMDFeature::AVX2)
            .prefer(SIMDFeature::FMA);
        
        let detector = HardwareDetector::for_target("x86_64");
        
        // Should not be supported without AVX2
        assert!(!features.is_supported_by(&detector));
    }
    
    #[test]
    fn test_required_features() {
        let detector = HardwareDetector::for_target("x86_64");
        let features = detector.required_features(&SIMDVectorType::F32x8);
        
        assert!(features.contains(&SIMDFeature::AVX));
    }
}