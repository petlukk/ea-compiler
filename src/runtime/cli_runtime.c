/*
 * CLI Runtime Support for Eä Compiler
 * Provides real command-line argument parsing and program execution support
 */

#define _GNU_SOURCE  // Enable GNU extensions for strdup
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdint.h>
#include <unistd.h>
#include <sys/time.h>
#include <sys/resource.h>

// Global program arguments (set by main)
static int g_argc = 0;
static char** g_argv = NULL;

// Initialize CLI runtime with program arguments
void cli_init(int argc, char* argv[]) {
    g_argc = argc;
    g_argv = argv;
}

// Get command line argument count
int32_t get_command_line_arg_count() {
    return (int32_t)g_argc;
}

// Get command line argument by index
char* get_command_line_arg(int32_t index) {
    if (index < 0 || index >= g_argc || !g_argv) {
        return NULL;
    }
    
    // Return a copy of the argument
    size_t len = strlen(g_argv[index]);
    char* arg_copy = (char*)malloc(len + 1);
    if (arg_copy) {
        strcpy(arg_copy, g_argv[index]);
    }
    return arg_copy;
}

// Get all command line arguments as array
char** get_command_line_args() {
    if (!g_argv || g_argc <= 0) {
        return NULL;
    }
    
    // Allocate array of string pointers
    char** args = (char**)malloc((g_argc + 1) * sizeof(char*));
    if (!args) return NULL;
    
    // Copy each argument
    for (int i = 0; i < g_argc; i++) {
        size_t len = strlen(g_argv[i]);
        args[i] = (char*)malloc(len + 1);
        if (args[i]) {
            strcpy(args[i], g_argv[i]);
        }
    }
    
    args[g_argc] = NULL; // Null terminate
    return args;
}

// Parse command line arguments for image processing
typedef struct {
    char* input_file;
    char* output_file;
    char* filter_type;
    int32_t brightness;
    int32_t valid;
} CLIArgs;

CLIArgs* parse_cli_args() {
    CLIArgs* args = (CLIArgs*)malloc(sizeof(CLIArgs));
    if (!args) return NULL;
    
    // Initialize with defaults
    args->input_file = NULL;
    args->output_file = NULL;
    args->filter_type = NULL;
    args->brightness = 50;
    args->valid = 0;
    
    // Parse arguments
    for (int i = 1; i < g_argc; i++) {
        if (strcmp(g_argv[i], "--input") == 0 && i + 1 < g_argc) {
            args->input_file = strdup(g_argv[i + 1]);
            i++; // Skip next arg
        } else if (strcmp(g_argv[i], "--output") == 0 && i + 1 < g_argc) {
            args->output_file = strdup(g_argv[i + 1]);
            i++; // Skip next arg
        } else if (strcmp(g_argv[i], "--filter") == 0 && i + 1 < g_argc) {
            args->filter_type = strdup(g_argv[i + 1]);
            i++; // Skip next arg
        } else if (strcmp(g_argv[i], "--brightness") == 0 && i + 1 < g_argc) {
            args->brightness = atoi(g_argv[i + 1]);
            i++; // Skip next arg
        }
    }
    
    // Set defaults if not provided
    if (!args->input_file) {
        args->input_file = strdup("input.pgm");
    }
    if (!args->output_file) {
        args->output_file = strdup("output.pgm");
    }
    if (!args->filter_type) {
        args->filter_type = strdup("brightness");
    }
    
    args->valid = 1;
    return args;
}

// Free CLI args structure
void free_cli_args(CLIArgs* args) {
    if (!args) return;
    
    if (args->input_file) free(args->input_file);
    if (args->output_file) free(args->output_file);
    if (args->filter_type) free(args->filter_type);
    free(args);
}

// Free command line argument string (for get_command_line_arg)
void free_command_line_arg(char* arg) {
    if (arg) {
        free(arg);
    }
}

// Free command line arguments array (for get_command_line_args)
void free_command_line_args(char** args) {
    if (!args) return;
    
    int i = 0;
    while (args[i] != NULL) {
        free(args[i]);
        i++;
    }
    free(args);
}

// Get current time in microseconds
int64_t get_time_microseconds() {
    struct timeval tv;
    gettimeofday(&tv, NULL);
    return (int64_t)tv.tv_sec * 1000000 + (int64_t)tv.tv_usec;
}

// Get current time in milliseconds
int64_t get_time_milliseconds() {
    return get_time_microseconds() / 1000;
}

// Get memory usage in bytes
int64_t get_memory_usage() {
    struct rusage usage;
    if (getrusage(RUSAGE_SELF, &usage) == 0) {
        // ru_maxrss is in kilobytes on Linux, bytes on other systems
        #ifdef __linux__
        return (int64_t)usage.ru_maxrss * 1024;
        #else
        return (int64_t)usage.ru_maxrss;
        #endif
    }
    return -1;
}

// Print help message
void print_help() {
    printf("Eä Image Filter - SIMD-accelerated image processing\n\n");
    printf("Usage: ea-imagefilter [OPTIONS]\n\n");
    printf("Options:\n");
    printf("  --input FILE     Input PGM file (default: input.pgm)\n");
    printf("  --output FILE    Output PGM file (default: output.pgm)\n");
    printf("  --filter TYPE    Filter type: brightness, blur, edge, sharpen\n");
    printf("  --brightness N   Brightness adjustment value (default: 50)\n");
    printf("  --help           Show this help message\n\n");
    printf("Examples:\n");
    printf("  ea-imagefilter --input photo.pgm --output bright.pgm --filter brightness\n");
    printf("  ea-imagefilter --input photo.pgm --output edge.pgm --filter edge\n");
}

// Check if help was requested
int32_t is_help_requested() {
    for (int i = 1; i < g_argc; i++) {
        if (strcmp(g_argv[i], "--help") == 0 || strcmp(g_argv[i], "-h") == 0) {
            return 1;
        }
    }
    return 0;
}

// Cleanup function for test files
int32_t cleanup_test_files() {
    int result = 0;
    
    // Remove test files
    if (unlink("test_input.pgm") != 0) {
        result = -1; // Non-fatal error
    }
    if (unlink("test_output.pgm") != 0) {
        result = -1; // Non-fatal error
    }
    
    return result;
}

// Exit with error message
void exit_with_error(const char* message) {
    fprintf(stderr, "Error: %s\n", message);
    exit(1);
}

// Exit with success message
void exit_with_success(const char* message) {
    printf("Success: %s\n", message);
    exit(0);
}