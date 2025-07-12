// Minimal reproduction of the SIGSEGV issue
// Following DEVELOPMENT_PROCESS.md - create end-to-end test FIRST

#[cfg(feature = "llvm")]
use ea_compiler::compile_to_llvm;

#[cfg(feature = "llvm")]
#[test]
fn test_control_flow_minimal() {
    // This is the exact code that causes SIGSEGV
    let source = r#"
func abs_value(x: i32) -> i32 {
    if (x >= 0) {
        return x;
    } else {
        return -x;
    }
}
"#;

    println!("About to compile control flow test...");
    let result = compile_to_llvm(source, "debug_sigsegv");
    println!("Compilation result: {:?}", result);

    // If we get here without SIGSEGV, the compilation succeeded
    assert!(result.is_ok(), "Control flow should compile successfully");

    // Clean up
    let _ = std::fs::remove_file("debug_sigsegv.ll");
}
