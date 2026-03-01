//! Archive extractor implementation
//!
//! This module provides BitExtractor for extracting files from archives.

use crate::ffi::{
    BitLibrary, IInArchive, IArchiveExtractCallback, IArchiveOpenCallback,
    ISequentialOutStream, ICryptoGetTextPassword, IInStream,
    PROPVARIANT, HRESULT, IArchiveExtractCallbackVTable, IInArchiveVTable,
    IArchiveOpenCallbackVTable, IUnknownVTable, ICryptoGetTextPasswordVTable,
    IArchiveOpenVolumeCallback, IArchiveOpenVolumeCallbackVTable,
    IArchiveOpenSetSubArchiveName, IArchiveOpenSetSubArchiveNameVTable,
};
use crate::format::ExtractFormat;
use crate::error::{Bit7zError, Result};
use crate::stream::{FileStreamWrite, BufferInStream};
use crate::callback::OpenCallback;
use std::cell::UnsafeCell;
use std::ffi::c_void;
use std::fs;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::ptr;
use std::ffi::c_uint;

/// Extractor for extracting files from archives
pub struct BitExtractor<'a> {
    library: &'a BitLibrary,
    format: ExtractFormat,
    password: Option<String>,
}

impl<'a> BitExtractor<'a> {
    pub fn new(library: &'a BitLibrary, format: ExtractFormat) -> Self {
        BitExtractor {
            library,
            format,
            password: None,
        }
    }

    pub fn password(&mut self, password: impl Into<String>) -> &mut Self {
        self.password = Some(password.into());
        self
    }

    pub fn extract<P: AsRef<Path>>(
        &self,
        archive_path: P,
        output_dir: P,
    ) -> Result<()> {
        unsafe {
            let archive_ptr = self.library.create_in_archive(&self.format.guid())?;

            let in_stream = Box::leak(Box::new(
                crate::stream::FileStream::new(archive_path.as_ref())?
            ));

            // For some formats (like TAR), we need to pass null or they fail to open
            let max_check_start_position: *const u64 = std::ptr::null();

            let open_callback = Box::leak(Box::new(OpenCallback::new(archive_path.as_ref())));

            let result = ((*(*archive_ptr.as_ptr()).vtable).open)(
                archive_ptr.as_ptr(),
                in_stream.as_i_in_stream(),
                max_check_start_position,
                open_callback.as_i_archive_open_callback(),
            );

            // S_OK (0) and S_FALSE (1) are both success for some formats
            if result != 0 && result != 1 {
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

            let output_path = output_dir.as_ref();
            if !output_path.exists() {
                fs::create_dir_all(output_path)?;
            }

            let callback = Box::leak(Box::new(ExtractCallback::new(
                output_path,
                self.password.clone(),
                archive_ptr.as_ptr(),
            )));

            let result = ((*(*archive_ptr.as_ptr()).vtable).extract)(
                archive_ptr.as_ptr(),
                ptr::null(),
                0xFFFFFFFF,
                0,
                callback.as_i_archive_extract_callback(),
            );

            let _ = ((*(*archive_ptr.as_ptr()).vtable).close)(archive_ptr.as_ptr());

            // S_OK (0) and S_FALSE (1) are both success codes
            // S_FALSE is returned for some formats (like GZip/BZip2/XZ) after successful extraction
            if result != 0 && result != 1 {
                return Err(Bit7zError::ExtractFailed(format!(
                    "Extraction failed: HRESULT 0x{:08X}", result
                )));
            }

            Ok(())
        }
    }

    pub fn extract_from_buffer<P: AsRef<Path>>(
        &self,
        buffer: Vec<u8>,
        output_dir: P,
    ) -> Result<()> {
        unsafe {
            let archive_ptr = self.library.create_in_archive(&self.format.guid())?;

            let in_stream = Box::leak(Box::new(
                BufferInStream::new(buffer)
            ));

            // Create open callback (required by 7-Zip)
            let open_callback = Box::leak(Box::new(OpenCallback::new(Path::new(""))));

            // max_check_start_position pointer (0 means search from beginning)
            let max_check_start_position: u64 = 0;

            let result = ((*(*archive_ptr.as_ptr()).vtable).open)(
                archive_ptr.as_ptr(),
                in_stream.as_i_in_stream(),
                &max_check_start_position,  // Pass as pointer
                open_callback.as_i_archive_open_callback(),
            );

            if result != 0 {
                return Err(Bit7zError::OpenFailed(format!(
                    "Failed to open archive from buffer: HRESULT 0x{:08X}", result
                )));
            }

            let output_path = output_dir.as_ref();
            if !output_path.exists() {
                fs::create_dir_all(output_path)?;
            }

            let callback = Box::leak(Box::new(ExtractCallback::new(
                output_path,
                self.password.clone(),
                archive_ptr.as_ptr(),
            )));

            let result = ((*(*archive_ptr.as_ptr()).vtable).extract)(
                archive_ptr.as_ptr(),
                ptr::null(),
                0xFFFFFFFF,
                0,
                callback.as_i_archive_extract_callback(),
            );

            let _ = ((*(*archive_ptr.as_ptr()).vtable).close)(archive_ptr.as_ptr());

            // S_OK (0) and S_FALSE (1) are both success codes
            // S_FALSE is returned for some formats (like GZip/BZip2/XZ) after successful extraction
            if result != 0 && result != 1 {
                return Err(Bit7zError::ExtractFailed(format!(
                    "Extraction failed: HRESULT 0x{:08X}", result
                )));
            }

            Ok(())
        }
    }

    pub fn extract_matching<P: AsRef<Path>>(
        &self,
        archive_path: P,
        output_dir: P,
        pattern: &str,
    ) -> Result<()> {
        unsafe {
            let archive_ptr = self.library.create_in_archive(&self.format.guid())?;

            let in_stream = Box::leak(Box::new(
                crate::stream::FileStream::new(archive_path.as_ref())?
            ));

            // Create open callback (required by 7-Zip)
            let open_callback = Box::leak(Box::new(OpenCallback::new(archive_path.as_ref())));

            // max_check_start_position pointer (0 means search from beginning)
            let max_check_start_position: u64 = 0;

            let result = ((*(*archive_ptr.as_ptr()).vtable).open)(
                archive_ptr.as_ptr(),
                in_stream.as_i_in_stream(),
                &max_check_start_position,  // Pass as pointer
                open_callback.as_i_archive_open_callback(),
            );

            if result != 0 {
                return Err(Bit7zError::OpenFailed(format!(
                    "Failed to open archive: HRESULT 0x{:08X}", result
                )));
            }

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
                    prop.clear(); // Clear after use
                    if Self::matches_pattern(&path, &wildcard) {
                        indices.push(i);
                    }
                }
            }

            let output_path = output_dir.as_ref();
            if !output_path.exists() {
                fs::create_dir_all(output_path)?;
            }

            let callback = Box::leak(Box::new(ExtractCallback::new(
                output_path,
                self.password.clone(),
                archive_ptr.as_ptr(),
            )));

            let result = ((*(*archive_ptr.as_ptr()).vtable).extract)(
                archive_ptr.as_ptr(),
                indices.as_ptr(),
                indices.len() as u32,
                0,
                callback.as_i_archive_extract_callback(),
            );

            let _ = ((*(*archive_ptr.as_ptr()).vtable).close)(archive_ptr.as_ptr());

            if result != 0 {
                return Err(Bit7zError::ExtractFailed(format!(
                    "Extraction failed: HRESULT 0x{:08X}", result
                )));
            }

            Ok(())
        }
    }

    fn validate_path(path: &Path, output_dir: &Path) -> Result<PathBuf> {
        // Simple path validation: check that the path doesn't contain parent directory
        // components that would escape the output directory

        for component in path.components() {
            if component == std::path::Component::ParentDir {
                return Err(Bit7zError::PathTraversal(format!(
                    "Path traversal detected: {}", path.display()
                )));
            }
        }

        // Get canonical/absolute form of output_dir (must exist)
        let output_canonical = output_dir.canonicalize().map_err(|e| {
            Bit7zError::Io(e)
        })?;

        // For paths that contain directory components (like "test_debug/test.txt"),
        // we only use the file name to avoid creating nested directories
        // This matches the behavior of most archive extractors
        
        // For single-file formats (GZip/BZip2/XZ), the path may be empty
        // In this case, we use a default file name based on the output directory
        let file_name = path.file_name()
            .and_then(|name| name.to_str())
            .filter(|s| !s.is_empty());
        
        let output_file_name = match file_name {
            Some(name) => name.to_string(),
            None => {
                // Use the output directory name as the file name for single-file formats
                output_dir.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("extracted_file")
                    .to_string()
            }
        };

        // Build the full output path using the file name
        let full_path = output_canonical.join(&output_file_name);

        // For the full path, we need to ensure parent directories exist
        if let Some(parent) = full_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        // Check if full_path starts with output_canonical
        let path_starts_with_output = full_path.starts_with(&output_canonical);

        if !path_starts_with_output {
            return Err(Bit7zError::PathTraversal(format!(
                "Path traversal detected: {} escapes output directory", path.display()
            )));
        }

        Ok(full_path)
    }

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

    fn matches_pattern(path: &str, pattern: &[String]) -> bool {
        if pattern.is_empty() {
            return true;
        }

        let mut path_chars = path.chars().peekable();
        let mut pattern_idx = 0;

        while pattern_idx < pattern.len() {
            match pattern[pattern_idx].as_str() {
                "*" => {
                    if pattern_idx == pattern.len() - 1 {
                        return true;
                    }

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

        path_chars.peek().is_none()
    }
}

/// ExtractCallback implements IArchiveExtractCallback for extracting files
///
/// Memory layout: vtable must be first to match C++ COM object layout
#[repr(C)]
struct ExtractCallback {
    vtable: Pin<Box<IArchiveExtractCallbackVTable>>,
    ref_count: UnsafeCell<u32>,
    output_dir: PathBuf,
    password: Option<String>,
    archive: *mut IInArchive,
    current_out_stream: UnsafeCell<Option<*mut ISequentialOutStream>>,
    current_path: UnsafeCell<Option<String>>,
}

impl ExtractCallback {
    fn new(output_dir: &Path, password: Option<String>, archive: *mut IInArchive) -> Self {
        let vtable = Box::pin(IArchiveExtractCallbackVTable {
            base: crate::ffi::IProgressVTable {
                base: crate::ffi::IUnknownVTable {
                    query_interface: Self::query_interface,
                    add_ref: Self::add_ref,
                    release: Self::release,
                },
                set_completed: Self::set_completed,
                set_total: Self::set_total,
            },
            get_stream: Self::get_stream,
            prepare_operation: Self::prepare_operation,
            set_operation_result: Self::set_operation_result,
        });

        ExtractCallback {
            vtable,
            ref_count: UnsafeCell::new(1),
            output_dir: output_dir.to_path_buf(),
            password,
            archive,
            current_out_stream: UnsafeCell::new(None),
            current_path: UnsafeCell::new(None),
        }
    }

    fn as_i_archive_extract_callback(&self) -> *mut IArchiveExtractCallback {
        self as *const ExtractCallback as *mut ExtractCallback as *mut IArchiveExtractCallback
    }

    fn as_i_crypto_get_text_password(&self) -> *mut ICryptoGetTextPassword {
        self as *const ExtractCallback as *mut ExtractCallback as *mut ICryptoGetTextPassword
    }

    unsafe extern "system" fn query_interface(
        _this: *mut crate::ffi::IUnknown,
        _iid: *const crate::ffi::GUID,
        _out: *mut *mut c_void,
    ) -> HRESULT {
        -1
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
        _this: *mut crate::ffi::IProgress,
        _total: u64,
    ) -> HRESULT {
        0
    }

    unsafe extern "system" fn set_completed(
        _this: *mut crate::ffi::IProgress,
        _complete_value: *const u64,
    ) -> HRESULT {
        0
    }

    unsafe extern "system" fn get_stream(
        this: *mut IArchiveExtractCallback,
        index: u32,
        out_stream: *mut *mut ISequentialOutStream,
        ask_extract_mode: *mut i32,
    ) -> HRESULT {
        let callback = &*(this as *const ExtractCallback);
        let archive_vtable = unsafe { &*(*callback.archive).vtable };

        // Get item path
        let mut prop = std::mem::zeroed::<PROPVARIANT>();
        let result = (archive_vtable.get_property)(
            callback.archive,
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
        prop.clear(); // Clear after use

        // Get item attributes
        let result = (archive_vtable.get_property)(
            callback.archive,
            index,
            crate::ffi::kpidAttrib,
            &mut prop,
        );

        if result == 0 {
            let attrib = crate::ffi::propvariant_to_u32(&prop);
            prop.clear(); // Clear after use
            let is_dir = (attrib & 0x10) != 0;

            if is_dir {
                // Validate the relative path and get output path
                match BitExtractor::validate_path(Path::new(&path), &callback.output_dir) {
                    Ok(output_path) => {
                        if let Err(e) = fs::create_dir_all(&output_path) {
                            eprintln!("Failed to create directory {}: {}", output_path.display(), e);
                        }
                    }
                    Err(e) => {
                        eprintln!("Path validation failed for directory: {}", e);
                    }
                }
                *ask_extract_mode = 0;
                *out_stream = ptr::null_mut();
                return 0;
            }
        }

        // Validate the relative path and get output path
        let output_path = match BitExtractor::validate_path(Path::new(&path), &callback.output_dir) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Path validation failed: {}", e);
                *ask_extract_mode = 0;
                *out_stream = ptr::null_mut();
                return 0;
            }
        };

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
                // Store the stream pointer for later cleanup
                unsafe {
                    *callback.current_out_stream.get() = Some(*out_stream);
                    *callback.current_path.get() = Some(path.clone());
                }
                if !ask_extract_mode.is_null() {
                    *ask_extract_mode = 0;
                }
                0
            }
            Err(e) => {
                eprintln!("Failed to create output stream for {}: {}", output_path.display(), e);
                if !ask_extract_mode.is_null() {
                    *ask_extract_mode = 0;
                }
                *out_stream = ptr::null_mut();
                0
            }
        }
    }

    unsafe extern "system" fn prepare_operation(
        _this: *mut IArchiveExtractCallback,
        _ask_extract_mode: i32,
    ) -> HRESULT {
        0
    }

    unsafe extern "system" fn set_operation_result(
        this: *mut IArchiveExtractCallback,
        result_e_operation_result: i32,
    ) -> HRESULT {
        if result_e_operation_result != 0 {
            let callback = this as *mut ExtractCallback;
            if let Some(path) = &*(*callback).current_path.get() {
                eprintln!("Extraction failed for {}: operation result {}", path, result_e_operation_result);
            }
        }
        0
    }
}