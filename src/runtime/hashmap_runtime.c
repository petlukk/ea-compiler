//! Runtime support for HashMap operations
//! This provides real memory management and operations for Eä HashMap type

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdint.h>

// HashMap entry structure
typedef struct HashMapEntry {
    int32_t key;
    int32_t value;
    int occupied;  // 1 if occupied, 0 if empty
} HashMapEntry;

// HashMap structure layout (matches LLVM IR expectations)
typedef struct {
    HashMapEntry* buckets;
    size_t capacity;
    size_t size;
} HashMap;

// Simple hash function for i32 keys
static size_t hash_i32(int32_t key, size_t capacity) {
    if (capacity == 0) return 0;
    // Simple hash function - can be improved later
    return ((size_t)key * 2654435761U) % capacity;
}

// Create a new empty HashMap
HashMap* hashmap_new() {
    HashMap* map = (HashMap*)malloc(sizeof(HashMap));
    if (!map) return NULL;
    
    map->buckets = NULL;
    map->capacity = 0;
    map->size = 0;
    return map;
}

// Create HashMap with specific capacity
HashMap* hashmap_with_capacity(size_t capacity) {
    HashMap* map = hashmap_new();
    if (!map) return NULL;
    
    if (capacity > 0) {
        map->buckets = (HashMapEntry*)calloc(capacity, sizeof(HashMapEntry));
        if (!map->buckets) {
            free(map);
            return NULL;
        }
        map->capacity = capacity;
    }
    
    return map;
}

// Grow HashMap capacity
static int hashmap_grow(HashMap* map) {
    if (!map) return 0;
    
    size_t old_capacity = map->capacity;
    size_t new_capacity = old_capacity == 0 ? 8 : old_capacity * 2;
    
    HashMapEntry* old_buckets = map->buckets;
    HashMapEntry* new_buckets = (HashMapEntry*)calloc(new_capacity, sizeof(HashMapEntry));
    
    if (!new_buckets) return 0;
    
    map->buckets = new_buckets;
    map->capacity = new_capacity;
    size_t old_size = map->size;
    map->size = 0;
    
    // Rehash all existing entries
    for (size_t i = 0; i < old_capacity; i++) {
        if (old_buckets[i].occupied) {
            // Find new position
            size_t hash = hash_i32(old_buckets[i].key, new_capacity);
            size_t index = hash;
            
            // Linear probing
            while (new_buckets[index].occupied) {
                index = (index + 1) % new_capacity;
            }
            
            new_buckets[index].key = old_buckets[i].key;
            new_buckets[index].value = old_buckets[i].value;
            new_buckets[index].occupied = 1;
            map->size++;
        }
    }
    
    if (old_buckets) {
        free(old_buckets);
    }
    
    return 1;
}

// Insert key-value pair
int hashmap_insert(HashMap* map, int32_t key, int32_t value) {
    if (!map) return 0;
    
    // Check if we need to grow (load factor > 0.75)
    if (map->size * 4 >= map->capacity * 3) {
        if (!hashmap_grow(map)) {
            return 0;
        }
    }
    
    // If capacity is still 0, initialize
    if (map->capacity == 0) {
        if (!hashmap_grow(map)) {
            return 0;
        }
    }
    
    size_t hash = hash_i32(key, map->capacity);
    size_t index = hash;
    
    // Linear probing
    while (map->buckets[index].occupied) {
        if (map->buckets[index].key == key) {
            // Update existing key
            map->buckets[index].value = value;
            return 1;
        }
        index = (index + 1) % map->capacity;
        
        // Prevent infinite loop (shouldn't happen with proper load factor)
        if (index == hash) {
            return 0;
        }
    }
    
    // Insert new entry
    map->buckets[index].key = key;
    map->buckets[index].value = value;
    map->buckets[index].occupied = 1;
    map->size++;
    
    return 1;
}

// Get value by key
int32_t hashmap_get(HashMap* map, int32_t key) {
    if (!map || map->capacity == 0) return 0;
    
    size_t hash = hash_i32(key, map->capacity);
    size_t index = hash;
    
    // Linear probing
    while (map->buckets[index].occupied) {
        if (map->buckets[index].key == key) {
            return map->buckets[index].value;
        }
        index = (index + 1) % map->capacity;
        
        // Prevent infinite loop
        if (index == hash) {
            break;
        }
    }
    
    return 0; // Key not found
}

// Check if key exists
int hashmap_contains_key(HashMap* map, int32_t key) {
    if (!map || map->capacity == 0) return 0;
    
    size_t hash = hash_i32(key, map->capacity);
    size_t index = hash;
    
    // Linear probing
    while (map->buckets[index].occupied) {
        if (map->buckets[index].key == key) {
            return 1;
        }
        index = (index + 1) % map->capacity;
        
        // Prevent infinite loop
        if (index == hash) {
            break;
        }
    }
    
    return 0; // Key not found
}

// Remove key-value pair
int hashmap_remove(HashMap* map, int32_t key) {
    if (!map || map->capacity == 0) return 0;
    
    size_t hash = hash_i32(key, map->capacity);
    size_t index = hash;
    
    // Linear probing
    while (map->buckets[index].occupied) {
        if (map->buckets[index].key == key) {
            map->buckets[index].occupied = 0;
            map->size--;
            return 1;
        }
        index = (index + 1) % map->capacity;
        
        // Prevent infinite loop
        if (index == hash) {
            break;
        }
    }
    
    return 0; // Key not found
}

// Get size
size_t hashmap_len(HashMap* map) {
    return map ? map->size : 0;
}

// Check if empty
int hashmap_is_empty(HashMap* map) {
    return map ? (map->size == 0 ? 1 : 0) : 1;
}

// Clear all entries
void hashmap_clear(HashMap* map) {
    if (map && map->buckets) {
        for (size_t i = 0; i < map->capacity; i++) {
            map->buckets[i].occupied = 0;
        }
        map->size = 0;
    }
}

// Free HashMap memory
void hashmap_free(HashMap* map) {
    if (map) {
        if (map->buckets) {
            free(map->buckets);
        }
        free(map);
    }
}

// Test function to verify runtime works
int hashmap_runtime_test() {
    HashMap* map = hashmap_new();
    if (!map) return 0;
    
    // Test basic operations
    if (!hashmap_insert(map, 42, 100)) {
        hashmap_free(map);
        return 0;
    }
    
    if (!hashmap_insert(map, 84, 200)) {
        hashmap_free(map);
        return 0;
    }
    
    // Test get
    if (hashmap_get(map, 42) != 100) {
        hashmap_free(map);
        return 0;
    }
    
    if (hashmap_get(map, 84) != 200) {
        hashmap_free(map);
        return 0;
    }
    
    // Test size
    if (hashmap_len(map) != 2) {
        hashmap_free(map);
        return 0;
    }
    
    // Test contains
    if (!hashmap_contains_key(map, 42)) {
        hashmap_free(map);
        return 0;
    }
    
    if (hashmap_contains_key(map, 999)) {
        hashmap_free(map);
        return 0;
    }
    
    // Test remove
    if (!hashmap_remove(map, 42)) {
        hashmap_free(map);
        return 0;
    }
    
    if (hashmap_len(map) != 1) {
        hashmap_free(map);
        return 0;
    }
    
    if (hashmap_contains_key(map, 42)) {
        hashmap_free(map);
        return 0;
    }
    
    // Test stress - insert many items
    for (int i = 0; i < 100; i++) {
        if (!hashmap_insert(map, i, i * 10)) {
            hashmap_free(map);
            return 0;
        }
    }
    
    if (hashmap_len(map) != 100) {
        hashmap_free(map);
        return 0;
    }
    
    // Verify all items
    for (int i = 0; i < 100; i++) {
        if (hashmap_get(map, i) != i * 10) {
            hashmap_free(map);
            return 0;
        }
    }
    
    hashmap_free(map);
    return 1; // Success
}