//! Runtime support for File I/O operations
//! This provides real file operations for Eä File type

#define _GNU_SOURCE  // Enable GNU extensions for getline
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdint.h>
#include <sys/stat.h>
#include <unistd.h>

// Helper function for strdup to ensure C99 compatibility
static char* ea_strdup(const char* str) {
    if (!str) return NULL;
    size_t len = strlen(str) + 1;
    char* copy = malloc(len);
    if (copy) {
        memcpy(copy, str, len);
    }
    return copy;
}

// File structure layout (matches LLVM IR expectations)
typedef struct {
    FILE* handle;    // C FILE pointer
    char* filename;  // Copy of filename for debugging
    char* mode;      // Copy of mode for debugging
    int is_open;     // Boolean flag for open status
} EaFile;

// Create a new File and open it
EaFile* file_open(const char* filename, const char* mode) {
    if (!filename || !mode) return NULL;
    
    EaFile* file = (EaFile*)malloc(sizeof(EaFile));
    if (!file) return NULL;
    
    // Open the file
    file->handle = fopen(filename, mode);
    if (!file->handle) {
        free(file);
        return NULL;
    }
    
    // Store filename and mode for debugging
    file->filename = ea_strdup(filename);
    file->mode = ea_strdup(mode);
    file->is_open = 1;
    
    return file;
}

// Create a new File for writing (convenience function)
EaFile* file_create(const char* filename) {
    if (!filename) return NULL;
    
    // Use file_open with "w" mode to create/overwrite a file
    return file_open(filename, "w");
}

// Check if a file exists
int32_t file_exists(const char* filename) {
    if (!filename) return 0;
    
    struct stat st;
    return (stat(filename, &st) == 0) ? 1 : 0;
}

// Get file size
int64_t file_size(const char* filename) {
    if (!filename) return -1;
    
    struct stat st;
    if (stat(filename, &st) != 0) {
        return -1; // File doesn't exist or can't access
    }
    
    return (int64_t)st.st_size;
}

// Delete a file
void file_delete(const char* filename) {
    if (!filename) return;
    
    unlink(filename);
}

// Write data to file
void file_write(EaFile* file, const char* data) {
    if (!file || !file->handle || !file->is_open || !data) return;
    
    fputs(data, file->handle);
    fflush(file->handle); // Ensure data is written immediately
}

// Read a line from file
char* file_read_line(EaFile* file) {
    if (!file || !file->handle || !file->is_open) return NULL;
    
    char* line = NULL;
    size_t len = 0;
    ssize_t read_len = getline(&line, &len, file->handle);
    
    if (read_len == -1) {
        if (line) free(line);
        return NULL; // EOF or error
    }
    
    // Remove trailing newline if present
    if (read_len > 0 && line[read_len - 1] == '\n') {
        line[read_len - 1] = '\0';
    }
    
    return line;
}

// Read entire file content
char* file_read_all(EaFile* file) {
    if (!file || !file->handle || !file->is_open) return NULL;
    
    // Get current position
    long current_pos = ftell(file->handle);
    
    // Seek to end to get file size
    fseek(file->handle, 0, SEEK_END);
    long file_size = ftell(file->handle);
    
    // Restore position
    fseek(file->handle, current_pos, SEEK_SET);
    
    // Calculate remaining size from current position
    long remaining_size = file_size - current_pos;
    if (remaining_size <= 0) return NULL;
    
    // Allocate buffer
    char* buffer = (char*)malloc(remaining_size + 1);
    if (!buffer) return NULL;
    
    // Read the data
    size_t read_size = fread(buffer, 1, remaining_size, file->handle);
    buffer[read_size] = '\0'; // Null-terminate
    
    return buffer;
}

// Close file
void file_close(EaFile* file) {
    if (!file) return;
    
    if (file->handle && file->is_open) {
        fclose(file->handle);
        file->handle = NULL;
        file->is_open = 0;
    }
    
    if (file->filename) {
        free(file->filename);
        file->filename = NULL;
    }
    
    if (file->mode) {
        free(file->mode);
        file->mode = NULL;
    }
    
    free(file);
}

// Helper function to free a file handle (for RAII-style cleanup)
void file_free(EaFile* file) {
    file_close(file); // Same as close for now
}