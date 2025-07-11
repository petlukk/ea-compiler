// Minimal test to isolate the SIGSEGV issue
#[cfg(feature = "llvm")]
fn test_compile_control_flow_minimal() {
    use ea_compiler::compile_to_llvm;
    
    let source = r#"
func abs_value(x: i32) -> i32 {
    if (x >= 0) {
        return x;
    } else {
        return -x;
    }
}
"#;

    println!("About to compile control flow...");
    let result = compile_to_llvm(source, "test_control_flow_minimal");
    println!("Result: {:?}", result);
    
    // Clean up
    let _ = std::fs::remove_file("test_control_flow_minimal.ll");
}

#[cfg(feature = "llvm")]
fn main() {
    test_compile_control_flow_minimal();
}