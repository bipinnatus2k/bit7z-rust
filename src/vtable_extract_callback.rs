//! ExtractCallback implementation using vtable crate
//!
//! This module provides a vtable-crate-based implementation of
//! IArchiveExtractCallback for 7-Zip extraction operations.
//!
//! **Note**: This is an experimental implementation.
//! The legacy manual implementation in `extractor.rs` is still the default.

use vtable::*;
use crate::vtable_base::*;
use crate::ffi::{
    GUID, HRESULT, IInArchive, ISequentialOutStream, PROPVARIANT, PROPID,
    IID_IUnknown, IID_IProgress, IID_IArchiveExtractCallback,
    IID_ICompressProgressInfo, IID_ICryptoGetTextPassword,
};
use crate::stream::FileStreamWrite;
use crate::ffi::variant::alloc_bstr;
use std::path::{Path, PathBuf};
use std::ptr;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

// ============================================================================
// VTable Definition using #[vtable] macro
// ============================================================================

/// Unified ExtractCallback vtable containing all interface methods
#[vtable]
#[repr(C)]
struct ExtractCallbackVTable {
    // IUnknown methods
    query_interface: fn(VRef<ExtractCallbackVTable>, &GUID) -> *mut std::ffi::c_void,
    add_ref: fn(VRef<ExtractCallbackVTable>) -> u32,
    release: fn(VRefMut<ExtractCallbackVTable>) -> u32,

    // IProgress methods
    set_completed: fn(VRef<ExtractCallbackVTable>, *const u64) -> HRESULT,
    set_total: fn(VRef<ExtractCallbackVTable>, u64) -> HRESULT,

    // IArchiveExtractCallback methods
    get_stream: fn(VRef<ExtractCallbackVTable>, u32, *mut *mut ISequentialOutStream, i32) -> HRESULT,
    prepare_operation: fn(VRef<ExtractCallbackVTable>, i32) -> HRESULT,
    set_operation_result: fn(VRef<ExtractCallbackVTable>, i32) -> HRESULT,

    // ICompressProgressInfo methods
    set_ratio_info: fn(VRef<ExtractCallbackVTable>, *const u64, *const u64) -> HRESULT,

    // ICryptoGetTextPassword methods
    crypto_get_text_password: fn(VRef<ExtractCallbackVTable>, *mut *mut u16) -> HRESULT,

    // Drop destructor
    drop: fn(VRefMut<ExtractCallbackVTable>),
}

// ============================================================================
// ExtractCallback Implementation
// ============================================================================

/// ExtractCallback using vtable crate for automatic vtable management
pub struct VTableExtractCallback {
    output_dir: PathBuf,
    password: Option<String>,
    archive: *mut IInArchive,
    ref_count: AtomicU32,
    current_stream: Mutex<Option<*mut ISequentialOutStream>>,
    current_path: Mutex<Option<String>>,
}

// SAFETY: ExtractCallback is only used in single-threaded 7-Zip callbacks
unsafe impl Send for VTableExtractCallback {}
unsafe impl Sync for VTableExtractCallback {}

impl VTableExtractCallback {
    /// Create a new ExtractCallback for the given output directory
    pub fn new(output_dir: &Path, password: Option<String>, archive: *mut IInArchive) -> Self {
        VTableExtractCallback {
            output_dir: output_dir.to_path_buf(),
            password,
            archive,
            ref_count: AtomicU32::new(1),
            current_stream: Mutex::new(None),
            current_path: Mutex::new(None),
        }
    }

    /// Get as IArchiveExtractCallback pointer for 7-Zip interop
    pub fn as_i_archive_extract_callback(&self) -> *mut crate::ffi::IArchiveExtractCallback {
        self as *const VTableExtractCallback as *mut VTableExtractCallback as *mut crate::ffi::IArchiveExtractCallback
    }

    /// Get as ICompressProgressInfo pointer
    pub fn as_i_compress_progress_info(&self) -> *mut crate::ffi::ICompressProgressInfo {
        self as *const VTableExtractCallback as *mut VTableExtractCallback as *mut crate::ffi::ICompressProgressInfo
    }

    /// Get as ICryptoGetTextPassword pointer
    pub fn as_i_crypto_get_text_password(&self) -> *mut crate::ffi::ICryptoGetTextPassword {
        self as *const VTableExtractCallback as *mut VTableExtractCallback as *mut crate::ffi::ICryptoGetTextPassword
    }

    /// Get item property from archive
    unsafe fn get_item_property(&self, index: u32, prop_id: PROPID) -> Option<PROPVARIANT> {
        if self.archive.is_null() {
            return None;
        }

        let archive_vtable = &*(*self.archive).vtable;
        let mut prop = std::mem::zeroed::<PROPVARIANT>();
        
        let result = (archive_vtable.get_property)(
            self.archive,
            index,
            prop_id,
            &mut prop,
        );

        if result == S_OK {
            Some(prop)
        } else {
            None
        }
    }

    /// Get item path as String
    unsafe fn get_item_path(&self, index: u32) -> String {
        if let Some(mut prop) = self.get_item_property(index, crate::ffi::kpidPath) {
            let path = crate::ffi::propvariant_to_string(&prop).unwrap_or_default();
            prop.clear();
            path
        } else {
            String::new()
        }
    }

    /// Get item attributes
    unsafe fn get_item_attrib(&self, index: u32) -> u32 {
        if let Some(mut prop) = self.get_item_property(index, crate::ffi::kpidAttrib) {
            let attrib = crate::ffi::propvariant_to_u32(&prop);
            prop.clear();
            attrib
        } else {
            0
        }
    }

    /// Validate and create output path
    fn create_output_path(&self, relative_path: &str) -> Option<PathBuf> {
        // Prevent directory traversal attacks
        let path = Path::new(relative_path);
        let mut components = Vec::new();
        
        for component in path.components() {
            if let std::path::Component::Normal(c) = component {
                components.push(c);
            }
        }

        if components.is_empty() {
            return None;
        }

        let clean_path: PathBuf = components.iter().collect();
        Some(self.output_dir.join(clean_path))
    }
}

// ============================================================================
// Trait Implementation
// ============================================================================

impl ExtractCallback for VTableExtractCallback {
    // IUnknown methods
    fn query_interface(&self, iid: &GUID) -> *mut std::ffi::c_void {
        if *iid == IID_IUnknown
            || *iid == IID_IProgress
            || *iid == IID_IArchiveExtractCallback
            || *iid == IID_ICompressProgressInfo
            || *iid == IID_ICryptoGetTextPassword
        {
            return self as *const _ as *mut _;
        }
        ptr::null_mut()
    }

    fn add_ref(&self) -> u32 {
        self.ref_count.fetch_add(1, Ordering::SeqCst) + 1
    }

    fn release(&mut self) -> u32 {
        self.ref_count.fetch_sub(1, Ordering::SeqCst) - 1
    }

    // IProgress methods
    fn set_completed(&self, _complete_value: *const u64) -> HRESULT {
        S_OK
    }

    fn set_total(&self, _total: u64) -> HRESULT {
        S_OK
    }

    // IArchiveExtractCallback methods
    fn get_stream(
        &self,
        index: u32,
        out_stream: *mut *mut ISequentialOutStream,
        ask_extract_mode: i32,
    ) -> HRESULT {
        if out_stream.is_null() {
            return E_POINTER;
        }

        unsafe {
            *out_stream = ptr::null_mut();

            if ask_extract_mode != 0 {
                return S_OK;
            }

            // Get item path
            let path = self.get_item_path(index);
            
            // Get item attributes
            let attrib = self.get_item_attrib(index);
            let is_dir = (attrib & 0x10) != 0; // FILE_ATTRIBUTE_DIRECTORY

            if is_dir {
                // Create directory
                if let Some(output_path) = self.create_output_path(&path) {
                    if let Err(e) = std::fs::create_dir_all(&output_path) {
                        eprintln!("Failed to create directory {}: {}", output_path.display(), e);
                    }
                }
                return S_OK;
            }

            // Create output path for file
            let output_path = match self.create_output_path(&path) {
                Some(p) => p,
                None => return S_OK,
            };

            // Create parent directories
            if let Some(parent) = output_path.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    eprintln!("Failed to create parent directory {}: {}", parent.display(), e);
                    return S_OK;
                }
            }

            // Create file output stream
            match FileStreamWrite::new(&output_path) {
                Ok(stream) => {
                    let stream_box = Box::new(stream);
                    let stream_ptr = Box::leak(stream_box);
                    *out_stream = stream_ptr as *mut FileStreamWrite as *mut ISequentialOutStream;
                    
                    // Store for cleanup
                    *self.current_stream.lock().unwrap() = Some(*out_stream);
                    *self.current_path.lock().unwrap() = Some(path);
                    
                    S_OK
                }
                Err(e) => {
                    eprintln!("Failed to create output stream for {}: {}", output_path.display(), e);
                    S_OK // Return S_OK but with null stream (skip extraction)
                }
            }
        }
    }

    fn prepare_operation(&self, _ask_extract_mode: i32) -> HRESULT {
        S_OK
    }

    fn set_operation_result(&self, _result_e_operation_result: i32) -> HRESULT {
        // Cleanup current stream
        if let Ok(mut stream_opt) = self.current_stream.lock() {
            if let Some(stream_ptr) = stream_opt.take() {
                // SAFETY: This stream was created with Box::leak and should be freed
                unsafe {
                    let _ = Box::from_raw(stream_ptr as *mut FileStreamWrite);
                }
            }
        }
        *self.current_path.lock().unwrap() = None;
        S_OK
    }

    // ICompressProgressInfo methods
    fn set_ratio_info(&self, _in_size: *const u64, _out_size: *const u64) -> HRESULT {
        S_OK
    }

    // ICryptoGetTextPassword methods
    fn crypto_get_text_password(&self, _password: *mut *mut u16) -> HRESULT {
        if let Some(ref pwd) = self.password {
            let pwd_utf16: Vec<u16> = pwd.encode_utf16().collect();
            let bstr = alloc_bstr(&pwd_utf16);
            if !bstr.is_null() {
                unsafe { *_password = bstr; }
                return S_OK;
            }
        }
        E_NOINTERFACE
    }
}

// ============================================================================
// Static VTable Generation
// ============================================================================

ExtractCallbackVTable_static!(static EXTRACT_CALLBACK_VT for VTableExtractCallback);

