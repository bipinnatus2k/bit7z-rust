//! Stream compressor for compressing data from streams
//!
//! This module provides BitStreamCompressor for compressing data from input streams into archives.

use crate::compressor::BitCompressor;
use crate::ffi::BitLibrary;
use crate::format::CompressionFormat;
use crate::error::{Bit7zError, Result};
use std::io::Read;
use std::vec::Vec;

/// Stream compressor for compressing data from input streams
///
/// This allows compressing data directly from input streams into archives.
pub struct BitStreamCompressor<'a> {
    compressor: BitCompressor<'a>,
}

impl<'a> BitStreamCompressor<'a> {
    /// Create a new stream compressor
    pub fn new(library: &'a BitLibrary, format: CompressionFormat) -> Self {
        BitStreamCompressor {
            compressor: BitCompressor::new(library, format),
        }
    }

    /// Set password for encryption
    pub fn password(&mut self, password: impl Into<String>) -> &mut Self {
        self.compressor.password(password);
        self
    }

    /// Set compression level
    pub fn compression_level(&mut self, level: crate::format::CompressionLevel) -> &mut Self {
        self.compressor.compression_level(level);
        self
    }

    /// Set compression method
    pub fn compression_method(&mut self, method: crate::format::CompressionMethod) -> &mut Self {
        self.compressor.compression_method(method);
        self
    }

    /// Set dictionary size
    pub fn dictionary_size(&mut self, size: u32) -> &mut Self {
        self.compressor.dictionary_size(size);
        self
    }

    /// Set word size
    pub fn word_size(&mut self, size: u32) -> &mut Self {
        self.compressor.word_size(size);
        self
    }

    /// Enable solid compression
    pub fn solid(&mut self, solid: bool) -> &mut Self {
        self.compressor.solid(solid);
        self
    }

    /// Enable header encryption (7z format only)
    pub fn crypt_headers(&mut self, encrypt: bool) -> &mut Self {
        self.compressor.crypt_headers(encrypt);
        self
    }

    /// Set volume size for multi-volume archives (0 = single volume)
    pub fn volume_size(&mut self, size_bytes: u64) -> &mut Self {
        self.compressor.volume_size(size_bytes);
        self
    }

    /// Set total callback - called with total size at start
    pub fn set_total_callback<F>(&mut self, callback: F) -> &mut Self
    where
        F: Fn(u64) + Send + Sync + 'static,
    {
        self.compressor.set_total_callback(callback);
        self
    }

    /// Set progress callback - called with (processed, total), returns true to continue
    pub fn set_progress_callback<F>(&mut self, callback: F) -> &mut Self
    where
        F: Fn(u64, u64) -> bool + Send + Sync + 'static,
    {
        self.compressor.set_progress_callback(callback);
        self
    }

    /// Set ratio callback - called with input and output sizes
    pub fn set_ratio_callback<F>(&mut self, callback: F) -> &mut Self
    where
        F: Fn(u64, u64) + Send + Sync + 'static,
    {
        self.compressor.set_ratio_callback(callback);
        self
    }

    /// Set file callback - called with file path before processing
    pub fn set_file_callback<F>(&mut self, callback: F) -> &mut Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.compressor.set_file_callback(callback);
        self
    }

    /// Set password callback - called when password is needed
    pub fn set_password_callback<F>(&mut self, callback: F) -> &mut Self
    where
        F: Fn() -> String + Send + Sync + 'static,
    {
        self.compressor.set_password_callback(callback);
        self
    }

    /// Compress data from a reader stream into an archive
    ///
    /// # Arguments
    /// * `reader` - Input stream to read data from
    /// * `item_name` - Name of the item in the archive
    ///
    /// # Returns
    /// * `Ok(Vec<u8>)` - Compressed archive data
    /// * `Err(Bit7zError)` - Error
    pub fn compress_from_stream<R: Read>(
        &self,
        mut reader: R,
        item_name: String,
    ) -> Result<Vec<u8>> {
        // Create a temporary file to store stream content
        let temp_path = std::env::temp_dir().join(format!(
            "bit7z_stream_{}.tmp",
            std::process::id()
        ));

        // Copy stream to temp file
        let mut temp_file = std::fs::File::create(&temp_path)
            .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?;
        
        std::io::copy(&mut reader, &mut temp_file)
            .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?;
        
        drop(temp_file);

        // Use the existing compress_from_stream method from BitCompressor
        // Note: We'll need to modify this approach since compress_from_stream expects a file path
        // For now, we'll return an error indicating this feature needs refinement
        let result = self.compressor.compress_from_stream(
            std::fs::File::open(&temp_path)?,
            std::env::temp_dir().join("output"),
            item_name
        );
        
        // Clean up temp file
        let _ = std::fs::remove_file(&temp_path);
        
        result?;
        // This is a simplified implementation - in reality we'd need to capture the compressed data
        // For now, we return an error to indicate this is not fully implemented
        Err(Bit7zError::FeatureNotSupported(
            "Stream compression returning buffer not yet fully implemented".to_string()
        ))
    }
}
