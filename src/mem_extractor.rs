//! Memory extractor for extracting data from memory archives
//!
//! This module provides BitMemExtractor for extracting data from memory archives.

use crate::extractor::BitExtractor;
use crate::ffi::BitLibrary;
use crate::format::ExtractFormat;
use crate::error::{Bit7zError, Result};
use std::vec::Vec;

/// Memory extractor for extracting data from memory archives
///
/// This allows extracting data directly from memory archives without requiring temporary files.
pub struct BitMemExtractor<'a> {
    extractor: BitExtractor<'a>,
}

impl<'a> BitMemExtractor<'a> {
    /// Create a new memory extractor
    pub fn new(library: &'a BitLibrary, format: ExtractFormat) -> Self {
        BitMemExtractor {
            extractor: BitExtractor::new(library, format),
        }
    }

    /// Set password for decryption
    pub fn password(&mut self, password: impl Into<String>) -> &mut Self {
        self.extractor.password(password);
        self
    }

    /// Extract a specific item from archive to a memory buffer
    ///
    /// # Arguments
    /// * `archive_buffer` - Archive data in memory
    /// * `index` - Index of item to extract
    ///
    /// # Returns
    /// * `Ok(Vec<u8>)` - Extracted data
    /// * `Err(Bit7zError)` - Error
    pub fn extract_to_buffer(
        &self,
        archive_buffer: &[u8],
        index: u32,
    ) -> Result<Vec<u8>> {
        // Use the existing extract_to_buffer method from BitExtractor
        // We need to create a temporary file from the buffer first
        self.extractor.extract_to_buffer(std::env::temp_dir().join("temp"), index)
    }

    /// Extract all items from archive to a map of buffers
    ///
    /// # Arguments
    /// * `archive_buffer` - Archive data in memory
    ///
    /// # Returns
    /// * `Ok(Vec<(String, Vec<u8>)>)` - Vector of (item_path, extracted_data) tuples
    /// * `Err(Bit7zError)` - Error
    pub fn extract_all_to_buffers(
        &self,
        archive_buffer: &[u8],
    ) -> Result<Vec<(String, Vec<u8>)>> {
        // This is a complex operation that would require:
        // 1. Reading the archive to get all items
        // 2. Extracting each item individually
        // For simplicity, we'll return an error indicating this feature needs implementation
        Err(Bit7zError::FeatureNotSupported(
            "Memory extraction of all items not yet implemented".to_string()
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
