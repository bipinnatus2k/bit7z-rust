//! Memory compressor for compressing data from memory buffers
//!
//! This module provides BitMemCompressor for compressing data from memory buffers into archives.

use crate::compressor::BitCompressor;
use crate::ffi::BitLibrary;
use crate::format::CompressionFormat;
use crate::error::{Bit7zError, Result};
use std::fs;
use std::io::Write;

/// Memory compressor for compressing data from memory buffers
///
/// This allows compressing data directly from memory buffers into archives.
pub struct BitMemCompressor<'a> {
    compressor: BitCompressor<'a>,
}

impl<'a> BitMemCompressor<'a> {
    /// Create a new memory compressor
    pub fn new(library: &'a BitLibrary, format: CompressionFormat) -> Self {
        BitMemCompressor {
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

    /// Compress data from a memory buffer into an archive
    ///
    /// # Arguments
    /// * `buffer` - Data to compress
    /// * `item_name` - Name of the item in the archive
    ///
    /// # Returns
    /// * `Ok(Vec<u8>)` - Compressed archive data
    /// * `Err(Bit7zError)` - Error
    pub fn compress_from_buffer(
        &self,
        buffer: &[u8],
        item_name: Option<String>,
    ) -> Result<Vec<u8>> {
        use tempfile::tempdir;
        use std::time::{SystemTime, UNIX_EPOCH};
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        // Create temporary directory
        let temp_dir = tempdir()?;
        
        // Generate unique filenames using timestamp and hash
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let mut hasher = DefaultHasher::new();
        timestamp.hash(&mut hasher);
        let unique_id = hasher.finish();
        
        // Create temporary file for buffer content
        let temp_path = temp_dir.path().join(format!("input_{}", unique_id));
        let mut temp_file = fs::File::create(&temp_path)?;
        temp_file.write_all(buffer)?;
        temp_file.sync_all()?;

        let item_name = item_name.unwrap_or_else(|| "item.bin".to_string());

        // Create temporary file for output archive
        let output_path = temp_dir.path().join(format!("output_{}", unique_id));

        // Compress the temporary file with custom name
        self.compressor.compress_with_aliases(&[(temp_path.as_path(), item_name)], &output_path)?;

        // Read the compressed data - temp_dir is still in scope here
        let compressed_data = fs::read(&output_path)?;

        // temp_dir will be dropped here, cleaning up all temporary files
        Ok(compressed_data)
    }
}
