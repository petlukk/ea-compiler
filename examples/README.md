# examples/README.md

# Eä Language Examples

This directory contains example programs demonstrating various features of the Eä programming language.

## Basic Examples

### hello_world.ea
```eä
func main() -> () {
    print("Hello, World!");
    return;
}
```

### arithmetic.ea
```eä
func add(a: i32, b: i32) -> i32 {
    return a + b;
}

func multiply(a: i32, b: i32) -> i32 {
    return a * b;
}

func main() -> () {
    let x = 10;
    let y = 20;
    
    let sum = add(x, y);
    let product = multiply(x, y);
    
    print("Calculations complete");
    return;
}
```

## Control Flow Examples

### fibonacci.ea
```eä
func fibonacci(n: i32) -> i32 {
    if (n <= 1) {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

func main() -> () {
    let result = fibonacci(10);
    print("Fibonacci calculation complete");
    return;
}
```

### factorial.ea
```eä
func factorial(n: i32) -> i32 {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}

func main() -> () {
    for (let i: i32 = 1; i <= 10; i += 1) {
        let fact = factorial(i);
        // Process factorial result
    }
    return;
}
```

### loops.ea
```eä
func test_loops() -> () {
    // For loop example
    let sum = 0;
    for (let i: i32 = 1; i <= 100; i += 1) {
        sum += i;
    }
    
    // While loop example
    let counter = 0;
    while (counter < 10) {
        counter += 1;
    }
    
    // Nested loops
    for (let i: i32 = 1; i <= 10; i += 1) {
        for (let j: i32 = 1; j <= 10; j += 1) {
            let product = i * j;
        }
    }
    
    return;
}

func main() -> () {
    test_loops();
    return;
}
```

## Type System Examples

### type_inference.ea
```eä
func demonstrate_inference() -> () {
    // Type inference for literals
    let integer_var = 42;          // Inferred as i64
    let float_var = 3.14;          // Inferred as f64
    let boolean_var = true;        // Inferred as bool
    let string_var = "hello";      // Inferred as string
    
    // Type inference for expressions
    let arithmetic_result = integer_var + 10;
    let comparison_result = float_var > 2.0;
    let logical_result = boolean_var && true;
    
    return;
}

func explicit_types() -> () {
    // Explicit type annotations
    let x: i32 = 100;
    let y: f64 = 3.14159;
    let z: bool = false;
    let name: string = "Eä Language";
    
    // Mutable variables
    let mut counter: i32 = 0;
    counter += 1;
    counter *= 2;
    
    return;
}

func main() -> () {
    demonstrate_inference();
    explicit_types();
    return;
}
```

### error_handling.ea
```eä
// Examples of programs that should produce type errors
// (These are for testing - they won't compile)

/*
func type_mismatch_return() -> i32 {
    return "hello";  // Error: string != i32
}

func type_mismatch_assignment() -> () {
    let x: i32 = true;  // Error: bool != i32
    return;
}

func invalid_condition() -> () {
    if (42) {  // Error: i32 != bool
        return;
    }
    return;
}

func undefined_variable() -> () {
    let x = unknown_var + 1;  // Error: undefined variable
    return;
}
*/
```

## Complex Examples

### prime_numbers.ea
```eä
func is_prime(n: i32) -> bool {
    if (n <= 1) {
        return false;
    }
    
    if (n <= 3) {
        return true;
    }
    
    if (n % 2 == 0 || n % 3 == 0) {
        return false;
    }
    
    let i = 5;
    while (i * i <= n) {
        if (n % i == 0 || n % (i + 2) == 0) {
            return false;
        }
        i += 6;
    }
    
    return true;
}

func count_primes(limit: i32) -> i32 {
    let count = 0;
    
    for (let i: i32 = 2; i <= limit; i += 1) {
        if (is_prime(i)) {
            count += 1;
        }
    }
    
    return count;
}

func main() -> () {
    let prime_count = count_primes(1000);
    print("Prime counting complete");
    return;
}
```

### mathematical_functions.ea
```eä
func power(base: i32, exponent: i32) -> i32 {
    if (exponent == 0) {
        return 1;
    }
    
    let result = 1;
    for (let i: i32 = 0; i < exponent; i += 1) {
        result *= base;
    }
    
    return result;
}

func gcd(a: i32, b: i32) -> i32 {
    while (b != 0) {
        let temp = b;
        b = a % b;
        a = temp;
    }
    return a;
}

func lcm(a: i32, b: i32) -> i32 {
    return (a * b) / gcd(a, b);
}

func sum_of_digits(n: i32) -> i32 {
    let sum = 0;
    let temp = n;
    
    while (temp > 0) {
        sum += temp % 10;
        temp /= 10;
    }
    
    return sum;
}

func main() -> () {
    let power_result = power(2, 10);
    let gcd_result = gcd(48, 18);
    let lcm_result = lcm(12, 8);
    let digit_sum = sum_of_digits(12345);
    
    print("Mathematical calculations complete");
    return;
}
```

### sorting_algorithms.ea
```eä
// Note: Array indexing syntax shown here is aspirational
// Current implementation doesn't support array indexing yet

/*
func bubble_sort(arr: []i32, size: i32) -> () {
    for (let i: i32 = 0; i < size - 1; i += 1) {
        for (let j: i32 = 0; j < size - i - 1; j += 1) {
            if (arr[j] > arr[j + 1]) {
                // Swap elements
                let temp = arr[j];
                arr[j] = arr[j + 1];
                arr[j + 1] = temp;
            }
        }
    }
    return;
}

func binary_search(arr: []i32, target: i32, low: i32, high: i32) -> i32 {
    if (high >= low) {
        let mid = low + (high - low) / 2;
        
        if (arr[mid] == target) {
            return mid;
        }
        
        if (arr[mid] > target) {
            return binary_search(arr, target, low, mid - 1);
        }
        
        return binary_search(arr, target, mid + 1, high);
    }
    
    return -1;  // Not found
}
*/

// Current working version without arrays
func bubble_sort_demo() -> () {
    // Demonstrate sorting logic without arrays
    let a = 64;
    let b = 34;
    let c = 25;
    
    // Sort three numbers
    if (a > b) {
        let temp = a;
        a = b;
        b = temp;
    }
    
    if (b > c) {
        let temp = b;
        b = c;
        c = temp;
    }
    
    if (a > b) {
        let temp = a;
        a = b;
        b = temp;
    }
    
    print("Sorting demonstration complete");
    return;
}

func main() -> () {
    bubble_sort_demo();
    return;
}
```

## Advanced Examples (Future Features)

These examples demonstrate syntax for features that will be implemented in Sprint 2:

### simd_operations.ea (Future)
```eä
// SIMD operations - Sprint 2 target
/*
use eä::simd::{f32x8, i32x16}

func vector_add(a: []f32, b: []f32) -> []f32 {
    parallel_map_zip(a, b) vectorize { |x, y| => x + y }
}

func process_audio(samples: []f32) -> []f32 {
    parallel_map(samples) vectorize { |sample| =>
        sample * 0.5 + 0.1
    }
}
*/
```

### memory_regions.ea (Future)
```eä
// Memory regions - Sprint 2 target
/*
func process_large_dataset() {
    mem_region working_space(size: 1GB, alignment: 64) {
        let data = Array::<f64>::with_capacity_in_region(1_000_000);
        // Process data...
    } // Automatic cleanup
}
*/
```

### optimization_attributes.ea (Future)
```eä
// Adaptive optimization - Sprint 2 target
/*
func fibonacci_optimized(n: u64) -> u64 
    @optimize(cache: true, compile_time: up_to(100))
{
    if n <= 1 { return n }
    return fibonacci_optimized(n - 1) + fibonacci_optimized(n - 2)
}
*/
```

## Running the Examples

To compile and test these examples:

```bash
# Run basic compilation test
cargo test test_showcase_program

# Run with LLVM code generation
cargo test test_llvm_integration --features=llvm

# Run benchmarks
cargo bench

# Test individual examples
ea examples/fibonacci.ea
ea examples/prime_numbers.ea
```

## Example Output

When running the fibonacci example:
```
$ ea examples/fibonacci.ea
Parsing... ✓
Type checking... ✓
Code generation... ✓
Fibonacci calculation complete
```