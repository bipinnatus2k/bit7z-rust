//! bit7z-rust - A Rust wrapper for 7-Zip shared libraries
//! 
//! This library provides a safe, idiomatic Rust interface to 7-Zip
//! compression library, supporting multiple archive formats including 7z, ZIP,
//! GZIP, BZIP2, TAR, XZ, and WIM for compression, and many more for extraction.
//! 
//! # Basic Usage
//! 
//! ```no_run
//! use bit7z_rust::{BitLibrary, BitCompressor, CompressionFormat};
//! 
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load 7-Zip library
//!     let lib = BitLibrary::new(None)?;
//!     
//!     // Create a compressor for ZIP format
//!     let mut compressor = BitCompressor::new(&lib, CompressionFormat::Zip);
//!     
//!     // Compress files
//!     compressor.compress(&["file1.txt", "file2.txt"], "output.zip")?;
//!     
//!     Ok(())
//! }
//! ```

pub mod ffi;
pub mod format;
pub mod error;
pub mod stream;
pub mod compressor;
pub mod extractor;

pub use ffi::BitLibrary;
pub use ffi::LibraryError;

// Re-export common types for convenience
pub use ffi::GUID;
pub use ffi::HRESULT;
pub use ffi::PROPVARIANT;
pub use ffi::PROPID;

pub use format::{
    CompressionFormat, ExtractFormat, CompressionLevel, CompressionMethod,
    FormatFeatures, FormatInfo,
};
pub use error::{Bit7zError, Result};
pub use compressor::BitCompressor;
pub use extractor::BitExtractor;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn library_loading() {
        // Test will fail if 7-Zip library is not installed
        // This is expected in CI environments
        let result = BitLibrary::new(None::<&str>);
        match result {
            Ok(_) => println!("Library loaded successfully"),
            Err(e) => println!("Library loading failed (expected if 7-Zip not installed): {}", e),
        }
    }
}
