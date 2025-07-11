// Build script for the Eä compiler
// Compiles the C runtime for Vec, HashMap, HashSet, and String operations

fn main() {
    println!("cargo:rerun-if-changed=src/runtime/vec_runtime.c");
    println!("cargo:rerun-if-changed=src/runtime/hashmap_runtime.c");
    println!("cargo:rerun-if-changed=src/runtime/hashset_runtime.c");
    println!("cargo:rerun-if-changed=src/runtime/string_runtime.c");
    
    // Use cc crate to compile the C runtime
    cc::Build::new()
        .file("src/runtime/vec_runtime.c")
        .file("src/runtime/hashmap_runtime.c")
        .file("src/runtime/hashset_runtime.c")
        .file("src/runtime/string_runtime.c")
        .opt_level(2)                    // Optimize for performance
        .flag("-std=c99")               // Use C99 standard
        .flag("-Wall")                  // Enable warnings
        .flag("-Wextra")               // Extra warnings
        .compile("ea_runtime");         // Output library name
    
    // The cc crate automatically handles linking
    println!("cargo:rustc-link-lib=static=ea_runtime");
}