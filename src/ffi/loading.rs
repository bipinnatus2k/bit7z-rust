//! Dynamic library loading for 7-Zip
//! 
//! This module handles loading 7-Zip shared library and creating archive objects.

use crate::ffi::{
    GUID, HRESULT, IInArchive, IOutArchive,
    IID_IInArchive, IID_IOutArchive,
};
use libloading::{Library, Symbol};
use std::ffi::c_void;
use std::path::Path;
use std::path::PathBuf;
use std::ptr::NonNull;

#[cfg(target_os = "windows")]
pub const DEFAULT_LIBRARY: &str = "7z.dll";

#[cfg(target_os = "linux")]
pub const DEFAULT_LIBRARY: &str = "/usr/lib/7zip/7z.so";

#[cfg(target_os = "macos")]
pub const DEFAULT_LIBRARY: &str = "7z.dylib";

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub const DEFAULT_LIBRARY: &str = "7z.so";

fn resolve_library_path<P: AsRef<Path>>(path: Option<P>) -> PathBuf {
    if let Some(path) = path {
        return path.as_ref().to_path_buf();
    }

    if let Ok(path) = std::env::var("BIT7Z_LIBRARY_PATH") {
        return PathBuf::from(path);
    }

    let candidates: &[&str] = if cfg!(target_os = "windows") {
        &[
            "C:\\Program Files\\7-Zip\\7z.dll",
            "C:\\Program Files (x86)\\7-Zip\\7z.dll",
            "C:\\7-Zip\\7z.dll",
        ]
    } else if cfg!(target_os = "macos") {
        &[
            "/opt/homebrew/lib/7z.dylib",
            "/usr/local/lib/7z.dylib",
            "/usr/lib/7z.dylib",
            "/opt/homebrew/lib/7zip/7z.dylib",
            "/usr/local/lib/7zip/7z.dylib",
        ]
    } else {
        &[
            "/usr/lib/7zip/7z.so",
            "/usr/lib/x86_64-linux-gnu/7zip/7z.so",
            "/usr/local/lib/7zip/7z.so",
            "/opt/7zip/7z.so",
            "/usr/lib/7z.so",
            "/usr/local/lib/7z.so",
        ]
    };

    for candidate in candidates {
        let path = Path::new(candidate);
        if path.exists() {
            return path.to_path_buf();
        }
    }

    Path::new(DEFAULT_LIBRARY).to_path_buf()
}

// Function pointer types
type CreateObjectFunc = unsafe extern "system" fn(
    clsid: *const GUID,
    iid: *const GUID,
    out_object: *mut *mut c_void,
) -> HRESULT;

type SetLargePageModeFunc = unsafe extern "system" fn() -> HRESULT;

/// Library loading error
#[derive(Debug, thiserror::Error)]
pub enum LibraryError {
    #[error("Failed to load library: {0}")]
    LoadFailed(#[from] libloading::Error),
    
    #[error("Failed to find symbol: {0}")]
    SymbolNotFound(String),
    
    #[error("Failed to create archive object: {0:?}")]
    CreateFailed(HRESULT),
    
    #[error("Null pointer returned from CreateObject")]
    NullPointer,
}

/// Wrapper for 7-Zip shared library
pub struct BitLibrary {
    _library: Box<Library>,
    create_object: Symbol<'static, CreateObjectFunc>,
}

impl BitLibrary {
    /// Load 7-Zip library from specified path or default location
    pub fn new<P: AsRef<Path>>(path: Option<P>) -> Result<Self, LibraryError> {
        let lib_path = resolve_library_path(path);

        let library = Box::new(unsafe { Library::new(&lib_path)? });

        let create_object = unsafe {
            library.get::<CreateObjectFunc>(b"CreateObject\0")
                .map_err(|_| LibraryError::SymbolNotFound("CreateObject".into()))?
        };

        // Safety: We transmute the symbol to 'static lifetime. This is safe because
        // the symbol is tied to the library's lifetime, and we store the library
        // in the same struct, ensuring the symbol cannot outlive the library.
        let create_object_static = unsafe {
            std::mem::transmute::<Symbol<'_, CreateObjectFunc>, Symbol<'static, CreateObjectFunc>>(create_object)
        };

        Ok(BitLibrary {
            _library: library,
            create_object: create_object_static,
        })
    }
    
    /// Enable large page mode (if supported by library)
    pub fn set_large_page_mode(&self) -> Result<(), LibraryError> {
        if let Ok(set_large_page_mode) = unsafe {
            self._library.get::<SetLargePageModeFunc>(b"SetLargePageMode\0")
        } {
            let result = unsafe { set_large_page_mode() };
            if result != 0 {
                return Err(LibraryError::CreateFailed(result));
            }
        }
        Ok(())
    }
    
    /// Create an input archive object for specified format
    pub fn create_in_archive(&self, format: &GUID) -> Result<NonNull<IInArchive>, LibraryError> {
        let mut archive_ptr: *mut c_void = std::ptr::null_mut();
        let result = unsafe { (self.create_object)(
            format as *const GUID,
            &IID_IInArchive as *const GUID,
            &mut archive_ptr as *mut *mut c_void,
        ) };
        
        if result != 0 {
            return Err(LibraryError::CreateFailed(result));
        }
        
        NonNull::new(archive_ptr.cast()).ok_or(LibraryError::NullPointer)
    }
    
    /// Create an output archive object for specified format
    pub fn create_out_archive(&self, format: &GUID) -> Result<NonNull<IOutArchive>, LibraryError> {
        let mut archive_ptr: *mut c_void = std::ptr::null_mut();
        let result = unsafe { (self.create_object)(
            format as *const GUID,
            &IID_IOutArchive as *const GUID,
            &mut archive_ptr as *mut *mut c_void,
        ) };
        
        if result != 0 {
            return Err(LibraryError::CreateFailed(result));
        }
        
        NonNull::new(archive_ptr.cast()).ok_or(LibraryError::NullPointer)
    }
}
