//! Automatic format detection based on file signatures
//!
//! This module provides functionality to detect archive formats by reading
//! file signatures (magic bytes).

use crate::format::ExtractFormat;
use std::path::Path;

/// File signature for format detection
struct FileSignature {
    format: ExtractFormat,
    bytes: &'static [u8],
    mask: Option<&'static [u8]>, // Optional mask for partial byte matching
}

/// List of known file signatures
const FILE_SIGNATURES: &[FileSignature] = &[
    // 7z format
    FileSignature {
        format: ExtractFormat::SevenZip,
        bytes: &[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C],
        mask: None,
    },
    // ZIP format
    FileSignature {
        format: ExtractFormat::Zip,
        bytes: &[0x50, 0x4B, 0x03, 0x04],
        mask: None,
    },
    // GZip format
    FileSignature {
        format: ExtractFormat::GZip,
        bytes: &[0x1F, 0x8B],
        mask: None,
    },
    // BZip2 format
    FileSignature {
        format: ExtractFormat::BZip2,
        bytes: &[0x42, 0x5A, 0x68], // "BZh"
        mask: None,
    },
    // RAR format (RAR 4.x)
    FileSignature {
        format: ExtractFormat::Rar,
        bytes: &[0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x00],
        mask: None,
    },
    // RAR5 format
    FileSignature {
        format: ExtractFormat::Rar5,
        bytes: &[0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x01, 0x00],
        mask: None,
    },
    // TAR format (at offset 257)
    FileSignature {
        format: ExtractFormat::Tar,
        bytes: &[0x75, 0x73, 0x74, 0x61, 0x72], // "ustar"
        mask: None,
    },
    // XZ format
    FileSignature {
        format: ExtractFormat::Xz,
        bytes: &[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00],
        mask: None,
    },
    // CAB format
    FileSignature {
        format: ExtractFormat::Cab,
        bytes: &[0x4D, 0x53, 0x43, 0x46], // "MSCF"
        mask: None,
    },
    // ISO format
    FileSignature {
        format: ExtractFormat::Iso,
        bytes: &[0x43, 0x44, 0x30, 0x30, 0x31], // "CD001"
        mask: None,
    },
    // ARJ format
    FileSignature {
        format: ExtractFormat::Arj,
        bytes: &[0x60, 0xEA],
        mask: None,
    },
    // LZH format
    FileSignature {
        format: ExtractFormat::Lzh,
        bytes: &[0x2D, 0x6C, 0x68], // "-lh"
        mask: None,
    },
    // RPM format
    FileSignature {
        format: ExtractFormat::Rpm,
        bytes: &[0xED, 0xAB, 0xEE, 0xDB],
        mask: None,
    },
    // DEB format
    FileSignature {
        format: ExtractFormat::Deb,
        bytes: &[0x21, 0x3C, 0x61, 0x72, 0x63, 0x68, 0x3E], // "!<arch>"
        mask: None,
    },
    // CPIO format
    FileSignature {
        format: ExtractFormat::Cpio,
        bytes: &[0x30, 0x37, 0x30, 0x37, 0x30], // "07070"
        mask: None,
    },
    // Z format (Unix compress)
    FileSignature {
        format: ExtractFormat::Z,
        bytes: &[0x1F, 0x9D],
        mask: None,
    },
    // LZMA format
    FileSignature {
        format: ExtractFormat::Lzma,
        bytes: &[0x5D, 0x00, 0x00],
        mask: None,
    },
    // WIM format
    FileSignature {
        format: ExtractFormat::Wim,
        bytes: &[0x4D, 0x53, 0x57, 0x49, 0x4D], // "MSWIM"
        mask: None,
    },
    // CHM format
    FileSignature {
        format: ExtractFormat::Chm,
        bytes: &[0x49, 0x54, 0x53, 0x46], // "ITSF"
        mask: None,
    },
    // DMG format
    FileSignature {
        format: ExtractFormat::Dmg,
        bytes: &[0x78, 0x01, 0x73, 0x0D, 0x62, 0x62, 0x60],
        mask: None,
    },
    // UDF format
    FileSignature {
        format: ExtractFormat::Udf,
        bytes: &[0x4E, 0x53, 0x52, 0x30], // "NSR0"
        mask: None,
    },
    // HFS format
    FileSignature {
        format: ExtractFormat::Hfs,
        bytes: &[0x48, 0x2B], // "H+"
        mask: None,
    },
    // NTFS format
    FileSignature {
        format: ExtractFormat::Ntfs,
        bytes: &[0x45, 0x42, 0x52, 0x20], // "EBR "
        mask: None,
    },
    // ELF format
    FileSignature {
        format: ExtractFormat::Elf,
        bytes: &[0x7F, 0x45, 0x4C, 0x46], // "\x7FELF"
        mask: None,
    },
    // Mach-O format
    FileSignature {
        format: ExtractFormat::Macho,
        bytes: &[0xFE, 0xED, 0xFA, 0xCE],
        mask: None,
    },
    // PE format
    FileSignature {
        format: ExtractFormat::Pe,
        bytes: &[0x4D, 0x5A], // "MZ"
        mask: None,
    },
    // GPT format
    FileSignature {
        format: ExtractFormat::Gpt,
        bytes: &[0x45, 0x46, 0x49, 0x20, 0x50, 0x41, 0x52, 0x54], // "EFI PART"
        mask: None,
    },
    // MBR format (boot signature at offset 510)
    FileSignature {
        format: ExtractFormat::Mbr,
        bytes: &[0x55, 0xAA],
        mask: None,
    },
    // Compound format (Microsoft Office documents)
    FileSignature {
        format: ExtractFormat::Compound,
        bytes: &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1],
        mask: None,
    },
    // Base64 format (starts with typical base64 characters)
    // Note: Base64 detection is heuristic and may have false positives
];

/// Detect archive format from a file
pub fn detect_format_from_file<P: AsRef<Path>>(file_path: P) -> Option<ExtractFormat> {
    let path = file_path.as_ref();
    
    if !path.exists() {
        return None;
    }
    
    // Read first 512 bytes for signature detection
    let mut file = std::fs::File::open(path).ok()?;
    let mut buffer = [0u8; 512];
    let bytes_read = file.read(&mut buffer).ok()?;
    
    if bytes_read == 0 {
        return None;
    }
    
    // Check each signature
    for sig in FILE_SIGNATURES {
        if matches_signature(&buffer, sig) {
            return Some(sig.format);
        }
    }
    
    // Special handling for TAR (signature at offset 257)
    if bytes_read >= 262 {
        let tar_sig = &[0x75, 0x73, 0x74, 0x61, 0x72]; // "ustar"
        if &buffer[257..262] == tar_sig {
            return Some(ExtractFormat::Tar);
        }
    }
    
    // Special handling for MBR (signature at offset 510)
    if bytes_read >= 512 {
        if buffer[510] == 0x55 && buffer[511] == 0xAA {
            return Some(ExtractFormat::Mbr);
        }
    }
    
    // Check by extension as fallback
    detect_format_from_extension(path)
}

/// Detect archive format from file extension
pub fn detect_format_from_extension<P: AsRef<Path>>(file_path: P) -> Option<ExtractFormat> {
    let path = file_path.as_ref();
    let extension = path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();
    
    match extension.as_str() {
        "7z" => Some(ExtractFormat::SevenZip),
        "zip" => Some(ExtractFormat::Zip),
        "gz" | "gzip" => Some(ExtractFormat::GZip),
        "bz2" | "bzip2" => Some(ExtractFormat::BZip2),
        "xz" => Some(ExtractFormat::Xz),
        "tar" => Some(ExtractFormat::Tar),
        "rar" => Some(ExtractFormat::Rar),
        "rar5" => Some(ExtractFormat::Rar5),
        "arj" => Some(ExtractFormat::Arj),
        "lzh" | "lha" => Some(ExtractFormat::Lzh),
        "cab" => Some(ExtractFormat::Cab),
        "iso" => Some(ExtractFormat::Iso),
        "wim" => Some(ExtractFormat::Wim),
        "chm" => Some(ExtractFormat::Chm),
        "dmg" => Some(ExtractFormat::Dmg),
        "udf" => Some(ExtractFormat::Udf),
        "rpm" => Some(ExtractFormat::Rpm),
        "deb" => Some(ExtractFormat::Deb),
        "cpio" => Some(ExtractFormat::Cpio),
        "z" => Some(ExtractFormat::Z),
        "lzma" => Some(ExtractFormat::Lzma),
        "hfs" => Some(ExtractFormat::Hfs),
        "ntfs" => Some(ExtractFormat::Ntfs),
        "elf" => Some(ExtractFormat::Elf),
        "macho" | "o" => Some(ExtractFormat::Macho),
        "exe" | "dll" | "pe" => Some(ExtractFormat::Pe),
        "gpt" => Some(ExtractFormat::Gpt),
        "mbr" => Some(ExtractFormat::Mbr),
        "doc" | "xls" | "ppt" | "msi" => Some(ExtractFormat::Compound),
        "base64" | "b64" => Some(ExtractFormat::Base64),
        _ => None,
    }
}

/// Check if buffer matches a file signature
fn matches_signature(buffer: &[u8], signature: &FileSignature) -> bool {
    if buffer.len() < signature.bytes.len() {
        return false;
    }
    
    if let Some(mask) = signature.mask {
        // Apply mask for partial byte matching
        for i in 0..signature.bytes.len() {
            if i >= mask.len() {
                break;
            }
            if (buffer[i] & mask[i]) != (signature.bytes[i] & mask[i]) {
                return false;
            }
        }
        true
    } else {
        // Direct byte comparison
        &buffer[..signature.bytes.len()] == signature.bytes
    }
}

use std::io::Read;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_detection() {
        assert_eq!(
            detect_format_from_extension("test.7z"),
            Some(ExtractFormat::SevenZip)
        );
        assert_eq!(
            detect_format_from_extension("archive.zip"),
            Some(ExtractFormat::Zip)
        );
        assert_eq!(
            detect_format_from_extension("file.tar.gz"),
            Some(ExtractFormat::GZip)
        );
        assert_eq!(
            detect_format_from_extension("data.bz2"),
            Some(ExtractFormat::BZip2)
        );
        assert_eq!(
            detect_format_from_extension("archive.xz"),
            Some(ExtractFormat::Xz)
        );
        assert_eq!(
            detect_format_from_extension("file.tar"),
            Some(ExtractFormat::Tar)
        );
        assert_eq!(
            detect_format_from_extension("file.rar"),
            Some(ExtractFormat::Rar)
        );
        assert_eq!(
            detect_format_from_extension("file.rar5"),
            Some(ExtractFormat::Rar5)
        );
        assert_eq!(
            detect_format_from_extension("file.arj"),
            Some(ExtractFormat::Arj)
        );
        assert_eq!(
            detect_format_from_extension("file.lzh"),
            Some(ExtractFormat::Lzh)
        );
        assert_eq!(
            detect_format_from_extension("file.cab"),
            Some(ExtractFormat::Cab)
        );
        assert_eq!(
            detect_format_from_extension("file.iso"),
            Some(ExtractFormat::Iso)
        );
        assert_eq!(
            detect_format_from_extension("file.wim"),
            Some(ExtractFormat::Wim)
        );
        assert_eq!(
            detect_format_from_extension("file.chm"),
            Some(ExtractFormat::Chm)
        );
        assert_eq!(
            detect_format_from_extension("file.dmg"),
            Some(ExtractFormat::Dmg)
        );
        assert_eq!(
            detect_format_from_extension("file.udf"),
            Some(ExtractFormat::Udf)
        );
        assert_eq!(
            detect_format_from_extension("file.rpm"),
            Some(ExtractFormat::Rpm)
        );
        assert_eq!(
            detect_format_from_extension("file.deb"),
            Some(ExtractFormat::Deb)
        );
        assert_eq!(
            detect_format_from_extension("file.cpio"),
            Some(ExtractFormat::Cpio)
        );
        assert_eq!(
            detect_format_from_extension("file.z"),
            Some(ExtractFormat::Z)
        );
        assert_eq!(
            detect_format_from_extension("file.lzma"),
            Some(ExtractFormat::Lzma)
        );
        assert_eq!(
            detect_format_from_extension("file.hfs"),
            Some(ExtractFormat::Hfs)
        );
        assert_eq!(
            detect_format_from_extension("file.ntfs"),
            Some(ExtractFormat::Ntfs)
        );
        assert_eq!(
            detect_format_from_extension("file.elf"),
            Some(ExtractFormat::Elf)
        );
        assert_eq!(
            detect_format_from_extension("file.macho"),
            Some(ExtractFormat::Macho)
        );
        assert_eq!(
            detect_format_from_extension("file.exe"),
            Some(ExtractFormat::Pe)
        );
        assert_eq!(
            detect_format_from_extension("file.gpt"),
            Some(ExtractFormat::Gpt)
        );
        assert_eq!(
            detect_format_from_extension("file.mbr"),
            Some(ExtractFormat::Mbr)
        );
        assert_eq!(
            detect_format_from_extension("file.doc"),
            Some(ExtractFormat::Compound)
        );
        assert_eq!(
            detect_format_from_extension("file.xls"),
            Some(ExtractFormat::Compound)
        );
        assert_eq!(
            detect_format_from_extension("file.ppt"),
            Some(ExtractFormat::Compound)
        );
        assert_eq!(
            detect_format_from_extension("file.msi"),
            Some(ExtractFormat::Compound)
        );
        assert_eq!(
            detect_format_from_extension("file.base64"),
            Some(ExtractFormat::Base64)
        );
        assert_eq!(
            detect_format_from_extension("file.b64"),
            Some(ExtractFormat::Base64)
        );

    }
}
