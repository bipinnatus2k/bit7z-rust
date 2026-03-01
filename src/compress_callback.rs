//! Compression callback implementations for 7-Zip archive creation
//!
//! This module provides UpdateCallback for handling compression operations.

use crate::ffi::{
    IArchiveUpdateCallback, IArchiveUpdateCallbackVTable,
    ISequentialInStream, IUnknown, IUnknownVTable,
    ICryptoGetTextPassword, ICryptoGetTextPasswordVTable,
    ICryptoGetTextPassword2, ICryptoGetTextPassword2VTable,
    IProgress, IProgressVTable,
    PROPVARIANT, PROPID, HRESULT, ULONG,
    IID_IUnknown, IID_IArchiveUpdateCallback, IID_ICryptoGetTextPassword,
    IID_ICryptoGetTextPassword2,
};
use crate::ffi::variant::alloc_bstr_utf32;
use crate::error::Result;
use crate::stream::FileStream;
use std::cell::UnsafeCell;
use std::ffi::c_void;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::ptr;

/// Input item representing a file to be compressed
#[derive(Clone)]
pub struct InputItem {
    pub path: PathBuf,
    pub name_in_archive: Option<String>,
}

impl InputItem {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        InputItem {
            path: path.as_ref().to_path_buf(),
            name_in_archive: None,
        }
    }

    pub fn with_name<P: AsRef<Path>>(path: P, name: String) -> Self {
        InputItem {
            path: path.as_ref().to_path_buf(),
            name_in_archive: Some(name),
        }
    }
}

/// Callback for archive update operations
/// Uses a single vtable containing all interface methods
#[repr(C)]
pub struct UpdateCallback {
    base_vtable: *const c_void,
    input_items: Vec<InputItem>,
    ref_count: UnsafeCell<u32>,
    password: Option<String>,
}

// Unified vtable structure containing all interface methods
struct UpdateCallbackVTable {
    // IUnknown
    query_interface: unsafe extern "system" fn(*mut IUnknown, *const crate::ffi::GUID, *mut *mut c_void) -> HRESULT,
    add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG,
    release: unsafe extern "system" fn(*mut IUnknown) -> ULONG,
    // IProgress (base of IArchiveUpdateCallback)
    set_completed: unsafe extern "system" fn(*mut IProgress, *const u64) -> HRESULT,
    set_total: unsafe extern "system" fn(*mut IProgress, u64) -> HRESULT,
    // IArchiveUpdateCallback
    get_update_item_info: unsafe extern "system" fn(
        *mut IArchiveUpdateCallback,
        u32,
        *mut i32,
        *mut i32,
        *mut u32,
    ) -> HRESULT,
    get_property: unsafe extern "system" fn(*mut IArchiveUpdateCallback, u32, PROPID, *mut PROPVARIANT) -> HRESULT,
    get_stream: unsafe extern "system" fn(*mut IArchiveUpdateCallback, u32, *mut *mut ISequentialInStream) -> HRESULT,
    set_operation_result: unsafe extern "system" fn(*mut IArchiveUpdateCallback, i32) -> HRESULT,
    // ICryptoGetTextPassword
    get_text_password: unsafe extern "system" fn(*mut ICryptoGetTextPassword, *mut *mut u16) -> HRESULT,
    // ICryptoGetTextPassword2
    get_text_password2: unsafe extern "system" fn(*mut ICryptoGetTextPassword2, *mut i32, *mut *mut u16) -> HRESULT,
}

static mut UPDATE_CALLBACK_VTABLE: Option<UpdateCallbackVTable> = None;

fn init_vtable() -> *const UpdateCallbackVTable {
    unsafe {
        if UPDATE_CALLBACK_VTABLE.is_none() {
            UPDATE_CALLBACK_VTABLE = Some(UpdateCallbackVTable {
                query_interface: UpdateCallback::query_interface,
                add_ref: UpdateCallback::add_ref,
                release: UpdateCallback::release,
                set_completed: UpdateCallback::set_completed,
                set_total: UpdateCallback::set_total,
                get_update_item_info: UpdateCallback::get_update_item_info,
                get_property: UpdateCallback::get_property,
                get_stream: UpdateCallback::get_stream,
                set_operation_result: UpdateCallback::set_operation_result,
                get_text_password: UpdateCallback::get_text_password,
                get_text_password2: UpdateCallback::get_text_password2,
            });
        }
        UPDATE_CALLBACK_VTABLE.as_ref().unwrap() as *const _
    }
}

impl UpdateCallback {
    /// Create a new UpdateCallback with input items
    pub fn new(input_items: Vec<InputItem>, password: Option<String>) -> Self {
        UpdateCallback {
            base_vtable: init_vtable() as *const c_void,
            input_items,
            ref_count: UnsafeCell::new(1),
            password,
        }
    }

    /// Get as IArchiveUpdateCallback pointer
    pub fn as_i_archive_update_callback(&self) -> *mut IArchiveUpdateCallback {
        self as *const UpdateCallback as *mut UpdateCallback as *mut IArchiveUpdateCallback
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
        let iid_update_callback = IID_IArchiveUpdateCallback;
        let iid_crypto = IID_ICryptoGetTextPassword;
        let iid_crypto2 = IID_ICryptoGetTextPassword2;
        let iid_progress = crate::ffi::IID_IProgress;

        if *iid == iid_iunknown {
            *out = this as *mut c_void;
            UpdateCallback::add_ref(this);
            return 0; // S_OK
        }

        if *iid == iid_progress {
            *out = this as *mut c_void;
            UpdateCallback::add_ref(this);
            return 0; // S_OK
        }

        if *iid == iid_update_callback {
            *out = this as *mut c_void;
            UpdateCallback::add_ref(this);
            return 0; // S_OK
        }

        if *iid == iid_crypto || *iid == iid_crypto2 {
            *out = this as *mut c_void;
            UpdateCallback::add_ref(this);
            return 0; // S_OK
        }

        *out = ptr::null_mut();
        -2147467262 // E_NOINTERFACE
    }

    unsafe extern "system" fn add_ref(this: *mut IUnknown) -> ULONG {
        let callback = this as *mut UpdateCallback;
        let ref_count = &(*callback).ref_count;
        let count = *ref_count.get();
        *ref_count.get() = count + 1;
        count + 1
    }

    unsafe extern "system" fn release(this: *mut IUnknown) -> ULONG {
        let callback = this as *mut UpdateCallback;
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
        _this: *mut IProgress,
        _size: u64,
    ) -> HRESULT {
        0 // S_OK
    }

    unsafe extern "system" fn set_completed(
        _this: *mut IProgress,
        _complete_value: *const u64,
    ) -> HRESULT {
        0 // S_OK
    }

    unsafe extern "system" fn get_update_item_info(
        _this: *mut IArchiveUpdateCallback,
        index: u32,
        new_data: *mut i32,
        new_properties: *mut i32,
        index_in_archive: *mut u32,
    ) -> HRESULT {
        // All items are new (not in existing archive)
        if !new_data.is_null() {
            *new_data = 1; // true - this is new data
        }
        if !new_properties.is_null() {
            *new_properties = 1; // true - properties are available
        }
        if !index_in_archive.is_null() {
            *index_in_archive = 0xFFFFFFFF; // -1 = not in archive (new item)
        }
        0 // S_OK
    }

    unsafe extern "system" fn get_property(
        this: *mut IArchiveUpdateCallback,
        index: u32,
        prop_id: PROPID,
        value: *mut PROPVARIANT,
    ) -> HRESULT {
        if value.is_null() {
            return -2147467261; // E_POINTER
        }

        let callback = this as *const UpdateCallback;
        let items = &(*callback).input_items;

        if index as usize >= items.len() {
            return -2147467259; // E_FAIL
        }

        let item = &items[index as usize];

        // Initialize PROPVARIANT as empty
        (*value).vt = 0; // VT_EMPTY
        (*value).wReserved1 = 0;
        (*value).wReserved2 = 0;
        (*value).wReserved3 = 0;
        (*value).data = [0; 16];

        match prop_id {
            PROPID::IsAnti => {
                // Not an anti-item
                (*value).vt = 11; // VT_BOOL
                (*value).data[0] = 0; // false
            }
            PROPID::Path => {
                // Set the path in the archive
                let path_str = if let Some(ref name) = item.name_in_archive {
                    name.clone()
                } else {
                    item.path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string()
                };

                // Convert to UTF-32 for Linux 7-Zip
                let utf32: Vec<u32> = path_str.chars().map(|c| c as u32).collect();
                let bstr = alloc_bstr_utf32(&utf32);

                if bstr.is_null() {
                    return -2147467259; // E_FAIL
                }

                (*value).vt = 8; // VT_BSTR
                let data_ptr = (*value).data.as_mut_ptr() as *mut *mut u16;
                *data_ptr = bstr as *mut u16;
            }
            PROPID::IsDir => {
                // Check if it's a directory
                let is_dir = item.path.is_dir();
                (*value).vt = 11; // VT_BOOL
                // VT_BOOL uses i16: TRUE = -1 (0xFFFF), FALSE = 0
                let bool_val: i16 = if is_dir { -1 } else { 0 };
                (*value).data[0] = (bool_val & 0xFF) as u8;
                (*value).data[1] = ((bool_val >> 8) & 0xFF) as u8;
            }
            PROPID::Size => {
                // Get file size (0 for directories)
                if !item.path.is_dir() {
                    if let Ok(metadata) = std::fs::metadata(&item.path) {
                        let size = metadata.len();
                        (*value).vt = 21; // VT_UI8
                        let data_ptr = (*value).data.as_mut_ptr() as *mut u64;
                        *data_ptr = size;
                    }
                }
            }
            PROPID::Attrib => {
                // File attributes (0 for now, could be extended)
                (*value).vt = 19; // VT_UI4
                (*value).data[0] = 0;
                (*value).data[1] = 0;
                (*value).data[2] = 0;
                (*value).data[3] = 0;
            }
            _ => {
                // Return empty for other properties
            }
        }

        0 // S_OK
    }

    unsafe extern "system" fn get_stream(
        this: *mut IArchiveUpdateCallback,
        index: u32,
        in_stream: *mut *mut ISequentialInStream,
    ) -> HRESULT {
        if in_stream.is_null() {
            return -2147467261; // E_POINTER
        }

        let callback = this as *const UpdateCallback;
        let items = &(*callback).input_items;

        if index as usize >= items.len() {
            *in_stream = ptr::null_mut();
            return -2147467259; // E_FAIL
        }

        let item = &items[index as usize];

        // Directories don't need a stream
        if item.path.is_dir() {
            *in_stream = ptr::null_mut();
            return 0; // S_OK
        }

        // Create a file stream for the input file
        match FileStream::new(&item.path) {
            Ok(file_stream) => {
                // Pin the stream to prevent moving
                let pinned_stream = Box::new(file_stream);
                let stream_ptr = Box::into_raw(pinned_stream);
                // Cast to ISequentialInStream (base interface of IInStream)
                *in_stream = (*stream_ptr).as_i_in_stream() as *mut ISequentialInStream;
                // Note: The stream will be released by 7-Zip via Release()
                0 // S_OK
            }
            Err(_) => {
                *in_stream = ptr::null_mut();
                -2147467259 // E_FAIL
            }
        }
    }

    unsafe extern "system" fn set_operation_result(
        _this: *mut IArchiveUpdateCallback,
        _result: i32,
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

        let callback = this as *const UpdateCallback;
        if let Some(ref pwd) = (*callback).password {
            // Convert password to UTF-32
            let utf32: Vec<u32> = pwd.chars().map(|c| c as u32).collect();
            let bstr = alloc_bstr_utf32(&utf32);

            if bstr.is_null() {
                return -2147467259; // E_FAIL
            }

            *password = bstr as *mut u16;
            0 // S_OK
        } else {
            // No password defined
            -2147467262 // E_NOINTERFACE
        }
    }

    unsafe extern "system" fn get_text_password2(
        this: *mut ICryptoGetTextPassword2,
        password_is_defined: *mut i32,
        password: *mut *mut u16,
    ) -> HRESULT {
        if password_is_defined.is_null() || password.is_null() {
            return -2147467261; // E_POINTER
        }

        let callback = this as *const UpdateCallback;
        if let Some(ref pwd) = (*callback).password {
            *password_is_defined = 1; // true - password is defined

            // Convert password to UTF-32
            let utf32: Vec<u32> = pwd.chars().map(|c| c as u32).collect();
            let bstr = alloc_bstr_utf32(&utf32);

            if bstr.is_null() {
                return -2147467259; // E_FAIL
            }

            *password = bstr as *mut u16;
            0 // S_OK
        } else {
            // No password defined
            *password_is_defined = 0; // false
            *password = ptr::null_mut();
            0 // S_OK
        }
    }
}
