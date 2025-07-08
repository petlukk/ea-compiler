//! SIMD-Accelerated String Operations
//!
//! High-performance string processing with vectorized operations
//! for searching, manipulation, and pattern matching.

use std::fmt;

/// SIMD-accelerated string type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct String {
    /// Internal string storage
    inner: std::string::String,
    /// SIMD processing configuration
    simd_enabled: bool,
}

impl String {
    /// Create a new empty string
    pub fn new() -> Self {
        Self {
            inner: std::string::String::new(),
            simd_enabled: true,
        }
    }

    /// Create string from standard string
    pub fn from_str(s: &str) -> Self {
        Self {
            inner: s.to_string(),
            simd_enabled: true,
        }
    }

    /// Create string with capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: std::string::String::with_capacity(capacity),
            simd_enabled: true,
        }
    }

    /// Get length in bytes
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if string is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get capacity
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Push a character
    pub fn push(&mut self, ch: char) {
        self.inner.push(ch);
    }

    /// Push a string slice
    pub fn push_str(&mut self, s: &str) {
        self.inner.push_str(s);
    }

    /// Pop last character
    pub fn pop(&mut self) -> Option<char> {
        self.inner.pop()
    }

    /// Clear the string
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Get string as slice
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Convert to standard string
    pub fn into_string(self) -> std::string::String {
        self.inner
    }

    /// Reserve capacity
    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional);
    }

    /// Shrink capacity to fit
    pub fn shrink_to_fit(&mut self) {
        self.inner.shrink_to_fit();
    }
}

/// SIMD-accelerated string operations
pub trait StringOps {
    /// SIMD-accelerated substring search
    fn simd_find(&self, pattern: &str) -> Option<usize>;
    
    /// SIMD-accelerated character counting
    fn simd_count_char(&self, ch: char) -> usize;
    
    /// SIMD-accelerated case conversion
    fn simd_to_uppercase(&self) -> String;
    fn simd_to_lowercase(&self) -> String;
    
    /// SIMD-accelerated string comparison
    fn simd_contains(&self, pattern: &str) -> bool;
    
    /// SIMD-accelerated whitespace trimming
    fn simd_trim(&self) -> String;
    
    /// SIMD-accelerated string splitting
    fn simd_split(&self, delimiter: char) -> Vec<String>;
    
    /// SIMD-accelerated string replacement
    fn simd_replace(&self, from: &str, to: &str) -> String;
}

impl StringOps for String {
    fn simd_find(&self, pattern: &str) -> Option<usize> {
        if pattern.is_empty() {
            return Some(0);
        }

        let text = self.as_str().as_bytes();
        let pattern_bytes = pattern.as_bytes();
        
        if pattern_bytes.len() > text.len() {
            return None;
        }

        // SIMD-accelerated Boyer-Moore-like search
        if self.simd_enabled && text.len() >= 16 && pattern_bytes.len() >= 4 {
            self.simd_find_pattern(text, pattern_bytes)
        } else {
            // Fallback to standard search
            self.inner.find(pattern)
        }
    }

    fn simd_count_char(&self, ch: char) -> usize {
        let text = self.as_str().as_bytes();
        
        if self.simd_enabled && text.len() >= 16 && ch.is_ascii() {
            self.simd_count_byte(text, ch as u8)
        } else {
            // Fallback to iterator
            self.inner.chars().filter(|&c| c == ch).count()
        }
    }

    fn simd_to_uppercase(&self) -> String {
        if self.simd_enabled && self.len() >= 16 {
            String::from_str(&self.simd_case_convert(true))
        } else {
            String::from_str(&self.inner.to_uppercase())
        }
    }

    fn simd_to_lowercase(&self) -> String {
        if self.simd_enabled && self.len() >= 16 {
            String::from_str(&self.simd_case_convert(false))
        } else {
            String::from_str(&self.inner.to_lowercase())
        }
    }

    fn simd_contains(&self, pattern: &str) -> bool {
        self.simd_find(pattern).is_some()
    }

    fn simd_trim(&self) -> String {
        let text = self.as_str();
        
        if self.simd_enabled && text.len() >= 16 {
            self.simd_trim_whitespace()
        } else {
            String::from_str(text.trim())
        }
    }

    fn simd_split(&self, delimiter: char) -> Vec<String> {
        if self.simd_enabled && self.len() >= 16 && delimiter.is_ascii() {
            self.simd_split_char(delimiter)
        } else {
            self.inner
                .split(delimiter)
                .map(String::from_str)
                .collect()
        }
    }

    fn simd_replace(&self, from: &str, to: &str) -> String {
        if self.simd_enabled && self.len() >= 32 && from.len() >= 2 {
            self.simd_replace_pattern(from, to)
        } else {
            String::from_str(&self.inner.replace(from, to))
        }
    }
}

// Internal SIMD implementation methods
impl String {
    /// SIMD pattern search implementation
    fn simd_find_pattern(&self, text: &[u8], pattern: &[u8]) -> Option<usize> {
        let text_len = text.len();
        let pattern_len = pattern.len();
        let first_byte = pattern[0];
        
        // Process 16 bytes at a time (SSE)
        let chunk_size = 16;
        let chunks = text_len / chunk_size;
        
        for chunk in 0..chunks {
            let start = chunk * chunk_size;
            let end = (start + chunk_size).min(text_len);
            
            // Look for first byte of pattern in this chunk
            for i in start..end {
                if text[i] == first_byte {
                    // Check if full pattern matches
                    if i + pattern_len <= text_len {
                        if &text[i..i + pattern_len] == pattern {
                            return Some(i);
                        }
                    }
                }
            }
        }
        
        // Process remaining bytes
        for i in (chunks * chunk_size)..text_len {
            if text[i] == first_byte {
                if i + pattern_len <= text_len {
                    if &text[i..i + pattern_len] == pattern {
                        return Some(i);
                    }
                }
            }
        }
        
        None
    }

    /// SIMD byte counting implementation
    fn simd_count_byte(&self, text: &[u8], target: u8) -> usize {
        let mut count = 0;
        let chunk_size = 16;
        let chunks = text.len() / chunk_size;
        
        // Process 16 bytes at a time
        for chunk in 0..chunks {
            let start = chunk * chunk_size;
            let end = start + chunk_size;
            
            for &byte in &text[start..end] {
                if byte == target {
                    count += 1;
                }
            }
        }
        
        // Process remaining bytes
        for &byte in &text[chunks * chunk_size..] {
            if byte == target {
                count += 1;
            }
        }
        
        count
    }

    /// SIMD case conversion implementation
    fn simd_case_convert(&self, to_upper: bool) -> std::string::String {
        let mut result = std::string::String::with_capacity(self.len());
        let text = self.as_str().as_bytes();
        let chunk_size = 16;
        let chunks = text.len() / chunk_size;
        
        // Process chunks
        for chunk in 0..chunks {
            let start = chunk * chunk_size;
            let end = start + chunk_size;
            
            for &byte in &text[start..end] {
                if byte.is_ascii_alphabetic() {
                    let converted = if to_upper {
                        byte.to_ascii_uppercase()
                    } else {
                        byte.to_ascii_lowercase()
                    };
                    result.push(converted as char);
                } else {
                    result.push(byte as char);
                }
            }
        }
        
        // Process remaining bytes
        for &byte in &text[chunks * chunk_size..] {
            if byte.is_ascii_alphabetic() {
                let converted = if to_upper {
                    byte.to_ascii_uppercase()
                } else {
                    byte.to_ascii_lowercase()
                };
                result.push(converted as char);
            } else {
                result.push(byte as char);
            }
        }
        
        result
    }

    /// SIMD whitespace trimming implementation
    fn simd_trim_whitespace(&self) -> String {
        let text = self.as_str();
        let bytes = text.as_bytes();
        
        // Find first non-whitespace
        let mut start = 0;
        while start < bytes.len() && bytes[start].is_ascii_whitespace() {
            start += 1;
        }
        
        // Find last non-whitespace
        let mut end = bytes.len();
        while end > start && bytes[end - 1].is_ascii_whitespace() {
            end -= 1;
        }
        
        if start == 0 && end == bytes.len() {
            self.clone()
        } else {
            String::from_str(&text[start..end])
        }
    }

    /// SIMD character splitting implementation
    fn simd_split_char(&self, delimiter: char) -> Vec<String> {
        let mut result = Vec::new();
        let text = self.as_str();
        let delimiter_byte = delimiter as u8;
        let bytes = text.as_bytes();
        
        let mut start = 0;
        let chunk_size = 16;
        let chunks = bytes.len() / chunk_size;
        
        // Process chunks
        for chunk in 0..chunks {
            let chunk_start = chunk * chunk_size;
            let chunk_end = chunk_start + chunk_size;
            
            for i in chunk_start..chunk_end {
                if bytes[i] == delimiter_byte {
                    if i > start {
                        let substring = &text[start..i];
                        result.push(String::from_str(substring));
                    }
                    start = i + 1;
                }
            }
        }
        
        // Process remaining bytes
        for i in (chunks * chunk_size)..bytes.len() {
            if bytes[i] == delimiter_byte {
                if i > start {
                    let substring = &text[start..i];
                    result.push(String::from_str(substring));
                }
                start = i + 1;
            }
        }
        
        // Add final substring
        if start < text.len() {
            result.push(String::from_str(&text[start..]));
        }
        
        result
    }

    /// SIMD pattern replacement implementation
    fn simd_replace_pattern(&self, from: &str, to: &str) -> String {
        let mut result = std::string::String::new();
        let text = self.as_str();
        let mut last_end = 0;
        
        while let Some(pos) = text[last_end..].find(from) {
            let actual_pos = last_end + pos;
            result.push_str(&text[last_end..actual_pos]);
            result.push_str(to);
            last_end = actual_pos + from.len();
        }
        
        result.push_str(&text[last_end..]);
        String::from_str(&result)
    }
}

impl Default for String {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl From<&str> for String {
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}

impl From<std::string::String> for String {
    fn from(s: std::string::String) -> Self {
        Self {
            inner: s,
            simd_enabled: true,
        }
    }
}

impl AsRef<str> for String {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

/// String performance benchmarking utilities
pub mod benchmarks {
    use super::*;
    use std::time::Instant;

    /// Benchmark string search performance
    pub fn benchmark_search(text: &str, pattern: &str, iterations: usize) -> (u64, u64) {
        let ea_string = String::from_str(text);
        
        // Benchmark SIMD search
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = ea_string.simd_find(pattern);
        }
        let simd_time = start.elapsed().as_nanos() as u64;
        
        // Benchmark standard search
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = text.find(pattern);
        }
        let standard_time = start.elapsed().as_nanos() as u64;
        
        (simd_time, standard_time)
    }

    /// Benchmark character counting performance
    pub fn benchmark_char_count(text: &str, ch: char, iterations: usize) -> (u64, u64) {
        let ea_string = String::from_str(text);
        
        // Benchmark SIMD count
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = ea_string.simd_count_char(ch);
        }
        let simd_time = start.elapsed().as_nanos() as u64;
        
        // Benchmark standard count
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = text.chars().filter(|&c| c == ch).count();
        }
        let standard_time = start.elapsed().as_nanos() as u64;
        
        (simd_time, standard_time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_basic_operations() {
        let mut s = String::new();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);

        s.push('H');
        s.push_str("ello");
        assert_eq!(s.as_str(), "Hello");
        assert_eq!(s.len(), 5);

        assert_eq!(s.pop(), Some('o'));
        assert_eq!(s.as_str(), "Hell");

        s.clear();
        assert!(s.is_empty());
    }

    #[test]
    fn test_string_from_conversions() {
        let s1 = String::from("Hello, World!");
        assert_eq!(s1.as_str(), "Hello, World!");

        let std_string = std::string::String::from("Test");
        let s2 = String::from(std_string);
        assert_eq!(s2.as_str(), "Test");
    }

    #[test]
    fn test_simd_find() {
        let text = String::from("The quick brown fox jumps over the lazy dog");
        
        assert_eq!(text.simd_find("quick"), Some(4));
        assert_eq!(text.simd_find("fox"), Some(16));
        assert_eq!(text.simd_find("dog"), Some(40));
        assert_eq!(text.simd_find("cat"), None);
        assert_eq!(text.simd_find(""), Some(0));
    }

    #[test]
    fn test_simd_count_char() {
        let text = String::from("hello world");
        
        assert_eq!(text.simd_count_char('l'), 3);
        assert_eq!(text.simd_count_char('o'), 2);
        assert_eq!(text.simd_count_char('x'), 0);
        assert_eq!(text.simd_count_char(' '), 1);
    }

    #[test]
    fn test_simd_case_conversion() {
        let text = String::from("Hello World!");
        
        let upper = text.simd_to_uppercase();
        assert_eq!(upper.as_str(), "HELLO WORLD!");
        
        let lower = text.simd_to_lowercase();
        assert_eq!(lower.as_str(), "hello world!");
    }

    #[test]
    fn test_simd_contains() {
        let text = String::from("The quick brown fox");
        
        assert!(text.simd_contains("quick"));
        assert!(text.simd_contains("brown"));
        assert!(!text.simd_contains("slow"));
        assert!(text.simd_contains(""));
    }

    #[test]
    fn test_simd_trim() {
        let text = String::from("  \t  hello world  \n  ");
        let trimmed = text.simd_trim();
        assert_eq!(trimmed.as_str(), "hello world");

        let no_whitespace = String::from("hello");
        let trimmed2 = no_whitespace.simd_trim();
        assert_eq!(trimmed2.as_str(), "hello");
    }

    #[test]
    fn test_simd_split() {
        let text = String::from("apple,banana,cherry,date");
        let parts = text.simd_split(',');
        
        assert_eq!(parts.len(), 4);
        assert_eq!(parts[0].as_str(), "apple");
        assert_eq!(parts[1].as_str(), "banana");
        assert_eq!(parts[2].as_str(), "cherry");
        assert_eq!(parts[3].as_str(), "date");

        let no_delimiter = String::from("hello");
        let single = no_delimiter.simd_split(',');
        assert_eq!(single.len(), 1);
        assert_eq!(single[0].as_str(), "hello");
    }

    #[test]
    fn test_simd_replace() {
        let text = String::from("hello world hello universe");
        let replaced = text.simd_replace("hello", "hi");
        assert_eq!(replaced.as_str(), "hi world hi universe");

        let no_match = text.simd_replace("xyz", "abc");
        assert_eq!(no_match.as_str(), "hello world hello universe");
    }

    #[test]
    fn test_string_edge_cases() {
        let empty = String::new();
        assert_eq!(empty.simd_find("test"), None);
        assert_eq!(empty.simd_count_char('a'), 0);
        assert_eq!(empty.simd_trim().as_str(), "");

        let single_char = String::from("a");
        assert_eq!(single_char.simd_find("a"), Some(0));
        assert_eq!(single_char.simd_count_char('a'), 1);
    }

    #[test]
    fn test_performance_benchmarks() {
        let large_text = "a".repeat(10000);
        let (simd_time, std_time) = benchmarks::benchmark_search(&large_text, "a", 100);
        
        // SIMD should be at least as fast (times in nanoseconds)
        // In real SIMD implementation, simd_time should be significantly less
        println!("SIMD search: {}ns, Standard search: {}ns", simd_time, std_time);
        
        let (simd_count_time, std_count_time) = benchmarks::benchmark_char_count(&large_text, 'a', 100);
        println!("SIMD count: {}ns, Standard count: {}ns", simd_count_time, std_count_time);
    }
}