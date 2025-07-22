#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include <limits.h>

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
// Works with raw char* strings as expected by the compiler
// IMPORTANT: Caller must call string_concat_free() on returned pointer
char* string_concat(const char* left, const char* right) {
    if (!left && !right) {
        char* empty = (char*)malloc(1);
        if (empty) empty[0] = '\0';
        return empty;
    }
    if (!left) {
        size_t len = strlen(right);
        char* result = (char*)malloc(len + 1);
        if (result) strcpy(result, right);
        return result;
    }
    if (!right) {
        size_t len = strlen(left);
        char* result = (char*)malloc(len + 1);
        if (result) strcpy(result, left);
        return result;
    }
    
    size_t left_len = strlen(left);
    size_t right_len = strlen(right);
    size_t new_len = left_len + right_len;
    
    char* result = (char*)malloc(new_len + 1);
    if (!result) {
        char* empty = (char*)malloc(1);
        if (empty) empty[0] = '\0';
        return empty;
    }
    
    strcpy(result, left);
    strcat(result, right);
    
    return result;
}

// Free string concatenation result
void string_concat_free(char* str) {
    if (str) {
        free(str);
    }
}

// String equality function (for == operator)
int string_equals(String* left, String* right) {
    if (!left && !right) return 1;
    if (!left || !right) return 0;
    if (left->length != right->length) return 0;
    return strcmp(left->data, right->data) == 0;
}

// String format function - simple version for single placeholder
String* string_format(const char* template, const char* value) {
    if (!template || !value) return string_new();
    
    // Find the placeholder {}
    char* placeholder = strstr(template, "{}");
    if (!placeholder) {
        return string_from(template);
    }
    
    size_t prefix_len = placeholder - template;
    size_t suffix_len = strlen(template) - prefix_len - 2; // -2 for {}
    size_t value_len = strlen(value);
    size_t total_len = prefix_len + value_len + suffix_len;
    
    String* result = (String*)malloc(sizeof(String));
    if (!result) return string_new();
    
    result->data = (char*)malloc(total_len + 1);
    if (!result->data) {
        free(result);
        return string_new();
    }
    
    // Copy prefix
    strncpy(result->data, template, prefix_len);
    // Copy value
    strcpy(result->data + prefix_len, value);
    // Copy suffix
    strcpy(result->data + prefix_len + value_len, template + prefix_len + 2);
    
    result->length = total_len;
    result->capacity = total_len + 1;
    
    return result;
}

// String format for integers
String* string_format_i32(const char* template, int value) {
    if (!template) return string_new();
    
    // Convert integer to string
    char buffer[32];
    snprintf(buffer, sizeof(buffer), "%d", value);
    
    return string_format(template, buffer);
}

// String format for floats
String* string_format_f32(const char* template, float value) {
    if (!template) return string_new();
    
    // Convert float to string
    char buffer[32];
    snprintf(buffer, sizeof(buffer), "%.5g", value);
    
    return string_format(template, buffer);
}

// String split function
typedef struct {
    String** items;
    size_t count;
    size_t capacity;
} StringArray;

StringArray* string_split(String* str, const char* delimiter) {
    if (!str || !delimiter || !str->data) {
        StringArray* empty = (StringArray*)malloc(sizeof(StringArray));
        if (!empty) return NULL;
        empty->items = NULL;
        empty->count = 0;
        empty->capacity = 0;
        return empty;
    }
    
    StringArray* result = (StringArray*)malloc(sizeof(StringArray));
    if (!result) return NULL;
    
    result->capacity = 4;
    result->items = (String**)malloc(sizeof(String*) * result->capacity);
    if (!result->items) {
        free(result);
        return NULL;
    }
    result->count = 0;
    
    // Handle empty delimiter (split into characters)
    if (strlen(delimiter) == 0) {
        for (size_t i = 0; i < str->length; i++) {
            if (result->count >= result->capacity) {
                result->capacity *= 2;
                String** new_items = (String**)realloc(result->items, sizeof(String*) * result->capacity);
                if (!new_items) break;
                result->items = new_items;
            }
            
            String* char_str = (String*)malloc(sizeof(String));
            if (!char_str) break;
            char_str->data = (char*)malloc(2);
            if (!char_str->data) {
                free(char_str);
                break;
            }
            char_str->data[0] = str->data[i];
            char_str->data[1] = '\0';
            char_str->length = 1;
            char_str->capacity = 2;
            
            result->items[result->count++] = char_str;
        }
        return result;
    }
    
    // Regular delimiter splitting
    char* data_copy = (char*)malloc(str->length + 1);
    if (!data_copy) {
        free(result->items);
        free(result);
        return NULL;
    }
    strcpy(data_copy, str->data);
    
    size_t delim_len = strlen(delimiter);
    char* start = data_copy;
    char* end = strstr(start, delimiter);
    
    while (end != NULL) {
        // Ensure capacity
        if (result->count >= result->capacity) {
            result->capacity *= 2;
            String** new_items = (String**)realloc(result->items, sizeof(String*) * result->capacity);
            if (!new_items) break;
            result->items = new_items;
        }
        
        // Create substring
        *end = '\0';
        result->items[result->count++] = string_from(start);
        
        start = end + delim_len;
        end = strstr(start, delimiter);
    }
    
    // Add final part
    if (result->count < result->capacity) {
        result->items[result->count++] = string_from(start);
    }
    
    free(data_copy);
    return result;
}

// String starts_with function
int string_starts_with(String* str, const char* prefix) {
    if (!str || !prefix || !str->data) return 0;
    
    size_t prefix_len = strlen(prefix);
    if (prefix_len > str->length) return 0;
    
    return strncmp(str->data, prefix, prefix_len) == 0;
}

// String ends_with function
int string_ends_with(String* str, const char* suffix) {
    if (!str || !suffix || !str->data) return 0;
    
    size_t suffix_len = strlen(suffix);
    if (suffix_len > str->length) return 0;
    
    return strcmp(str->data + str->length - suffix_len, suffix) == 0;
}

// String to_i32 function
int string_to_i32(String* str) {
    if (!str || !str->data) return 0;
    
    // Use strtol for robust parsing
    char* endptr;
    long result = strtol(str->data, &endptr, 10);
    
    // Check for parsing errors
    if (endptr == str->data || *endptr != '\0') {
        return 0; // Invalid input
    }
    
    // Check for overflow
    if (result > INT_MAX || result < INT_MIN) {
        return 0;
    }
    
    return (int)result;
}

// String to_f32 function
float string_to_f32(String* str) {
    if (!str || !str->data) return 0.0f;
    
    // Use strtof for robust parsing
    char* endptr;
    float result = strtof(str->data, &endptr);
    
    // Check for parsing errors
    if (endptr == str->data || *endptr != '\0') {
        return 0.0f; // Invalid input
    }
    
    return result;
}

// Free string array
void string_array_free(StringArray* arr) {
    if (!arr) return;
    
    for (size_t i = 0; i < arr->count; i++) {
        string_free(arr->items[i]);
    }
    free(arr->items);
    free(arr);
}