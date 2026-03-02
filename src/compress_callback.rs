//! Compression callback implementations for 7-Zip archive creation
//!
//! This module provides UpdateCallback for handling compression operations.
//! 
//! ## Interface Layout
//! 
//! To match C++ multiple inheritance vtable layout, we use separate wrapper
//! structs for each interface. Each wrapper has its own vtable pointer at offset 0.
//!
//! UpdateCallback (main object)
//! ├── IUnknown vtable (indices 0-2)
//! ├── IProgress vtable (indices 0-2) <- shares IUnknown methods
//! ├── IArchiveUpdateCallback vtable (indices 0-6) <- inherits IProgress
//! └── ICryptoGetTextPassword vtable (indices 0-2)
//! └── ICryptoGetTextPassword2 vtable (indices 0-2)

use crate::ffi::{
    IArchiveUpdateCallback, IArchiveUpdateCallbackVTable,
    ISequentialInStream, ISequentialOutStream, IUnknown, IUnknownVTable,
    ICryptoGetTextPassword, ICryptoGetTextPasswordVTable,
    ICryptoGetTextPassword2, ICryptoGetTextPassword2VTable,
    IProgress, IProgressVTable,
    ICompressProgressInfo, ICompressProgressInfoVTable,
    IArchiveUpdateCallback2, IArchiveUpdateCallback2VTable,
    PROPVARIANT, PROPID, HRESULT, ULONG,
    IID_IUnknown, IID_IArchiveUpdateCallback, IID_ICryptoGetTextPassword,
    IID_ICryptoGetTextPassword2, IID_IProgress,
    IID_IArchiveUpdateCallback2, IID_ICompressProgressInfo,
};
use crate::ffi::variant::alloc_bstr_from_utf32;
use crate::error::Result;
use crate::stream::FileStream;
use std::cell::UnsafeCell;
use std::ffi::c_void;
use std::path::{Path, PathBuf};
use std::ptr;
use std::sync::Arc;
use std::sync::Mutex;

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

/// Input item type for update operations
#[derive(Clone)]
pub enum InputItemType {
    /// New file from filesystem
    NewFile(PathBuf),
    /// Keep existing item from archive (by index)
    KeepExisting(u32),
    /// Update existing item with new file
    UpdateExisting(u32, PathBuf),
}

/// Extended input item for update operations
#[derive(Clone)]
pub struct ExtendedInputItem {
    pub item_type: InputItemType,
    pub name_in_archive: Option<String>,
}

impl ExtendedInputItem {
    pub fn new_file<P: AsRef<Path>>(path: P) -> Self {
        ExtendedInputItem {
            item_type: InputItemType::NewFile(path.as_ref().to_path_buf()),
            name_in_archive: None,
        }
    }

    pub fn keep_existing(index: u32) -> Self {
        ExtendedInputItem {
            item_type: InputItemType::KeepExisting(index),
            name_in_archive: None,
        }
    }

    pub fn update_existing<P: AsRef<Path>>(index: u32, path: P) -> Self {
        ExtendedInputItem {
            item_type: InputItemType::UpdateExisting(index, path.as_ref().to_path_buf()),
            name_in_archive: None,
        }
    }
}

/// Progress callback type - receives (completed, total), returns true to continue
pub type ProgressCallback = Arc<Mutex<dyn Fn(u64, u64) -> bool + Send + Sync>>;

/// Ratio callback type - receives (in_size, out_size)
pub type RatioCallback = Arc<Mutex<dyn Fn(u64, u64) + Send + Sync>>;

/// File callback type - receives file path
pub type FileCallback = Arc<Mutex<dyn Fn(&str) + Send + Sync>>;

/// Password callback type - returns password
pub type PasswordCallback = Arc<Mutex<dyn Fn() -> String + Send + Sync>>;

/// Total callback type - receives total size
pub type TotalCallbackType = Arc<Mutex<dyn Fn(u64) + Send + Sync>>;

/// Main callback object containing all data
/// This is the "master" object that owns all the data
#[repr(C)]
pub struct UpdateCallback {
    // IUnknown vtable - must be first for IUnknown casts
    unknown_vtable: *const IUnknownVTable,
    // IProgress vtable - for IProgress casts
    progress_vtable: *const IProgressVTable,
    // IArchiveUpdateCallback vtable - for IArchiveUpdateCallback casts
    update_callback_vtable: *const IArchiveUpdateCallbackVTable,
    // IArchiveUpdateCallback2 vtable
    update_callback2_vtable: *const IArchiveUpdateCallback2VTable,
    // ICompressProgressInfo vtable
    compress_progress_vtable: *const ICompressProgressInfoVTable,
    // ICryptoGetTextPassword vtable
    crypto_password_vtable: *const ICryptoGetTextPasswordVTable,
    // ICryptoGetTextPassword2 vtable
    crypto_password2_vtable: *const ICryptoGetTextPassword2VTable,
    // Data fields
    input_items: Vec<InputItem>,
    ref_count: UnsafeCell<u32>,
    password: Option<String>,
    // Callbacks
    total_callback: Option<TotalCallbackType>,
    progress_callback: Option<ProgressCallback>,
    ratio_callback: Option<RatioCallback>,
    file_callback: Option<FileCallback>,
    password_callback: Option<PasswordCallback>,
}

// Static vtables - initialized once
static mut UNKNOWN_VTABLE: Option<IUnknownVTable> = None;
static mut PROGRESS_VTABLE: Option<IProgressVTable> = None;
static mut UPDATE_CALLBACK_VTABLE: Option<IArchiveUpdateCallbackVTable> = None;
static mut UPDATE_CALLBACK2_VTABLE: Option<IArchiveUpdateCallback2VTable> = None;
static mut COMPRESS_PROGRESS_VTABLE: Option<ICompressProgressInfoVTable> = None;
static mut CRYPTO_PASSWORD_VTABLE: Option<ICryptoGetTextPasswordVTable> = None;
static mut CRYPTO_PASSWORD2_VTABLE: Option<ICryptoGetTextPassword2VTable> = None;

/// Initialize all vtables
fn init_vtables() {
    unsafe {
        if UNKNOWN_VTABLE.is_none() {
            UNKNOWN_VTABLE = Some(IUnknownVTable {
                query_interface: UpdateCallback::unknown_query_interface,
                add_ref: UpdateCallback::unknown_add_ref,
                release: UpdateCallback::unknown_release,
            });
            
            PROGRESS_VTABLE = Some(IProgressVTable {
                base: IUnknownVTable {
                    query_interface: std::mem::transmute::<
                        unsafe extern "system" fn(*mut IProgress, *const crate::ffi::GUID, *mut *mut c_void) -> HRESULT,
                        unsafe extern "system" fn(*mut IUnknown, *const crate::ffi::GUID, *mut *mut c_void) -> HRESULT,
                    >(UpdateCallback::progress_query_interface),
                    add_ref: std::mem::transmute::<
                        unsafe extern "system" fn(*mut IProgress) -> ULONG,
                        unsafe extern "system" fn(*mut IUnknown) -> ULONG,
                    >(UpdateCallback::progress_add_ref),
                    release: std::mem::transmute::<
                        unsafe extern "system" fn(*mut IProgress) -> ULONG,
                        unsafe extern "system" fn(*mut IUnknown) -> ULONG,
                    >(UpdateCallback::progress_release),
                },
                set_completed: UpdateCallback::set_completed,
                set_total: UpdateCallback::set_total,
            });
            
            UPDATE_CALLBACK_VTABLE = Some(IArchiveUpdateCallbackVTable {
                base: IProgressVTable {
                    base: IUnknownVTable {
                        query_interface: std::mem::transmute::<
                            unsafe extern "system" fn(*mut IArchiveUpdateCallback, *const crate::ffi::GUID, *mut *mut c_void) -> HRESULT,
                            unsafe extern "system" fn(*mut IUnknown, *const crate::ffi::GUID, *mut *mut c_void) -> HRESULT,
                        >(UpdateCallback::update_callback_query_interface),
                        add_ref: std::mem::transmute::<
                            unsafe extern "system" fn(*mut IArchiveUpdateCallback) -> ULONG,
                            unsafe extern "system" fn(*mut IUnknown) -> ULONG,
                        >(UpdateCallback::update_callback_add_ref),
                        release: std::mem::transmute::<
                            unsafe extern "system" fn(*mut IArchiveUpdateCallback) -> ULONG,
                            unsafe extern "system" fn(*mut IUnknown) -> ULONG,
                        >(UpdateCallback::update_callback_release),
                    },
                    set_completed: std::mem::transmute::<
                        unsafe extern "system" fn(*mut IArchiveUpdateCallback, *const u64) -> HRESULT,
                        unsafe extern "system" fn(*mut IProgress, *const u64) -> HRESULT,
                    >(UpdateCallback::set_completed_impl),
                    set_total: std::mem::transmute::<
                        unsafe extern "system" fn(*mut IArchiveUpdateCallback, u64) -> HRESULT,
                        unsafe extern "system" fn(*mut IProgress, u64) -> HRESULT,
                    >(UpdateCallback::set_total_impl),
                },
                get_update_item_info: UpdateCallback::get_update_item_info,
                get_property: UpdateCallback::get_property,
                get_stream: UpdateCallback::get_stream,
                set_operation_result: UpdateCallback::set_operation_result,
            });
            
            UPDATE_CALLBACK2_VTABLE = Some(IArchiveUpdateCallback2VTable {
                base: IArchiveUpdateCallbackVTable {
                    base: IProgressVTable {
                        base: IUnknownVTable {
                            query_interface: std::mem::transmute::<
                                unsafe extern "system" fn(*mut IArchiveUpdateCallback2, *const crate::ffi::GUID, *mut *mut c_void) -> HRESULT,
                                unsafe extern "system" fn(*mut IUnknown, *const crate::ffi::GUID, *mut *mut c_void) -> HRESULT,
                            >(UpdateCallback::update_callback2_query_interface),
                            add_ref: std::mem::transmute::<
                                unsafe extern "system" fn(*mut IArchiveUpdateCallback2) -> ULONG,
                                unsafe extern "system" fn(*mut IUnknown) -> ULONG,
                            >(UpdateCallback::update_callback2_add_ref),
                            release: std::mem::transmute::<
                                unsafe extern "system" fn(*mut IArchiveUpdateCallback2) -> ULONG,
                                unsafe extern "system" fn(*mut IUnknown) -> ULONG,
                            >(UpdateCallback::update_callback2_release),
                        },
                        set_completed: std::mem::transmute::<
                            unsafe extern "system" fn(*mut IArchiveUpdateCallback2, *const u64) -> HRESULT,
                            unsafe extern "system" fn(*mut IProgress, *const u64) -> HRESULT,
                        >(UpdateCallback::set_completed_impl2),
                        set_total: std::mem::transmute::<
                            unsafe extern "system" fn(*mut IArchiveUpdateCallback2, u64) -> HRESULT,
                            unsafe extern "system" fn(*mut IProgress, u64) -> HRESULT,
                        >(UpdateCallback::set_total_impl2),
                    },
                    get_update_item_info: UpdateCallback::get_update_item_info,
                    get_property: UpdateCallback::get_property,
                    get_stream: UpdateCallback::get_stream,
                    set_operation_result: UpdateCallback::set_operation_result,
                },
                get_volume_size: UpdateCallback::get_volume_size,
                get_volume_stream: UpdateCallback::get_volume_stream,
            });
            
            COMPRESS_PROGRESS_VTABLE = Some(ICompressProgressInfoVTable {
                base: IUnknownVTable {
                    query_interface: std::mem::transmute::<
                        unsafe extern "system" fn(*mut ICompressProgressInfo, *const crate::ffi::GUID, *mut *mut c_void) -> HRESULT,
                        unsafe extern "system" fn(*mut IUnknown, *const crate::ffi::GUID, *mut *mut c_void) -> HRESULT,
                    >(UpdateCallback::compress_progress_query_interface),
                    add_ref: std::mem::transmute::<
                        unsafe extern "system" fn(*mut ICompressProgressInfo) -> ULONG,
                        unsafe extern "system" fn(*mut IUnknown) -> ULONG,
                    >(UpdateCallback::compress_progress_add_ref),
                    release: std::mem::transmute::<
                        unsafe extern "system" fn(*mut ICompressProgressInfo) -> ULONG,
                        unsafe extern "system" fn(*mut IUnknown) -> ULONG,
                    >(UpdateCallback::compress_progress_release),
                },
                set_ratio_info: UpdateCallback::set_ratio_info,
            });
            
            CRYPTO_PASSWORD_VTABLE = Some(ICryptoGetTextPasswordVTable {
                base: IUnknownVTable {
                    query_interface: std::mem::transmute::<
                        unsafe extern "system" fn(*mut ICryptoGetTextPassword, *const crate::ffi::GUID, *mut *mut c_void) -> HRESULT,
                        unsafe extern "system" fn(*mut IUnknown, *const crate::ffi::GUID, *mut *mut c_void) -> HRESULT,
                    >(UpdateCallback::crypto_password_query_interface),
                    add_ref: std::mem::transmute::<
                        unsafe extern "system" fn(*mut ICryptoGetTextPassword) -> ULONG,
                        unsafe extern "system" fn(*mut IUnknown) -> ULONG,
                    >(UpdateCallback::crypto_password_add_ref),
                    release: std::mem::transmute::<
                        unsafe extern "system" fn(*mut ICryptoGetTextPassword) -> ULONG,
                        unsafe extern "system" fn(*mut IUnknown) -> ULONG,
                    >(UpdateCallback::crypto_password_release),
                },
                crypto_get_text_password: UpdateCallback::get_text_password,
            });
            
            CRYPTO_PASSWORD2_VTABLE = Some(ICryptoGetTextPassword2VTable {
                base: IUnknownVTable {
                    query_interface: std::mem::transmute::<
                        unsafe extern "system" fn(*mut ICryptoGetTextPassword2, *const crate::ffi::GUID, *mut *mut c_void) -> HRESULT,
                        unsafe extern "system" fn(*mut IUnknown, *const crate::ffi::GUID, *mut *mut c_void) -> HRESULT,
                    >(UpdateCallback::crypto_password2_query_interface),
                    add_ref: std::mem::transmute::<
                        unsafe extern "system" fn(*mut ICryptoGetTextPassword2) -> ULONG,
                        unsafe extern "system" fn(*mut IUnknown) -> ULONG,
                    >(UpdateCallback::crypto_password2_add_ref),
                    release: std::mem::transmute::<
                        unsafe extern "system" fn(*mut ICryptoGetTextPassword2) -> ULONG,
                        unsafe extern "system" fn(*mut IUnknown) -> ULONG,
                    >(UpdateCallback::crypto_password2_release),
                },
                crypto_get_text_password2: UpdateCallback::get_text_password2,
            });
        }
    }
}

impl UpdateCallback {
    /// Create a new UpdateCallback with input items
    pub fn new(input_items: Vec<InputItem>, password: Option<String>) -> Self {
        init_vtables();

        unsafe {
            UpdateCallback {
                unknown_vtable: UNKNOWN_VTABLE.as_ref().unwrap() as *const _,
                progress_vtable: PROGRESS_VTABLE.as_ref().unwrap() as *const _,
                update_callback_vtable: UPDATE_CALLBACK_VTABLE.as_ref().unwrap() as *const _,
                update_callback2_vtable: UPDATE_CALLBACK2_VTABLE.as_ref().unwrap() as *const _,
                compress_progress_vtable: COMPRESS_PROGRESS_VTABLE.as_ref().unwrap() as *const _,
                crypto_password_vtable: CRYPTO_PASSWORD_VTABLE.as_ref().unwrap() as *const _,
                crypto_password2_vtable: CRYPTO_PASSWORD2_VTABLE.as_ref().unwrap() as *const _,
                input_items,
                ref_count: UnsafeCell::new(1),
                password,
                total_callback: None,
                progress_callback: None,
                ratio_callback: None,
                file_callback: None,
                password_callback: None,
            }
        }
    }

    /// Create a new UpdateCallback with callbacks
    pub fn with_callbacks(
        input_items: Vec<InputItem>,
        password: Option<String>,
        total_callback: Option<TotalCallbackType>,
        progress_callback: Option<ProgressCallback>,
        ratio_callback: Option<RatioCallback>,
        file_callback: Option<FileCallback>,
        password_callback: Option<PasswordCallback>,
    ) -> Self {
        init_vtables();

        UpdateCallback {
            unknown_vtable: unsafe { UNKNOWN_VTABLE.as_ref().unwrap() as *const _ },
            progress_vtable: unsafe { PROGRESS_VTABLE.as_ref().unwrap() as *const _ },
            update_callback_vtable: unsafe { UPDATE_CALLBACK_VTABLE.as_ref().unwrap() as *const _ },
            update_callback2_vtable: unsafe { UPDATE_CALLBACK2_VTABLE.as_ref().unwrap() as *const _ },
            compress_progress_vtable: unsafe { COMPRESS_PROGRESS_VTABLE.as_ref().unwrap() as *const _ },
            crypto_password_vtable: unsafe { CRYPTO_PASSWORD_VTABLE.as_ref().unwrap() as *const _ },
            crypto_password2_vtable: unsafe { CRYPTO_PASSWORD2_VTABLE.as_ref().unwrap() as *const _ },
            input_items,
            ref_count: UnsafeCell::new(1),
            password,
            total_callback,
            progress_callback,
            ratio_callback,
            file_callback,
            password_callback,
        }
    }

    /// Get as IUnknown pointer
    pub fn as_i_unknown(&self) -> *mut IUnknown {
        self as *const UpdateCallback as *mut UpdateCallback as *mut IUnknown
    }

    /// Get as IArchiveUpdateCallback pointer
    pub fn as_i_archive_update_callback(&self) -> *mut IArchiveUpdateCallback {
        self as *const UpdateCallback as *mut UpdateCallback as *mut IArchiveUpdateCallback
    }

    // Helper to get self from IUnknown pointer
    unsafe fn from_unknown(this: *mut IUnknown) -> *mut UpdateCallback {
        // For IUnknown, the vtable is at offset 0, same as UpdateCallback
        this as *mut UpdateCallback
    }

    // Helper to get self from IProgress pointer
    unsafe fn from_progress(this: *mut IProgress) -> *mut UpdateCallback {
        // IProgress vtable is at offset 1 (after unknown_vtable)
        // We need to calculate the offset
        let progress_vtable_offset = std::mem::size_of::<*const IUnknownVTable>();
        let callback_ptr = (this as *const u8).sub(progress_vtable_offset);
        callback_ptr as *mut UpdateCallback
    }

    // Helper to get self from IArchiveUpdateCallback pointer
    unsafe fn from_update_callback(this: *mut IArchiveUpdateCallback) -> *mut UpdateCallback {
        // IArchiveUpdateCallback vtable is at offset 2
        let offset = std::mem::size_of::<*const IUnknownVTable>()
                   + std::mem::size_of::<*const IProgressVTable>();
        let callback_ptr = (this as *const u8).sub(offset);
        callback_ptr as *mut UpdateCallback
    }

    // Helper to get self from IArchiveUpdateCallback2 pointer
    unsafe fn from_update_callback2(this: *mut IArchiveUpdateCallback2) -> *mut UpdateCallback {
        let offset = std::mem::size_of::<*const IUnknownVTable>()
                   + std::mem::size_of::<*const IProgressVTable>()
                   + std::mem::size_of::<*const IArchiveUpdateCallbackVTable>();
        let callback_ptr = (this as *const u8).sub(offset);
        callback_ptr as *mut UpdateCallback
    }

    // Helper to get self from ICompressProgressInfo pointer
    unsafe fn from_compress_progress(this: *mut ICompressProgressInfo) -> *mut UpdateCallback {
        let offset = std::mem::size_of::<*const IUnknownVTable>()
                   + std::mem::size_of::<*const IProgressVTable>()
                   + std::mem::size_of::<*const IArchiveUpdateCallbackVTable>()
                   + std::mem::size_of::<*const IArchiveUpdateCallback2VTable>();
        let callback_ptr = (this as *const u8).sub(offset);
        callback_ptr as *mut UpdateCallback
    }

    // Helper to get self from ICryptoGetTextPassword pointer
    unsafe fn from_crypto_password(this: *mut ICryptoGetTextPassword) -> *mut UpdateCallback {
        let offset = std::mem::size_of::<*const IUnknownVTable>()
                   + std::mem::size_of::<*const IProgressVTable>()
                   + std::mem::size_of::<*const IArchiveUpdateCallbackVTable>()
                   + std::mem::size_of::<*const IArchiveUpdateCallback2VTable>()
                   + std::mem::size_of::<*const ICompressProgressInfoVTable>();
        let callback_ptr = (this as *const u8).sub(offset);
        callback_ptr as *mut UpdateCallback
    }

    // Helper to get self from ICryptoGetTextPassword2 pointer
    unsafe fn from_crypto_password2(this: *mut ICryptoGetTextPassword2) -> *mut UpdateCallback {
        let offset = std::mem::size_of::<*const IUnknownVTable>()
                   + std::mem::size_of::<*const IProgressVTable>()
                   + std::mem::size_of::<*const IArchiveUpdateCallbackVTable>()
                   + std::mem::size_of::<*const IArchiveUpdateCallback2VTable>()
                   + std::mem::size_of::<*const ICompressProgressInfoVTable>()
                   + std::mem::size_of::<*const ICryptoGetTextPasswordVTable>();
        let callback_ptr = (this as *const u8).sub(offset);
        callback_ptr as *mut UpdateCallback
    }

    // Finalize callback (close any open streams)
    unsafe fn finalize(&mut self) -> HRESULT {
        // In our implementation, streams are closed automatically when dropped
        // This matches bit7z's behavior of closing streams between items
        0 // S_OK
    }

    // ========== IUnknown implementation ==========
    
    unsafe extern "system" fn unknown_query_interface(
        this: *mut IUnknown,
        iid: *const crate::ffi::GUID,
        out: *mut *mut c_void,
    ) -> HRESULT {
        if out.is_null() || iid.is_null() {
            return -2147467261; // E_POINTER
        }

        let callback = Self::from_unknown(this);
        *out = ptr::null_mut();

        let iid_iunknown = IID_IUnknown;
        let iid_progress = IID_IProgress;
        let iid_update_callback = IID_IArchiveUpdateCallback;
        let iid_update_callback2 = IID_IArchiveUpdateCallback2;
        let iid_compress_progress = IID_ICompressProgressInfo;
        let iid_crypto = IID_ICryptoGetTextPassword;
        let iid_crypto2 = IID_ICryptoGetTextPassword2;

        if *iid == iid_iunknown {
            *out = callback as *mut c_void;
        } else if *iid == iid_progress {
            let progress_ptr = (callback as *mut u8).add(std::mem::size_of::<*const IUnknownVTable>()) as *mut IProgress;
            *out = progress_ptr as *mut c_void;
        } else if *iid == iid_update_callback {
            let update_ptr = (callback as *mut u8)
                .add(std::mem::size_of::<*const IUnknownVTable>() 
                   + std::mem::size_of::<*const IProgressVTable>()) as *mut IArchiveUpdateCallback;
            *out = update_ptr as *mut c_void;
        } else if *iid == iid_update_callback2 {
            let update2_ptr = (callback as *mut u8)
                .add(std::mem::size_of::<*const IUnknownVTable>() 
                   + std::mem::size_of::<*const IProgressVTable>()
                   + std::mem::size_of::<*const IArchiveUpdateCallbackVTable>()) as *mut IArchiveUpdateCallback2;
            *out = update2_ptr as *mut c_void;
        } else if *iid == iid_compress_progress {
            let progress_ptr = (callback as *mut u8)
                .add(std::mem::size_of::<*const IUnknownVTable>() 
                   + std::mem::size_of::<*const IProgressVTable>()
                   + std::mem::size_of::<*const IArchiveUpdateCallbackVTable>()
                   + std::mem::size_of::<*const IArchiveUpdateCallback2VTable>()) as *mut ICompressProgressInfo;
            *out = progress_ptr as *mut c_void;
        } else if *iid == iid_crypto {
            let crypto_ptr = (callback as *mut u8)
                .add(std::mem::size_of::<*const IUnknownVTable>() 
                   + std::mem::size_of::<*const IProgressVTable>()
                   + std::mem::size_of::<*const IArchiveUpdateCallbackVTable>()
                   + std::mem::size_of::<*const IArchiveUpdateCallback2VTable>()
                   + std::mem::size_of::<*const ICompressProgressInfoVTable>()) as *mut ICryptoGetTextPassword;
            *out = crypto_ptr as *mut c_void;
        } else if *iid == iid_crypto2 {
            let crypto2_ptr = (callback as *mut u8)
                .add(std::mem::size_of::<*const IUnknownVTable>() 
                   + std::mem::size_of::<*const IProgressVTable>()
                   + std::mem::size_of::<*const IArchiveUpdateCallbackVTable>()
                   + std::mem::size_of::<*const IArchiveUpdateCallback2VTable>()
                   + std::mem::size_of::<*const ICompressProgressInfoVTable>()
                   + std::mem::size_of::<*const ICryptoGetTextPasswordVTable>()) as *mut ICryptoGetTextPassword2;
            *out = crypto2_ptr as *mut c_void;
        } else {
            return -2147467262; // E_NOINTERFACE
        }

        Self::unknown_add_ref(this);
        0 // S_OK
    }

    unsafe extern "system" fn unknown_add_ref(this: *mut IUnknown) -> ULONG {
        let callback = Self::from_unknown(this);
        let ref_count = &(*callback).ref_count;
        let count = *ref_count.get();
        *ref_count.get() = count + 1;
        count + 1
    }

    unsafe extern "system" fn unknown_release(this: *mut IUnknown) -> ULONG {
        let callback = Self::from_unknown(this);
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

    // ========== IProgress implementation ==========
    
    unsafe extern "system" fn progress_query_interface(
        this: *mut IProgress,
        iid: *const crate::ffi::GUID,
        out: *mut *mut c_void,
    ) -> HRESULT {
        // Delegate to IUnknown's QueryInterface
        let callback = Self::from_progress(this);
        Self::unknown_query_interface(Self::as_i_unknown(&*callback), iid, out)
    }

    unsafe extern "system" fn progress_add_ref(this: *mut IProgress) -> ULONG {
        let callback = Self::from_progress(this);
        Self::unknown_add_ref(Self::as_i_unknown(&*callback))
    }

    unsafe extern "system" fn progress_release(this: *mut IProgress) -> ULONG {
        let callback = Self::from_progress(this);
        Self::unknown_release(Self::as_i_unknown(&*callback))
    }

    unsafe extern "system" fn set_total(
        this: *mut IProgress,
        size: u64,
    ) -> HRESULT {
        let callback = Self::from_progress(this);

        // Call total callback if registered
        if let Some(ref total_cb) = (*callback).total_callback {
            if let Ok(cb) = total_cb.lock() {
                cb(size);
            }
        }

        // Call progress callback if registered
        if let Some(ref progress_cb) = (*callback).progress_callback {
            if let Ok(cb) = progress_cb.lock() {
                cb(0, size);
            }
        }

        0 // S_OK
    }

    unsafe extern "system" fn set_total_impl(
        this: *mut IArchiveUpdateCallback,
        size: u64,
    ) -> HRESULT {
        let callback = Self::from_update_callback(this);

        // Call total callback if registered
        if let Some(ref total_cb) = (*callback).total_callback {
            if let Ok(cb) = total_cb.lock() {
                cb(size);
            }
        }

        0 // S_OK
    }

    unsafe extern "system" fn set_total_impl2(
        this: *mut IArchiveUpdateCallback2,
        size: u64,
    ) -> HRESULT {
        let callback = Self::from_update_callback2(this);

        // Call total callback if registered
        if let Some(ref total_cb) = (*callback).total_callback {
            if let Ok(cb) = total_cb.lock() {
                cb(size);
            }
        }

        0 // S_OK
    }

    unsafe extern "system" fn set_completed(
        this: *mut IProgress,
        complete_value: *const u64,
    ) -> HRESULT {
        let callback = Self::from_progress(this);

        // Call progress callback if registered
        if let Some(ref progress_cb) = (*callback).progress_callback {
            let completed = if !complete_value.is_null() { *complete_value } else { 0 };
            if let Ok(cb) = progress_cb.lock() {
                cb(completed, 0);
            }
        }

        0 // S_OK
    }

    unsafe extern "system" fn set_completed_impl(
        this: *mut IArchiveUpdateCallback,
        complete_value: *const u64,
    ) -> HRESULT {
        let callback = Self::from_update_callback(this);

        // Call progress callback if registered
        if let Some(ref progress_cb) = (*callback).progress_callback {
            let completed = if !complete_value.is_null() { *complete_value } else { 0 };
            if let Ok(cb) = progress_cb.lock() {
                cb(completed, 0);
            }
        }

        0 // S_OK
    }

    unsafe extern "system" fn set_completed_impl2(
        this: *mut IArchiveUpdateCallback2,
        complete_value: *const u64,
    ) -> HRESULT {
        let callback = Self::from_update_callback2(this);

        // Call progress callback if registered
        if let Some(ref progress_cb) = (*callback).progress_callback {
            let completed = if !complete_value.is_null() { *complete_value } else { 0 };
            if let Ok(cb) = progress_cb.lock() {
                cb(completed, 0);
            }
        }

        0 // S_OK
    }

    // ========== IArchiveUpdateCallback implementation ==========
    
    unsafe extern "system" fn update_callback_query_interface(
        this: *mut IArchiveUpdateCallback,
        iid: *const crate::ffi::GUID,
        out: *mut *mut c_void,
    ) -> HRESULT {
        let callback = Self::from_update_callback(this);
        Self::unknown_query_interface(Self::as_i_unknown(&*callback), iid, out)
    }

    unsafe extern "system" fn update_callback_add_ref(this: *mut IArchiveUpdateCallback) -> ULONG {
        let callback = Self::from_update_callback(this);
        Self::unknown_add_ref(Self::as_i_unknown(&*callback))
    }

    unsafe extern "system" fn update_callback_release(this: *mut IArchiveUpdateCallback) -> ULONG {
        let callback = Self::from_update_callback(this);
        Self::unknown_release(Self::as_i_unknown(&*callback))
    }

    unsafe extern "system" fn get_update_item_info(
        this: *mut IArchiveUpdateCallback,
        index: u32,
        new_data: *mut i32,
        new_properties: *mut i32,
        index_in_archive: *mut u32,
    ) -> HRESULT {
        eprintln!("[Callback] GetUpdateItemInfo: index={}", index);

        let callback = Self::from_update_callback(this);
        let items = &(*callback).input_items;

        if index as usize >= items.len() {
            eprintln!("[Callback] GetUpdateItemInfo: index out of bounds");
            return -2147467259; // E_FAIL
        }

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
        eprintln!("[Callback] GetUpdateItemInfo: returning newData=1, newProperties=1, indexInArchive=-1");
        0 // S_OK
    }

    unsafe extern "system" fn get_property(
        this: *mut IArchiveUpdateCallback,
        index: u32,
        prop_id: PROPID,
        value: *mut PROPVARIANT,
    ) -> HRESULT {
        eprintln!("[Callback] GetProperty: index={}, prop_id={:?}", index, prop_id);
        
        if value.is_null() {
            return -2147467261; // E_POINTER
        }

        let callback = Self::from_update_callback(this);
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
                (*value).vt = 11; // VT_BOOL
                (*value).data[0] = 0; // false
            }
            PROPID::Path => {
                let path_str = if let Some(ref name) = item.name_in_archive {
                    name.clone()
                } else {
                    item.path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string()
                };

                let bstr = alloc_bstr_from_utf32(&path_str);
                if bstr.is_null() {
                    return -2147467259; // E_FAIL
                }

                (*value).vt = 8; // VT_BSTR
                // Use write_unaligned to avoid alignment issues
                let data_ptr = (*value).data.as_mut_ptr() as *mut *mut u16;
                std::ptr::write_unaligned(data_ptr, bstr);
            }
            PROPID::IsDir => {
                let is_dir = item.path.is_dir();
                (*value).vt = 11; // VT_BOOL
                let bool_val: i16 = if is_dir { -1 } else { 0 };
                (*value).data[0] = (bool_val & 0xFF) as u8;
                (*value).data[1] = ((bool_val >> 8) & 0xFF) as u8;
            }
            PROPID::Size => {
                if !item.path.is_dir() {
                    if let Ok(metadata) = std::fs::metadata(&item.path) {
                        let size = metadata.len();
                        (*value).vt = 21; // VT_UI8
                        // Use write_unaligned to avoid alignment issues
                        let data_ptr = (*value).data.as_mut_ptr() as *mut u64;
                        std::ptr::write_unaligned(data_ptr, size);
                    }
                }
            }
            PROPID::Attrib => {
                (*value).vt = 19; // VT_UI4
                (*value).data[0] = 0;
                (*value).data[1] = 0;
                (*value).data[2] = 0;
                (*value).data[3] = 0;
            }
            _ => {}
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

        let callback = Self::from_update_callback(this);

        // Call finalize to close any previous stream (matching bit7z behavior)
        // Note: finalize is a no-op in our implementation since streams auto-close on drop

        let items = &(*callback).input_items;

        if index as usize >= items.len() {
            *in_stream = ptr::null_mut();
            return -2147467259; // E_FAIL
        }

        let item = &items[index as usize];

        // Call file callback if registered (matching bit7z behavior)
        if let Some(ref file_cb) = (*callback).file_callback {
            let path_str = item.path.to_string_lossy();
            if let Ok(cb) = file_cb.lock() {
                cb(&path_str);
            }
        }

        // Directories don't need a stream
        if item.path.is_dir() {
            *in_stream = ptr::null_mut();
            return 0; // S_OK
        }

        // Create a file stream for the input file
        match FileStream::new(&item.path) {
            Ok(file_stream) => {
                let pinned_stream = Box::new(file_stream);
                let stream_ptr = Box::into_raw(pinned_stream);
                *in_stream = (*stream_ptr).as_i_in_stream() as *mut ISequentialInStream;
                0 // S_OK
            }
            Err(e) => {
                eprintln!("[Callback] GetStream: failed to create stream: {}", e);
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

    // ========== ICryptoGetTextPassword implementation ==========
    
    unsafe extern "system" fn crypto_password_query_interface(
        this: *mut ICryptoGetTextPassword,
        iid: *const crate::ffi::GUID,
        out: *mut *mut c_void,
    ) -> HRESULT {
        let callback = Self::from_crypto_password(this);
        Self::unknown_query_interface(Self::as_i_unknown(&*callback), iid, out)
    }

    unsafe extern "system" fn crypto_password_add_ref(this: *mut ICryptoGetTextPassword) -> ULONG {
        let callback = Self::from_crypto_password(this);
        Self::unknown_add_ref(Self::as_i_unknown(&*callback))
    }

    unsafe extern "system" fn crypto_password_release(this: *mut ICryptoGetTextPassword) -> ULONG {
        let callback = Self::from_crypto_password(this);
        Self::unknown_release(Self::as_i_unknown(&*callback))
    }

    unsafe extern "system" fn get_text_password(
        this: *mut ICryptoGetTextPassword,
        password: *mut *mut u16,
    ) -> HRESULT {
        if password.is_null() {
            return -2147467261; // E_POINTER
        }

        let callback = Self::from_crypto_password(this);
        if let Some(ref pwd) = (*callback).password {
            let bstr = alloc_bstr_from_utf32(pwd);
            if bstr.is_null() {
                return -2147467259; // E_FAIL
            }
            *password = bstr;
            0 // S_OK
        } else {
            -2147467262 // E_NOINTERFACE
        }
    }

    // ========== ICryptoGetTextPassword2 implementation ==========
    
    unsafe extern "system" fn crypto_password2_query_interface(
        this: *mut ICryptoGetTextPassword2,
        iid: *const crate::ffi::GUID,
        out: *mut *mut c_void,
    ) -> HRESULT {
        let callback = Self::from_crypto_password2(this);
        Self::unknown_query_interface(Self::as_i_unknown(&*callback), iid, out)
    }

    unsafe extern "system" fn crypto_password2_add_ref(this: *mut ICryptoGetTextPassword2) -> ULONG {
        let callback = Self::from_crypto_password2(this);
        Self::unknown_add_ref(Self::as_i_unknown(&*callback))
    }

    unsafe extern "system" fn crypto_password2_release(this: *mut ICryptoGetTextPassword2) -> ULONG {
        let callback = Self::from_crypto_password2(this);
        Self::unknown_release(Self::as_i_unknown(&*callback))
    }

    unsafe extern "system" fn get_text_password2(
        this: *mut ICryptoGetTextPassword2,
        password_is_defined: *mut i32,
        password: *mut *mut u16,
    ) -> HRESULT {
        if password_is_defined.is_null() || password.is_null() {
            return -2147467261; // E_POINTER
        }

        let callback = Self::from_crypto_password2(this);

        // Try to get password from callback first
        if let Some(ref password_cb) = (*callback).password_callback {
            if let Ok(cb) = password_cb.lock() {
                let pwd = cb();
                *password_is_defined = 1;
                let bstr = alloc_bstr_from_utf32(&pwd);
                if bstr.is_null() {
                    return -2147467259; // E_FAIL
                }
                *password = bstr;
                return 0; // S_OK
            }
        }
        
        // Fall back to stored password
        if let Some(ref pwd) = (*callback).password {
            *password_is_defined = 1; // true - password is defined

            let bstr = alloc_bstr_from_utf32(pwd);
            if bstr.is_null() {
                return -2147467259; // E_FAIL
            }

            *password = bstr;
            0 // S_OK
        } else {
            *password_is_defined = 0; // false
            *password = ptr::null_mut();
            0 // S_OK
        }
    }

    // ========== IArchiveUpdateCallback2 implementation ==========

    unsafe extern "system" fn update_callback2_query_interface(
        this: *mut IArchiveUpdateCallback2,
        iid: *const crate::ffi::GUID,
        out: *mut *mut c_void,
    ) -> HRESULT {
        let callback = Self::from_update_callback2(this);
        Self::unknown_query_interface(Self::as_i_unknown(&*callback), iid, out)
    }

    unsafe extern "system" fn update_callback2_add_ref(this: *mut IArchiveUpdateCallback2) -> ULONG {
        let callback = Self::from_update_callback2(this);
        Self::unknown_add_ref(Self::as_i_unknown(&*callback))
    }

    unsafe extern "system" fn update_callback2_release(this: *mut IArchiveUpdateCallback2) -> ULONG {
        let callback = Self::from_update_callback2(this);
        Self::unknown_release(Self::as_i_unknown(&*callback))
    }

    unsafe extern "system" fn get_volume_size(
        _this: *mut IArchiveUpdateCallback2,
        _index: u32,
        _size: *mut u64,
    ) -> HRESULT {
        // Not supported for single-volume archives
        -2147467262 // E_NOINTERFACE
    }

    unsafe extern "system" fn get_volume_stream(
        _this: *mut IArchiveUpdateCallback2,
        _index: u32,
        _volume_stream: *mut *mut ISequentialOutStream,
    ) -> HRESULT {
        // Not supported for single-volume archives
        -2147467262 // E_NOINTERFACE
    }

    // ========== ICompressProgressInfo implementation ==========

    unsafe extern "system" fn compress_progress_query_interface(
        this: *mut ICompressProgressInfo,
        iid: *const crate::ffi::GUID,
        out: *mut *mut c_void,
    ) -> HRESULT {
        let callback = Self::from_compress_progress(this);
        Self::unknown_query_interface(Self::as_i_unknown(&*callback), iid, out)
    }

    unsafe extern "system" fn compress_progress_add_ref(this: *mut ICompressProgressInfo) -> ULONG {
        let callback = Self::from_compress_progress(this);
        Self::unknown_add_ref(Self::as_i_unknown(&*callback))
    }

    unsafe extern "system" fn compress_progress_release(this: *mut ICompressProgressInfo) -> ULONG {
        let callback = Self::from_compress_progress(this);
        Self::unknown_release(Self::as_i_unknown(&*callback))
    }

    unsafe extern "system" fn set_ratio_info(
        this: *mut ICompressProgressInfo,
        in_size: *const u64,
        out_size: *const u64,
    ) -> HRESULT {
        let callback = Self::from_compress_progress(this);

        // Call ratio callback if registered
        if let Some(ref ratio_cb) = (*callback).ratio_callback {
            let in_val = if !in_size.is_null() { *in_size } else { 0 };
            let out_val = if !out_size.is_null() { *out_size } else { 0 };
            if let Ok(cb) = ratio_cb.lock() {
                cb(in_val, out_val);
            }
        }

        0 // S_OK
    }
}
