#include <stdio.h>
#ifdef __SSE__
#include <xmmintrin.h>
#endif

// SIMD benchmark - C version
void simd_operations() {
#ifdef __SSE__
    __m128 vec1 = _mm_set_ps(4.0f, 3.0f, 2.0f, 1.0f);
    __m128 vec2 = _mm_set_ps(8.0f, 7.0f, 6.0f, 5.0f);
    
    // Perform 100000 SIMD operations
    for (int i = 0; i < 100000; i++) {
        __m128 sum = _mm_add_ps(vec1, vec2);
        __m128 product = _mm_mul_ps(vec1, vec2);
        __m128 diff = _mm_sub_ps(vec1, vec2);
    }
#else
    // Fallback for systems without SSE
    for (int i = 0; i < 100000; i++) {
        float sum1 = 1.0f + 5.0f;
        float sum2 = 2.0f + 6.0f;
        float sum3 = 3.0f + 7.0f;
        float sum4 = 4.0f + 8.0f;
    }
#endif
    
    printf("SIMD operations completed\n");
}

int main() {
    simd_operations();
    return 0;
}