//! Common callback implementations for 7-Zip operations

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
use std::ptr;

/// Open callback for 7-Zip archive opening
/// Uses a single vtable containing all interface methods
#[repr(C)]
pub struct OpenCallback {
    base_vtable: *const c_void,  // Points to a unified vtable
    archive_path: PathBuf,
    ref_count: UnsafeCell<u32>,
}

// Unified vtable structure containing all interface methods
struct OpenCallbackVTable {
    // IUnknown
    query_interface: unsafe extern "system" fn(*mut IUnknown, *const crate::ffi::GUID, *mut *mut c_void) -> HRESULT,
    add_ref: unsafe extern "system" fn(*mut IUnknown) -> u32,
    release: unsafe extern "system" fn(*mut IUnknown) -> u32,
    // IArchiveOpenCallback
    set_completed: unsafe extern "system" fn(*mut IArchiveOpenCallback, *const u64, *const u64) -> HRESULT,
    set_total: unsafe extern "system" fn(*mut IArchiveOpenCallback, *const u64, *const u64) -> HRESULT,
    // IArchiveOpenVolumeCallback
    get_property: unsafe extern "system" fn(*mut IArchiveOpenVolumeCallback, u32, *mut PROPVARIANT) -> HRESULT,
    get_stream: unsafe extern "system" fn(*mut IArchiveOpenVolumeCallback, *const u16, *mut *mut IInStream) -> HRESULT,
    // IArchiveOpenSetSubArchiveName
    set_sub_archive_name: unsafe extern "system" fn(*mut IArchiveOpenSetSubArchiveName, *const u16) -> HRESULT,
    // ICryptoGetTextPassword
    get_text_password: unsafe extern "system" fn(*mut ICryptoGetTextPassword, *mut *mut u16) -> HRESULT,
}

static mut OPEN_CALLBACK_VTABLE: Option<OpenCallbackVTable> = None;

fn init_vtable() -> *const OpenCallbackVTable {
    unsafe {
        if OPEN_CALLBACK_VTABLE.is_none() {
            OPEN_CALLBACK_VTABLE = Some(OpenCallbackVTable {
                query_interface: OpenCallback::query_interface,
                add_ref: OpenCallback::add_ref,
                release: OpenCallback::release,
                set_completed: OpenCallback::set_completed,
                set_total: OpenCallback::set_total,
                get_property: OpenCallback::get_property,
                get_stream: OpenCallback::get_stream,
                set_sub_archive_name: OpenCallback::set_sub_archive_name,
                get_text_password: OpenCallback::get_text_password,
            });
        }
        OPEN_CALLBACK_VTABLE.as_ref().unwrap() as *const _
    }
}

impl OpenCallback {
    /// Create a new OpenCallback for the given archive path
    pub fn new(archive_path: &Path) -> Self {
        OpenCallback {
            base_vtable: init_vtable() as *const c_void,
            archive_path: archive_path.to_path_buf(),
            ref_count: UnsafeCell::new(1),
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
        if count > 0 {
            *ref_count.get() = count - 1;
            count - 1
        } else {
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
            let utf32: Vec<u32> = file_name.chars().map(|c| c as u32).collect();

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

    unsafe extern "system" fn get_stream(
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
        _this: *mut ICryptoGetTextPassword,
        _password: *mut *mut u16,
    ) -> HRESULT {
        -2147467262 // E_NOINTERFACE - no password support
    }
}
