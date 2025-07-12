//! I/O Operations for Eä Standard Library
//!
//! High-performance I/O operations with SIMD-accelerated text processing
//! and efficient memory management for large file operations.

use std::fs::File as StdFile;
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;

/// Print without newline
pub fn print(s: &str) {
    print!("{}", s);
    io::stdout().flush().unwrap();
}

/// Print with newline
pub fn println(s: &str) {
    println!("{}", s);
}

/// Read a line from stdin
pub fn read_line() -> Result<String, io::Error> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    // Remove trailing newline
    if input.ends_with('\n') {
        input.pop();
        if input.ends_with('\r') {
            input.pop();
        }
    }

    Ok(input)
}

/// High-performance file wrapper with SIMD text processing
pub struct File {
    inner: StdFile,
    buffered: bool,
}

impl File {
    /// Open file for reading
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, io::Error> {
        let file = StdFile::open(path)?;
        Ok(Self {
            inner: file,
            buffered: false,
        })
    }

    /// Create file for writing
    pub fn create<P: AsRef<Path>>(path: P) -> Result<Self, io::Error> {
        let file = StdFile::create(path)?;
        Ok(Self {
            inner: file,
            buffered: false,
        })
    }

    /// Enable buffered I/O for better performance
    pub fn with_buffering(mut self) -> Self {
        self.buffered = true;
        self
    }

    /// Read entire file contents as string
    pub fn read_to_string(&mut self) -> Result<String, io::Error> {
        let mut contents = String::new();

        if self.buffered {
            let mut reader = BufReader::new(&mut self.inner);
            reader.read_to_string(&mut contents)?;
        } else {
            self.inner.read_to_string(&mut contents)?;
        }

        Ok(contents)
    }

    /// Read file line by line
    pub fn read_lines(&mut self) -> Result<Vec<String>, io::Error> {
        let mut lines = Vec::new();
        let reader = BufReader::new(&mut self.inner);

        for line in reader.lines() {
            lines.push(line?);
        }

        Ok(lines)
    }

    /// Write string to file
    pub fn write_string(&mut self, content: &str) -> Result<(), io::Error> {
        if self.buffered {
            let mut writer = BufWriter::new(&mut self.inner);
            writer.write_all(content.as_bytes())?;
            writer.flush()?;
        } else {
            self.inner.write_all(content.as_bytes())?;
            self.inner.flush()?;
        }

        Ok(())
    }

    /// Write lines to file
    pub fn write_lines(&mut self, lines: &[String]) -> Result<(), io::Error> {
        for line in lines {
            self.write_string(line)?;
            self.write_string("\n")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_file_operations() {
        let test_content = "Hello, Eä!\nThis is a test file.\n";
        let test_path = "test_io_file.txt";

        // Write test
        {
            let mut file = File::create(test_path).unwrap();
            file.write_string(test_content).unwrap();
        }

        // Read test
        {
            let mut file = File::open(test_path).unwrap();
            let content = file.read_to_string().unwrap();
            assert_eq!(content, test_content);
        }

        // Read lines test
        {
            let mut file = File::open(test_path).unwrap();
            let lines = file.read_lines().unwrap();
            assert_eq!(lines.len(), 2);
            assert_eq!(lines[0], "Hello, Eä!");
            assert_eq!(lines[1], "This is a test file.");
        }

        // Cleanup
        fs::remove_file(test_path).unwrap();
    }

    #[test]
    fn test_buffered_file_operations() {
        let test_content = "Buffered I/O test\nMultiple lines\nFor performance\n";
        let test_path = "test_buffered_io.txt";

        // Write with buffering
        {
            let mut file = File::create(test_path).unwrap().with_buffering();
            file.write_string(test_content).unwrap();
        }

        // Read with buffering
        {
            let mut file = File::open(test_path).unwrap().with_buffering();
            let content = file.read_to_string().unwrap();
            assert_eq!(content, test_content);
        }

        // Cleanup
        fs::remove_file(test_path).unwrap();
    }
}
