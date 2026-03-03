//! Base vtable infrastructure for COM interface implementation
//!
//! This module provides the foundation for implementing COM interfaces
//! using the `vtable` crate, including:
//! - Core COM trait definitions
//! - HRESULT constants
//! - FFI conversion utilities

use vtable::*;
use std::ffi::c_void;
use std::sync::atomic::{AtomicU32, Ordering};

// ============================================================================
// HRESULT Constants
// ============================================================================

/// Success code
pub const S_OK: HRESULT = 0;
/// Success code (alternative)
pub const S_FALSE: HRESULT = 1;
/// No such interface supported
pub const E_NOINTERFACE: HRESULT = -2147467262;
/// Invalid pointer
pub const E_POINTER: HRESULT = -2147467261;
/// General failure
pub const E_FAIL: HRESULT = -2147467259;
/// Out of memory
pub const E_OUTOFMEMORY: HRESULT = -2147024882;
/// Invalid argument
pub const E_INVALIDARG: HRESULT = -2147024809;

pub type HRESULT = i32;

// ============================================================================
// COM Trait Definitions
// ============================================================================

/// Core COM IUnknown interface trait
///
/// All COM objects must implement this trait.
#[vtable]
pub struct IUnknownVTable {
    /// Queries the object for a specific interface
    query_interface: fn(VRef<IUnknownVTable>, &crate::ffi::GUID) -> *mut c_void,
    /// Increments the reference count
    add_ref: fn(VRef<IUnknownVTable>) -> u32,
    /// Decrements the reference count
    release: fn(VRefMut<IUnknownVTable>) -> u32,
    /// Destructor
    drop: fn(VRefMut<IUnknownVTable>),
}

/// ISequentialInStream interface trait
#[vtable]
pub struct ISequentialInStreamVTable {
    /// Base IUnknown interface
    base: IUnknownVTable,
    /// Reads data from the stream
    read: fn(VRefMut<ISequentialInStreamVTable>, *mut c_void, u32, *mut u32) -> HRESULT,
}

/// IInStream interface trait (extends ISequentialInStream)
#[vtable]
pub struct IInStreamVTable {
    /// Base ISequentialInStream interface
    base: ISequentialInStreamVTable,
    /// Seeks to a position in the stream
    seek: fn(VRefMut<IInStreamVTable>, i64, u32, *mut u64) -> HRESULT,
}

/// ISequentialOutStream interface trait
#[vtable]
pub struct ISequentialOutStreamVTable {
    /// Base IUnknown interface
    base: IUnknownVTable,
    /// Writes data to the stream
    write: fn(VRefMut<ISequentialOutStreamVTable>, *const c_void, u32, *mut u32) -> HRESULT,
}

/// IOutStream interface trait (extends ISequentialOutStream)
#[vtable]
pub struct IOutStreamVTable {
    /// Base ISequentialOutStream interface
    base: ISequentialOutStreamVTable,
    /// Seeks to a position in the stream
    seek: fn(VRefMut<IOutStreamVTable>, i64, u32, *mut u64) -> HRESULT,
    /// Sets the size of the stream
    set_size: fn(VRefMut<IOutStreamVTable>, u64) -> HRESULT,
}

/// IProgress interface trait
#[vtable]
pub struct IProgressVTable {
    /// Base IUnknown interface
    base: IUnknownVTable,
    /// Reports completed work
    set_completed: fn(VRef<IProgressVTable>, *const u64) -> HRESULT,
    /// Reports total work
    set_total: fn(VRef<IProgressVTable>, u64) -> HRESULT,
}

/// IArchiveOpenCallback interface trait
#[vtable]
pub struct IArchiveOpenCallbackVTable {
    /// Base IUnknown interface
    base: IUnknownVTable,
    /// Reports completed files/bytes during open
    set_completed: fn(VRef<IArchiveOpenCallbackVTable>, *const u64, *const u64) -> HRESULT,
    /// Reports total files/bytes to open
    set_total: fn(VRef<IArchiveOpenCallbackVTable>, *const u64, *const u64) -> HRESULT,
}

/// IArchiveOpenVolumeCallback interface trait
#[vtable]
pub struct IArchiveOpenVolumeCallbackVTable {
    /// Base IUnknown interface
    base: IUnknownVTable,
    /// Gets a property of the volume
    get_property: fn(VRef<IArchiveOpenVolumeCallbackVTable>, u32, *mut crate::ffi::PROPVARIANT) -> HRESULT,
    /// Gets a stream for a volume file
    get_stream: fn(VRef<IArchiveOpenVolumeCallbackVTable>, *const u16, *mut *mut crate::ffi::IInStream) -> HRESULT,
}

/// ICryptoGetTextPassword interface trait
#[vtable]
pub struct ICryptoGetTextPasswordVTable {
    /// Base IUnknown interface
    base: IUnknownVTable,
    /// Gets the password for encrypted archives
    crypto_get_text_password: fn(VRef<ICryptoGetTextPasswordVTable>, *mut *mut u16) -> HRESULT,
}

/// IArchiveExtractCallback interface trait
#[vtable]
pub struct IArchiveExtractCallbackVTable {
    /// Base IProgress interface
    base: IProgressVTable,
    /// Gets the output stream for an item
    get_stream: fn(VRef<IArchiveExtractCallbackVTable>, u32, *mut *mut crate::ffi::ISequentialOutStream, *mut i32) -> HRESULT,
    /// Prepares for an operation
    prepare_operation: fn(VRef<IArchiveExtractCallbackVTable>, i32) -> HRESULT,
    /// Sets the result of an operation
    set_operation_result: fn(VRef<IArchiveExtractCallbackVTable>, i32) -> HRESULT,
}

/// IArchiveUpdateCallback interface trait
#[vtable]
pub struct IArchiveUpdateCallbackVTable {
    /// Base IProgress interface
    base: IProgressVTable,
    /// Gets update item info
    get_update_item_info: fn(VRef<IArchiveUpdateCallbackVTable>, u32, *mut i32, *mut i32, *mut u32) -> HRESULT,
    /// Gets a property of an item
    get_property: fn(VRef<IArchiveUpdateCallbackVTable>, u32, crate::ffi::PROPID, *mut crate::ffi::PROPVARIANT) -> HRESULT,
    /// Gets the input stream for an item
    get_stream: fn(VRef<IArchiveUpdateCallbackVTable>, u32, *mut *mut crate::ffi::ISequentialInStream) -> HRESULT,
    /// Sets the result of an operation
    set_operation_result: fn(VRef<IArchiveUpdateCallbackVTable>, i32) -> HRESULT,
}

/// ICompressProgressInfo interface trait
#[vtable]
pub struct ICompressProgressInfoVTable {
    /// Base IUnknown interface
    base: IUnknownVTable,
    /// Reports compression ratio info
    set_ratio_info: fn(VRef<ICompressProgressInfoVTable>, *const u64, *const u64) -> HRESULT,
}

// ============================================================================
// Helper Types and Utilities
// ============================================================================

/// Reference counter for COM objects
pub struct ComRefCounter {
    count: AtomicU32,
}

impl ComRefCounter {
    /// Creates a new reference counter with initial value of 1
    pub fn new() -> Self {
        ComRefCounter {
            count: AtomicU32::new(1),
        }
    }

    /// Creates a new reference counter with a specific initial value
    pub fn with_initial(initial: u32) -> Self {
        ComRefCounter {
            count: AtomicU32::new(initial),
        }
    }

    /// Increments the reference count
    pub fn add_ref(&self) -> u32 {
        self.count.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Decrements the reference count and returns the new value
    pub fn release(&self) -> u32 {
        self.count.fetch_sub(1, Ordering::SeqCst) - 1
    }

    /// Gets the current reference count
    pub fn count(&self) -> u32 {
        self.count.load(Ordering::SeqCst)
    }
}

impl Default for ComRefCounter {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper trait for COM interface QueryInterface implementation
pub trait ComInterface {
    /// Checks if the object supports the given interface
    fn supports_interface(&self, iid: &crate::ffi::GUID) -> bool;
    
    /// Gets the interface pointer for a supported IID
    /// Returns null if not supported
    fn get_interface(&self, iid: &crate::ffi::GUID) -> *mut c_void;
}

/// Macro to generate static vtable for a type
#[macro_export]
macro_rules! vtable_static {
    ($name:ident, $type:ty, $vtable:ty) => {
        $crate::vtable_base::vtable::$vtable::_static!($name for $type);
    };
}

// Re-export vtable crate items
pub use vtable::{VRef, VRefMut, VBox};
