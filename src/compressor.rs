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
    crypt_headers: bool,
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
            crypt_headers: false,
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

    /// Enable header encryption (7z format only)
    pub fn crypt_headers(&mut self, encrypt: bool) -> &mut Self {
        self.crypt_headers = encrypt;
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
            eprintln!("[compress_internal] Creating archive object...");
            
            // Create output archive object
            let format_guid = self.format.info().guid;
            let archive_ptr = self.library.create_out_archive(&format_guid)
                .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?;

            let archive = archive_ptr.as_ptr();
            eprintln!("[compress_internal] Archive created: {:?}", archive);

            // Set archive properties (compression level, method, etc.)
            eprintln!("[compress_internal] Setting archive properties...");
            self.set_archive_properties(archive)?;

            // Create update callback
            eprintln!("[compress_internal] Creating update callback...");
            let update_callback = UpdateCallback::new(
                input_items.to_vec(),
                self.password.clone(),
            );

            // Call UpdateItems
            eprintln!("[compress_internal] Calling UpdateItems with {} items...", input_items.len());
            let num_items = input_items.len() as u32;
            let archive_vtable = &*(*archive).vtable;
            let result = (archive_vtable.update_items)(
                archive,
                out_stream.as_i_out_stream() as *mut crate::ffi::ISequentialOutStream,
                num_items,
                update_callback.as_i_archive_update_callback(),
            );
            eprintln!("[compress_internal] UpdateItems returned: 0x{:X}", result);

            // S_OK (0) and S_FALSE (1) are both success codes
            // S_FALSE may indicate some items were not compressed but overall operation succeeded
            if result != 0 && result != 1 {
                return Err(Bit7zError::CompressFailed(
                    format!("UpdateItems failed with HRESULT: 0x{:X}", result)
                ));
            }
        }

        Ok(())
    }

    /// Set archive properties (compression level, method, etc.)
    unsafe fn set_archive_properties(&self, archive: *mut IOutArchive) -> Result<()> {
        use crate::ffi::{ISetProperties, IID_ISetProperties, PROPVARIANT, VARENUM};
        use crate::ffi::variant::{alloc_bstr_from_utf32, free_bstr};

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
            // This is OK - some formats don't support custom properties
            return Ok(());
        }

        let set_properties: *mut ISetProperties = set_props_ptr as *mut ISetProperties;

        // Build property names and values
        let mut prop_names: Vec<*const u16> = Vec::new();
        let mut prop_values: Vec<PROPVARIANT> = Vec::new();

        // Add compression level property ("x")
        {
            let name_bstr = alloc_bstr_from_utf32("x");
            if !name_bstr.is_null() {
                prop_names.push(name_bstr);
                let mut prop_value = PROPVARIANT::default();
                prop_value.vt = VARENUM::VT_UI4 as u16;
                let value = self.compression_level.to_value();
                prop_value.data[0] = (value & 0xFF) as u8;
                prop_value.data[1] = ((value >> 8) & 0xFF) as u8;
                prop_value.data[2] = ((value >> 16) & 0xFF) as u8;
                prop_value.data[3] = ((value >> 24) & 0xFF) as u8;
                prop_values.push(prop_value);
            }
        }

        // Add compression method property ("m") if specified
        if let Some(method) = self.compression_method {
            let method_str = method.to_string();
            let name_bstr = alloc_bstr_from_utf32("m");
            if !name_bstr.is_null() {
                let value_bstr = alloc_bstr_from_utf32(&method_str);
                if !value_bstr.is_null() {
                    prop_names.push(name_bstr);
                    let mut prop_value = PROPVARIANT::default();
                    prop_value.vt = VARENUM::VT_BSTR as u16;
                    let data_ptr = prop_value.data.as_mut_ptr() as *mut *mut u16;
                    *data_ptr = value_bstr;
                    prop_values.push(prop_value);
                } else {
                    free_bstr(name_bstr as *mut u16);
                }
            }
        }

        // Add dictionary size property ("d") if specified
        if let Some(size) = self.dictionary_size {
            let name_bstr = alloc_bstr_from_utf32("d");
            if !name_bstr.is_null() {
                prop_names.push(name_bstr);
                let mut prop_value = PROPVARIANT::default();
                prop_value.vt = VARENUM::VT_UI4 as u16;
                prop_value.data[0] = (size & 0xFF) as u8;
                prop_value.data[1] = ((size >> 8) & 0xFF) as u8;
                prop_value.data[2] = ((size >> 16) & 0xFF) as u8;
                prop_value.data[3] = ((size >> 24) & 0xFF) as u8;
                prop_values.push(prop_value);
            }
        }

        // Add word size property ("w") if specified
        if let Some(size) = self.word_size {
            let name_bstr = alloc_bstr_from_utf32("w");
            if !name_bstr.is_null() {
                prop_names.push(name_bstr);
                let mut prop_value = PROPVARIANT::default();
                prop_value.vt = VARENUM::VT_UI4 as u16;
                prop_value.data[0] = (size & 0xFF) as u8;
                prop_value.data[1] = ((size >> 8) & 0xFF) as u8;
                prop_value.data[2] = ((size >> 16) & 0xFF) as u8;
                prop_value.data[3] = ((size >> 24) & 0xFF) as u8;
                prop_values.push(prop_value);
            }
        }

        // Add solid compression property ("s") if enabled
        if self.solid {
            let name_bstr = alloc_bstr_from_utf32("s");
            if !name_bstr.is_null() {
                prop_names.push(name_bstr);
                let mut prop_value = PROPVARIANT::default();
                prop_value.vt = VARENUM::VT_BOOL as u16;
                // BOOL: true = -1 (0xFFFF), false = 0
                prop_value.data[0] = 0xFF;
                prop_value.data[1] = 0xFF;
                prop_values.push(prop_value);
            }
        }

        // Add header encryption property ("hc") if enabled (7z only)
        if self.crypt_headers && matches!(self.format, CompressionFormat::SevenZip) {
            let name_bstr = alloc_bstr_from_utf32("hc");
            if !name_bstr.is_null() {
                prop_names.push(name_bstr);
                let mut prop_value = PROPVARIANT::default();
                prop_value.vt = VARENUM::VT_BOOL as u16;
                prop_value.data[0] = 0xFF;
                prop_value.data[1] = 0xFF;
                prop_values.push(prop_value);
            }
        }

        // Call SetProperties if we have any properties
        if !prop_names.is_empty() && !prop_values.is_empty() {
            let set_vtable = &*(*set_properties).vtable;
            let names_ptr = prop_names.as_ptr();
            let values_ptr = prop_values.as_ptr();
            let num_props = prop_names.len() as u32;

            let set_result = (set_vtable.set_properties)(
                set_properties,
                names_ptr,
                values_ptr as *const *const std::ffi::c_void,
                num_props,
            );

            if set_result != 0 && set_result != 1 {
                eprintln!("[WARN] SetProperties returned 0x{:X}", set_result);
                // Don't fail - some formats may not support all properties
            }
        }

        // Release the interface
        let set_vtable = &*(*set_properties).vtable;
        (set_vtable.base.release)(set_properties as *mut IUnknown);

        // Free allocated BSTRs (property names)
        for name in prop_names {
            free_bstr(name as *mut u16);
        }
        
        // Free BSTRs in property values (VT_BSTR only)
        for value in prop_values {
            if value.vt == VARENUM::VT_BSTR as u16 {
                // Use read_unaligned to avoid alignment issues
                let bstr_ptr = std::ptr::read_unaligned(value.data.as_ptr() as *const *const u16);
                if !bstr_ptr.is_null() {
                    free_bstr(bstr_ptr as *mut u16);
                }
            }
        }

        Ok(())
    }
}
