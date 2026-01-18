//! HRESULT error handling
//! 
//! This module provides error constants and helper functions for HRESULT values.

pub const S_OK: i32 = 0;
pub const S_FALSE: i32 = 1;

// Common error codes
pub const E_FAIL: i32 = -2147467259; // 0x80004005
pub const E_OUTOFMEMORY: i32 = -2147024882; // 0x8007000E
pub const E_INVALIDARG: i32 = -2147024809; // 0x80070057
pub const E_NOINTERFACE: i32 = -2147467262; // 0x80004002

// 7-Zip specific error codes
pub const E_NOTIMPL: i32 = -2147467263; // 0x80004001
pub const E_ABORT: i32 = -2147467260; // 0x80004004

/// Check if an HRESULT indicates success
#[inline]
pub const fn succeeded(hr: i32) -> bool {
    hr >= 0
}

/// Check if an HRESULT indicates failure
#[inline]
pub const fn failed(hr: i32) -> bool {
    hr < 0
}
