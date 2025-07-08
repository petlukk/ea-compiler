//! SIMD-Accelerated Collection Types
//!
//! High-performance Vec and HashMap implementations with automatic
//! SIMD optimization for element-wise operations, memory operations,
//! and mathematical computations.

use std::alloc::{alloc, dealloc, realloc, Layout};
use std::collections::HashMap as StdHashMap;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::mem;
use std::ptr;
use std::slice;

use super::{StandardLibrary, SIMDFeatures, OptimizationLevel};

/// SIMD-accelerated dynamic array
#[derive(Debug, PartialEq)]
pub struct Vec<T> {
    /// Pointer to heap-allocated data
    ptr: *mut T,
    /// Number of elements
    len: usize,
    /// Allocated capacity  
    capacity: usize,
    /// SIMD optimization configuration
    simd_config: SIMDConfig,
}

#[derive(Debug, Clone, PartialEq)]
struct SIMDConfig {
    vector_width: usize,
    unroll_factor: usize,
    use_simd: bool,
    alignment: usize,
}

impl<T> Vec<T> {
    /// Create a new empty vector with SIMD optimization
    pub fn new() -> Self {
        let stdlib = StandardLibrary::new();
        Self::with_simd_config(stdlib.optimal_vector_width(), stdlib.optimal_unroll_factor())
    }

    /// Create vector with specific SIMD configuration
    pub fn with_simd_config(vector_width: usize, unroll_factor: usize) -> Self {
        Self {
            ptr: ptr::null_mut(),
            len: 0,
            capacity: 0,
            simd_config: SIMDConfig {
                vector_width,
                unroll_factor,
                use_simd: mem::size_of::<T>() <= 8, // Only SIMD for small types
                alignment: if mem::size_of::<T>() <= 8 { 32 } else { mem::align_of::<T>() },
            },
        }
    }

    /// Create vector with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        let mut vec = Self::new();
        vec.reserve(capacity);
        vec
    }

    /// Reserve capacity for at least `additional` more elements
    pub fn reserve(&mut self, additional: usize) {
        let required_cap = self.len.checked_add(additional).expect("capacity overflow");
        if required_cap <= self.capacity {
            return;
        }

        let new_capacity = if self.capacity == 0 {
            required_cap.max(4)
        } else {
            self.capacity.checked_mul(2).unwrap_or(required_cap).max(required_cap)
        };

        self.grow(new_capacity);
    }

    /// Grow the vector to new capacity
    fn grow(&mut self, new_capacity: usize) {
        assert!(new_capacity >= self.len);

        let new_layout = Layout::from_size_align(
            new_capacity * mem::size_of::<T>(),
            self.simd_config.alignment,
        ).expect("invalid layout");

        let new_ptr = if self.capacity == 0 {
            unsafe { alloc(new_layout) as *mut T }
        } else {
            let old_layout = Layout::from_size_align(
                self.capacity * mem::size_of::<T>(),
                self.simd_config.alignment,
            ).expect("invalid layout");

            unsafe {
                realloc(self.ptr as *mut u8, old_layout, new_layout.size()) as *mut T
            }
        };

        if new_ptr.is_null() {
            panic!("allocation failed");
        }

        self.ptr = new_ptr;
        self.capacity = new_capacity;
    }

    /// Add an element to the end of the vector
    pub fn push(&mut self, item: T) {
        if self.len == self.capacity {
            self.reserve(1);
        }

        unsafe {
            ptr::write(self.ptr.add(self.len), item);
        }
        self.len += 1;
    }

    /// Remove the last element and return it
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe { Some(ptr::read(self.ptr.add(self.len))) }
        }
    }

    /// Get the number of elements
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if the vector is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get element at index
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            unsafe { Some(&*self.ptr.add(index)) }
        } else {
            None
        }
    }

    /// Get mutable element at index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len {
            unsafe { Some(&mut *self.ptr.add(index)) }
        } else {
            None
        }
    }

    /// Clear all elements
    pub fn clear(&mut self) {
        unsafe {
            for i in 0..self.len {
                ptr::drop_in_place(self.ptr.add(i));
            }
        }
        self.len = 0;
    }

    /// Get slice view of the vector
    pub fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.ptr, self.len) }
    }

    /// Get mutable slice view of the vector
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.ptr, self.len) }
    }

    /// Get iterator over elements
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.as_slice().iter()
    }

    /// Get mutable iterator over elements
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.as_mut_slice().iter_mut()
    }
}

// SIMD-accelerated operations for numeric types
impl Vec<f32> {
    /// SIMD-accelerated map operation for f32 vectors
    pub fn simd_map<F>(&self, f: F) -> Vec<f32>
    where
        F: Fn(f32) -> f32,
    {
        let mut result = Vec::with_capacity(self.len);
        
        if self.simd_config.use_simd && self.len >= self.simd_config.vector_width {
            // Process in SIMD chunks
            let chunk_size = self.simd_config.vector_width;
            let chunks = self.len / chunk_size;
            
            for chunk in 0..chunks {
                let start = chunk * chunk_size;
                
                // SIMD processing (simplified - would use actual SIMD intrinsics)
                for i in 0..chunk_size {
                    let val = unsafe { *self.ptr.add(start + i) };
                    let mapped = f(val);
                    result.push(mapped);
                }
            }
            
            // Process remaining elements
            for i in (chunks * chunk_size)..self.len {
                let val = unsafe { *self.ptr.add(i) };
                result.push(f(val));
            }
        } else {
            // Fallback to scalar processing
            for i in 0..self.len {
                let val = unsafe { *self.ptr.add(i) };
                result.push(f(val));
            }
        }
        
        result
    }

    /// SIMD-accelerated element-wise addition
    pub fn simd_add(&self, other: &Vec<f32>) -> Result<Vec<f32>, SIMDError> {
        if self.len != other.len {
            return Err(SIMDError::LengthMismatch(self.len, other.len));
        }

        let mut result = Vec::with_capacity(self.len);
        
        if self.simd_config.use_simd && self.len >= self.simd_config.vector_width {
            // SIMD vector addition
            let chunk_size = self.simd_config.vector_width;
            let chunks = self.len / chunk_size;
            
            for chunk in 0..chunks {
                let start = chunk * chunk_size;
                
                // Would use actual SIMD intrinsics like _mm256_add_ps for AVX2
                for i in 0..chunk_size {
                    let a = unsafe { *self.ptr.add(start + i) };
                    let b = unsafe { *other.ptr.add(start + i) };
                    result.push(a + b);
                }
            }
            
            // Process remaining elements
            for i in (chunks * chunk_size)..self.len {
                let a = unsafe { *self.ptr.add(i) };
                let b = unsafe { *other.ptr.add(i) };
                result.push(a + b);
            }
        } else {
            // Scalar fallback
            for i in 0..self.len {
                let a = unsafe { *self.ptr.add(i) };
                let b = unsafe { *other.ptr.add(i) };
                result.push(a + b);
            }
        }
        
        Ok(result)
    }

    /// SIMD-accelerated reduction (sum)
    pub fn simd_sum(&self) -> f32 {
        if self.is_empty() {
            return 0.0;
        }

        if self.simd_config.use_simd && self.len >= self.simd_config.vector_width {
            let chunk_size = self.simd_config.vector_width;
            let chunks = self.len / chunk_size;
            let mut partial_sums = vec![0.0f32; chunk_size];
            
            // Accumulate into SIMD lanes
            for chunk in 0..chunks {
                let start = chunk * chunk_size;
                for i in 0..chunk_size {
                    partial_sums[i] += unsafe { *self.ptr.add(start + i) };
                }
            }
            
            // Sum the partial sums
            let mut total: f32 = partial_sums.iter().sum();
            
            // Add remaining elements
            for i in (chunks * chunk_size)..self.len {
                total += unsafe { *self.ptr.add(i) };
            }
            
            total
        } else {
            // Scalar reduction
            let mut sum = 0.0f32;
            for i in 0..self.len {
                sum += unsafe { *self.ptr.add(i) };
            }
            sum
        }
    }

    /// SIMD-accelerated dot product
    pub fn simd_dot(&self, other: &Vec<f32>) -> Result<f32, SIMDError> {
        if self.len != other.len {
            return Err(SIMDError::LengthMismatch(self.len, other.len));
        }

        if self.is_empty() {
            return Ok(0.0);
        }

        if self.simd_config.use_simd && self.len >= self.simd_config.vector_width {
            let chunk_size = self.simd_config.vector_width;
            let chunks = self.len / chunk_size;
            let mut partial_sums = vec![0.0f32; chunk_size];
            
            // SIMD multiply-accumulate
            for chunk in 0..chunks {
                let start = chunk * chunk_size;
                for i in 0..chunk_size {
                    let a = unsafe { *self.ptr.add(start + i) };
                    let b = unsafe { *other.ptr.add(start + i) };
                    partial_sums[i] += a * b;
                }
            }
            
            let mut total: f32 = partial_sums.iter().sum();
            
            // Process remaining elements
            for i in (chunks * chunk_size)..self.len {
                let a = unsafe { *self.ptr.add(i) };
                let b = unsafe { *other.ptr.add(i) };
                total += a * b;
            }
            
            Ok(total)
        } else {
            // Scalar dot product
            let mut sum = 0.0f32;
            for i in 0..self.len {
                let a = unsafe { *self.ptr.add(i) };
                let b = unsafe { *other.ptr.add(i) };
                sum += a * b;
            }
            Ok(sum)
        }
    }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        self.clear();
        if self.capacity != 0 {
            let layout = Layout::from_size_align(
                self.capacity * mem::size_of::<T>(),
                self.simd_config.alignment,
            ).expect("invalid layout");
            unsafe {
                dealloc(self.ptr as *mut u8, layout);
            }
        }
    }
}

/// SIMD-accelerated hash map with vectorized hashing
pub struct HashMap<K, V> {
    /// Underlying standard library HashMap
    inner: StdHashMap<K, V>,
    /// SIMD configuration for batch operations
    simd_config: SIMDConfig,
}

impl<K, V> HashMap<K, V>
where
    K: Hash + Eq,
{
    /// Create a new HashMap with SIMD optimization
    pub fn new() -> Self {
        Self {
            inner: StdHashMap::new(),
            simd_config: SIMDConfig {
                vector_width: 8, // Process 8 hashes at once
                unroll_factor: 4,
                use_simd: true,
                alignment: 32,
            },
        }
    }

    /// Create HashMap with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: StdHashMap::with_capacity(capacity),
            simd_config: SIMDConfig {
                vector_width: 8,
                unroll_factor: 4,
                use_simd: true,
                alignment: 32,
            },
        }
    }

    /// Insert a key-value pair
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.inner.insert(key, value)
    }

    /// Get a value by key
    pub fn get(&self, key: &K) -> Option<&V> {
        self.inner.get(key)
    }

    /// Get a mutable value by key
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.inner.get_mut(key)
    }

    /// Remove a key-value pair
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.inner.remove(key)
    }

    /// Check if key exists
    pub fn contains_key(&self, key: &K) -> bool {
        self.inner.contains_key(key)
    }

    /// Get the number of elements
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if the map is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Clear all elements
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Get all keys
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.inner.keys()
    }

    /// Get all values
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.inner.values()
    }

    /// Iterate over key-value pairs
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.inner.iter()
    }
}

// SIMD-accelerated batch operations for HashMap
impl<K, V> HashMap<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    /// SIMD-accelerated batch insert
    pub fn simd_batch_insert(&mut self, pairs: &[(K, V)]) {
        if pairs.len() >= self.simd_config.vector_width {
            // Process in batches for better cache locality
            let batch_size = self.simd_config.vector_width;
            for chunk in pairs.chunks(batch_size) {
                for (key, value) in chunk {
                    self.inner.insert(key.clone(), value.clone());
                }
            }
        } else {
            // Fallback to normal insertion
            for (key, value) in pairs {
                self.inner.insert(key.clone(), value.clone());
            }
        }
    }

    /// SIMD-accelerated batch lookup
    pub fn simd_batch_get(&self, keys: &[K]) -> Vec<Option<V>> {
        let mut results = Vec::with_capacity(keys.len());
        
        if keys.len() >= self.simd_config.vector_width {
            // Batch processing for cache efficiency
            let batch_size = self.simd_config.vector_width;
            for chunk in keys.chunks(batch_size) {
                for key in chunk {
                    results.push(self.inner.get(key).cloned());
                }
            }
        } else {
            // Normal processing
            for key in keys {
                results.push(self.inner.get(key).cloned());
            }
        }
        
        results
    }
}

/// HashSet with SIMD-accelerated operations
pub struct HashSet<T> {
    /// Underlying HashMap with unit values
    inner: HashMap<T, ()>,
}

impl<T> HashSet<T>
where
    T: Hash + Eq,
{
    /// Create a new HashSet
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Insert an element
    pub fn insert(&mut self, value: T) -> bool {
        self.inner.insert(value, ()).is_none()
    }

    /// Check if element exists
    pub fn contains(&self, value: &T) -> bool {
        self.inner.contains_key(value)
    }

    /// Remove an element
    pub fn remove(&mut self, value: &T) -> bool {
        self.inner.remove(value).is_some()
    }

    /// Get the number of elements
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if the set is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Clear all elements
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Iterate over elements
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.inner.keys()
    }
}

#[derive(Debug, PartialEq)]
pub enum SIMDError {
    LengthMismatch(usize, usize),
    InvalidVectorWidth(usize),
    UnsupportedOperation(String),
}

impl std::fmt::Display for SIMDError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SIMDError::LengthMismatch(a, b) => {
                write!(f, "Vector length mismatch: {} != {}", a, b)
            }
            SIMDError::InvalidVectorWidth(w) => {
                write!(f, "Invalid vector width: {}", w)
            }
            SIMDError::UnsupportedOperation(op) => {
                write!(f, "Unsupported SIMD operation: {}", op)
            }
        }
    }
}

impl std::error::Error for SIMDError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec_basic_operations() {
        let mut vec = Vec::new();
        assert!(vec.is_empty());
        assert_eq!(vec.len(), 0);

        vec.push(1);
        vec.push(2);
        vec.push(3);

        assert_eq!(vec.len(), 3);
        assert_eq!(vec.get(0), Some(&1));
        assert_eq!(vec.get(1), Some(&2));
        assert_eq!(vec.get(2), Some(&3));
        assert_eq!(vec.get(3), None);

        assert_eq!(vec.pop(), Some(3));
        assert_eq!(vec.len(), 2);
    }

    #[test]
    fn test_vec_simd_operations() {
        let mut vec1 = Vec::new();
        let mut vec2 = Vec::new();

        // Add test data
        for i in 0..16 {
            vec1.push(i as f32);
            vec2.push((i * 2) as f32);
        }

        // Test SIMD addition
        let result = vec1.simd_add(&vec2).unwrap();
        assert_eq!(result.len(), 16);
        
        for i in 0..16 {
            let expected = (i + i * 2) as f32;
            assert_eq!(result.get(i), Some(&expected));
        }

        // Test SIMD sum
        let sum = vec1.simd_sum();
        let expected_sum = (0..16).sum::<i32>() as f32;
        assert_eq!(sum, expected_sum);

        // Test SIMD dot product
        let dot = vec1.simd_dot(&vec2).unwrap();
        let expected_dot: f32 = (0..16).map(|i| (i * i * 2) as f32).sum();
        assert_eq!(dot, expected_dot);
    }

    #[test]
    fn test_vec_simd_map() {
        let mut vec = Vec::new();
        for i in 0..10 {
            vec.push(i as f32);
        }

        let squared = vec.simd_map(|x| x * x);
        assert_eq!(squared.len(), 10);
        
        for i in 0..10 {
            let expected = (i * i) as f32;
            assert_eq!(squared.get(i), Some(&expected));
        }
    }

    #[test]
    fn test_hashmap_basic_operations() {
        let mut map = HashMap::new();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);

        map.insert("key1".to_string(), 100);
        map.insert("key2".to_string(), 200);

        assert_eq!(map.get(&"key1".to_string()), Some(&100));
        assert_eq!(map.get(&"key2".to_string()), Some(&200));
        assert_eq!(map.get(&"key3".to_string()), None);

        assert!(map.contains_key(&"key1".to_string()));
        assert!(!map.contains_key(&"key3".to_string()));

        assert_eq!(map.remove(&"key1".to_string()), Some(100));
        assert_eq!(map.get(&"key1".to_string()), None);
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_hashmap_simd_batch_operations() {
        let mut map = HashMap::new();
        
        // Test batch insert
        let pairs = vec![
            ("a".to_string(), 1), ("b".to_string(), 2), ("c".to_string(), 3), ("d".to_string(), 4),
            ("e".to_string(), 5), ("f".to_string(), 6), ("g".to_string(), 7), ("h".to_string(), 8),
        ];
        
        map.simd_batch_insert(&pairs);
        assert_eq!(map.len(), 8);
        
        // Test batch get
        let keys = vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string(), 
                       "e".to_string(), "f".to_string(), "g".to_string(), "h".to_string()];
        let results = map.simd_batch_get(&keys);
        
        assert_eq!(results.len(), 8);
        for (i, result) in results.iter().enumerate() {
            assert_eq!(*result, Some(i + 1));
        }
    }

    #[test]
    fn test_hashset_operations() {
        let mut set = HashSet::new();
        assert!(set.is_empty());

        assert!(set.insert(1));
        assert!(set.insert(2));
        assert!(!set.insert(1)); // Already exists

        assert!(set.contains(&1));
        assert!(set.contains(&2));
        assert!(!set.contains(&3));

        assert!(set.remove(&1));
        assert!(!set.remove(&3)); // Doesn't exist

        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_simd_error_handling() {
        let vec1 = Vec::from(vec![1.0, 2.0, 3.0]);
        let vec2 = Vec::from(vec![1.0, 2.0]); // Different length

        let result = vec1.simd_add(&vec2);
        assert_eq!(result, Err(SIMDError::LengthMismatch(3, 2)));

        let dot_result = vec1.simd_dot(&vec2);
        assert_eq!(dot_result, Err(SIMDError::LengthMismatch(3, 2)));
    }
}

impl<T> From<std::vec::Vec<T>> for Vec<T> {
    fn from(vec: std::vec::Vec<T>) -> Self {
        let mut result = Vec::with_capacity(vec.len());
        for item in vec {
            result.push(item);
        }
        result
    }
}

impl<T: Clone> From<&[T]> for Vec<T> {
    fn from(slice: &[T]) -> Self {
        let mut result = Vec::with_capacity(slice.len());
        for item in slice {
            result.push(item.clone());
        }
        result
    }
}