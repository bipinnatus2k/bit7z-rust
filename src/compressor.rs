//! Archive compressor implementation
//!
//! This module provides BitCompressor for compressing files into archives.

use crate::ffi::{
    BitLibrary, IOutArchive, IUnknown,
};
use crate::format::{CompressionFormat, CompressionLevel, CompressionMethod};
use crate::error::{Bit7zError, Result};
use crate::stream::FileStreamWrite;
use crate::compress_callback::{UpdateCallback, InputItem, TotalCallbackType, ProgressCallback as CompressProgressCallback, RatioCallback as CompressRatioCallback, FileCallback as CompressFileCallback, PasswordCallback as CompressPasswordCallback};
use std::path::Path;
use std::ptr;
use std::sync::{Arc, Mutex};
use std::io::Write;

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
    volume_size: u64,
    // Callbacks
    total_callback: Option<TotalCallbackType>,
    progress_callback: Option<CompressProgressCallback>,
    ratio_callback: Option<CompressRatioCallback>,
    file_callback: Option<CompressFileCallback>,
    password_callback: Option<CompressPasswordCallback>,
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
            volume_size: 0, // Single volume by default
            total_callback: None,
            progress_callback: None,
            ratio_callback: None,
            file_callback: None,
            password_callback: None,
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

    /// Set volume size for multi-volume archives (0 = single volume)
    pub fn volume_size(&mut self, size_bytes: u64) -> &mut Self {
        self.volume_size = size_bytes;
        self
    }

    /// Set total callback - called with total size at start
    pub fn set_total_callback<F>(&mut self, callback: F) -> &mut Self
    where
        F: Fn(u64) + Send + Sync + 'static,
    {
        self.total_callback = Some(Arc::new(Mutex::new(callback)));
        self
    }

    /// Set progress callback - called with (processed, total), returns true to continue
    pub fn set_progress_callback<F>(&mut self, callback: F) -> &mut Self
    where
        F: Fn(u64, u64) -> bool + Send + Sync + 'static,
    {
        self.progress_callback = Some(Arc::new(Mutex::new(callback)));
        self
    }

    /// Set ratio callback - called with input and output sizes
    pub fn set_ratio_callback<F>(&mut self, callback: F) -> &mut Self
    where
        F: Fn(u64, u64) + Send + Sync + 'static,
    {
        self.ratio_callback = Some(Arc::new(Mutex::new(callback)));
        self
    }

    /// Set file callback - called with file path before processing
    pub fn set_file_callback<F>(&mut self, callback: F) -> &mut Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.file_callback = Some(Arc::new(Mutex::new(callback)));
        self
    }

    /// Set password callback - called when password is needed
    pub fn set_password_callback<F>(&mut self, callback: F) -> &mut Self
    where
        F: Fn() -> String + Send + Sync + 'static,
    {
        self.password_callback = Some(Arc::new(Mutex::new(callback)));
        self
    }

    /// Compress only files (ignore directories)
    pub fn compress_files<P: AsRef<Path>, O: AsRef<Path>>(
        &self,
        files: &[P],
        output_path: O,
    ) -> Result<()> {
        // Filter to only include files (not directories) and collect as PathBuf
        let file_paths: Vec<std::path::PathBuf> = files
            .iter()
            .filter(|p| p.as_ref().is_file())
            .map(|p| p.as_ref().to_path_buf())
            .collect();

        self.compress(&file_paths, output_path.as_ref())
    }

    /// Compress directory contents with optional recursion and filter
    pub fn compress_directory_contents<P: AsRef<Path>, O: AsRef<Path>>(
        &self,
        dir_path: P,
        output_path: O,
        recursive: bool,
        filter: Option<&str>,
    ) -> Result<()> {
        let dir_path = dir_path.as_ref();
        
        if !dir_path.exists() {
            return Err(Bit7zError::CompressFailed(
                format!("Directory does not exist: {:?}", dir_path)
            ));
        }

        if !dir_path.is_dir() {
            return Err(Bit7zError::CompressFailed(
                format!("Not a directory: {:?}", dir_path)
            ));
        }

        // Collect files to compress
        let mut files_to_compress = Vec::new();
        self.collect_directory_files(dir_path, dir_path, recursive, filter, &mut files_to_compress)?;

        // Compress collected files
        self.compress(&files_to_compress, output_path.as_ref())
    }

    /// Compress files with custom archive names (aliases)
    ///
    /// # Arguments
    /// * `files_with_aliases` - List of (file_path, archive_name) pairs
    /// * `output_path` - Output archive path
    ///
    /// # Returns
    /// * `Ok(())` - Success
    /// * `Err(Bit7zError)` - Error
    pub fn compress_with_aliases<P: AsRef<Path>>(
        &self,
        files_with_aliases: &[(P, String)],
        output_path: P,
    ) -> Result<()> {
        use crate::output_archive::BitOutputArchive;

        // Create output archive
        let mut output_archive = BitOutputArchive::new(self.format);
        
        // Set compression parameters
        output_archive
            .compression_level(self.compression_level)
            .solid(self.solid);

        if let Some(method) = self.compression_method {
            output_archive.compression_method(method);
        }

        if let Some(size) = self.dictionary_size {
            output_archive.dictionary_size(size);
        }

        if let Some(size) = self.word_size {
            output_archive.word_size(size);
        }

        if let Some(ref password) = self.password {
            output_archive.password(password.clone());
        }

        // Add files with aliases
        for (file_path, alias) in files_with_aliases {
            output_archive.add_file_with_name(file_path, alias.clone());
        }

        // Compress to file
        output_archive.compress_to(output_path)
    }

    /// Collect files from directory recursively
    fn collect_directory_files(
        &self,
        root: &Path,
        current: &Path,
        recursive: bool,
        filter: Option<&str>,
        files: &mut Vec<std::path::PathBuf>,
    ) -> Result<()> {
        let entries = std::fs::read_dir(current)
            .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?;

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_file() {
                // Apply filter if specified
                if let Some(pattern) = filter {
                    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                        if !Self::match_wildcard(pattern, file_name) {
                            continue;
                        }
                    }
                }
                files.push(path);
            } else if path.is_dir() && recursive {
                // Recurse into subdirectory
                self.collect_directory_files(root, &path, recursive, filter, files)?;
            }
        }

        Ok(())
    }

    /// Match a filename against a wildcard pattern
    fn match_wildcard(pattern: &str, filename: &str) -> bool {
        let pattern_chars: Vec<char> = pattern.chars().collect();
        let filename_chars: Vec<char> = filename.chars().collect();
        Self::wildcard_match_recursive(&pattern_chars, &filename_chars, 0, 0)
    }

    fn wildcard_match_recursive(pattern: &[char], text: &[char], pi: usize, ti: usize) -> bool {
        if pi == pattern.len() && ti == text.len() {
            return true;
        }

        if pi == pattern.len() {
            return false;
        }

        if pattern[pi] == '*' {
            for i in ti..=text.len() {
                if Self::wildcard_match_recursive(pattern, text, pi + 1, i) {
                    return true;
                }
            }
            return false;
        }

        if ti == text.len() {
            return false;
        }

        if pattern[pi] == '?' || pattern[pi] == text[ti] {
            return Self::wildcard_match_recursive(pattern, text, pi + 1, ti + 1);
        }

        false
    }

    /// Compress files to an archive
    pub fn compress<P: AsRef<Path>, O: AsRef<Path>>(
        &self,
        input_paths: &[P],
        output_path: O,
    ) -> Result<()> {
        eprintln!("[compress] Enter, format={:?}", self.format);
        let _ = std::io::stderr().flush();
        // Check if format supports multiple files
        if input_paths.len() > 1 && !self.format.info().features.multiple_files {
            return Err(Bit7zError::FeatureNotSupported(
                "Format does not support multiple files".to_string()
            ));
        }

        // Build input items
        eprintln!("[compress] Building {} input items", input_paths.len());
        let _ = std::io::stderr().flush();
        let input_items: Vec<InputItem> = input_paths
            .iter()
            .map(|p| InputItem::new(p.as_ref()))
            .collect();

        // Create output file stream
        eprintln!("[compress] Creating output stream: {:?}", output_path.as_ref());
        let _ = std::io::stderr().flush();
        let out_stream = FileStreamWrite::new(output_path.as_ref())
            .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?;
        eprintln!("[compress] Output stream created");
        let _ = std::io::stderr().flush();

        // Perform compression
        eprintln!("[compress] Calling compress_internal");
        let _ = std::io::stderr().flush();
        self.compress_internal(&input_items, &out_stream)?;
        eprintln!("[compress] Done");
        let _ = std::io::stderr().flush();

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

    /// Compress from a memory buffer to an archive file
    ///
    /// # Arguments
    /// * `buffer` - Data to compress
    /// * `output_path` - Output archive path
    /// * `item_name` - Name of the item in the archive (optional)
    ///
    /// # Returns
    /// * `Ok(())` - Success
    /// * `Err(Bit7zError)` - Error
    pub fn compress_from_buffer<P: AsRef<Path>>(
        &self,
        buffer: &[u8],
        output_path: P,
        item_name: Option<String>,
    ) -> Result<()> {
        // Create a temporary file with the buffer content
        let temp_path = std::env::temp_dir().join(format!(
            "bit7z_buffer_{}.tmp",
            std::process::id()
        ));

        // Write buffer to temp file
        std::fs::write(&temp_path, buffer)
            .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?;

        // Build input item
        let mut input_item = InputItem::new(&temp_path);
        if let Some(name) = item_name {
            input_item.name_in_archive = Some(name);
        }

        // Create output file stream
        let out_stream = FileStreamWrite::new(output_path)
            .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?;

        // Perform compression
        self.compress_internal(&[input_item], &out_stream)?;

        // Clean up temp file
        let _ = std::fs::remove_file(&temp_path);

        Ok(())
    }

    /// Compress from a reader stream to an archive file
    ///
    /// # Arguments
    /// * `reader` - Input stream to read data from
    /// * `output_path` - Output archive path
    /// * `item_name` - Name of the item in the archive
    ///
    /// # Returns
    /// * `Ok(())` - Success
    /// * `Err(Bit7zError)` - Error
    pub fn compress_from_stream<P: AsRef<Path>, R: std::io::Read>(
        &self,
        mut reader: R,
        output_path: P,
        item_name: String,
    ) -> Result<()> {
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

        // Build input item with custom name
        let input_item = InputItem::with_name(&temp_path, item_name);

        // Create output file stream
        let out_stream = FileStreamWrite::new(output_path)
            .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?;

        // Perform compression
        self.compress_internal(&[input_item], &out_stream)?;

        // Clean up temp file
        let _ = std::fs::remove_file(&temp_path);

        Ok(())
    }

    /// Internal compression implementation
    fn compress_internal(
        &self,
        input_items: &[InputItem],
        out_stream: &FileStreamWrite,
    ) -> Result<()> {
        use crate::compress_callback::UpdateCallback;
        use crate::ffi::IArchiveUpdateCallback;

        unsafe {
            // Create output archive object
            let format_guid = self.format.info().guid;
            let archive_ptr = self.library.create_out_archive(&format_guid)
                .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?;

            let archive = archive_ptr.as_ptr();

            // Set archive properties (compression level, method, etc.)
            // Note: p7zip may not support custom properties, so we ignore errors
            eprintln!("[compress_internal] Calling set_archive_properties");
            let _ = std::io::stderr().flush();
            let _ = self.set_archive_properties(archive); // Ignore errors
            eprintln!("[compress_internal] set_archive_properties done");
            let _ = std::io::stderr().flush();

            // Create update callback on the heap
            // We need to use Box::new and Box::into_raw because the COM interface
            // uses reference counting and may call release after update_items returns
            eprintln!("[DEBUG] Creating UpdateCallback with {} items", input_items.len());
            let update_callback = UpdateCallback::with_callbacks(
                input_items.to_vec(),
                self.password.clone(),
                self.total_callback.clone(),
                self.progress_callback.clone(),
                self.ratio_callback.clone(),
                self.file_callback.clone(),
                self.password_callback.clone(),
            );
            let callback_box = Box::new(update_callback);
            let callback_ptr = Box::into_raw(callback_box);
            eprintln!("[DEBUG] UpdateCallback created at {:p}", callback_ptr);

            // Call UpdateItems
            let num_items = input_items.len() as u32;
            eprintln!("[DEBUG] Calling UpdateItems with {} items", num_items);
            let _ = std::io::stderr().flush();
            let archive_vtable = &*(*archive).vtable;
            eprintln!("[DEBUG] archive_vtable={:p}, update_items_fn={:p}",
                archive_vtable, archive_vtable.update_items);
            let out_stream_ptr = out_stream.as_i_out_stream() as *mut crate::ffi::ISequentialOutStream;
            eprintln!("[DEBUG] out_stream_ptr={:p}", out_stream_ptr);
            let callback_iface_ptr = (*callback_ptr).as_i_archive_update_callback();
            eprintln!("[DEBUG] callback_ptr={:p}, callback_iface_ptr={:p}", callback_ptr, callback_iface_ptr);
            let _ = std::io::stderr().flush();
            let result = (archive_vtable.update_items)(
                archive,
                out_stream_ptr,
                num_items,
                callback_iface_ptr as *mut IArchiveUpdateCallback,
            );
            eprintln!("[DEBUG] UpdateItems returned 0x{:X}", result);
            let _ = std::io::stderr().flush();

            // After update_items returns, release our reference
            // The callback will be freed when ref_count reaches 0
            eprintln!("[DEBUG] Releasing caller reference");
            UpdateCallback::release_caller_reference(callback_ptr);
            eprintln!("[DEBUG] Done");

            // Match bit7z behavior: UpdateItems must return S_OK.
            if result != 0 {
                return Err(Bit7zError::CompressFailed(
                    format!("UpdateItems failed with HRESULT: 0x{:X}", result)
                ));
            }
        }

        Ok(())
    }

    /// Set archive properties (compression level, method, etc.)
    fn set_archive_properties(&self, archive: *mut IOutArchive) -> Result<()> {
        use crate::ffi::{ISetProperties, IID_ISetProperties, PROPVARIANT, VARENUM};
        use crate::ffi::variant::{alloc_bstr_from_utf32, free_bstr};

        // Try to get ISetProperties interface
        let mut set_props_ptr: *mut std::ffi::c_void = ptr::null_mut();
        let iid = IID_ISetProperties;

        // Cast archive to IUnknown for QueryInterface
        let unknown = archive as *mut IUnknown;
        let archive_vtable = unsafe {&*(*archive).vtable };
        let result = unsafe {
            (archive_vtable.base.query_interface)(
                unknown,
                &iid,
                &mut set_props_ptr,
            )
        };

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
                    // Use write_unaligned to avoid alignment issues
                    let data_ptr = prop_value.data.as_mut_ptr() as *mut *mut u16;
                    unsafe { std::ptr::write_unaligned(data_ptr, value_bstr); }
                    prop_values.push(prop_value);
                } else {
                    unsafe { free_bstr(name_bstr as *mut u16); }
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
            let set_vtable = unsafe { &*(*set_properties).vtable };
            let names_ptr = prop_names.as_ptr();
            let values_ptr = prop_values.as_ptr();
            let num_props = prop_names.len() as u32;

            eprintln!("[DEBUG] Calling SetProperties with {} props", num_props);
            let set_result = unsafe {
                (set_vtable.set_properties)(
                    set_properties,
                    names_ptr,
                    values_ptr as *const *const std::ffi::c_void,
                    num_props,
                )
            };
            eprintln!("[DEBUG] SetProperties returned 0x{:X}", set_result);
            let _ = std::io::stderr().flush();

            if set_result != 0 && set_result != 1 {
                eprintln!("[WARN] SetProperties returned 0x{:X}", set_result);
                let _ = std::io::stderr().flush();
                // Don't fail - some formats may not support all properties
            }
        } else {
            eprintln!("[DEBUG] No properties to set");
            let _ = std::io::stderr().flush();
        }

        eprintln!("[DEBUG] Releasing SetProperties interface");
        let _ = std::io::stderr().flush();
        unsafe {
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
        }

        Ok(())
    }
}
