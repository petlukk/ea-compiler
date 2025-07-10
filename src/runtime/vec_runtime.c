//! Runtime support for Vec operations
//! This provides real memory management and operations for Eä Vec type

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdint.h>

// Vec structure layout (matches LLVM IR expectations)
typedef struct {
    void* data;      // Pointer to data array
    size_t len;      // Number of elements
    size_t capacity; // Allocated capacity
} Vec;

// Create a new empty Vec
Vec* vec_new() {
    Vec* vec = (Vec*)malloc(sizeof(Vec));
    if (!vec) return NULL;
    
    vec->data = NULL;
    vec->len = 0;
    vec->capacity = 0;
    return vec;
}

// Create Vec with specific capacity
Vec* vec_with_capacity(size_t capacity) {
    Vec* vec = vec_new();
    if (!vec) return NULL;
    
    if (capacity > 0) {
        vec->data = malloc(capacity * sizeof(int32_t)); // Assuming i32 for now
        if (!vec->data) {
            free(vec);
            return NULL;
        }
        vec->capacity = capacity;
    }
    
    return vec;
}

// Grow Vec capacity
int vec_grow(Vec* vec, size_t new_capacity) {
    if (!vec || new_capacity <= vec->capacity) return 0;
    
    void* new_data = realloc(vec->data, new_capacity * sizeof(int32_t));
    if (!new_data) return 0; // Allocation failed
    
    vec->data = new_data;
    vec->capacity = new_capacity;
    return 1; // Success
}

// Push element to Vec
int vec_push(Vec* vec, int32_t item) {
    if (!vec) return 0;
    
    // Check if we need to grow
    if (vec->len >= vec->capacity) {
        size_t new_capacity = vec->capacity == 0 ? 4 : vec->capacity * 2;
        if (!vec_grow(vec, new_capacity)) {
            return 0; // Failed to grow
        }
    }
    
    // Add the item
    ((int32_t*)vec->data)[vec->len] = item;
    vec->len++;
    return 1; // Success
}

// Pop element from Vec
int vec_pop(Vec* vec, int32_t* out_item) {
    if (!vec || vec->len == 0) return 0;
    
    vec->len--;
    if (out_item) {
        *out_item = ((int32_t*)vec->data)[vec->len];
    }
    return 1; // Success
}

// Get element at index (returns pointer to element or NULL)
int32_t* vec_get(Vec* vec, size_t index) {
    if (!vec || index >= vec->len) return NULL;
    
    return &((int32_t*)vec->data)[index];
}

// Get length
size_t vec_len(Vec* vec) {
    return vec ? vec->len : 0;
}

// Check if empty
int vec_is_empty(Vec* vec) {
    return vec ? (vec->len == 0 ? 1 : 0) : 1;
}

// Get capacity
size_t vec_capacity(Vec* vec) {
    return vec ? vec->capacity : 0;
}

// Clear all elements
void vec_clear(Vec* vec) {
    if (vec) {
        vec->len = 0;
    }
}

// Free Vec memory
void vec_free(Vec* vec) {
    if (vec) {
        if (vec->data) {
            free(vec->data);
        }
        free(vec);
    }
}

// F32 Vec operations for SIMD
typedef struct {
    float* data;
    size_t len;
    size_t capacity;
} VecF32;

VecF32* vec_f32_new() {
    VecF32* vec = (VecF32*)malloc(sizeof(VecF32));
    if (!vec) return NULL;
    
    vec->data = NULL;
    vec->len = 0;
    vec->capacity = 0;
    return vec;
}

int vec_f32_push(VecF32* vec, float item) {
    if (!vec) return 0;
    
    if (vec->len >= vec->capacity) {
        size_t new_capacity = vec->capacity == 0 ? 4 : vec->capacity * 2;
        float* new_data = (float*)realloc(vec->data, new_capacity * sizeof(float));
        if (!new_data) return 0;
        
        vec->data = new_data;
        vec->capacity = new_capacity;
    }
    
    vec->data[vec->len] = item;
    vec->len++;
    return 1;
}

float* vec_f32_get(VecF32* vec, size_t index) {
    if (!vec || index >= vec->len) return NULL;
    return &vec->data[index];
}

size_t vec_f32_len(VecF32* vec) {
    return vec ? vec->len : 0;
}

// SIMD operations
VecF32* vec_f32_simd_add(VecF32* a, VecF32* b) {
    if (!a || !b || a->len != b->len) return NULL;
    
    VecF32* result = vec_f32_new();
    if (!result) return NULL;
    
    // Allocate capacity
    if (a->len > 0) {
        result->data = (float*)malloc(a->len * sizeof(float));
        if (!result->data) {
            free(result);
            return NULL;
        }
        result->capacity = a->len;
        result->len = a->len;
        
        // Perform addition
        for (size_t i = 0; i < a->len; i++) {
            result->data[i] = a->data[i] + b->data[i];
        }
    }
    
    return result;
}

float vec_f32_simd_sum(VecF32* vec) {
    if (!vec || vec->len == 0) return 0.0f;
    
    float sum = 0.0f;
    for (size_t i = 0; i < vec->len; i++) {
        sum += vec->data[i];
    }
    return sum;
}

float vec_f32_simd_dot(VecF32* a, VecF32* b) {
    if (!a || !b || a->len != b->len) return 0.0f;
    
    float dot = 0.0f;
    for (size_t i = 0; i < a->len; i++) {
        dot += a->data[i] * b->data[i];
    }
    return dot;
}

void vec_f32_free(VecF32* vec) {
    if (vec) {
        if (vec->data) {
            free(vec->data);
        }
        free(vec);
    }
}

// Test function to verify runtime works
int vec_runtime_test() {
    // Test basic Vec operations
    Vec* vec = vec_new();
    if (!vec) return 0;
    
    // Test push
    for (int i = 0; i < 10; i++) {
        if (!vec_push(vec, i)) {
            vec_free(vec);
            return 0;
        }
    }
    
    // Test length
    if (vec_len(vec) != 10) {
        vec_free(vec);
        return 0;
    }
    
    // Test get
    for (int i = 0; i < 10; i++) {
        int32_t* val = vec_get(vec, i);
        if (!val || *val != i) {
            vec_free(vec);
            return 0;
        }
    }
    
    // Test pop
    int32_t popped;
    if (!vec_pop(vec, &popped) || popped != 9) {
        vec_free(vec);
        return 0;
    }
    
    if (vec_len(vec) != 9) {
        vec_free(vec);
        return 0;
    }
    
    vec_free(vec);
    return 1; // Success
}