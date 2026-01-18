//! Archive format definitions
//! 
//! This module defines all supported archive formats, compression levels,
//! and compression methods.

use crate::ffi::GUID;

/// Compression format for creating archives
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompressionFormat {
    /// 7z format
    SevenZip,
    /// ZIP format
    Zip,
    /// GZIP format
    GZip,
    /// BZIP2 format
    BZip2,
    /// TAR format
    Tar,
    /// XZ format
    Xz,
    /// WIM format
    Wim,
}

/// Extraction format (includes read-only formats)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtractFormat {
    /// 7z format
    SevenZip,
    /// ZIP format
    Zip,
    /// GZIP format
    GZip,
    /// BZIP2 format
    BZip2,
    /// TAR format
    Tar,
    /// XZ format
    Xz,
    /// WIM format
    Wim,
    /// RAR format (read-only)
    Rar,
    /// RAR5 format (read-only)
    Rar5,
    /// ARJ format (read-only)
    Arj,
    /// LZH format (read-only)
    Lzh,
    /// CAB format (read-only)
    Cab,
    /// NSIS format (read-only)
    Nsis,
    /// LZMA format (read-only)
    Lzma,
    /// ISO format (read-only)
    Iso,
    /// UDF format (read-only)
    Udf,
    /// CHM format (read-only)
    Chm,
    /// Split format (read-only)
    Split,
    /// RPM format (read-only)
    Rpm,
    /// DEB format (read-only)
    Deb,
    /// CPIO format (read-only)
    Cpio,
}

/// Compression level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompressionLevel {
    /// No compression (store only)
    None,
    /// Fastest compression
    Fastest,
    /// Fast compression
    Fast,
    /// Normal compression (default)
    Normal,
    /// Maximum compression
    Max,
    /// Ultra compression (slowest, best ratio)
    Ultra,
}

impl CompressionLevel {
    /// Get the numeric value for the compression level
    pub fn to_value(&self) -> u32 {
        match self {
            CompressionLevel::None => 0,
            CompressionLevel::Fastest => 1,
            CompressionLevel::Fast => 3,
            CompressionLevel::Normal => 5,
            CompressionLevel::Max => 7,
            CompressionLevel::Ultra => 9,
        }
    }
}

/// Compression method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompressionMethod {
    /// Copy (no compression)
    Copy,
    /// Deflate
    Deflate,
    /// Deflate64
    Deflate64,
    /// BZIP2
    BZip2,
    /// LZMA
    Lzma,
    /// LZMA2
    Lzma2,
    /// PPMd
    Ppmd,
    /// Delta
    Delta,
    /// BCJ
    Bcj,
    /// BCJ2
    Bcj2,
}

impl CompressionMethod {
    /// Get the 7-Zip method identifier
    pub fn to_id(&self) -> &'static str {
        match self {
            CompressionMethod::Copy => "Copy",
            CompressionMethod::Deflate => "Deflate",
            CompressionMethod::Deflate64 => "Deflate64",
            CompressionMethod::BZip2 => "BZip2",
            CompressionMethod::Lzma => "LZMA",
            CompressionMethod::Lzma2 => "LZMA2",
            CompressionMethod::Ppmd => "PPMd",
            CompressionMethod::Delta => "Delta",
            CompressionMethod::Bcj => "BCJ",
            CompressionMethod::Bcj2 => "BCJ2",
        }
    }
}

/// Format features
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatFeatures {
    /// Can contain multiple files
    pub multiple_files: bool,
    /// Supports solid archives
    pub solid_archive: bool,
    /// Supports different compression levels
    pub compression_level: bool,
    /// Supports encryption
    pub encryption: bool,
    /// Supports header encryption
    pub header_encryption: bool,
    /// Supports multiple compression methods
    pub multiple_methods: bool,
}

impl FormatFeatures {
    /// Create a format features set with all features disabled
    pub fn none() -> Self {
        FormatFeatures {
            multiple_files: false,
            solid_archive: false,
            compression_level: false,
            encryption: false,
            header_encryption: false,
            multiple_methods: false,
        }
    }
}

/// Information about an archive format
pub struct FormatInfo {
    /// Format GUID
    pub guid: GUID,
    /// Default file extension
    pub extension: &'static str,
    /// Default compression method
    pub default_method: CompressionMethod,
    /// Supported features
    pub features: FormatFeatures,
}

impl CompressionFormat {
    /// Get the format information
    pub const fn info(&self) -> FormatInfo {
        match self {
            CompressionFormat::SevenZip => FormatInfo {
                guid: crate::ffi::CLSID_CFormat7z,
                extension: "7z",
                default_method: CompressionMethod::Lzma2,
                features: FormatFeatures {
                    multiple_files: true,
                    solid_archive: true,
                    compression_level: true,
                    encryption: true,
                    header_encryption: true,
                    multiple_methods: true,
                },
            },
            CompressionFormat::Zip => FormatInfo {
                guid: crate::ffi::CLSID_CFormatZip,
                extension: "zip",
                default_method: CompressionMethod::Deflate,
                features: FormatFeatures {
                    multiple_files: true,
                    solid_archive: false,
                    compression_level: true,
                    encryption: true,
                    header_encryption: false,
                    multiple_methods: false,
                },
            },
            CompressionFormat::GZip => FormatInfo {
                guid: crate::ffi::CLSID_CFormatGZip,
                extension: "gz",
                default_method: CompressionMethod::Deflate,
                features: FormatFeatures {
                    multiple_files: false,
                    solid_archive: false,
                    compression_level: true,
                    encryption: false,
                    header_encryption: false,
                    multiple_methods: false,
                },
            },
            CompressionFormat::BZip2 => FormatInfo {
                guid: crate::ffi::CLSID_CFormatBZip2,
                extension: "bz2",
                default_method: CompressionMethod::BZip2,
                features: FormatFeatures {
                    multiple_files: false,
                    solid_archive: false,
                    compression_level: true,
                    encryption: false,
                    header_encryption: false,
                    multiple_methods: false,
                },
            },
            CompressionFormat::Tar => FormatInfo {
                guid: crate::ffi::CLSID_CFormatTar,
                extension: "tar",
                default_method: CompressionMethod::Copy,
                features: FormatFeatures {
                    multiple_files: true,
                    solid_archive: false,
                    compression_level: false,
                    encryption: false,
                    header_encryption: false,
                    multiple_methods: false,
                },
            },
            CompressionFormat::Xz => FormatInfo {
                guid: crate::ffi::CLSID_CFormatXz,
                extension: "xz",
                default_method: CompressionMethod::Lzma2,
                features: FormatFeatures {
                    multiple_files: false,
                    solid_archive: false,
                    compression_level: true,
                    encryption: false,
                    header_encryption: false,
                    multiple_methods: false,
                },
            },
            CompressionFormat::Wim => FormatInfo {
                guid: crate::ffi::CLSID_CFormatWim,
                extension: "wim",
                default_method: CompressionMethod::Lzma2,
                features: FormatFeatures {
                    multiple_files: true,
                    solid_archive: false,
                    compression_level: true,
                    encryption: false,
                    header_encryption: false,
                    multiple_methods: false,
                },
            },
        }
    }
    
    /// Get the default file extension for this format
    pub const fn extension(&self) -> &'static str {
        self.info().extension
    }
}

impl ExtractFormat {
    /// Get the format GUID
    pub const fn guid(&self) -> GUID {
        match self {
            ExtractFormat::SevenZip => crate::ffi::CLSID_CFormat7z,
            ExtractFormat::Zip => crate::ffi::CLSID_CFormatZip,
            ExtractFormat::GZip => crate::ffi::CLSID_CFormatGZip,
            ExtractFormat::BZip2 => crate::ffi::CLSID_CFormatBZip2,
            ExtractFormat::Tar => crate::ffi::CLSID_CFormatTar,
            ExtractFormat::Xz => crate::ffi::CLSID_CFormatXz,
            ExtractFormat::Wim => crate::ffi::CLSID_CFormatWim,
            ExtractFormat::Rar => crate::ffi::CLSID_CFormatRar,
            ExtractFormat::Rar5 => crate::ffi::CLSID_CFormatRar5,
            ExtractFormat::Arj => crate::ffi::CLSID_CFormatArj,
            ExtractFormat::Lzh => crate::ffi::CLSID_CFormatLzh,
            ExtractFormat::Cab => crate::ffi::CLSID_CFormatCab,
            ExtractFormat::Nsis => crate::ffi::CLSID_CFormatNsis,
            ExtractFormat::Lzma => crate::ffi::CLSID_CFormatLzma,
            ExtractFormat::Iso => crate::ffi::CLSID_CFormatIso,
            ExtractFormat::Udf => crate::ffi::CLSID_CFormatUdf,
            ExtractFormat::Chm => crate::ffi::CLSID_CFormatChm,
            ExtractFormat::Split => crate::ffi::CLSID_CFormatSplit,
            ExtractFormat::Rpm => crate::ffi::CLSID_CFormatRpm,
            ExtractFormat::Deb => crate::ffi::CLSID_CFormatDeb,
            ExtractFormat::Cpio => crate::ffi::CLSID_CFormatCpio,
        }
    }
}
