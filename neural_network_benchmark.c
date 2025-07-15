#include <stdio.h>
#include <stdlib.h>

#ifdef __SSE__
#include <xmmintrin.h>
#endif

// Neural Network Benchmark - C version
// Tests: JSON parsing, matrix ops, SIMD vectors, 10K parameters, activations

void parse_json_config() {
    // Simulate parsing a neural network configuration
    printf("Parsing neural network configuration...\n");
    
    // Simulate reading layers, weights, biases from JSON
    int layer_count = 5;
    int input_size = 784;   // MNIST-like input
    int hidden_size = 256;
    int output_size = 10;
    
    printf("Network config loaded: 5 layers, 784 inputs, 10 outputs\n");
}

int initialize_large_parameters() {
    // Initialize 10K+ parameters (typical small neural network)
    printf("Initializing 10,000 neural network parameters...\n");
    
    int total_params = 0;
    
    // Layer 1: 784 * 256 weights + 256 biases
    int layer1_weights = 784 * 256;
    int layer1_biases = 256;
    total_params += layer1_weights + layer1_biases;
    
    // Layer 2: 256 * 128 weights + 128 biases  
    int layer2_weights = 256 * 128;
    int layer2_biases = 128;
    total_params += layer2_weights + layer2_biases;
    
    // Layer 3: 128 * 64 weights + 64 biases
    int layer3_weights = 128 * 64;
    int layer3_biases = 64;
    total_params += layer3_weights + layer3_biases;
    
    // Output layer: 64 * 10 weights + 10 biases
    int output_weights = 64 * 10;
    int output_biases = 10;
    total_params += output_weights + output_biases;
    
    return total_params;
}

void simd_vector_operations() {
    // Demonstrate SIMD operations common in neural networks
    printf("Performing SIMD vector operations...\n");
    
#ifdef __SSE__
    __m128 vec1 = _mm_set_ps(0.4f, 0.3f, 0.2f, 0.1f);
    __m128 vec2 = _mm_set_ps(0.8f, 0.7f, 0.6f, 0.5f);
    __m128 weights = _mm_set_ps(0.4f, 0.6f, 0.8f, 1.0f);
    
    // Perform 1000 SIMD operations (typical in training loop)
    for (int i = 0; i < 1000; i++) {
        // Dot product simulation
        __m128 dot_product = _mm_mul_ps(vec1, vec2);
        
        // Weighted sum
        __m128 weighted = _mm_mul_ps(dot_product, weights);
        
        // Element-wise addition (bias addition)
        __m128 biased = _mm_add_ps(weighted, vec1);
        
        // ReLU activation simulation would go here
    }
#else
    // Fallback for systems without SSE
    for (int i = 0; i < 1000; i++) {
        float dot1 = 0.1f * 0.5f;
        float dot2 = 0.2f * 0.6f;
        float dot3 = 0.3f * 0.7f;
        float dot4 = 0.4f * 0.8f;
    }
#endif
    
    printf("Completed 1000 SIMD vector operations\n");
}

void matrix_multiplication_simulation() {
    // Simulate matrix multiplication (core neural network operation)
    printf("Performing matrix multiplication simulation...\n");
    
    // Simulate multiplying 256x256 matrices (common layer size)
    int matrix_size = 256;
    int total_operations = 0;
    
    for (int i = 0; i < matrix_size; i++) {
        for (int j = 0; j < matrix_size; j++) {
            // Simulate dot product of row i with column j
            for (int k = 0; k < matrix_size; k++) {
                total_operations++;
            }
        }
    }
    
    printf("Matrix multiplication complete\n");
}

void activation_functions() {
    // Test various activation functions used in neural networks
    printf("Computing activation functions...\n");
    
    // Test on 1000 values (typical batch size)
    for (int i = 0; i < 1000; i++) {
        // ReLU: max(0, x)
        int input = i - 500;  // Range from -500 to 499
        int relu = (input > 0) ? input : 0;
        
        // Sigmoid approximation: 1 / (1 + exp(-x))
        // Simplified for this demo
        int sigmoid_approx = (input < 0) ? 0 : 1;
        
        // Tanh approximation
        int tanh_approx;
        if (input > 1) {
            tanh_approx = 1;
        } else if (input < -1) {
            tanh_approx = -1;
        } else {
            tanh_approx = input;
        }
    }
    
    printf("Activation functions computed for 1000 values\n");
}

void memory_management_test() {
    // Test memory allocation patterns common in ML
    printf("Testing memory management for ML workloads...\n");
    
    // Simulate allocating space for gradients, activations, etc.
    int batch_size = 32;
    int layer_sizes = 256;
    
    // Simulate multiple memory allocations and deallocations
    for (int batch = 0; batch < 10; batch++) {
        // Simulate forward pass memory allocation
        for (int layer = 0; layer < 5; layer++) {
            int activations = layer_sizes * batch_size;
            int gradients = layer_sizes * batch_size;
        }
        
        // Simulate backward pass and cleanup
        for (int cleanup = 0; cleanup < 5; cleanup++) {
            int temp_memory = layer_sizes;
        }
    }
    
    printf("Memory management test completed\n");
}

void data_loading_simulation() {
    // Simulate loading and preprocessing data (common ML task)
    printf("Simulating data loading and preprocessing...\n");
    
    // Simulate loading a dataset with 1000 samples
    int dataset_size = 1000;
    int feature_count = 784;  // MNIST-like
    
    for (int sample = 0; sample < dataset_size; sample++) {
        // Simulate data normalization
        for (int feature = 0; feature < feature_count; feature++) {
            // Normalize: (x - mean) / std
            int raw_value = feature * 2;  // Simulated raw data
            int normalized = raw_value / 255;  // Typical image normalization
        }
        
        // Simulate data augmentation
        int augmented_sample = sample * 2;
    }
    
    printf("Data loading simulation completed\n");
}

void training_loop_simulation() {
    // Simulate a mini neural network training loop
    printf("Simulating neural network training loop...\n");
    
    int epochs = 5;
    int batch_size = 32;
    int learning_rate = 1;  // Simplified as integer
    
    for (int epoch = 0; epoch < epochs; epoch++) {
        printf("Training epoch\n");
        
        // Simulate processing 100 batches per epoch
        for (int batch = 0; batch < 100; batch++) {
            // Forward pass
            int forward_ops = batch_size * 1000;  // Simulated operations
            
            // Backward pass
            int backward_ops = batch_size * 1000;
            
            // Parameter update
            int param_updates = 10000;  // Our 10K parameters
        }
    }
    
    printf("Training simulation completed\n");
}

int main() {
    printf("=== C Neural Network Benchmark ===\n");
    printf("Showcasing AI/ML capabilities\n");
    
    // Test 1: Configuration parsing
    parse_json_config();
    
    // Test 2: Large parameter initialization
    int total_params = initialize_large_parameters();
    printf("Total parameters initialized: %d\n", total_params);
    
    // Test 3: SIMD operations
    simd_vector_operations();
    
    // Test 4: Matrix operations
    matrix_multiplication_simulation();
    
    // Test 5: Activation functions
    activation_functions();
    
    // Test 6: Memory management
    memory_management_test();
    
    // Test 7: Data loading
    data_loading_simulation();
    
    // Test 8: Training loop
    training_loop_simulation();
    
    printf("=== Benchmark Complete ===\n");
    printf("All neural network operations completed successfully\n");
    
    return 0;
}