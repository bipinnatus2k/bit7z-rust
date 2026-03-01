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
    /// Z format (read-only, Unix compress)
    Z,
    /// DMG format (macOS disk image)
    Dmg,
    /// Ext format (Linux filesystem)
    Ext,
    /// Fat format (Windows filesystem)
    Fat,
    /// HFS format (macOS filesystem)
    Hfs,
    /// NTFS format (Windows filesystem)
    Ntfs,
    /// QCOW format (QEMU disk image)
    Qcow,
    /// VDI format (VirtualBox disk image)
    Vdi,
    /// VHD format (Virtual PC disk image)
    Vhd,
    /// VHDX format (Hyper-V disk image)
    Vhdx,
    /// VMDK format (VMware disk image)
    Vmdk,
    /// CramFS format (compressed filesystem)
    Cramfs,
    /// SquashFS format (compressed filesystem)
    Squashfs,
    /// APFS format (Apple filesystem)
    Apfs,
    /// ELF format (Linux executable)
    Elf,
    /// Mach-O format (macOS executable)
    Macho,
    /// PE format (Windows executable)
    Pe,
    /// UEFIc format (UEFI capsule)
    Uefic,
    /// UEFIf format (UEFI firmware)
    Uefif,
    /// TE format (Tiano Core EFI)
    Te,
    /// GPT format (GUID Partition Table)
    Gpt,
    /// MBR format (Master Boot Record)
    Mbr,
    /// APM format (Apple Partition Map)
    Apm,
    /// Xar format (XAR archive)
    Xar,
    /// Ar format (Unix archive)
    Ar,
    /// Compound format (Microsoft Compound Document)
    Compound,
    /// Base64 format (encoded data)
    Base64,
    /// COFF format (object file)
    Coff,
    /// IHex format (Intel HEX)
    IHex,
    /// Mub format (Mach-O Universal Binary)
    Mub,
    /// LP format (printer mirror)
    LP,
    /// Hxs format (Microsoft Help)
    Hxs,
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
    
    /// Get the 7-Zip method name as String
    pub fn to_string(&self) -> String {
        self.to_id().to_string()
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
            ExtractFormat::Z => crate::ffi::CLSID_CFormatZ,
            ExtractFormat::Dmg => crate::ffi::CLSID_CFormatDmg,
            ExtractFormat::Ext => crate::ffi::CLSID_CFormatExt,
            ExtractFormat::Fat => crate::ffi::CLSID_CFormatFat,
            ExtractFormat::Hfs => crate::ffi::CLSID_CFormatHfs,
            ExtractFormat::Ntfs => crate::ffi::CLSID_CFormatNtfs,
            ExtractFormat::Qcow => crate::ffi::CLSID_CFormatQcow,
            ExtractFormat::Vdi => crate::ffi::CLSID_CFormatVdi,
            ExtractFormat::Vhd => crate::ffi::CLSID_CFormatVhd,
            ExtractFormat::Vhdx => crate::ffi::CLSID_CFormatVhdx,
            ExtractFormat::Vmdk => crate::ffi::CLSID_CFormatVmdk,
            ExtractFormat::Cramfs => crate::ffi::CLSID_CFormatCramfs,
            ExtractFormat::Squashfs => crate::ffi::CLSID_CFormatSquashfs,
            ExtractFormat::Apfs => crate::ffi::CLSID_CFormatApfs,
            ExtractFormat::Elf => crate::ffi::CLSID_CFormatElf,
            ExtractFormat::Macho => crate::ffi::CLSID_CFormatMacho,
            ExtractFormat::Pe => crate::ffi::CLSID_CFormatPe,
            ExtractFormat::Uefic => crate::ffi::CLSID_CFormatUefic,
            ExtractFormat::Uefif => crate::ffi::CLSID_CFormatUefif,
            ExtractFormat::Te => crate::ffi::CLSID_CFormatTe,
            ExtractFormat::Gpt => crate::ffi::CLSID_CFormatGpt,
            ExtractFormat::Mbr => crate::ffi::CLSID_CFormatMbr,
            ExtractFormat::Apm => crate::ffi::CLSID_CFormatApm,
            ExtractFormat::Xar => crate::ffi::CLSID_CFormatXar,
            ExtractFormat::Ar => crate::ffi::CLSID_CFormatAr,
            ExtractFormat::Compound => crate::ffi::CLSID_CFormatCompound,
            ExtractFormat::Base64 => crate::ffi::CLSID_CFormatBase64,
            ExtractFormat::Coff => crate::ffi::CLSID_CFormatCoff,
            ExtractFormat::IHex => crate::ffi::CLSID_CFormatIHex,
            ExtractFormat::Mub => crate::ffi::CLSID_CFormatMub,
            ExtractFormat::LP => crate::ffi::CLSID_CFormatLP,
            ExtractFormat::Hxs => crate::ffi::CLSID_CFormatHxs,
        }
    }
}
