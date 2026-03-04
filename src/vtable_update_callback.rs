//! UpdateCallback implementation using vtable crate
//!
//! This module provides a vtable-crate-based implementation of
//! IArchiveUpdateCallback for 7-Zip compression operations.
//!
//! **Note**: This is a prototype implementation.
//! The legacy manual implementation in `compress_callback.rs` is still the default.

use vtable::*;
use crate::vtable_base::*;
use crate::ffi::{
    GUID, HRESULT, ISequentialInStream, PROPVARIANT, PROPID,
    IID_IUnknown, IID_IProgress, IID_IArchiveUpdateCallback,
    IID_IArchiveUpdateCallback2, IID_ICompressProgressInfo,
    IID_ICryptoGetTextPassword, IID_ICryptoGetTextPassword2,
};
use crate::stream::FileStream;
use crate::ffi::variant::alloc_bstr;
use std::path::{Path, PathBuf};
use std::ptr;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

// ============================================================================
// VTable Definition using #[vtable] macro
// ============================================================================

/// Unified UpdateCallback vtable containing all interface methods
#[vtable]
#[repr(C)]
struct UpdateCallbackVTable {
    // IUnknown methods
    query_interface: fn(VRef<UpdateCallbackVTable>, &GUID) -> *mut std::ffi::c_void,
    add_ref: fn(VRef<UpdateCallbackVTable>) -> u32,
    release: fn(VRefMut<UpdateCallbackVTable>) -> u32,

    // IProgress methods
    set_completed: fn(VRef<UpdateCallbackVTable>, *const u64) -> HRESULT,
    set_total: fn(VRef<UpdateCallbackVTable>, u64) -> HRESULT,

    // IArchiveUpdateCallback methods
    get_update_item_info: fn(VRef<UpdateCallbackVTable>, u32, *mut i32, *mut i32, *mut u32) -> HRESULT,
    get_property: fn(VRef<UpdateCallbackVTable>, u32, u32, *mut PROPVARIANT) -> HRESULT,
    get_stream: fn(VRef<UpdateCallbackVTable>, u32, *mut *mut ISequentialInStream) -> HRESULT,
    set_operation_result: fn(VRef<UpdateCallbackVTable>, i32) -> HRESULT,

    // IArchiveUpdateCallback2 methods (optional)
    // Note: These are commented out as they require additional handling
    // get_volume_size: fn(VRef<UpdateCallbackVTable>, u32, *mut u64) -> HRESULT,
    // get_volume_stream: fn(VRef<UpdateCallbackVTable>, u32, *mut *mut std::ffi::c_void) -> HRESULT,

    // ICompressProgressInfo methods
    set_ratio_info: fn(VRef<UpdateCallbackVTable>, *const u64, *const u64) -> HRESULT,

    // ICryptoGetTextPassword methods
    crypto_get_text_password: fn(VRef<UpdateCallbackVTable>, *mut *mut u16) -> HRESULT,

    // Drop destructor
    drop: fn(VRefMut<UpdateCallbackVTable>),
}

// ============================================================================
// UpdateCallback Implementation
// ============================================================================

/// Input item for compression
#[derive(Clone)]
pub struct UpdateItem {
    pub path: PathBuf,
    pub name_in_archive: Option<String>,
}

impl UpdateItem {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        UpdateItem {
            path: path.as_ref().to_path_buf(),
            name_in_archive: None,
        }
    }

    pub fn with_name<P: AsRef<Path>>(path: P, name: String) -> Self {
        UpdateItem {
            path: path.as_ref().to_path_buf(),
            name_in_archive: Some(name),
        }
    }
}

/// Callback types
pub type ProgressCallbackType = Arc<Mutex<dyn Fn(u64, u64) -> bool + Send + Sync>>;
pub type RatioCallbackType = Arc<Mutex<dyn Fn(u64, u64) + Send + Sync>>;
pub type FileCallbackType = Arc<Mutex<dyn Fn(&str) + Send + Sync>>;
pub type TotalCallbackType = Arc<Mutex<dyn Fn(u64) + Send + Sync>>;
pub type PasswordCallbackType = Arc<Mutex<dyn Fn() -> String + Send + Sync>>;

/// UpdateCallback using vtable crate
pub struct VTableUpdateCallback {
    input_items: Vec<UpdateItem>,
    password: Option<String>,
    ref_count: AtomicU32,
    // Optional callbacks
    progress_callback: Option<ProgressCallbackType>,
    ratio_callback: Option<RatioCallbackType>,
    file_callback: Option<FileCallbackType>,
    total_callback: Option<TotalCallbackType>,
    password_callback: Option<PasswordCallbackType>,
}

// SAFETY: UpdateCallback is only used in single-threaded 7-Zip callbacks
unsafe impl Send for VTableUpdateCallback {}
unsafe impl Sync for VTableUpdateCallback {}

impl VTableUpdateCallback {
    /// Create a new UpdateCallback with input items
    pub fn new(input_items: Vec<UpdateItem>) -> Self {
        VTableUpdateCallback {
            input_items,
            password: None,
            ref_count: AtomicU32::new(1),
            progress_callback: None,
            ratio_callback: None,
            file_callback: None,
            total_callback: None,
            password_callback: None,
        }
    }

    /// Set password for encrypted archives
    pub fn with_password(mut self, password: String) -> Self {
        self.password = Some(password);
        self
    }

    /// Set progress callback
    pub fn with_progress_callback(mut self, cb: ProgressCallbackType) -> Self {
        self.progress_callback = Some(cb);
        self
    }

    /// Set ratio callback
    pub fn with_ratio_callback(mut self, cb: RatioCallbackType) -> Self {
        self.ratio_callback = Some(cb);
        self
    }

    /// Set file callback
    pub fn with_file_callback(mut self, cb: FileCallbackType) -> Self {
        self.file_callback = Some(cb);
        self
    }

    /// Set total callback
    pub fn with_total_callback(mut self, cb: TotalCallbackType) -> Self {
        self.total_callback = Some(cb);
        self
    }

    /// Set password callback
    pub fn with_password_callback(mut self, cb: PasswordCallbackType) -> Self {
        self.password_callback = Some(cb);
        self
    }

    /// Get as IArchiveUpdateCallback pointer
    pub fn as_i_archive_update_callback(&self) -> *mut crate::ffi::IArchiveUpdateCallback {
        self as *const VTableUpdateCallback as *mut VTableUpdateCallback as *mut crate::ffi::IArchiveUpdateCallback
    }

    /// Get item info
    fn get_item_info(&self, index: u32) -> Option<&UpdateItem> {
        self.input_items.get(index as usize)
    }
}

// ============================================================================
// Trait Implementation
// ============================================================================

impl UpdateCallback for VTableUpdateCallback {
    // IUnknown methods
    fn query_interface(&self, iid: &GUID) -> *mut std::ffi::c_void {
        if *iid == IID_IUnknown
            || *iid == IID_IProgress
            || *iid == IID_IArchiveUpdateCallback
            || *iid == IID_ICompressProgressInfo
            || *iid == IID_ICryptoGetTextPassword
        {
            return self as *const _ as *mut _;
        }
        // Note: IArchiveUpdateCallback2 and ICryptoGetTextPassword2 not supported in this prototype
        ptr::null_mut()
    }

    fn add_ref(&self) -> u32 {
        self.ref_count.fetch_add(1, Ordering::SeqCst) + 1
    }

    fn release(&mut self) -> u32 {
        self.ref_count.fetch_sub(1, Ordering::SeqCst) - 1
    }

    // IProgress methods
    fn set_completed(&self, complete_value: *const u64) -> HRESULT {
        if !complete_value.is_null() {
            if let Some(ref cb) = self.progress_callback {
                if let Ok(guard) = cb.lock() {
                    let completed = unsafe { *complete_value };
                    guard(completed, self.input_items.len() as u64);
                }
            }
        }
        S_OK
    }

    fn set_total(&self, total: u64) -> HRESULT {
        if let Some(ref cb) = self.total_callback {
            if let Ok(guard) = cb.lock() {
                guard(total);
            }
        }
        S_OK
    }

    // IArchiveUpdateCallback methods
    fn get_update_item_info(
        &self,
        index: u32,
        new_data: *mut i32,
        new_properties: *mut i32,
        index_in_archive: *mut u32,
    ) -> HRESULT {
        // For new items (not updates)
        if !new_data.is_null() {
            unsafe { *new_data = 1; } // True - this is new data
        }
        if !new_properties.is_null() {
            unsafe { *new_properties = 1; } // True - has new properties
        }
        if !index_in_archive.is_null() {
            unsafe { *index_in_archive = u32::MAX; } // No index in archive (new item)
        }
        S_OK
    }

    fn get_property(&self, index: u32, prop_id: u32, value: *mut PROPVARIANT) -> HRESULT {
        if value.is_null() {
            return E_POINTER;
        }

        let item = match self.get_item_info(index) {
            Some(i) => i,
            None => return E_FAIL,
        };

        unsafe {
            // kpidPath
            if prop_id == crate::ffi::PROPID::Path as u32 {
                let path_str = item.path.to_string_lossy();
                let utf16: Vec<u16> = path_str.encode_utf16().collect();
                let bstr = alloc_bstr(&utf16);
                if bstr.is_null() {
                    return E_FAIL;
                }
                (*value).vt = 8; // VT_BSTR
                (*value).wReserved1 = 0;
                (*value).wReserved2 = 0;
                (*value).wReserved3 = 0;
                std::ptr::write_unaligned((*value).data.as_mut_ptr() as *mut *mut u16, bstr);
                return S_OK;
            }

            // kpidIsDir
            if prop_id == crate::ffi::PROPID::IsDir as u32 {
                let is_dir = item.path.is_dir();
                (*value).vt = 11; // VT_BOOL
                (*value).wReserved1 = 0;
                (*value).wReserved2 = 0;
                (*value).wReserved3 = 0;
                // BOOL in VARIANT is stored as i16: -1 (0xFFFF) for true, 0 for false
                let bool_val: i16 = if is_dir { -1 } else { 0 };
                std::ptr::write_unaligned((*value).data.as_mut_ptr() as *mut i16, bool_val);
                return S_OK;
            }

            // Default: return empty property
            (*value).vt = 0; // VT_EMPTY
        }
        S_OK
    }

    fn get_stream(&self, index: u32, in_stream: *mut *mut ISequentialInStream) -> HRESULT {
        if in_stream.is_null() {
            return E_POINTER;
        }

        let item = match self.get_item_info(index) {
            Some(i) => i,
            None => return E_FAIL,
        };

        // Call file callback if set
        if let Some(ref cb) = self.file_callback {
            if let Ok(guard) = cb.lock() {
                guard(&item.path.to_string_lossy());
            }
        }

        // Create file stream
        match FileStream::new(&item.path) {
            Ok(stream) => {
                let stream_box = Box::new(stream);
                let stream_ptr = Box::leak(stream_box);
                unsafe {
                    *in_stream = stream_ptr as *mut FileStream as *mut ISequentialInStream;
                }
                S_OK
            }
            Err(e) => {
                eprintln!("Failed to open file {:?}: {}", item.path, e);
                E_FAIL
            }
        }
    }

    fn set_operation_result(&self, _result_e_operation_result: i32) -> HRESULT {
        S_OK
    }

    // ICompressProgressInfo methods
    fn set_ratio_info(&self, in_size: *const u64, out_size: *const u64) -> HRESULT {
        if let Some(ref cb) = self.ratio_callback {
            if let Ok(guard) = cb.lock() {
                let in_val = if !in_size.is_null() { unsafe { *in_size } } else { 0 };
                let out_val = if !out_size.is_null() { unsafe { *out_size } } else { 0 };
                guard(in_val, out_val);
            }
        }
        S_OK
    }

    // ICryptoGetTextPassword methods
    fn crypto_get_text_password(&self, password: *mut *mut u16) -> HRESULT {
        if password.is_null() {
            return E_POINTER;
        }

        // Try password callback first
        if let Some(ref cb) = self.password_callback {
            if let Ok(guard) = cb.lock() {
                let pwd_string = guard();
                let pwd_utf16: Vec<u16> = pwd_string.encode_utf16().collect();
                let bstr = alloc_bstr(&pwd_utf16);
                if !bstr.is_null() {
                    unsafe { *password = bstr; }
                    return S_OK;
                }
            }
        }

        // Fall back to stored password
        if let Some(ref pwd) = self.password {
            let pwd_utf16: Vec<u16> = pwd.encode_utf16().collect();
            let bstr = alloc_bstr(&pwd_utf16);
            if !bstr.is_null() {
                unsafe { *password = bstr; }
                return S_OK;
            }
        }

        E_NOINTERFACE
    }
}

// ============================================================================
// Static VTable Generation
// ============================================================================

UpdateCallbackVTable_static!(static UPDATE_CALLBACK_VT for VTableUpdateCallback);

