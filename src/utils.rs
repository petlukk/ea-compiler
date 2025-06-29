//! Utility functions and helpers for the Eä compiler.

use std::path::Path;

/// Checks if a file has a specific extension.
pub fn has_extension<P: AsRef<Path>>(path: P, ext: &str) -> bool {
    path.as_ref()
        .extension()
        .and_then(|e| e.to_str())
        .map_or(false, |e| e.eq_ignore_ascii_case(ext))
}

/// Normalizes path separators to forward slashes.
pub fn normalize_path<P: AsRef<Path>>(path: P) -> String {
    path.as_ref()
        .to_string_lossy()
        .replace('\\', "/")
}

/// Formats a diagnostic message with file location.
pub fn format_diagnostic(
    file: &str,
    line: usize,
    column: usize,
    message: &str,
) -> String {
    format!("{}:{}:{}: {}", file, line, column, message)
}
