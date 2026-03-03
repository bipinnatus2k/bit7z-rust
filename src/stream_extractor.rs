//! Stream extractor for extracting data to streams
//!
//! This module provides BitStreamExtractor for extracting data to output streams from archives.

use crate::extractor::BitExtractor;
use crate::ffi::BitLibrary;
use crate::format::ExtractFormat;
use crate::error::{Bit7zError, Result};
use std::fs;
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};

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
        // Create temporary file from archive buffer
        let mut temp_archive = NamedTempFile::new()?;
        temp_archive.write_all(archive_buffer)?;
        temp_archive.flush()?;
        
        let temp_archive_path = temp_archive.path().to_path_buf();
        
        // Extract the specific item to the writer stream
        self.extractor.extract_to_stream(&temp_archive_path, writer, index)
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
        // Create temporary file from archive buffer
        let mut temp_archive = NamedTempFile::new()?;
        temp_archive.write_all(archive_buffer)?;
        temp_archive.flush()?;
        
        let temp_archive_path = temp_archive.path().to_path_buf();
        
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
        let mut temp_archive = NamedTempFile::new()?;
        temp_archive.write_all(archive_buffer)?;
        temp_archive.flush()?;
        
        let temp_archive_path = temp_archive.path().to_path_buf();
        
        // Test the archive
        self.extractor.test(&temp_archive_path)
    }
}
