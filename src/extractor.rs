//! Archive extractor implementation
//! 
//! This module provides BitExtractor for extracting files from archives.

use crate::ffi::{
    BitLibrary, IInArchive, IArchiveExtractCallback,
    ISequentialOutStream, ICryptoGetTextPassword,
    PROPVARIANT, HRESULT, IArchiveExtractCallbackVTable, IInArchiveVTable,
};
use crate::format::ExtractFormat;
use crate::error::{Bit7zError, Result};
use crate::stream::FileStreamWrite;
use std::cell::UnsafeCell;
use std::ffi::c_void;
use std::fs;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::ptr;

/// Extractor for extracting files from archives
pub struct BitExtractor<'a> {
    library: &'a BitLibrary<'a>,
    format: ExtractFormat,
    password: Option<String>,
}

impl<'a> BitExtractor<'a> {
    /// Create a new extractor
    pub fn new(library: &'a BitLibrary<'a>, format: ExtractFormat) -> Self {
        BitExtractor {
            library,
            format,
            password: None,
        }
    }

    /// Set password for encrypted archives
    pub fn password(&mut self, password: impl Into<String>) -> &mut Self {
        self.password = Some(password.into());
        self
    }

    /// Extract all files from archive to output directory
    pub fn extract<P: AsRef<Path>>(
        &self,
        archive_path: P,
        output_dir: P,
    ) -> Result<()> {
        unsafe {
            let archive_ptr = self.library.create_in_archive(&self.format.guid())?;

            // Create input stream for archive file
            let in_stream = Box::leak(Box::new(
                crate::stream::FileStream::new(archive_path.as_ref())?
            ));

            // Open archive
            let result = ((*(*archive_ptr.as_ptr()).vtable).open)(
                archive_ptr.as_ptr(),
                in_stream.as_i_in_stream(),
                0, // max_check_start_position
                ptr::null_mut(), // open_callback
            );

            if result != 0 {
                return Err(Bit7zError::OpenFailed(format!(
                    "Failed to open archive: HRESULT 0x{:08X}", result
                )));
            }

            // Ensure output directory exists
            let output_path = output_dir.as_ref();
            if !output_path.exists() {
                fs::create_dir_all(output_path)?;
            }

            // Create extract callback
            let callback = Box::leak(Box::new(ExtractCallback::new(
                output_path,
                self.password.clone(),
                archive_ptr.as_ptr(),
            )));

            // Extract all items
            let result = ((*(*archive_ptr.as_ptr()).vtable).extract)(
                archive_ptr.as_ptr(),
                ptr::null(), // indices (null = all)
                0xFFFFFFFF,    // num_items (0xFFFFFFFF = all)
                0,            // test_mode (0 = extract)
                callback.as_i_archive_extract_callback(),
            );

            // Close archive
            let _ = ((*(*archive_ptr.as_ptr()).vtable).close)(archive_ptr.as_ptr());

            if result != 0 {
                return Err(Bit7zError::ExtractFailed(format!(
                    "Extraction failed: HRESULT 0x{:08X}", result
                )));
            }

            Ok(())
        }
    }

    /// Extract files matching a wildcard pattern
    pub fn extract_matching<P: AsRef<Path>>(
        &self,
        archive_path: P,
        output_dir: P,
        pattern: &str,
    ) -> Result<()> {
        unsafe {
            let archive_ptr = self.library.create_in_archive(&self.format.guid())?;

            // Create input stream
            let in_stream = Box::leak(Box::new(
                crate::stream::FileStream::new(archive_path.as_ref())?
            ));

            // Open archive
            let result = ((*(*archive_ptr.as_ptr()).vtable).open)(
                archive_ptr.as_ptr(),
                in_stream.as_i_in_stream(),
                0,
                ptr::null_mut(),
            );

            if result != 0 {
                return Err(Bit7zError::OpenFailed(format!(
                    "Failed to open archive: HRESULT 0x{:08X}", result
                )));
            }

            // Get number of items
            let mut num_items: u32 = 0;
            let result = ((*(*archive_ptr.as_ptr()).vtable).get_number_of_items)(
                archive_ptr.as_ptr(),
                &mut num_items,
            );

            if result != 0 {
                return Err(Bit7zError::OpenFailed(format!(
                    "Failed to get item count: HRESULT 0x{:08X}", result
                )));
            }

            // Find matching indices
            let mut indices = Vec::new();
            let wildcard = Self::compile_wildcard(pattern);

            for i in 0..num_items {
                let mut prop = std::mem::zeroed::<PROPVARIANT>();
                let result = ((*(*archive_ptr.as_ptr()).vtable).get_property)(
                    archive_ptr.as_ptr(),
                    i,
                    crate::ffi::kpidPath,
                    &mut prop,
                );

                if result == 0 {
                    let path = crate::ffi::propvariant_to_string(&prop)?;
                    if Self::matches_pattern(&path, &wildcard) {
                        indices.push(i);
                    }
                }
            }

            // Ensure output directory exists
            let output_path = output_dir.as_ref();
            if !output_path.exists() {
                fs::create_dir_all(output_path)?;
            }

            // Create extract callback
            let callback = Box::leak(Box::new(ExtractCallback::new(
                output_path,
                self.password.clone(),
                archive_ptr.as_ptr(),
            )));

            // Extract matching items
            let result = ((*(*archive_ptr.as_ptr()).vtable).extract)(
                archive_ptr.as_ptr(),
                indices.as_ptr(),
                indices.len() as u32,
                0,
                callback.as_i_archive_extract_callback(),
            );

            // Close archive
            let _ = ((*(*archive_ptr.as_ptr()).vtable).close)(archive_ptr.as_ptr());

            if result != 0 {
                return Err(Bit7zError::ExtractFailed(format!(
                    "Extraction failed: HRESULT 0x{:08X}", result
                )));
            }

            Ok(())
        }
    }

    /// Validate path to prevent directory traversal attacks
    fn validate_path(path: &Path, output_dir: &Path) -> Result<()> {
        // Try to canonicalize both paths
        let canonical = path.canonicalize().map_err(|_| {
            Bit7zError::PathTraversal(format!("Invalid path: {}", path.display()))
        })?;

        let output_canonical = output_dir.canonicalize().map_err(|e| {
            Bit7zError::Io(e)
        })?;

        // Check if path starts with output directory
        if !canonical.starts_with(&output_canonical) {
            return Err(Bit7zError::PathTraversal(format!(
                "Path traversal detected: {} escapes output directory {}",
                path.display(),
                output_dir.display()
            )));
        }

        Ok(())
    }

    /// Compile wildcard pattern into regex parts
    fn compile_wildcard(pattern: &str) -> Vec<String> {
        let mut parts = Vec::new();
        let mut current = String::new();

        for ch in pattern.chars() {
            match ch {
                '*' => {
                    if !current.is_empty() {
                        parts.push(current.clone());
                        current.clear();
                    }
                    parts.push("*".to_string());
                }
                '?' => {
                    if !current.is_empty() {
                        parts.push(current.clone());
                        current.clear();
                    }
                    parts.push("?".to_string());
                }
                '.' | '+' | '(' | ')' | '[' | ']' | '{' | '}' | '^' | '$' | '|' | '\\' => {
                    current.push('\\');
                    current.push(ch);
                }
                _ => {
                    current.push(ch);
                }
            }
        }

        if !current.is_empty() {
            parts.push(current);
        }

        parts
    }

    /// Check if path matches wildcard pattern
    fn matches_pattern(path: &str, pattern: &[String]) -> bool {
        if pattern.is_empty() {
            return true;
        }

        let mut path_chars = path.chars().peekable();
        let mut pattern_idx = 0;

        while pattern_idx < pattern.len() {
            match pattern[pattern_idx].as_str() {
                "*" => {
                    // Match any sequence
                    if pattern_idx == pattern.len() - 1 {
                        return true; // Last pattern is *, matches everything
                    }

                    // Try to match rest of the pattern
                    while path_chars.peek().is_some() {
                        let rest_path: String = path_chars.clone().collect();
                        if Self::matches_pattern(&rest_path, &pattern[pattern_idx + 1..]) {
                            return true;
                        }
                        path_chars.next();
                    }

                    return Self::matches_pattern("", &pattern[pattern_idx + 1..]);
                }
                "?" => {
                    if path_chars.next().is_none() {
                        return false;
                    }
                }
                literal => {
                    for ch in literal.chars() {
                        let path_ch = path_chars.next();
                        if path_ch.map(|c| c.to_lowercase().collect::<String>())
                            != Some(ch.to_lowercase().collect::<String>())
                        {
                            return false;
                        }
                    }
                }
            }

            pattern_idx += 1;
        }

        // Check if we consumed all path characters
        path_chars.peek().is_none()
    }
}

/// Extract callback implementation
struct ExtractCallback {
    output_dir: PathBuf,
    password: Option<String>,
    archive: *mut IInArchive,
    current_out_stream: UnsafeCell<Option<*mut ISequentialOutStream>>,
    current_path: UnsafeCell<Option<String>>,
    ref_count: UnsafeCell<u32>,
    vtable: Pin<Box<IArchiveExtractCallbackVTable>>,
}

impl ExtractCallback {
    fn new(output_dir: &Path, password: Option<String>, archive: *mut IInArchive) -> Self {
        let vtable = Box::pin(IArchiveExtractCallbackVTable {
            base: crate::ffi::IUnknownVTable {
                query_interface: Self::query_interface,
                add_ref: Self::add_ref,
                release: Self::release,
            },
            set_completed: Self::set_completed,
            set_total: Self::set_total,
            get_stream: Self::get_stream,
            prepare_operation: Self::prepare_operation,
            set_operation_result: Self::set_operation_result,
        });

        ExtractCallback {
            output_dir: output_dir.to_path_buf(),
            password,
            archive,
            current_out_stream: UnsafeCell::new(None),
            current_path: UnsafeCell::new(None),
            ref_count: UnsafeCell::new(1),
            vtable,
        }
    }

    fn as_i_archive_extract_callback(&self) -> *mut IArchiveExtractCallback {
        self as *const ExtractCallback as *mut ExtractCallback as *mut IArchiveExtractCallback
    }

    /// Get the ICryptoGetTextPassword interface pointer
    fn as_i_crypto_get_text_password(&self) -> *mut ICryptoGetTextPassword {
        self as *const ExtractCallback as *mut ExtractCallback as *mut ICryptoGetTextPassword
    }

    unsafe extern "system" fn query_interface(
        _this: *mut crate::ffi::IUnknown,
        _iid: *const crate::ffi::GUID,
        _out: *mut *mut c_void,
    ) -> HRESULT {
        -1 // E_NOINTERFACE
    }

    unsafe extern "system" fn add_ref(this: *mut crate::ffi::IUnknown) -> u32 {
        let callback = this as *mut ExtractCallback;
        let ref_count = &(*callback).ref_count;
        let count = *ref_count.get();
        *ref_count.get() = count + 1;
        count + 1
    }

    unsafe extern "system" fn release(this: *mut crate::ffi::IUnknown) -> u32 {
        let callback = this as *mut ExtractCallback;
        let ref_count = &(*callback).ref_count;
        let count = *ref_count.get();
        if count > 0 {
            *ref_count.get() = count - 1;
            count - 1
        } else {
            0
        }
    }

    unsafe extern "system" fn set_total(
        _this: *mut IArchiveExtractCallback,
        _total: u64,
    ) -> HRESULT {
        0 // S_OK
    }

    unsafe extern "system" fn set_completed(
        _this: *mut IArchiveExtractCallback,
        _complete_value: *const u64,
    ) -> HRESULT {
        0 // S_OK
    }

    unsafe extern "system" fn get_stream(
        this: *mut IArchiveExtractCallback,
        index: u32,
        out_stream: *mut *mut ISequentialOutStream,
        ask_extract_mode: *mut i32,
    ) -> HRESULT {
        let callback = this as *mut ExtractCallback;
        let archive_vtable = (*(*callback).archive).vtable;

        // Get item path
        let mut prop = std::mem::zeroed::<PROPVARIANT>();
        let result = (archive_vtable.get_property)(
            (*callback).archive,
            index,
            crate::ffi::kpidPath,
            &mut prop,
        );

        if result != 0 {
            *ask_extract_mode = 0; // kExtract = 0
            *out_stream = ptr::null_mut();
            return 0; // S_OK, but skip
        }

        let path = crate::ffi::propvariant_to_string(&prop).unwrap_or_default();

        // Get item attributes
        let result = (archive_vtable.get_property)(
            (*callback).archive,
            index,
            crate::ffi::kpidAttrib,
            &mut prop,
        );

        if result == 0 {
            let attrib = crate::ffi::propvariant_to_u32(&prop);
            let is_dir = (attrib & 0x10) != 0; // FILE_ATTRIBUTE_DIRECTORY

            if is_dir {
                // Create directory
                let output_path = (*callback).output_dir.join(&path);
                if let Err(e) = fs::create_dir_all(&output_path) {
                    eprintln!("Failed to create directory {}: {}", output_path.display(), e);
                }
                *ask_extract_mode = 0;
                *out_stream = ptr::null_mut();
                return 0;
            }
        }

        // Create output stream for file
        let output_path = (*callback).output_dir.join(&path);

        // Validate path to prevent traversal attacks
        if let Err(e) = BitExtractor::validate_path(&output_path, &(*callback).output_dir) {
            eprintln!("Path validation failed: {}", e);
            *ask_extract_mode = 0;
            *out_stream = ptr::null_mut();
            return 0;
        }

        // Create parent directories if needed
        if let Some(parent) = output_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("Failed to create parent directory {}: {}", parent.display(), e);
                *ask_extract_mode = 0;
                *out_stream = ptr::null_mut();
                return 0;
            }
        }

        match FileStreamWrite::new(&output_path) {
            Ok(stream) => {
                let stream_ptr = Box::leak(Box::new(stream));
                *out_stream = stream_ptr as *mut FileStreamWrite as *mut ISequentialOutStream;
                *(*callback).current_out_stream.get() = Some(*out_stream);
                *(*callback).current_path.get() = Some(path);
                *ask_extract_mode = 0; // kExtract
                0 // S_OK
            }
            Err(e) => {
                eprintln!("Failed to create output stream for {}: {}", output_path.display(), e);
                *ask_extract_mode = 0;
                *out_stream = ptr::null_mut();
                0 // S_OK, but skip
            }
        }
    }

    unsafe extern "system" fn prepare_operation(
        _this: *mut IArchiveExtractCallback,
        _ask_extract_mode: i32,
    ) -> HRESULT {
        0 // S_OK
    }

    unsafe extern "system" fn set_operation_result(
        this: *mut IArchiveExtractCallback,
        result_e_operation_result: i32,
    ) -> HRESULT {
        // result_e_operation_result: 0 = kOK, 1 = kUnSupportedMethod, etc.
        if result_e_operation_result != 0 {
            let callback = this as *mut ExtractCallback;
            if let Some(path) = &*(*callback).current_path.get() {
                eprintln!("Extraction failed for {}: operation result {}", path, result_e_operation_result);
            }
        }
        0 // S_OK
    }
}

// Note: In a full implementation, we would need to add QueryInterface support
// to return ICryptoGetTextPassword interface when requested