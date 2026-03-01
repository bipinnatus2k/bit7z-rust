//! Archive compressor implementation
//!
//! This module provides BitCompressor for compressing files into archives.

use crate::ffi::{
    BitLibrary, IOutArchive, IOutStream, ISetProperties, IUnknown,
    PROPVARIANT, PROPID, HRESULT,
    IID_ISetProperties,
};
use crate::format::{CompressionFormat, CompressionLevel, CompressionMethod};
use crate::error::{Bit7zError, Result};
use crate::stream::FileStreamWrite;
use crate::compress_callback::{UpdateCallback, InputItem};
use std::path::Path;
use std::ptr;

/// Compressor for creating archives
pub struct BitCompressor<'a> {
    library: &'a BitLibrary,
    format: CompressionFormat,
    password: Option<String>,
    compression_level: CompressionLevel,
    compression_method: Option<CompressionMethod>,
    dictionary_size: Option<u32>,
    word_size: Option<u32>,
    solid: bool,
}

impl<'a> BitCompressor<'a> {
    /// Create a new compressor
    pub fn new(library: &'a BitLibrary, format: CompressionFormat) -> Self {
        BitCompressor {
            library,
            format,
            password: None,
            compression_level: CompressionLevel::Normal,
            compression_method: None,
            dictionary_size: None,
            word_size: None,
            solid: false,
        }
    }

    /// Set password for encryption
    pub fn password(&mut self, password: impl Into<String>) -> &mut Self {
        self.password = Some(password.into());
        self
    }

    /// Set compression level
    pub fn compression_level(&mut self, level: CompressionLevel) -> &mut Self {
        self.compression_level = level;
        self
    }

    /// Set compression method
    pub fn compression_method(&mut self, method: CompressionMethod) -> &mut Self {
        self.compression_method = Some(method);
        self
    }

    /// Set dictionary size
    pub fn dictionary_size(&mut self, size: u32) -> &mut Self {
        self.dictionary_size = Some(size);
        self
    }

    /// Set word size
    pub fn word_size(&mut self, size: u32) -> &mut Self {
        self.word_size = Some(size);
        self
    }

    /// Enable solid compression
    pub fn solid(&mut self, solid: bool) -> &mut Self {
        self.solid = solid;
        self
    }

    /// Compress files to an archive
    pub fn compress<P: AsRef<Path>>(
        &self,
        input_paths: &[P],
        output_path: P,
    ) -> Result<()> {
        // Check if format supports multiple files
        if input_paths.len() > 1 && !self.format.info().features.multiple_files {
            return Err(Bit7zError::FeatureNotSupported(
                "Format does not support multiple files".to_string()
            ));
        }

        // Build input items
        let input_items: Vec<InputItem> = input_paths
            .iter()
            .map(|p| InputItem::new(p.as_ref()))
            .collect();

        // Create output file stream
        let out_stream = FileStreamWrite::new(output_path.as_ref())
            .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?;

        // Perform compression
        self.compress_internal(&input_items, &out_stream)?;

        Ok(())
    }

    /// Compress files to a memory buffer
    pub fn compress_to_buffer<P: AsRef<Path>>(
        &self,
        input_paths: &[P],
    ) -> Result<Vec<u8>> {
        // Check if format supports multiple files
        if input_paths.len() > 1 && !self.format.info().features.multiple_files {
            return Err(Bit7zError::FeatureNotSupported(
                "Format does not support multiple files".to_string()
            ));
        }

        // Build input items
        let input_items: Vec<InputItem> = input_paths
            .iter()
            .map(|p| InputItem::new(p.as_ref()))
            .collect();

        // Create a temporary file for output
        use std::io::Write;
        let temp_path = std::env::temp_dir().join(format!(
            "bit7z_temp_{}.tmp",
            std::process::id()
        ));
        
        {
            let temp_file = std::fs::File::create(&temp_path)
                .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?;
            let out_stream = FileStreamWrite::new(&temp_path)
                .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?;
            
            // Perform compression
            self.compress_internal(&input_items, &out_stream)?;
            
            drop(out_stream);
            drop(temp_file);
        }

        // Read the compressed data
        let buffer = std::fs::read(&temp_path)
            .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?;
        
        // Clean up temp file
        let _ = std::fs::remove_file(&temp_path);

        Ok(buffer)
    }

    /// Internal compression implementation
    fn compress_internal(
        &self,
        input_items: &[InputItem],
        out_stream: &FileStreamWrite,
    ) -> Result<()> {
        unsafe {
            // Create output archive object
            let format_guid = self.format.info().guid;
            let archive_ptr = self.library.create_out_archive(&format_guid)
                .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?;

            let archive = archive_ptr.as_ptr();

            // Create update callback
            let mut update_callback = UpdateCallback::new(
                input_items.to_vec(),
                self.password.clone(),
            );

            // Call UpdateItems
            let num_items = input_items.len() as u32;
            let archive_vtable = &*(*archive).vtable;
            let result = (archive_vtable.update_items)(
                archive,
                out_stream.as_i_out_stream() as *mut crate::ffi::ISequentialOutStream,
                num_items,
                update_callback.as_i_archive_update_callback(),
            );

            if result != 0 {
                return Err(Bit7zError::CompressFailed(
                    format!("UpdateItems failed with HRESULT: 0x{:X}", result)
                ));
            }
        }

        Ok(())
    }

    /// Set archive properties (compression level, method, etc.)
    unsafe fn set_archive_properties(&self, archive: *mut IOutArchive) -> Result<()> {
        // Try to get ISetProperties interface
        let mut set_props_ptr: *mut std::ffi::c_void = ptr::null_mut();
        let iid = IID_ISetProperties;
        
        // Cast archive to IUnknown for QueryInterface
        let unknown = archive as *mut IUnknown;
        let archive_vtable = &*(*archive).vtable;
        let result = (archive_vtable.base.query_interface)(
            unknown,
            &iid,
            &mut set_props_ptr,
        );

        if result != 0 || set_props_ptr.is_null() {
            // ISetProperties not supported, skip property setting
            return Ok(());
        }

        let set_properties: *mut ISetProperties = set_props_ptr as *mut ISetProperties;

        // Build property names and values
        // Note: 7-Zip expects property values as PROPVARIANT types, not raw values
        // For now, we skip setting properties to get basic compression working
        // TODO: Implement proper PROPVARIANT-based property setting
        
        // Release the interface
        let set_vtable = &*(*set_properties).vtable;
        (set_vtable.base.release)(set_properties as *mut IUnknown);

        Ok(())
    }
}
