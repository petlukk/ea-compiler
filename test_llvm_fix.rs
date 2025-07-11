// Test the LLVM fix directly
use ea_compiler::compile_to_llvm;
use std::fs;

fn main() {
    let source = r#"
func abs_value(x: i32) -> i32 {
    if (x >= 0) {
        return x;
    } else {
        return -x;
    }
}
"#;
    
    println!("Testing LLVM compilation with control flow...");
    match compile_to_llvm(source, "test_llvm_fix") {
        Ok(_) => {
            println!("✅ LLVM compilation succeeded!");
            
            // Check if the .ll file was created
            if let Ok(content) = fs::read_to_string("test_llvm_fix.ll") {
                println!("✅ LLVM IR file created successfully");
                println!("File size: {} bytes", content.len());
                
                // Check for key patterns
                if content.contains("abs_value") {
                    println!("✅ Function 'abs_value' found in LLVM IR");
                }
                if content.contains("if_then") && content.contains("if_else") {
                    println!("✅ Control flow blocks found in LLVM IR");
                }
            } else {
                println!("❌ LLVM IR file was not created");
            }
        }
        Err(e) => {
            println!("❌ LLVM compilation failed: {:?}", e);
        }
    }
    
    // Clean up
    let _ = fs::remove_file("test_llvm_fix.ll");
}