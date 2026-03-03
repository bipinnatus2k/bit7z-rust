//! Common callback implementations for 7-Zip operations
//!
//! This module provides both manual and vtable-crate-based implementations:
//! - `OpenCallback` - Manual vtable implementation (legacy)
//! - `VTableOpenCallback` - VTable crate implementation (recommended, in vtable_callback.rs)
//!
//! OpenCallback implements multiple interfaces:
//! - IArchiveOpenCallback
//! - IArchiveOpenVolumeCallback
//! - IArchiveOpenSetSubArchiveName
//! - ICryptoGetTextPassword

use crate::ffi::{
    IArchiveOpenCallback, IArchiveOpenVolumeCallback, IArchiveOpenSetSubArchiveName,
    ICryptoGetTextPassword, IInStream, IUnknown,
    PROPVARIANT, HRESULT,
    IID_IUnknown, IID_IArchiveOpenCallback, IID_IArchiveOpenVolumeCallback,
    IID_IArchiveOpenSetSubArchiveName, IID_ICryptoGetTextPassword,
};
use crate::ffi::variant::alloc_bstr_from_utf32;
use std::cell::UnsafeCell;
use std::ffi::c_void;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::ptr;
use std::sync::Arc;
use std::sync::Mutex;

// Re-export VTableOpenCallback from vtable_callback module
pub use crate::vtable_callback::VTableOpenCallback;

/// Total callback type - called with total size
pub type TotalCallback = Arc<Mutex<dyn FnMut(u64) + Send + Sync>>;

/// Progress callback type - called with processed size, returns true to continue
pub type ProgressCallback = Arc<Mutex<dyn FnMut(u64) -> bool + Send + Sync>>;

/// Ratio callback type - called with input and output sizes
pub type RatioCallback = Arc<Mutex<dyn FnMut(u64, u64) + Send + Sync>>;

/// File callback type - called with file path
pub type FileCallback = Arc<Mutex<dyn FnMut(String) + Send + Sync>>;

/// Password callback type - returns password string
pub type PasswordCallback = Arc<Mutex<dyn FnMut() -> String + Send + Sync>>;

/// Open callback for 7-Zip archive opening (Manual vtable implementation)
/// 
/// **Note**: This is the legacy manual implementation. 
/// For new code, consider using `VTableOpenCallback` from the `vtable_callback` module.
///
/// Memory layout: vtable must be first to match C++ COM object layout
#[repr(C)]
pub struct OpenCallback {
    vtable: Pin<Box<OpenCallbackVTable>>,
    archive_path: PathBuf,
    ref_count: UnsafeCell<u32>,
    password_callback: Option<PasswordCallback>,
}

/// Unified vtable structure containing all interface methods
/// This matches bit7z's multi-interface inheritance pattern
#[repr(C)]
struct OpenCallbackVTable {
    // IArchiveOpenCallback vtable (includes IUnknown base)
    open_callback_vtable: crate::ffi::IArchiveOpenCallbackVTable,
    // IArchiveOpenVolumeCallback vtable
    volume_callback_vtable: crate::ffi::IArchiveOpenVolumeCallbackVTable,
    // IArchiveOpenSetSubArchiveName vtable
    set_name_vtable: crate::ffi::IArchiveOpenSetSubArchiveNameVTable,
    // ICryptoGetTextPassword vtable
    crypto_vtable: crate::ffi::ICryptoGetTextPasswordVTable,
}

impl OpenCallback {
    /// Create a new OpenCallback for the given archive path
    pub fn new(archive_path: &Path) -> Self {
        Self::with_password_callback(archive_path, None)
    }

    /// Create a new OpenCallback with password callback
    pub fn with_password_callback(archive_path: &Path, password_callback: Option<PasswordCallback>) -> Self {
        let vtable = Box::pin(OpenCallbackVTable {
            open_callback_vtable: crate::ffi::IArchiveOpenCallbackVTable {
                base: crate::ffi::IUnknownVTable {
                    query_interface: Self::query_interface,
                    add_ref: Self::add_ref,
                    release: Self::release,
                },
                set_completed: Self::set_completed,
                set_total: Self::set_total,
            },
            volume_callback_vtable: crate::ffi::IArchiveOpenVolumeCallbackVTable {
                base: crate::ffi::IUnknownVTable {
                    query_interface: Self::query_interface,
                    add_ref: Self::add_ref,
                    release: Self::release,
                },
                get_property: Self::get_property,
                get_stream: Self::get_stream_volume,
            },
            set_name_vtable: crate::ffi::IArchiveOpenSetSubArchiveNameVTable {
                base: crate::ffi::IUnknownVTable {
                    query_interface: Self::query_interface,
                    add_ref: Self::add_ref,
                    release: Self::release,
                },
                set_sub_archive_name: Self::set_sub_archive_name,
            },
            crypto_vtable: crate::ffi::ICryptoGetTextPasswordVTable {
                base: crate::ffi::IUnknownVTable {
                    query_interface: Self::query_interface,
                    add_ref: Self::add_ref,
                    release: Self::release,
                },
                crypto_get_text_password: Self::get_text_password,
            },
        });

        OpenCallback {
            vtable,
            archive_path: archive_path.to_path_buf(),
            ref_count: UnsafeCell::new(1),
            password_callback,
        }
    }

    /// Get as IArchiveOpenCallback pointer
    pub fn as_i_archive_open_callback(&self) -> *mut IArchiveOpenCallback {
        self as *const OpenCallback as *mut OpenCallback as *mut IArchiveOpenCallback
    }

    unsafe extern "system" fn query_interface(
        this: *mut IUnknown,
        iid: *const crate::ffi::GUID,
        out: *mut *mut c_void,
    ) -> HRESULT {
        if out.is_null() || iid.is_null() {
            return -2147467261; // E_POINTER
        }

        let iid_iunknown = IID_IUnknown;
        let iid_open_callback = IID_IArchiveOpenCallback;
        let iid_volume_callback = IID_IArchiveOpenVolumeCallback;
        let iid_set_name = IID_IArchiveOpenSetSubArchiveName;
        let iid_crypto = IID_ICryptoGetTextPassword;

        // For all supported interfaces, return the same pointer
        if *iid == iid_iunknown ||
           *iid == iid_open_callback ||
           *iid == iid_volume_callback ||
           *iid == iid_set_name ||
           *iid == iid_crypto {
            *out = this as *mut c_void;
            OpenCallback::add_ref(this);
            return 0; // S_OK
        }

        *out = ptr::null_mut();
        -2147467262 // E_NOINTERFACE
    }

    unsafe extern "system" fn add_ref(this: *mut IUnknown) -> u32 {
        let callback = this as *mut OpenCallback;
        let ref_count = &(*callback).ref_count;
        let count = *ref_count.get();
        *ref_count.get() = count + 1;
        count + 1
    }

    unsafe extern "system" fn release(this: *mut IUnknown) -> u32 {
        let callback = this as *mut OpenCallback;
        let ref_count = &(*callback).ref_count;
        let count = *ref_count.get();
        if count > 1 {
            *ref_count.get() = count - 1;
            count - 1
        } else {
            // Reference count reached 0, free the object
            *ref_count.get() = 0;
            // Drop the callback - this will also drop the Pin<Box<>> vtable
            let _ = Box::from_raw(callback);
            0
        }
    }

    unsafe extern "system" fn set_total(
        _this: *mut IArchiveOpenCallback,
        _files: *const u64,
        _bytes: *const u64,
    ) -> HRESULT {
        0 // S_OK
    }

    unsafe extern "system" fn set_completed(
        _this: *mut IArchiveOpenCallback,
        _files: *const u64,
        _bytes: *const u64,
    ) -> HRESULT {
        0 // S_OK
    }

    unsafe extern "system" fn get_property(
        this: *mut IArchiveOpenVolumeCallback,
        prop_id: u32,
        value: *mut PROPVARIANT,
    ) -> HRESULT {
        if value.is_null() {
            return -2147467261; // E_POINTER
        }

        // kpidName = 0
        if prop_id == 0 {
            let callback = this as *const OpenCallback;
            let path = &(*callback).archive_path;

            // Use only the filename (like bit7z does)
            let file_name = path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");

            // On Linux, 7-Zip uses UTF-32 (wchar_t is 4 bytes)
            // Convert to UTF-32
            let _utf32: Vec<u32> = file_name.chars().map(|c| c as u32).collect();

            // Allocate BSTR (UTF-16 version for cross-platform compatibility)
            let bstr = alloc_bstr_from_utf32(&file_name);
            if bstr.is_null() {
                return -2147467259; // E_FAIL
            }

            // Set PROPVARIANT
            (*value).vt = 8; // VT_BSTR
            (*value).wReserved1 = 0;
            (*value).wReserved2 = 0;
            (*value).wReserved3 = 0;
            // Write BSTR pointer to data array (first 8 bytes on 64-bit)
            // Use write_unaligned to avoid alignment issues
            let data_ptr = (*value).data.as_mut_ptr() as *mut *mut u16;
            std::ptr::write_unaligned(data_ptr, bstr as *mut u16);

            return 0; // S_OK
        }

        0 // S_OK - return empty property for other props
    }

    unsafe extern "system" fn get_stream_volume(
        _this: *mut IArchiveOpenVolumeCallback,
        _name: *const u16,
        _in_stream: *mut *mut IInStream,
    ) -> HRESULT {
        // Return S_FALSE to indicate no multi-volume archive support
        1 // S_FALSE
    }

    unsafe extern "system" fn set_sub_archive_name(
        _this: *mut IArchiveOpenSetSubArchiveName,
        _name: *const u16,
    ) -> HRESULT {
        0 // S_OK
    }

    unsafe extern "system" fn get_text_password(
        this: *mut ICryptoGetTextPassword,
        password: *mut *mut u16,
    ) -> HRESULT {
        if password.is_null() {
            return -2147467261; // E_POINTER
        }

        let callback = this as *const OpenCallback;
        
        // Check if we have a password callback
        if let Some(ref pwd_callback) = (*callback).password_callback {
            // Call the callback to get password
            if let Ok(mut cb) = pwd_callback.lock() {
                let pwd_string = cb();
                
                // Convert password to UTF-16 BSTR
                // alloc_bstr expects &[u16], so we need to convert the string
                let pwd_utf16: Vec<u16> = pwd_string.encode_utf16().collect();
                let bstr = crate::ffi::variant::alloc_bstr(&pwd_utf16);
                
                if !bstr.is_null() {
                    *password = bstr;
                    return 0; // S_OK
                }
            }
        }

        -2147467262 // E_NOINTERFACE - no password support
    }
}
