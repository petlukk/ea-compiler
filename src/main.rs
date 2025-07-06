use ea_compiler::{NAME, VERSION};
use std::env;
use std::fs;
use std::path::Path;
use std::process;
use std::time::Instant;

#[cfg(feature = "llvm")]
use ea_compiler::{compile_to_llvm, diagnose_jit_execution, jit_execute};

/// Command line arguments
struct Args {
    input_file: Option<String>,
    output_file: Option<String>,
    emit_ast: bool,
    emit_tokens: bool,
    emit_llvm: bool,
    emit_llvm_only: bool,
    run: bool,
    diagnose_jit: bool,
    verbose: bool,
    quiet: bool,
    help: bool,
    version: bool,
    run_tests: bool,
}

impl Args {
    fn parse() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut parsed = Args {
            input_file: None,
            output_file: None,
            emit_ast: false,
            emit_tokens: false,
            emit_llvm: false,
            emit_llvm_only: false,
            run: false,
            diagnose_jit: false,
            verbose: false,
            quiet: false,
            help: false,
            version: false,
            run_tests: false,
        };

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--help" | "-h" => parsed.help = true,
                "--version" | "-V" => parsed.version = true,
                "--verbose" | "-v" => parsed.verbose = true,
                "--quiet" | "-q" => parsed.quiet = true,
                "--emit-ast" => parsed.emit_ast = true,
                "--emit-tokens" => parsed.emit_tokens = true,
                "--emit-llvm" => parsed.emit_llvm = true,
                "--emit-llvm-only" => parsed.emit_llvm_only = true,
                "--run" | "-r" => parsed.run = true,
                "--diagnose-jit" => parsed.diagnose_jit = true,
                "--test" => parsed.run_tests = true,
                "--output" | "-o" => {
                    if i + 1 < args.len() {
                        parsed.output_file = Some(args[i + 1].clone());
                        i += 1;
                    } else {
                        eprintln!("Error: --output requires a filename");
                        process::exit(1);
                    }
                }
                arg if arg.starts_with('-') => {
                    eprintln!("Error: Unknown option '{}'", arg);
                    process::exit(1);
                }
                _ => {
                    if parsed.input_file.is_none() {
                        parsed.input_file = Some(args[i].clone());
                    } else {
                        eprintln!("Error: Multiple input files not supported");
                        process::exit(1);
                    }
                }
            }
            i += 1;
        }

        parsed
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.help {
        print_help();
        return Ok(());
    }

    if args.version {
        print_version();
        return Ok(());
    }

    if args.run_tests {
        run_builtin_tests();
        return Ok(());
    }

    match args.input_file.as_ref() {
        Some(filename) => compile_file(filename, &args)?,
        None => {
            if env::args().len() == 1 {
                // No arguments - show usage
                print_usage();
            } else {
                eprintln!("Error: No input file specified");
                process::exit(1);
            }
        }
    }

    Ok(())
}

fn print_help() {
    println!("{} v{}", NAME, VERSION);
    println!("A compiler for the Eä programming language");
    println!();
    println!("USAGE:");
    println!("    ea [OPTIONS] <INPUT_FILE>");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help          Print help information");
    println!("    -V, --version       Print version information");
    println!("    -v, --verbose       Enable verbose output");
    println!("    -q, --quiet         Suppress diagnostic messages");
    println!("    -r, --run           Compile and execute immediately (JIT)");
    println!("    -o, --output FILE   Specify output file");
    println!("        --emit-tokens   Print tokenization output");
    println!("        --emit-ast      Print AST output");
    println!("        --emit-llvm     Print LLVM IR output (with diagnostics)");
    println!("        --emit-llvm-only Print LLVM IR only (clean for piping)");
    println!("        --diagnose-jit  Diagnose JIT execution issues");
    println!("        --test          Run built-in compiler tests");
    println!();
    println!("EXAMPLES:");
    println!("    ea hello.ea                         # Compile hello.ea");
    println!("    ea --run fibonacci.ea               # Compile and execute immediately");
    println!("    ea --emit-ast program.ea            # Show AST for program.ea");
    println!("    ea --emit-llvm-only program.ea | lli  # Pipe clean IR to lli");
    println!("    ea --verbose fibonacci.ea           # Compile with verbose output");
    println!("    ea --test                           # Run compiler self-tests");
}

fn print_version() {
    println!("{} {}", NAME, VERSION);
}

fn print_usage() {
    println!("{} v{}", NAME, VERSION);
    println!("A systems programming language with built-in SIMD and memory safety");
    println!();
    println!("USAGE:");
    println!("    ea [OPTIONS] <INPUT_FILE>");
    println!();
    println!("Try 'ea --help' for more information.");
}

fn compile_file(filename: &str, args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    // Determine output mode
    let show_diagnostics = !args.quiet && !args.emit_llvm_only;
    let verbose_mode = args.verbose && show_diagnostics;

    if verbose_mode {
        eprintln!("🚀 {} v{}", NAME, VERSION);
        eprintln!("📁 Compiling: {}", filename);
    }

    // Check if file exists
    if !Path::new(filename).exists() {
        eprintln!("Error: File '{}' not found", filename);
        process::exit(1);
    }

    // Read source file
    let start_time = Instant::now();
    let source = fs::read_to_string(filename)?;

    if verbose_mode {
        eprintln!("📖 Read {} bytes from {}", source.len(), filename);
    }

    // Tokenization
    if verbose_mode || args.emit_tokens {
        eprintln!("🔍 Tokenizing...");
    }

    let tokens = ea_compiler::tokenize(&source)?;

    if args.emit_tokens {
        println!("📋 Tokens:");
        for (i, token) in tokens.iter().enumerate() {
            println!(
                "  {}: {:?} at {}:{}",
                i, token.kind, token.position.line, token.position.column
            );
        }
        println!();
    }

    // Parsing
    if verbose_mode {
        eprintln!("🌳 Parsing...");
    }

    let program = ea_compiler::parse(&source)?;

    if args.emit_ast {
        println!("🌳 Abstract Syntax Tree:");
        for (i, stmt) in program.iter().enumerate() {
            println!("  Statement {}: {}", i + 1, stmt);
        }
        println!();
    }

    // Type checking
    if verbose_mode {
        eprintln!("🎯 Type checking...");
    }

    let (_program, context) = ea_compiler::compile_to_ast(&source)?;

    if verbose_mode {
        eprintln!("✅ Type checking completed");
        eprintln!("   Functions: {}", context.functions.len());
        eprintln!("   Variables in scope: {}", context.variables.len());
    }

    // LLVM code generation (if available)
    #[cfg(feature = "llvm")]
    {
        if args.emit_llvm || args.emit_llvm_only {
            if verbose_mode {
                eprintln!("⚙️  Generating LLVM IR...");
            }

            let output_name = args
                .output_file
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or_else(|| {
                    Path::new(filename)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("output")
                });

            compile_to_llvm(&source, output_name)?;

            let ir_file = format!("{}.ll", output_name);
            if Path::new(&ir_file).exists() {
                let ir_content = fs::read_to_string(&ir_file)?;

                if args.emit_llvm_only {
                    // Clean output for piping - just the IR to stdout
                    print!("{}", ir_content);
                } else if args.emit_llvm {
                    // Regular emit-llvm with diagnostics
                    println!("🔧 LLVM IR:");
                    println!("{}", ir_content);
                }
            }

            if verbose_mode {
                eprintln!("📄 Generated LLVM IR: {}.ll", output_name);
            }
        } else if verbose_mode {
            eprintln!("⚙️  Generating LLVM IR...");

            let output_name = args
                .output_file
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or_else(|| {
                    Path::new(filename)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("output")
                });

            compile_to_llvm(&source, output_name)?;
            eprintln!("📄 Generated LLVM IR: {}.ll", output_name);
        }

        // Handle JIT diagnostics
        if args.diagnose_jit {
            if show_diagnostics {
                eprintln!("🔍 Diagnosing JIT execution...");
            }

            let output_name = args
                .output_file
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or_else(|| {
                    Path::new(filename)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("output")
                });

            match diagnose_jit_execution(&source, output_name) {
                Ok(diagnostics) => {
                    println!("🔍 JIT Execution Diagnostics:");
                    println!("{}", diagnostics);
                }
                Err(e) => {
                    eprintln!("❌ JIT diagnostic error: {}", e);
                    process::exit(1);
                }
            }
        }

        // Handle JIT execution
        if args.run {
            if show_diagnostics {
                eprintln!("🚀 Executing program...");
            }

            let output_name = args
                .output_file
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or_else(|| {
                    Path::new(filename)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("output")
                });

            match jit_execute(&source, output_name) {
                Ok(exit_code) => {
                    if verbose_mode {
                        eprintln!(
                            "✅ Program executed successfully with exit code: {}",
                            exit_code
                        );
                    }
                    if exit_code != 0 {
                        process::exit(exit_code);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Runtime error: {}", e);
                    process::exit(1);
                }
            }
        }
    }

    #[cfg(not(feature = "llvm"))]
    {
        if args.emit_llvm || args.emit_llvm_only || args.run || args.diagnose_jit {
            eprintln!("⚠️  LLVM code generation not available (compile with --features=llvm)");
        }
    }

    let elapsed = start_time.elapsed();

    if verbose_mode {
        eprintln!(
            "✅ Compilation completed in {:.2}ms",
            elapsed.as_secs_f64() * 1000.0
        );
    } else if show_diagnostics {
        eprintln!("✅ Compiled successfully");
    }
    // If emit_llvm_only, don't print any success message to keep output clean

    Ok(())
}

fn run_builtin_tests() {
    println!("🧪 Running built-in compiler tests...");
    println!();

    // Test 1: Simple arithmetic
    let test1 = r#"
func main() -> () {
    let result = 1 + 2 * 3;
    return;
}
"#;

    print!("📋 Test 1 (Arithmetic): ");
    match ea_compiler::compile_to_ast(test1) {
        Ok(_) => println!("✅ PASS"),
        Err(e) => {
            println!("❌ FAIL - {}", e);
            return;
        }
    }

    // Test 2: Function calls
    let test2 = r#"
func add(a: i32, b: i32) -> i32 {
    return a + b;
}

func main() -> () {
    let result = add(5, 10);
    return;
}
"#;

    print!("📋 Test 2 (Functions): ");
    match ea_compiler::compile_to_ast(test2) {
        Ok(_) => println!("✅ PASS"),
        Err(e) => {
            println!("❌ FAIL - {}", e);
            return;
        }
    }

    // Test 3: Control flow
    let test3 = r#"
func fibonacci(n: i32) -> i32 {
    if (n <= 1) {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

func main() -> () {
    let result = fibonacci(5);
    return;
}
"#;

    print!("📋 Test 3 (Control Flow): ");
    match ea_compiler::compile_to_ast(test3) {
        Ok(_) => println!("✅ PASS"),
        Err(e) => {
            println!("❌ FAIL - {}", e);
            return;
        }
    }

    // Test 4: Type error detection
    let test4 = r#"
func bad_function() -> i32 {
    return "hello";
}
"#;

    print!("📋 Test 4 (Error Detection): ");
    match ea_compiler::compile_to_ast(test4) {
        Ok(_) => println!("❌ FAIL - Should have detected type error"),
        Err(_) => println!("✅ PASS"),
    }

    println!();
    println!("🎉 All built-in tests completed!");
    println!();
    println!("💡 For comprehensive testing, run:");
    println!("   cargo test --features=llvm");
    println!("   cargo bench");
}
