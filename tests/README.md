# Eä Compiler

A compiler for the Eä programming language.

## Project Structure

```
eä-compiler/
├── .vscode/               # VSCode configuration
│   ├── settings.json     # Workspace settings
│   ├── tasks.json        # Build tasks
│   ├── launch.json       # Debug configurations
│   └── extensions.json   # Recommended extensions
├── src/                  # Source code
│   ├── lexer/           # Lexical analysis
│   ├── parser/          # Syntax analysis
│   ├── type_system/     # Type checking
│   ├── codegen/         # Code generation
│   ├── error.rs         # Error handling
│   ├── utils.rs         # Utility functions
│   ├── lib.rs           # Library root
│   └── main.rs          # Binary entry point
├── tests/               # Integration tests
├── examples/            # Example Eä programs
├── docs/                # Documentation
├── Cargo.toml           # Project manifest
└── README.md            # This file
```

## Building

```bash
# Build in debug mode
cargo build

# Build in release mode
cargo build --release
```

## Running

```bash
# Run the compiler
cargo run -- [options] <input_file>
```

## Testing

```bash
# Run all tests
cargo test

# Run tests with detailed output
cargo test -- --nocapture
```

## Documentation

```bash
# Generate documentation
cargo doc --open
```

## License

Licensed under either of:

 * Apache License, Version 2.0
 * MIT license

at your option.
