//! Memory extractor for extracting data from memory archives
//!
//! This module provides BitMemExtractor for extracting data from memory archives.

use crate::extractor::BitExtractor;
use crate::ffi::BitLibrary;
use crate::format::ExtractFormat;
use crate::error::{Bit7zError, Result};
use std::fs;
use std::io::Write;
use tempfile::TempDir;
use std::time::{SystemTime, UNIX_EPOCH};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

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

    /// Helper function to create a temporary file from buffer
    fn create_temp_file_from_buffer(buffer: &[u8]) -> Result<(TempDir, std::path::PathBuf)> {
        let temp_dir = TempDir::new()?;
        
        // Generate unique filename
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let mut hasher = DefaultHasher::new();
        timestamp.hash(&mut hasher);
        let unique_id = hasher.finish();
        
        let temp_path = temp_dir.path().join(format!("archive_{}", unique_id));
        let mut temp_file = fs::File::create(&temp_path)?;
        temp_file.write_all(buffer)?;
        temp_file.sync_all()?;
        
        Ok((temp_dir, temp_path))
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
        // Create temporary file from archive buffer
        let (_temp_dir, temp_archive_path) = Self::create_temp_file_from_buffer(archive_buffer)?;

        // Extract the specific item
        self.extractor.extract_to_buffer(&temp_archive_path, index)
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
        // Create temporary file from archive buffer
        let (_temp_dir, temp_archive_path) = Self::create_temp_file_from_buffer(archive_buffer)?;

        // Create temporary directory for extraction
        let temp_dir = TempDir::new()?;
        let temp_dir_path = temp_dir.path().to_path_buf();

        // Extract all items
        self.extractor.extract(&temp_archive_path, &temp_dir_path)?;

        // Read all extracted files into memory buffers
        let mut result = Vec::new();

        // Walk through the extracted files
        for entry in fs::read_dir(temp_dir.path())? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                // Get relative path from temp_dir
                let relative_path = path.strip_prefix(temp_dir.path())
                    .map_err(|e| Bit7zError::ExtractFailed(format!("Failed to get relative path: {}", e)))?
                    .to_str()
                    .ok_or_else(|| Bit7zError::ExtractFailed("Invalid path string".to_string()))?
                    .to_string();

                // Read file content
                let content = fs::read(&path)?;

                result.push((relative_path, content));
            }
        }

        Ok(result)
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
        // Create temporary file from archive buffer
        let (_temp_dir, temp_archive_path) = Self::create_temp_file_from_buffer(archive_buffer)?;

        // Test the archive
        self.extractor.test(&temp_archive_path)
    }
}
