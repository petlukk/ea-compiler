#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>

// String structure
typedef struct {
    char* data;
    size_t length;
    size_t capacity;
} String;

// String::new() -> *String
String* string_new() {
    String* str = (String*)malloc(sizeof(String));
    if (!str) return NULL;
    
    str->data = (char*)malloc(1);
    if (!str->data) {
        free(str);
        return NULL;
    }
    str->data[0] = '\0';
    str->length = 0;
    str->capacity = 1;
    return str;
}

// String::from(literal: *char) -> *String
String* string_from(const char* literal) {
    if (!literal) return string_new();
    
    String* str = (String*)malloc(sizeof(String));
    if (!str) return NULL;
    
    size_t len = strlen(literal);
    str->data = (char*)malloc(len + 1);
    if (!str->data) {
        free(str);
        return NULL;
    }
    
    strcpy(str->data, literal);
    str->length = len;
    str->capacity = len + 1;
    return str;
}

// String::len(str: *String) -> i32
int string_len(String* str) {
    if (!str) return 0;
    return (int)str->length;
}

// String::push_str(str: *String, other: *char) -> void
void string_push_str(String* str, const char* other) {
    if (!str || !other) return;
    
    size_t other_len = strlen(other);
    size_t new_len = str->length + other_len;
    
    // Resize if necessary
    if (new_len >= str->capacity) {
        size_t new_capacity = (new_len + 1) * 2;
        char* new_data = (char*)realloc(str->data, new_capacity);
        if (!new_data) return;
        str->data = new_data;
        str->capacity = new_capacity;
    }
    
    strcat(str->data, other);
    str->length = new_len;
}

// String::as_str(str: *String) -> *char
const char* string_as_str(String* str) {
    if (!str || !str->data) return "";
    return str->data;
}

// String::clone(str: *String) -> *String
String* string_clone(String* str) {
    if (!str) return string_new();
    return string_from(str->data);
}

// String::substring(str: *String, start: i32, end: i32) -> *String
String* string_substring(String* str, int start, int end) {
    if (!str || start < 0 || end < start || start >= (int)str->length) {
        return string_new();
    }
    
    if (end > (int)str->length) {
        end = (int)str->length;
    }
    
    int sub_len = end - start;
    String* result = (String*)malloc(sizeof(String));
    if (!result) return string_new();
    
    result->data = (char*)malloc(sub_len + 1);
    if (!result->data) {
        free(result);
        return string_new();
    }
    
    strncpy(result->data, str->data + start, sub_len);
    result->data[sub_len] = '\0';
    result->length = sub_len;
    result->capacity = sub_len + 1;
    
    return result;
}

// String::find(str: *String, needle: *char) -> i32
int string_find(String* str, const char* needle) {
    if (!str || !needle || !str->data) return -1;
    
    char* found = strstr(str->data, needle);
    if (!found) return -1;
    
    return (int)(found - str->data);
}

// String::replace(str: *String, from: *char, to: *char) -> *String
String* string_replace(String* str, const char* from, const char* to) {
    if (!str || !from || !to || !str->data) return string_clone(str);
    
    size_t from_len = strlen(from);
    size_t to_len = strlen(to);
    
    // Find first occurrence
    char* pos = strstr(str->data, from);
    if (!pos) return string_clone(str);
    
    // Calculate new length
    size_t prefix_len = pos - str->data;
    size_t suffix_len = str->length - prefix_len - from_len;
    size_t new_len = prefix_len + to_len + suffix_len;
    
    String* result = (String*)malloc(sizeof(String));
    if (!result) return string_clone(str);
    
    result->data = (char*)malloc(new_len + 1);
    if (!result->data) {
        free(result);
        return string_clone(str);
    }
    
    // Copy prefix
    strncpy(result->data, str->data, prefix_len);
    // Copy replacement
    strcpy(result->data + prefix_len, to);
    // Copy suffix
    strcpy(result->data + prefix_len + to_len, str->data + prefix_len + from_len);
    
    result->length = new_len;
    result->capacity = new_len + 1;
    
    return result;
}

// String::to_uppercase(str: *String) -> *String
String* string_to_uppercase(String* str) {
    if (!str) return string_new();
    
    String* result = string_clone(str);
    if (!result) return string_new();
    
    for (size_t i = 0; i < result->length; i++) {
        result->data[i] = toupper(result->data[i]);
    }
    
    return result;
}

// String::to_lowercase(str: *String) -> *String
String* string_to_lowercase(String* str) {
    if (!str) return string_new();
    
    String* result = string_clone(str);
    if (!result) return string_new();
    
    for (size_t i = 0; i < result->length; i++) {
        result->data[i] = tolower(result->data[i]);
    }
    
    return result;
}

// String::trim(str: *String) -> *String
String* string_trim(String* str) {
    if (!str || !str->data) return string_new();
    
    // Find start of non-whitespace
    size_t start = 0;
    while (start < str->length && isspace(str->data[start])) {
        start++;
    }
    
    // Find end of non-whitespace
    size_t end = str->length;
    while (end > start && isspace(str->data[end - 1])) {
        end--;
    }
    
    size_t trimmed_len = end - start;
    String* result = (String*)malloc(sizeof(String));
    if (!result) return string_new();
    
    result->data = (char*)malloc(trimmed_len + 1);
    if (!result->data) {
        free(result);
        return string_new();
    }
    
    strncpy(result->data, str->data + start, trimmed_len);
    result->data[trimmed_len] = '\0';
    result->length = trimmed_len;
    result->capacity = trimmed_len + 1;
    
    return result;
}

// String::free(str: *String) -> void
void string_free(String* str) {
    if (!str) return;
    if (str->data) {
        free(str->data);
    }
    free(str);
}

// String concatenation function (for + operator)
String* string_concat(String* left, String* right) {
    if (!left && !right) return string_new();
    if (!left) return string_clone(right);
    if (!right) return string_clone(left);
    
    size_t new_len = left->length + right->length;
    String* result = (String*)malloc(sizeof(String));
    if (!result) return string_new();
    
    result->data = (char*)malloc(new_len + 1);
    if (!result->data) {
        free(result);
        return string_new();
    }
    
    strcpy(result->data, left->data);
    strcat(result->data, right->data);
    result->length = new_len;
    result->capacity = new_len + 1;
    
    return result;
}

// String equality function (for == operator)
int string_equals(String* left, String* right) {
    if (!left && !right) return 1;
    if (!left || !right) return 0;
    if (left->length != right->length) return 0;
    return strcmp(left->data, right->data) == 0;
}