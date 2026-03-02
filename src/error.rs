//! Error types for the bit7z-rust library
//! 
//! This module provides comprehensive error handling for all operations.

use thiserror::Error;

/// Main error type for bit7z-rust
#[derive(Debug, Error)]
pub enum Bit7zError {
    #[error("Failed to load 7-Zip library: {0}")]
    LibraryLoadFailed(String),
    
    #[error("Failed to find symbol in library: {0}")]
    SymbolNotFound(String),
    
    #[error("Failed to create archive object: 0x{0:X}")]
    CreateFailed(i32),
    
    #[error("Failed to open archive: {0}")]
    OpenFailed(String),
    
    #[error("Failed to compress: {0}")]
    CompressFailed(String),
    
    #[error("Failed to extract: {0}")]
    ExtractFailed(String),
    
    #[error("Archive is encrypted, password required")]
    EncryptedArchive,
    
    #[error("Invalid password for encrypted archive")]
    InvalidPassword,
    
    #[error("Operation cancelled by user")]
    Cancelled,
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("String contains NUL byte")]
    NulError(#[from] std::ffi::NulError),
    
    #[error("Path traversal detected: {0}")]
    PathTraversal(String),
    
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    
    #[error("Feature not supported by format: {0}")]
    FeatureNotSupported(String),

    #[error("Archive is corrupted or invalid")]
    CorruptedArchive,

    #[error("Operation failed: {0}")]
    UnknownError(String),

    #[error("Invalid item index: {0}")]
    InvalidItemIndex(u32),

    #[error("Archive integrity check failed: {0}")]
    ArchiveIntegrityCheckFailed(String),

    #[error("Failed to create temporary file: {0}")]
    TempFileCreationFailed(String),
}

impl From<crate::ffi::LibraryError> for Bit7zError {
    fn from(err: crate::ffi::LibraryError) -> Self {
        Bit7zError::LibraryLoadFailed(err.to_string())
    }
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, Bit7zError>;