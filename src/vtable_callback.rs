//! OpenCallback implementation using vtable crate
//!
//! This module demonstrates using the vtable crate to simplify
//! COM interface implementation for 7-Zip callbacks.
//!
//! **Note**: This is a prototype/experimental implementation.
//! Enable with `--features vtable_impl` to use.

#[cfg(feature = "vtable_impl")]
use vtable::*;

use crate::ffi::{
    GUID, HRESULT, PROPVARIANT, IInStream,
    IID_IUnknown, IID_IArchiveOpenCallback,
    IID_IArchiveOpenVolumeCallback, IID_ICryptoGetTextPassword,
};
use crate::ffi::variant::alloc_bstr_from_utf32;
use std::path::{Path, PathBuf};
use std::ptr;
use std::sync::atomic::{AtomicU32, Ordering};

// Re-export vtable crate generated types for public use
#[cfg(feature = "vtable_impl")]
pub use vtable::*;

#[cfg(feature = "vtable_impl")]
pub use self::OpenCallback_vtable_mod::OpenCallbackVTable;

// ============================================================================
// VTable Definitions using #[vtable] macro
// ============================================================================

/// IUnknown vtable - base COM interface
#[cfg(feature = "vtable_impl")]
#[vtable]
#[repr(C)]
struct IUnknownVTable {
    query_interface: fn(VRef<IUnknownVTable>, &GUID) -> *mut std::ffi::c_void,
    add_ref: fn(VRef<IUnknownVTable>) -> u32,
    release: fn(VRefMut<IUnknownVTable>) -> u32,
    drop: fn(VRefMut<IUnknownVTable>),
}

/// Unified OpenCallback vtable containing all interface methods
/// 
/// This combines:
/// - IUnknown (base)
/// - IArchiveOpenCallback
/// - IArchiveOpenVolumeCallback
/// - ICryptoGetTextPassword (returns E_NOINTERFACE)
#[cfg(feature = "vtable_impl")]
#[vtable]
#[repr(C)]
struct OpenCallbackVTable {
    // IUnknown methods
    query_interface: fn(VRef<OpenCallbackVTable>, &GUID) -> *mut std::ffi::c_void,
    add_ref: fn(VRef<OpenCallbackVTable>) -> u32,
    release: fn(VRefMut<OpenCallbackVTable>) -> u32,
    
    // IArchiveOpenCallback methods
    set_completed: fn(VRef<OpenCallbackVTable>, *const u64, *const u64) -> HRESULT,
    set_total: fn(VRef<OpenCallbackVTable>, *const u64, *const u64) -> HRESULT,
    
    // IArchiveOpenVolumeCallback methods
    get_property: fn(VRef<OpenCallbackVTable>, u32, *mut PROPVARIANT) -> HRESULT,
    get_stream: fn(VRef<OpenCallbackVTable>, *const u16, *mut *mut IInStream) -> HRESULT,
    
    // Drop destructor
    drop: fn(VRefMut<OpenCallbackVTable>),
}

// ============================================================================
// OpenCallback Implementation
// ============================================================================

/// OpenCallback using vtable crate for automatic vtable management
#[cfg(feature = "vtable_impl")]
pub struct VTableOpenCallback {
    archive_path: PathBuf,
    ref_count: AtomicU32,
}

#[cfg(feature = "vtable_impl")]
impl VTableOpenCallback {
    /// Create a new OpenCallback for the given archive path
    pub fn new(archive_path: &Path) -> Self {
        VTableOpenCallback {
            archive_path: archive_path.to_path_buf(),
            ref_count: AtomicU32::new(1),
        }
    }
    
    /// Get as IArchiveOpenCallback pointer for 7-Zip interop
    /// 
    /// This is the key integration point: converting vtable crate's VBox
    /// to a raw pointer that 7-Zip can use.
    pub fn as_i_archive_open_callback(&self) -> *mut crate::ffi::IArchiveOpenCallback {
        // Safety: The memory layout is compatible - vtable pointer at the beginning
        // However, this requires careful handling because we're crossing between
        // vtable crate's abstraction and raw FFI
        self as *const VTableOpenCallback as *mut VTableOpenCallback as *mut crate::ffi::IArchiveOpenCallback
    }
    
    /// Create a boxed version that can be used with 7-Zip
    /// 
    /// Returns a raw pointer that 7-Zip can use directly.
    /// The caller is responsible for managing the lifetime.
    pub fn into_raw(archive_path: &Path) -> *mut crate::ffi::IArchiveOpenCallback {
        let callback = VTableOpenCallback::new(archive_path);
        let boxed: VBox<OpenCallbackVTable> = VBox::new(callback);
        
        // Get the raw pointer from VBox
        // Note: This is a simplified approach - we're leaking the VBox
        // In production, you'd need to properly handle the lifecycle
        // through COM reference counting
        let leaked = Box::leak(Box::new(boxed));
        &mut **leaked as *mut _ as *mut crate::ffi::IArchiveOpenCallback
    }
}

// ============================================================================
// Trait Implementations
// ============================================================================

#[cfg(feature = "vtable_impl")]
impl OpenCallback for VTableOpenCallback {
    // IUnknown methods
    fn query_interface(&self, iid: &GUID) -> *mut std::ffi::c_void {
        // Support all required interfaces
        if *iid == IID_IUnknown 
            || *iid == IID_IArchiveOpenCallback 
            || *iid == IID_IArchiveOpenVolumeCallback
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
    
    // IArchiveOpenCallback methods
    fn set_completed(&self, _files: *const u64, _bytes: *const u64) -> HRESULT {
        0 // S_OK
    }
    
    fn set_total(&self, _files: *const u64, _bytes: *const u64) -> HRESULT {
        0 // S_OK
    }
    
    // IArchiveOpenVolumeCallback methods
    fn get_property(&self, prop_id: u32, value: *mut PROPVARIANT) -> HRESULT {
        if value.is_null() {
            return -2147467261; // E_POINTER
        }
        
        // kpidName = 0
        if prop_id == 0 {
            let file_name = self.archive_path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            
            // On Linux, 7-Zip uses UTF-32 (wchar_t is 4 bytes)
            // For compatibility, we use the existing alloc_bstr_from_utf32
            let bstr = alloc_bstr_from_utf32(&file_name);
            if bstr.is_null() {
                return -2147467259; // E_FAIL
            }
            
            unsafe {
                (*value).vt = 8; // VT_BSTR
                (*value).wReserved1 = 0;
                (*value).wReserved2 = 0;
                (*value).wReserved3 = 0;
                // Use write_unaligned to avoid alignment issues
                let data_ptr = (*value).data.as_mut_ptr() as *mut *mut u16;
                std::ptr::write_unaligned(data_ptr, bstr as *mut u16);
            }
            return 0; // S_OK
        }
        
        0 // S_OK - return empty property for other props
    }
    
    fn get_stream(&self, _name: *const u16, _in_stream: *mut *mut IInStream) -> HRESULT {
        // Return S_FALSE to indicate no multi-volume archive support
        1 // S_FALSE
    }
}

// ============================================================================
// Static VTable Generation
// ============================================================================

// The OpenCallbackVTable_static! macro generates a static OPEN_CALLBACK_VT
// We re-export it for public use
#[cfg(feature = "vtable_impl")]
OpenCallbackVTable_static!(static OPEN_CALLBACK_VT for VTableOpenCallback);

// Re-export the generated static vtable
// Note: We use a workaround since the macro-generated static is private
#[cfg(feature = "vtable_impl")]
pub fn get_open_callback_vt() -> &'static OpenCallbackVTable {
    &OPEN_CALLBACK_VT
}

// ============================================================================
// Alternative: Manual VTable Implementation (for comparison)
// ============================================================================

/// Alternative implementation showing the manual approach for comparison
/// This demonstrates what the vtable crate is saving us from writing
#[allow(dead_code)]
pub struct ManualOpenCallback {
    pub vtable: *const ManualOpenCallbackVTable,
    archive_path: PathBuf,
    ref_count: AtomicU32,
}

#[repr(C)]
pub struct ManualOpenCallbackVTable {
    query_interface: unsafe extern "system" fn(*mut ManualOpenCallback, *const GUID, *mut *mut std::ffi::c_void) -> HRESULT,
    add_ref: unsafe extern "system" fn(*mut ManualOpenCallback) -> u32,
    release: unsafe extern "system" fn(*mut ManualOpenCallback) -> u32,
    set_completed: unsafe extern "system" fn(*mut ManualOpenCallback, *const u64, *const u64) -> HRESULT,
    set_total: unsafe extern "system" fn(*mut ManualOpenCallback, *const u64, *const u64) -> HRESULT,
    get_property: unsafe extern "system" fn(*mut ManualOpenCallback, u32, *mut PROPVARIANT) -> HRESULT,
    get_stream: unsafe extern "system" fn(*mut ManualOpenCallback, *const u16, *mut *mut IInStream) -> HRESULT,
}

#[allow(dead_code)]
impl ManualOpenCallback {
    pub fn new(archive_path: &Path) -> Self {
        static VTABLE: ManualOpenCallbackVTable = ManualOpenCallbackVTable {
            query_interface: ManualOpenCallback::query_interface,
            add_ref: ManualOpenCallback::add_ref,
            release: ManualOpenCallback::release,
            set_completed: ManualOpenCallback::set_completed,
            set_total: ManualOpenCallback::set_total,
            get_property: ManualOpenCallback::get_property,
            get_stream: ManualOpenCallback::get_stream,
        };
        
        ManualOpenCallback {
            vtable: &VTABLE,
            archive_path: archive_path.to_path_buf(),
            ref_count: AtomicU32::new(1),
        }
    }
    
    unsafe extern "system" fn query_interface(
        this: *mut ManualOpenCallback,
        iid: *const GUID,
        out: *mut *mut std::ffi::c_void,
    ) -> HRESULT {
        if out.is_null() || iid.is_null() {
            return -2147467261;
        }
        *out = this as *mut _;
        Self::add_ref(this);
        0
    }
    
    unsafe extern "system" fn add_ref(this: *mut ManualOpenCallback) -> u32 {
        let count = (*this).ref_count.fetch_add(1, Ordering::SeqCst) + 1;
        count
    }
    
    unsafe extern "system" fn release(this: *mut ManualOpenCallback) -> u32 {
        let count = (*this).ref_count.fetch_sub(1, Ordering::SeqCst) - 1;
        if count == 0 {
            drop(Box::from_raw(this));
        }
        count
    }
    
    unsafe extern "system" fn set_completed(
        _this: *mut ManualOpenCallback,
        _files: *const u64,
        _bytes: *const u64,
    ) -> HRESULT {
        0
    }
    
    unsafe extern "system" fn set_total(
        _this: *mut ManualOpenCallback,
        _files: *const u64,
        _bytes: *const u64,
    ) -> HRESULT {
        0
    }
    
    unsafe extern "system" fn get_property(
        this: *mut ManualOpenCallback,
        prop_id: u32,
        value: *mut PROPVARIANT,
    ) -> HRESULT {
        if value.is_null() {
            return -2147467261;
        }
        
        if prop_id == 0 {
            let file_name = (*this).archive_path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            
            let bstr = alloc_bstr_from_utf32(&file_name);
            if bstr.is_null() {
                return -2147467259;
            }

            (*value).vt = 8;
            (*value).wReserved1 = 0;
            (*value).wReserved2 = 0;
            (*value).wReserved3 = 0;
            // Use write_unaligned to avoid alignment issues
            let data_ptr = (*value).data.as_mut_ptr() as *mut *mut u16;
            std::ptr::write_unaligned(data_ptr, bstr as *mut u16);
            return 0;
        }
        0
    }
    
    unsafe extern "system" fn get_stream(
        _this: *mut ManualOpenCallback,
        _name: *const u16,
        _in_stream: *mut *mut IInStream,
    ) -> HRESULT {
        1 // S_FALSE
    }
}

// ============================================================================
// Trait Definition (generated by #[vtable] in the macro version)
// ============================================================================

/// Trait for OpenCallback operations
/// When using #[vtable], this is auto-generated by the macro
/// The macro generates a trait named `OpenCallback` (same as the vtable struct name without "VTable")

// Stub for when vtable_impl feature is disabled
#[cfg(not(feature = "vtable_impl"))]
pub trait OpenCallbackTrait {
    fn set_completed(&self, files: *const u64, bytes: *const u64) -> HRESULT;
    fn set_total(&self, files: *const u64, bytes: *const u64) -> HRESULT;
    fn get_property(&self, prop_id: u32, value: *mut PROPVARIANT) -> HRESULT;
    fn get_stream(&self, name: *const u16, in_stream: *mut *mut IInStream) -> HRESULT;
}
