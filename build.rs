use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=src/runtime/");
    
    // Get the output directory
    let out_dir = env::var("OUT_DIR").unwrap();
    
    // Configure the C compiler
    let mut builder = cc::Build::new();
    
    // Set compiler flags
    builder
        .include("src/runtime")
        .flag("-std=c99")
        .flag("-fPIC")
        .flag("-O2")
        .flag("-Wall")
        .flag("-Wextra");
    
    // Add debug flags in debug mode
    if env::var("PROFILE").unwrap() == "debug" {
        builder.flag("-g").flag("-DDEBUG");
    }
    
    // Compile all C runtime files
    builder.file("src/runtime/vec_runtime.c");
    builder.file("src/runtime/string_runtime.c");
    builder.file("src/runtime/hashmap_runtime.c");
    builder.file("src/runtime/hashset_runtime.c");
    builder.file("src/runtime/file_runtime.c");
    builder.file("src/runtime/cli_runtime.c");
    // PGM runtime removed - applications should implement PGM parsing using core features
    
    // Compile the runtime library
    builder.compile("ea_runtime");
    
    // Link with system libraries
    println!("cargo:rustc-link-lib=c");
    println!("cargo:rustc-link-lib=m");
    
    // Tell cargo to look for the compiled library in the output directory
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=ea_runtime");
    
    println!("cargo:rustc-env=EA_RUNTIME_COMPILED=1");
}