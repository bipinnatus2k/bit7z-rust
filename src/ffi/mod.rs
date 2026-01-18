//! FFI bindings for 7-Zip library
//! 
//! This module provides low-level bindings to 7-Zip COM interfaces.
//! It is primarily used internally by higher-level abstractions.

pub mod guid;
pub mod interfaces;
pub mod loading;
pub mod hresult;
pub mod variant;

pub use guid::*;
pub use interfaces::*;
pub use loading::*;
pub use hresult::*;
pub use variant::*;

// Re-export vtable types
pub use interfaces::IArchiveExtractCallbackVTable;
pub use interfaces::IArchiveUpdateCallbackVTable;
pub use interfaces::ICryptoGetTextPasswordVTable;
pub use interfaces::IProgressVTable;

use std::ffi::c_void;

pub type HRESULT = i32;
pub type ULONG = u32;
pub type ULONGLONG = u64;