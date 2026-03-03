//! Archive extractor implementation
//!
//! This module provides BitExtractor for extracting files from archives.

use crate::ffi::{
    BitLibrary, IInArchive, IArchiveExtractCallback,
    ISequentialOutStream, ICryptoGetTextPassword,
    PROPVARIANT, HRESULT, IArchiveExtractCallbackVTable, ICryptoGetTextPasswordVTable,
    ICompressProgressInfo, ICompressProgressInfoVTable,
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

    /// Extract a specific item from archive to a memory buffer
    ///
    /// # Arguments
    /// * `archive_path` - Path to archive
    /// * `index` - Index of item to extract
    ///
    /// # Returns
    /// * `Ok(Vec<u8>)` - Extracted data
    /// * `Err(Bit7zError)` - Error
    pub fn extract_to_buffer<P: AsRef<Path>>(
        &self,
        archive_path: P,
        index: u32,
    ) -> Result<Vec<u8>> {
        use crate::stream::BufferOutStream;

        unsafe {
            let archive_ptr = self.library.create_in_archive(&self.format.guid())?;

            // Create input stream for archive file
            let in_stream = crate::stream::FileStream::new(archive_path.as_ref())?;
            let in_stream_box = Box::new(in_stream);
            let in_stream_ptr = Box::into_raw(in_stream_box);

            // Create open callback
            let open_callback = OpenCallback::new(archive_path.as_ref());
            let open_callback_box = Box::new(open_callback);
            let open_callback_ptr = Box::into_raw(open_callback_box);

            let open_result = ((*(*archive_ptr.as_ptr()).vtable).open)(
                archive_ptr.as_ptr(),
                (*in_stream_ptr).as_i_in_stream(),
                ptr::null(),
                (*open_callback_ptr).as_i_archive_open_callback(),
            );

            if open_result != 0 {
                // Clean up on failure
                let _ = Box::from_raw(in_stream_ptr);
                let _ = Box::from_raw(open_callback_ptr);
                return Err(Bit7zError::OpenFailed(format!(
                    "Failed to open archive: HRESULT 0x{:08X}", open_result
                )));
            }

            // Create buffer output stream
            let buffer_stream = BufferOutStream::new();
            let buffer_stream_box = Box::new(buffer_stream);
            let buffer_stream_ptr = Box::into_raw(buffer_stream_box);

            // Create extract callback for buffer extraction
            let callback = ExtractCallback::with_buffer(
                buffer_stream_ptr,
                self.password.clone(),
                archive_ptr.as_ptr(),
            );
            let callback_box = Box::new(callback);
            let callback_ptr = Box::into_raw(callback_box);

            // Extract specific item by index
            let mut indices = [index];
            let indices_ptr = indices.as_mut_ptr();

            let result = ((*(*archive_ptr.as_ptr()).vtable).extract)(
                archive_ptr.as_ptr(),
                indices_ptr,
                1, // number of indices
                0, // extract mode: 0 = extract
                callback_ptr as *mut IArchiveExtractCallback,
            );

            // After extract returns, release our reference
            // The callback will be freed when ref_count reaches 0
            ExtractCallback::release_caller_reference(callback_ptr);

            if result != 0 {
                // Clean up on failure
                let _ = Box::from_raw(in_stream_ptr);
                let _ = Box::from_raw(open_callback_ptr);
                return Err(Bit7zError::ExtractFailed(format!(
                    "Extract failed with HRESULT: 0x{:X}", result
                )));
            }

            // Get buffer content
            let buffer = (*buffer_stream_ptr).get_buffer();

            // Clean up
            let _ = Box::from_raw(in_stream_ptr);
            let _ = Box::from_raw(open_callback_ptr);
            // buffer_stream is owned by callback, which will free it

            Ok(buffer)
        }
    }

    /// Extract a specific item from archive to a writer stream
    ///
    /// # Arguments
    /// * `archive_path` - Path to archive
    /// * `writer` - Output stream to write data to
    /// * `index` - Index of item to extract
    ///
    /// # Returns
    /// * `Ok(())` - Success
    /// * `Err(Bit7zError)` - Error
    pub fn extract_to_stream<P: AsRef<Path>, W: std::io::Write>(
        &self,
        archive_path: P,
        writer: &mut W,
        index: u32,
    ) -> Result<()> {
        
        
        // Extract to temporary directory first, then copy to stream
        let temp_dir = std::env::temp_dir().join(format!(
            "bit7z_extract_{}_{}",
            std::process::id(),
            index
        ));
        
        // Create temp directory
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| Bit7zError::ExtractFailed(e.to_string()))?;

        // Extract to temp directory
        self.extract(archive_path.as_ref(), &temp_dir)?;

        // Find the extracted file (it should be the only file in temp_dir)
        let mut temp_file_path = None;
        if let Ok(entries) = std::fs::read_dir(&temp_dir) {
            for entry in entries.flatten() {
                if entry.path().is_file() {
                    temp_file_path = Some(entry.path());
                    break;
                }
            }
        }

        let temp_file_path = temp_file_path
            .ok_or_else(|| Bit7zError::ExtractFailed("No file extracted".to_string()))?;

        // Read temp file and write to stream
        let mut temp_file = std::fs::File::open(&temp_file_path)
            .map_err(|e| Bit7zError::ExtractFailed(e.to_string()))?;
        
        std::io::copy(&mut temp_file, writer)
            .map_err(|e| Bit7zError::ExtractFailed(e.to_string()))?;

        // Clean up temp directory
        let _ = std::fs::remove_dir_all(&temp_dir);

        Ok(())
    }

    /// Extract specific items by index list
    ///
    /// # Arguments
    /// * `archive_path` - Path to archive
    /// * `indices` - List of item indices to extract
    /// * `output_dir` - Output directory
    ///
    /// # Returns
    /// * `Ok(())` - Success
    /// * `Err(Bit7zError)` - Error
    pub fn extract_items<P: AsRef<Path>>(
        &self,
        archive_path: P,
        indices: &[u32],
        output_dir: P,
    ) -> Result<()> {
        if indices.is_empty() {
            return Ok(()); // Nothing to extract
        }

        unsafe {
            let archive_ptr = self.library.create_in_archive(&self.format.guid())?;

            let in_stream = Box::leak(Box::new(
                crate::stream::FileStream::new(archive_path.as_ref())?
            ));

            let open_callback = Box::leak(Box::new(OpenCallback::new(archive_path.as_ref())));

            let open_result = ((*(*archive_ptr.as_ptr()).vtable).open)(
                archive_ptr.as_ptr(),
                in_stream.as_i_in_stream(),
                ptr::null(),
                open_callback.as_i_archive_open_callback(),
            );

            if open_result != 0 {
                return Err(Bit7zError::OpenFailed(format!(
                    "Failed to open archive: HRESULT 0x{:08X}", open_result
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

            // Extract specific indices
            let indices_ptr = indices.as_ptr() as *mut u32;
            let num_indices = indices.len() as u32;

            let result = ((*(*archive_ptr.as_ptr()).vtable).extract)(
                archive_ptr.as_ptr(),
                indices_ptr,
                num_indices,
                0, // extract mode: 0 = extract
                callback as *mut ExtractCallback as *mut IArchiveExtractCallback,
            );

            if result != 0 {
                return Err(Bit7zError::ExtractFailed(format!(
                    "Extract failed with HRESULT: 0x{:X}", result
                )));
            }

            Ok(())
        }
    }

    /// Extract items matching a wildcard pattern
    ///
    /// # Arguments
    /// * `archive_path` - Path to archive
    /// * `pattern` - Wildcard pattern (e.g., "*.txt", "docs/*")
    /// * `output_dir` - Output directory
    ///
    /// # Returns
    /// * `Ok(())` - Success
    /// * `Err(Bit7zError)` - Error
    pub fn extract_matching<P: AsRef<Path>>(
        &self,
        archive_path: P,
        pattern: &str,
        output_dir: P,
    ) -> Result<()> {
        use crate::archive_reader::BitArchiveReader;
        
        // Open archive to get items
        let mut reader = BitArchiveReader::new(self.library, self.format);
        reader.open(archive_path.as_ref())?;
        
        // Get all items and filter by pattern
        let items = reader.items()?;
        let matching_indices: Vec<u32> = items
            .iter()
            .filter(|item| Self::match_wildcard(pattern, &item.path))
            .map(|item| item.index)
            .collect();
        
        if matching_indices.is_empty() {
            return Ok(()); // No matching items
        }
        
        // Extract matching items
        self.extract_items(archive_path, &matching_indices, output_dir)
    }

    /// Extract items matching a regex pattern
    ///
    /// # Arguments
    /// * `archive_path` - Path to archive
    /// * `pattern` - Regex pattern
    /// * `output_dir` - Output directory
    ///
    /// # Returns
    /// * `Ok(())` - Success
    /// * `Err(Bit7zError)` - Error
    pub fn extract_matching_regex<P: AsRef<Path>>(
        &self,
        archive_path: P,
        pattern: &str,
        output_dir: P,
    ) -> Result<()> {
        use crate::archive_reader::BitArchiveReader;
        
        // Compile regex
        let regex = regex::Regex::new(pattern)
            .map_err(|e| Bit7zError::ExtractFailed(format!("Invalid regex: {}", e)))?;
        
        // Open archive to get items
        let mut reader = BitArchiveReader::new(self.library, self.format);
        reader.open(archive_path.as_ref())?;
        
        // Get all items and filter by regex
        let items = reader.items()?;
        let matching_indices: Vec<u32> = items
            .iter()
            .filter(|item| regex.is_match(&item.path))
            .map(|item| item.index)
            .collect();
        
        if matching_indices.is_empty() {
            return Ok(()); // No matching items
        }
        
        // Extract matching items
        self.extract_items(archive_path, &matching_indices, output_dir)
    }

    /// Match a path against a wildcard pattern
    fn match_wildcard(pattern: &str, path: &str) -> bool {
        // Simple wildcard matching: * matches any sequence, ? matches single char
        let pattern_chars: Vec<char> = pattern.chars().collect();
        let path_chars: Vec<char> = path.chars().collect();
        
        Self::wildcard_match_recursive(&pattern_chars, &path_chars, 0, 0)
    }

    fn wildcard_match_recursive(pattern: &[char], path: &[char], pi: usize, si: usize) -> bool {
        // Base case: both pattern and path are exhausted
        if pi == pattern.len() && si == path.len() {
            return true;
        }
        
        // If pattern is exhausted but path isn't, check for trailing *
        if pi == pattern.len() {
            return false;
        }
        
        // Handle * wildcard
        if pattern[pi] == '*' {
            // Try matching zero or more characters
            for i in si..=path.len() {
                if Self::wildcard_match_recursive(pattern, path, pi + 1, i) {
                    return true;
                }
            }
            return false;
        }
        
        // If path is exhausted but pattern isn't
        if si == path.len() {
            return false;
        }
        
        // Match current character or ? wildcard
        if pattern[pi] == '?' || pattern[pi] == path[si] {
            return Self::wildcard_match_recursive(pattern, path, pi + 1, si + 1);
        }
        
        false
    }

    /// Test archive integrity without extracting
    pub fn test<P: AsRef<Path>>(&self, archive_path: P) -> Result<()> {
        unsafe {
            let archive_ptr = self.library.create_in_archive(&self.format.guid())?;

            let in_stream = Box::leak(Box::new(
                crate::stream::FileStream::new(archive_path.as_ref())?
            ));

            let open_callback = Box::leak(Box::new(OpenCallback::new(archive_path.as_ref())));

            let open_result = ((*(*archive_ptr.as_ptr()).vtable).open)(
                archive_ptr.as_ptr(),
                in_stream.as_i_in_stream(),
                ptr::null(),
                open_callback.as_i_archive_open_callback(),
            );

            if open_result != 0 {
                return Err(Bit7zError::OpenFailed(format!(
                    "Failed to open archive: HRESULT 0x{:08X}", open_result
                )));
            }

            // Get number of items
            let mut num_items: u32 = 0;
            ((*(*archive_ptr.as_ptr()).vtable).get_number_of_items)(
                archive_ptr.as_ptr(),
                &mut num_items,
            );

            // Create a null output stream for testing (we don't want to extract)
            // Using kExtractMode::Test (2) - test archive integrity
            let callback = Box::leak(Box::new(ExtractCallback::new(
                Path::new("/dev/null"), // Dummy path - won't be used for test mode
                self.password.clone(),
                archive_ptr.as_ptr(),
            )));

            // Call Extract with kExtractMode::Test (2)
            // This tests the archive without actually extracting files
            let _extract_mode: i32 = 2; // kExtractMode::Test
            let mut indices: [i32; 1] = [-1]; // -1 means all items
            let indices_ptr = indices.as_mut_ptr() as *mut u32;

            let result = ((*(*archive_ptr.as_ptr()).vtable).extract)(
                archive_ptr.as_ptr(),
                indices_ptr,
                num_items,
                0, // test_all = 0 when indices is -1
                callback as *mut ExtractCallback as *mut IArchiveExtractCallback,
            );

            if result != 0 {
                return Err(Bit7zError::ArchiveIntegrityCheckFailed(format!(
                    "Archive test failed with HRESULT: 0x{:X}", result
                )));
            }

            Ok(())
        }
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

            let open_callback = Box::leak(Box::new(OpenCallback::new(archive_path.as_ref())));

            // Match bit7z default behavior: pass nullptr for maxCheckStartPosition.
            let open_result = ((*(*archive_ptr.as_ptr()).vtable).open)(
                archive_ptr.as_ptr(),
                in_stream.as_i_in_stream(),
                ptr::null(),
                open_callback.as_i_archive_open_callback(),
            );

            // Match bit7z: only S_OK is success for open().
            if open_result != 0 {
                return Err(Bit7zError::OpenFailed(format!(
                    "Failed to open archive: HRESULT 0x{:08X}", open_result
                )));
            }

            // Get number of items
            let mut num_items: u32 = 0;
            let _get_num_result = ((*(*archive_ptr.as_ptr()).vtable).get_number_of_items)(
                archive_ptr.as_ptr(),
                &mut num_items,
            );

            // Note: We don't check for invalid archives here because:
            // 1. Some valid single-file formats may return 0 items initially
            // 2. bit7z also doesn't check items count after open
            // The validity check is done during extract operation

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

            if result != 0 {
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

            let result = ((*(*archive_ptr.as_ptr()).vtable).open)(
                archive_ptr.as_ptr(),
                in_stream.as_i_in_stream(),
                ptr::null(),
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
/// 
/// According to 7-Zip source code, ExtractCallback also implements:
/// - ICompressProgressInfo (for progress reporting via SetRatioInfo)
/// - ICryptoGetTextPassword (for password-protected archives)
/// 
/// We store additional vtables for these interfaces.
#[repr(C)]
struct ExtractCallback {
    vtable: Pin<Box<IArchiveExtractCallbackVTable>>,
    // Additional vtables for supported interfaces
    compress_progress_vtable: Pin<Box<ICompressProgressInfoVTable>>,
    crypto_password_vtable: Pin<Box<ICryptoGetTextPasswordVTable>>,
    ref_count: UnsafeCell<u32>,
    output_dir: PathBuf,
    password: Option<String>,
    archive: *mut IInArchive,
    current_out_stream: UnsafeCell<Option<*mut ISequentialOutStream>>,
    current_path: UnsafeCell<Option<String>>,
    // For buffer extraction
    buffer_stream: Option<*mut crate::stream::BufferOutStream>,
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

        let compress_progress_vtable = Box::pin(ICompressProgressInfoVTable {
            base: crate::ffi::IUnknownVTable {
                query_interface: unsafe { std::mem::transmute::<
                    unsafe extern "system" fn(*mut ICompressProgressInfo, *const crate::ffi::GUID, *mut *mut c_void) -> HRESULT,
                    unsafe extern "system" fn(*mut crate::ffi::IUnknown, *const crate::ffi::GUID, *mut *mut c_void) -> HRESULT,
                >(Self::compress_progress_query_interface) },
                add_ref: unsafe { std::mem::transmute::<
                    unsafe extern "system" fn(*mut ICompressProgressInfo) -> u32,
                    unsafe extern "system" fn(*mut crate::ffi::IUnknown) -> u32,
                >(Self::compress_progress_add_ref) },
                release: unsafe { std::mem::transmute::<
                    unsafe extern "system" fn(*mut ICompressProgressInfo) -> u32,
                    unsafe extern "system" fn(*mut crate::ffi::IUnknown) -> u32,
                >(Self::compress_progress_release) },
            },
            set_ratio_info: Self::set_ratio_info,
        });

        let crypto_password_vtable = Box::pin(ICryptoGetTextPasswordVTable {
            base: crate::ffi::IUnknownVTable {
                query_interface: unsafe { std::mem::transmute::<
                    unsafe extern "system" fn(*mut ICryptoGetTextPassword, *const crate::ffi::GUID, *mut *mut c_void) -> HRESULT,
                    unsafe extern "system" fn(*mut crate::ffi::IUnknown, *const crate::ffi::GUID, *mut *mut c_void) -> HRESULT,
                >(Self::crypto_password_query_interface) },
                add_ref: unsafe { std::mem::transmute::<
                    unsafe extern "system" fn(*mut ICryptoGetTextPassword) -> u32,
                    unsafe extern "system" fn(*mut crate::ffi::IUnknown) -> u32,
                >(Self::crypto_password_add_ref) },
                release: unsafe { std::mem::transmute::<
                    unsafe extern "system" fn(*mut ICryptoGetTextPassword) -> u32,
                    unsafe extern "system" fn(*mut crate::ffi::IUnknown) -> u32,
                >(Self::crypto_password_release) },
            },
            crypto_get_text_password: Self::get_text_password,
        });

        ExtractCallback {
            vtable,
            compress_progress_vtable,
            crypto_password_vtable,
            ref_count: UnsafeCell::new(1),
            output_dir: output_dir.to_path_buf(),
            password,
            archive,
            current_out_stream: UnsafeCell::new(None),
            current_path: UnsafeCell::new(None),
            buffer_stream: None,
        }
    }

    /// Create ExtractCallback for buffer extraction
    fn with_buffer(
        buffer_stream: *mut crate::stream::BufferOutStream,
        password: Option<String>,
        archive: *mut IInArchive,
    ) -> Self {
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
            get_stream: Self::get_stream_buffer,
            prepare_operation: Self::prepare_operation,
            set_operation_result: Self::set_operation_result,
        });

        let compress_progress_vtable = Box::pin(ICompressProgressInfoVTable {
            base: crate::ffi::IUnknownVTable {
                query_interface: unsafe { std::mem::transmute::<
                    unsafe extern "system" fn(*mut ICompressProgressInfo, *const crate::ffi::GUID, *mut *mut c_void) -> HRESULT,
                    unsafe extern "system" fn(*mut crate::ffi::IUnknown, *const crate::ffi::GUID, *mut *mut c_void) -> HRESULT,
                >(Self::compress_progress_query_interface) },
                add_ref: unsafe { std::mem::transmute::<
                    unsafe extern "system" fn(*mut ICompressProgressInfo) -> u32,
                    unsafe extern "system" fn(*mut crate::ffi::IUnknown) -> u32,
                >(Self::compress_progress_add_ref) },
                release: unsafe { std::mem::transmute::<
                    unsafe extern "system" fn(*mut ICompressProgressInfo) -> u32,
                    unsafe extern "system" fn(*mut crate::ffi::IUnknown) -> u32,
                >(Self::compress_progress_release) },
            },
            set_ratio_info: Self::set_ratio_info,
        });

        let crypto_password_vtable = Box::pin(ICryptoGetTextPasswordVTable {
            base: crate::ffi::IUnknownVTable {
                query_interface: unsafe { std::mem::transmute::<
                    unsafe extern "system" fn(*mut ICryptoGetTextPassword, *const crate::ffi::GUID, *mut *mut c_void) -> HRESULT,
                    unsafe extern "system" fn(*mut crate::ffi::IUnknown, *const crate::ffi::GUID, *mut *mut c_void) -> HRESULT,
                >(Self::crypto_password_query_interface) },
                add_ref: unsafe { std::mem::transmute::<
                    unsafe extern "system" fn(*mut ICryptoGetTextPassword) -> u32,
                    unsafe extern "system" fn(*mut crate::ffi::IUnknown) -> u32,
                >(Self::crypto_password_add_ref) },
                release: unsafe { std::mem::transmute::<
                    unsafe extern "system" fn(*mut ICryptoGetTextPassword) -> u32,
                    unsafe extern "system" fn(*mut crate::ffi::IUnknown) -> u32,
                >(Self::crypto_password_release) },
            },
            crypto_get_text_password: Self::get_text_password,
        });

        ExtractCallback {
            vtable,
            compress_progress_vtable,
            crypto_password_vtable,
            ref_count: UnsafeCell::new(1),
            output_dir: PathBuf::new(),
            password,
            archive,
            current_out_stream: UnsafeCell::new(None),
            current_path: UnsafeCell::new(None),
            buffer_stream: Some(buffer_stream),
        }
    }

    fn as_i_archive_extract_callback(&self) -> *mut IArchiveExtractCallback {
        self as *const ExtractCallback as *mut ExtractCallback as *mut IArchiveExtractCallback
    }

    fn as_i_compress_progress_info(&self) -> *mut ICompressProgressInfo {
        let base = self as *const ExtractCallback as *const u8;
        let offset = std::mem::size_of::<Pin<Box<IArchiveExtractCallbackVTable>>>();
        unsafe { base.add(offset) as *mut ICompressProgressInfo }
    }

    /// Release the caller's reference to the callback.
    /// This should be called after extract returns to decrement the ref count.
    /// If this was the last reference, the callback will be freed.
    /// 
    /// # Safety
    /// This takes ownership of the callback and may free it.
    pub unsafe fn release_caller_reference(callback_ptr: *mut ExtractCallback) {
        let ref_count = &(*callback_ptr).ref_count;
        let count = *ref_count.get();
        if count > 1 {
            // 7-Zip is still holding a reference, decrement and let release free it
            *ref_count.get() = count - 1;
        } else {
            // 7-Zip didn't add a reference, we need to free it
            *ref_count.get() = 0;
            let _ = Box::from_raw(callback_ptr);
        }
    }

    fn as_i_crypto_get_text_password(&self) -> *mut ICryptoGetTextPassword {
        let base = self as *const ExtractCallback as *const u8;
        let offset = std::mem::size_of::<Pin<Box<IArchiveExtractCallbackVTable>>>()
            + std::mem::size_of::<Pin<Box<ICompressProgressInfoVTable>>>();
        unsafe { base.add(offset) as *mut ICryptoGetTextPassword }
    }

    unsafe fn from_archive_extract_callback(this: *mut IArchiveExtractCallback) -> *mut ExtractCallback {
        this as *mut ExtractCallback
    }

    unsafe fn from_compress_progress(this: *mut ICompressProgressInfo) -> *mut ExtractCallback {
        let offset = std::mem::size_of::<Pin<Box<IArchiveExtractCallbackVTable>>>();
        (this as *const u8).sub(offset) as *mut ExtractCallback
    }

    unsafe fn from_crypto_password(this: *mut ICryptoGetTextPassword) -> *mut ExtractCallback {
        let offset = std::mem::size_of::<Pin<Box<IArchiveExtractCallbackVTable>>>()
            + std::mem::size_of::<Pin<Box<ICompressProgressInfoVTable>>>();
        (this as *const u8).sub(offset) as *mut ExtractCallback
    }

    unsafe extern "system" fn extract_query_interface(
        this: *mut IArchiveExtractCallback,
        iid: *const crate::ffi::GUID,
        out: *mut *mut c_void,
    ) -> HRESULT {
        let callback = Self::from_archive_extract_callback(this);
        Self::query_interface(callback as *mut crate::ffi::IUnknown, iid, out)
    }

    unsafe extern "system" fn extract_add_ref(this: *mut IArchiveExtractCallback) -> u32 {
        let callback = Self::from_archive_extract_callback(this);
        Self::add_ref(callback as *mut crate::ffi::IUnknown)
    }

    unsafe extern "system" fn extract_release(this: *mut IArchiveExtractCallback) -> u32 {
        let callback = Self::from_archive_extract_callback(this);
        Self::release(callback as *mut crate::ffi::IUnknown)
    }

    unsafe extern "system" fn compress_progress_query_interface(
        this: *mut ICompressProgressInfo,
        iid: *const crate::ffi::GUID,
        out: *mut *mut c_void,
    ) -> HRESULT {
        let callback = Self::from_compress_progress(this);
        Self::query_interface(callback as *mut crate::ffi::IUnknown, iid, out)
    }

    unsafe extern "system" fn compress_progress_add_ref(this: *mut ICompressProgressInfo) -> u32 {
        let callback = Self::from_compress_progress(this);
        Self::add_ref(callback as *mut crate::ffi::IUnknown)
    }

    unsafe extern "system" fn compress_progress_release(this: *mut ICompressProgressInfo) -> u32 {
        let callback = Self::from_compress_progress(this);
        Self::release(callback as *mut crate::ffi::IUnknown)
    }

    unsafe extern "system" fn crypto_password_query_interface(
        this: *mut ICryptoGetTextPassword,
        iid: *const crate::ffi::GUID,
        out: *mut *mut c_void,
    ) -> HRESULT {
        let callback = Self::from_crypto_password(this);
        Self::query_interface(callback as *mut crate::ffi::IUnknown, iid, out)
    }

    unsafe extern "system" fn crypto_password_add_ref(this: *mut ICryptoGetTextPassword) -> u32 {
        let callback = Self::from_crypto_password(this);
        Self::add_ref(callback as *mut crate::ffi::IUnknown)
    }

    unsafe extern "system" fn crypto_password_release(this: *mut ICryptoGetTextPassword) -> u32 {
        let callback = Self::from_crypto_password(this);
        Self::release(callback as *mut crate::ffi::IUnknown)
    }

    unsafe extern "system" fn query_interface(
        this: *mut crate::ffi::IUnknown,
        iid: *const crate::ffi::GUID,
        out: *mut *mut c_void,
    ) -> HRESULT {
        if out.is_null() || iid.is_null() {
            return -2147467261; // E_POINTER
        }

        let callback = this as *mut ExtractCallback;

        // IID_IUnknown
        let iid_iunknown = crate::ffi::IID_IUnknown;
        if *iid == iid_iunknown {
            *out = this as *mut c_void;
            ExtractCallback::add_ref(this);
            return 0; // S_OK
        }

        // IID_IProgress (IArchiveExtractCallback inherits from IProgress)
        let iid_progress = crate::ffi::IID_IProgress;
        if *iid == iid_progress {
            *out = this as *mut c_void;
            ExtractCallback::add_ref(this);
            return 0; // S_OK
        }

        // IID_IArchiveExtractCallback
        let iid_extract_callback = crate::ffi::IID_IArchiveExtractCallback;
        if *iid == iid_extract_callback {
            *out = this as *mut c_void;
            ExtractCallback::add_ref(this);
            return 0; // S_OK
        }

        // IID_ICompressProgressInfo (for progress reporting during extraction)
        // According to 7-Zip source, ExtractCallback implements this interface
        let iid_compress_progress = crate::ffi::IID_ICompressProgressInfo;
        if *iid == iid_compress_progress {
            *out = (*callback).as_i_compress_progress_info() as *mut c_void;
            ExtractCallback::add_ref(this);
            return 0; // S_OK
        }

        // IID_ICryptoGetTextPassword (for password-protected archives)
        let iid_crypto = crate::ffi::IID_ICryptoGetTextPassword;
        if *iid == iid_crypto {
            *out = (*callback).as_i_crypto_get_text_password() as *mut c_void;
            ExtractCallback::add_ref(this);
            return 0; // S_OK
        }

        *out = ptr::null_mut();
        -2147467262 // E_NOINTERFACE
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
        if count > 1 {
            *ref_count.get() = count - 1;
            count - 1
        } else {
            *ref_count.get() = 0;
            // Free the callback
            let _ = Box::from_raw(callback);
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
        if out_stream.is_null() {
            return -2147467261; // E_POINTER
        }

        *out_stream = ptr::null_mut();
        
        // ask_extract_mode may be NULL for some formats
        if !ask_extract_mode.is_null() {
            *ask_extract_mode = 0; // kExtract = 0
        }

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
                return 0;
            }
        }

        // Validate the relative path and get output path
        let output_path = match BitExtractor::validate_path(Path::new(&path), &callback.output_dir) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Path validation failed: {}", e);
                return 0;
            }
        };

        if let Some(parent) = output_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("Failed to create parent directory {}: {}", parent.display(), e);
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
                0
            }
            Err(e) => {
                eprintln!("Failed to create output stream for {}: {}", output_path.display(), e);
                *out_stream = ptr::null_mut();
                0
            }
        }
    }

    /// Get stream for buffer extraction
    unsafe extern "system" fn get_stream_buffer(
        this: *mut IArchiveExtractCallback,
        _index: u32,
        out_stream: *mut *mut ISequentialOutStream,
        ask_extract_mode: *mut i32,
    ) -> HRESULT {
        if out_stream.is_null() {
            return -2147467261; // E_POINTER
        }

        *out_stream = ptr::null_mut();

        // ask_extract_mode may be NULL for some formats
        if !ask_extract_mode.is_null() {
            *ask_extract_mode = 0; // kExtract = 0
        }

        let callback = &*(this as *const ExtractCallback);

        // Use buffer stream if available
        if let Some(buffer_stream) = callback.buffer_stream {
            *out_stream = (*buffer_stream).as_i_out_stream() as *mut ISequentialOutStream;
            
            // AddRef the stream
            let stream_vtable = &*(*(*buffer_stream).as_i_out_stream()).vtable;
            (stream_vtable.base.base.add_ref)((*buffer_stream).as_i_out_stream() as *mut crate::ffi::IUnknown);
            
            return 0; // S_OK
        }

        0 // S_OK
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

    /// ICompressProgressInfo::SetRatioInfo - called during extraction to report progress
    /// This is required by 7-Zip for some formats
    unsafe extern "system" fn set_ratio_info(
        _this: *mut ICompressProgressInfo,
        _in_size: *const u64,
        _out_size: *const u64,
    ) -> HRESULT {
        // Just ignore, this is optional progress reporting
        0 // S_OK
    }

    /// ICryptoGetTextPassword::CryptoGetTextPassword - called for password-protected archives
    unsafe extern "system" fn get_text_password(
        this: *mut ICryptoGetTextPassword,
        password: *mut *mut u16,
    ) -> HRESULT {
        if password.is_null() {
            return -2147467261; // E_POINTER
        }

        let callback = this as *mut ExtractCallback;

        // Check if we have a password
        if let Some(ref pwd) = (*callback).password {
            // Convert password to UTF-16 BSTR
            let pwd_utf16: Vec<u16> = pwd.encode_utf16().collect();
            let bstr = crate::ffi::variant::alloc_bstr(&pwd_utf16);

            if !bstr.is_null() {
                *password = bstr;
                return 0; // S_OK
            }
        }

        // No password available
        -2147467263 // E_NOTIMPL
    }
}
