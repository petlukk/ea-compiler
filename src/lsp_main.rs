//! Main entry point for the Eä Language Server
//!
//! This binary provides LSP (Language Server Protocol) support for Eä,
//! enabling intelligent code completion, real-time error checking, and
//! performance analysis in editors like VS Code.

#[cfg(feature = "lsp")]
#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::init();

    // Run the LSP server
    ea_compiler::lsp::run_lsp_server().await;
}

#[cfg(not(feature = "lsp"))]
fn main() {
    eprintln!("LSP server support not compiled in. Please build with --features=lsp");
    std::process::exit(1);
}
