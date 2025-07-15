// SIMD benchmark - Rust version using portable_simd (if available) or manual SIMD
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

fn simd_operations() {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        let vec1 = _mm_set_ps(4.0, 3.0, 2.0, 1.0);
        let vec2 = _mm_set_ps(8.0, 7.0, 6.0, 5.0);
        
        // Perform 100000 SIMD operations
        for _ in 0..100000 {
            let sum = _mm_add_ps(vec1, vec2);
            let product = _mm_mul_ps(vec1, vec2);
            let diff = _mm_sub_ps(vec1, vec2);
        }
    }
    
    #[cfg(not(target_arch = "x86_64"))]
    {
        // Fallback for non-x86_64 architectures
        for _ in 0..100000 {
            let sum1 = 1.0f32 + 5.0f32;
            let sum2 = 2.0f32 + 6.0f32;
            let sum3 = 3.0f32 + 7.0f32;
            let sum4 = 4.0f32 + 8.0f32;
        }
    }
    
    println!("SIMD operations completed");
}

fn main() {
    simd_operations();
}