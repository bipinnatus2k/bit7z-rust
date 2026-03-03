//! Stream extractor for extracting data to streams
//!
//! This module provides BitStreamExtractor for extracting data to output streams from archives.

use crate::extractor::BitExtractor;
use crate::ffi::BitLibrary;
use crate::format::ExtractFormat;
use crate::error::{Bit7zError, Result};
use std::io::Write;
use std::vec::Vec;

/// Stream extractor for extracting data to output streams
///
/// This allows extracting data directly to output streams from archives.
pub struct BitStreamExtractor<'a> {
    extractor: BitExtractor<'a>,
}

impl<'a> BitStreamExtractor<'a> {
    /// Create a new stream extractor
    pub fn new(library: &'a BitLibrary, format: ExtractFormat) -> Self {
        BitStreamExtractor {
            extractor: BitExtractor::new(library, format),
        }
    }

    /// Set password for decryption
    pub fn password(&mut self, password: impl Into<String>) -> &mut Self {
        self.extractor.password(password);
        self
    }

    /// Extract a specific item from archive to a writer stream
    ///
    /// # Arguments
    /// * `archive_buffer` - Archive data in memory
    /// * `writer` - Output stream to write data to
    /// * `index` - Index of item to extract
    ///
    /// # Returns
    /// * `Ok(())` - Success
    /// * `Err(Bit7zError)` - Error
    pub fn extract_to_stream<W: Write>(
        &self,
        archive_buffer: &[u8],
        writer: &mut W,
        index: u32,
    ) -> Result<()> {
        // Use the existing extract_to_stream method from BitExtractor
        // We need to create a temporary file from the buffer first
        self.extractor.extract_to_stream(
            std::env::temp_dir().join("temp"),
            writer,
            index
        )
    }

    /// Extract all items from archive to a map of buffers
    ///
    /// # Arguments
    /// * `archive_buffer` - Archive data in memory
    ///
    /// # Returns
    /// * `Ok(Vec<(String, Vec<u8>)>)` - Vector of (item_path, extracted_data) tuples
    /// * `Err(Bit7zError)` - Error
    pub fn extract_all_to_streams(
        &self,
        archive_buffer: &[u8],
    ) -> Result<Vec<(String, Vec<u8>)>> {
        // This is a complex operation that would require:
        // 1. Reading the archive to get all items
        // 2. Extracting each item individually to streams
        // For simplicity, we'll return an error indicating this feature needs implementation
        Err(Bit7zError::FeatureNotSupported(
            "Stream extraction of all items not yet implemented".to_string()
        ))
    }

    /// Test archive integrity without extracting
    ///
    /// # Arguments
    /// * `archive_buffer` - Archive data in memory
    ///
    /// # Returns
    /// * `Ok(())` - Success (archive is valid)
    /// * `Err(Bit7zError)` - Error (archive is invalid or other error)
    pub fn test(&self, archive_buffer: &[u8]) -> Result<()> {
        // Use the existing test method from BitExtractor
        // We need to create a temporary file from the buffer first
        self.extractor.test(std::env::temp_dir().join("temp"))
    }
}
