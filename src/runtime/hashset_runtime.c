// HashSet Runtime Implementation for Eä Language
// Complete C runtime functions for HashSet operations

#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <stdio.h>

// HashSet node structure
typedef struct HashSetNode {
    int key;
    struct HashSetNode* next;
} HashSetNode;

// HashSet structure
typedef struct HashSet {
    HashSetNode** buckets;
    int size;
    int capacity;
    int count;
} HashSet;

// Hash function for integers
static int hash_int(int key, int capacity) {
    return abs(key) % capacity;
}

// Create a new HashSet
HashSet* hashset_new() {
    HashSet* set = malloc(sizeof(HashSet));
    if (!set) return NULL;
    
    set->capacity = 16;
    set->size = 0;
    set->count = 0;
    set->buckets = calloc(set->capacity, sizeof(HashSetNode*));
    
    if (!set->buckets) {
        free(set);
        return NULL;
    }
    
    return set;
}

// Resize HashSet when load factor gets too high
static void hashset_resize(HashSet* set) {
    if (!set || set->count < set->capacity * 0.75) return;
    
    HashSetNode** old_buckets = set->buckets;
    int old_capacity = set->capacity;
    
    set->capacity *= 2;
    set->buckets = calloc(set->capacity, sizeof(HashSetNode*));
    if (!set->buckets) {
        set->buckets = old_buckets;
        set->capacity = old_capacity;
        return;
    }
    
    // Rehash all elements
    for (int i = 0; i < old_capacity; i++) {
        HashSetNode* node = old_buckets[i];
        while (node) {
            HashSetNode* next = node->next;
            int new_index = hash_int(node->key, set->capacity);
            
            node->next = set->buckets[new_index];
            set->buckets[new_index] = node;
            
            node = next;
        }
    }
    
    free(old_buckets);
}

// Insert element into HashSet
bool hashset_insert(HashSet* set, int key) {
    if (!set) return false;
    
    int index = hash_int(key, set->capacity);
    HashSetNode* node = set->buckets[index];
    
    // Check if key already exists
    while (node) {
        if (node->key == key) {
            return false; // Key already exists
        }
        node = node->next;
    }
    
    // Create new node
    HashSetNode* new_node = malloc(sizeof(HashSetNode));
    if (!new_node) return false;
    
    new_node->key = key;
    new_node->next = set->buckets[index];
    set->buckets[index] = new_node;
    
    set->count++;
    hashset_resize(set);
    
    return true;
}

// Check if element exists in HashSet
bool hashset_contains(HashSet* set, int key) {
    if (!set) return false;
    
    int index = hash_int(key, set->capacity);
    HashSetNode* node = set->buckets[index];
    
    while (node) {
        if (node->key == key) {
            return true;
        }
        node = node->next;
    }
    
    return false;
}

// Remove element from HashSet
bool hashset_remove(HashSet* set, int key) {
    if (!set) return false;
    
    int index = hash_int(key, set->capacity);
    HashSetNode* node = set->buckets[index];
    HashSetNode* prev = NULL;
    
    while (node) {
        if (node->key == key) {
            if (prev) {
                prev->next = node->next;
            } else {
                set->buckets[index] = node->next;
            }
            free(node);
            set->count--;
            return true;
        }
        prev = node;
        node = node->next;
    }
    
    return false;
}

// Get the number of elements in HashSet
int hashset_len(HashSet* set) {
    if (!set) return 0;
    return set->count;
}

// Check if HashSet is empty
bool hashset_is_empty(HashSet* set) {
    if (!set) return true;
    return set->count == 0;
}

// Clear all elements from HashSet
void hashset_clear(HashSet* set) {
    if (!set) return;
    
    for (int i = 0; i < set->capacity; i++) {
        HashSetNode* node = set->buckets[i];
        while (node) {
            HashSetNode* next = node->next;
            free(node);
            node = next;
        }
        set->buckets[i] = NULL;
    }
    
    set->count = 0;
}

// Free HashSet and all its resources
void hashset_free(HashSet* set) {
    if (!set) return;
    
    hashset_clear(set);
    free(set->buckets);
    free(set);
}

// Debug: Print HashSet contents
void hashset_debug_print(HashSet* set) {
    if (!set) {
        printf("HashSet: NULL\n");
        return;
    }
    
    printf("HashSet: capacity=%d, count=%d\n", set->capacity, set->count);
    for (int i = 0; i < set->capacity; i++) {
        HashSetNode* node = set->buckets[i];
        if (node) {
            printf("  bucket[%d]: ", i);
            while (node) {
                printf("%d ", node->key);
                node = node->next;
            }
            printf("\n");
        }
    }
}

// Export functions for LLVM linkage
extern HashSet* HashSet_new() {
    return hashset_new();
}

extern bool HashSet_insert(HashSet* set, int key) {
    return hashset_insert(set, key);
}

extern bool HashSet_contains(HashSet* set, int key) {
    return hashset_contains(set, key);
}

extern bool HashSet_remove(HashSet* set, int key) {
    return hashset_remove(set, key);
}

extern int HashSet_len(HashSet* set) {
    return hashset_len(set);
}

extern bool HashSet_is_empty(HashSet* set) {
    return hashset_is_empty(set);
}

extern void HashSet_clear(HashSet* set) {
    hashset_clear(set);
}

extern void HashSet_free(HashSet* set) {
    hashset_free(set);
}