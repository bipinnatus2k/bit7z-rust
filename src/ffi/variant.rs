//! PROPVARIANT and property handling
//! 
//! This module defines PROPVARIANT union type for passing properties
//! to and from 7-Zip interfaces.

use std::ffi::c_void;
use crate::error::{Bit7zError, Result};

/// Variant type enumeration
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VARENUM {
    VT_EMPTY = 0,
    VT_NULL = 1,
    VT_I2 = 2,
    VT_I4 = 3,
    VT_R4 = 4,
    VT_R8 = 5,
    VT_CY = 6,
    VT_DATE = 7,
    VT_BSTR = 8,
    VT_DISPATCH = 9,
    VT_ERROR = 10,
    VT_BOOL = 11,
    VT_VARIANT = 12,
    VT_UNKNOWN = 13,
    VT_DECIMAL = 14,
    VT_I1 = 16,
    VT_UI1 = 17,
    VT_UI2 = 18,
    VT_UI4 = 19,
    VT_I8 = 20,
    VT_UI8 = 21,
    VT_INT = 22,
    VT_UINT = 23,
    VT_VOID = 24,
    VT_HRESULT = 25,
    VT_PTR = 26,
    VT_SAFEARRAY = 27,
    VT_CARRAY = 28,
    VT_USERDEFINED = 29,
    VT_LPSTR = 30,
    VT_LPWSTR = 31,
    VT_FILETIME = 64,
    VT_BLOB = 65,
    VT_STREAM = 66,
    VT_STORAGE = 67,
    VT_STREAMED_OBJECT = 68,
    VT_STORED_OBJECT = 69,
    VT_BLOB_OBJECT = 70,
    VT_CF = 71,
    VT_CLSID = 72,
    VT_VERSIONED_STREAM = 73,
}

/// Property identifier enumeration
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PROPID {
    NoProperty = 0,
    MainSubfile,
    HandlerItemIndex,
    Path,
    Name,
    Extension,
    IsDir,
    Size,
    PackSize,
    Attrib,
    CTime,
    ATime,
    MTime,
    Solid,
    Commented,
    Encrypted,
    SplitBefore,
    SplitAfter,
    DictionarySize,
    CRC,
    Type,
    IsAnti,
    Method,
    HostOS,
    FileSystem,
    User,
    Group,
    Block,
    Comment,
    Position,
    Prefix,
    NumSubDirs,
    NumSubFiles,
    UnpackVer,
    Volume,
    IsVolume,
    Offset,
    Links,
    Blocks,
    NumVolumes,
    TimeType,
    Bit64,
    BigEndian,
    Cpu,
    Os,
    TextMode,
    CodePage,
    IsTree,
    CRCError,
    NumErrors,
    ErrorFlags,
    ErrorDataIndex,
    NumAltStreams,
    AltStreamsSize,
    IsAltStream,
    CopyLink,
    HardLink,
    Inode,
    Device,
    UserId,
    GroupId,
    Attributes,
    TotalSize,
    FreeSpace,
    ClusterSize,
    VolumeName,
    LocalName,
    Provider,
    NTSecurity,
    IsSorted,
    Extension_A,
    CreatorApp,
    SectorSize,
    PosixAttrib,
    Link,
    ErrorType,
    Setattr,
    TotalBlocks,
    VolumeIndex,
    SubType,
    ShortName,
    CreatorApp64,
    UserDefinedStart,
}

pub const kpidNoProperty: PROPID = PROPID::NoProperty;
pub const kpidMainSubfile: PROPID = PROPID::MainSubfile;
pub const kpidHandlerItemIndex: PROPID = PROPID::HandlerItemIndex;
pub const kpidPath: PROPID = PROPID::Path;
pub const kpidName: PROPID = PROPID::Name;
pub const kpidExtension: PROPID = PROPID::Extension;
pub const kpidIsDir: PROPID = PROPID::IsDir;
pub const kpidSize: PROPID = PROPID::Size;
pub const kpidPackSize: PROPID = PROPID::PackSize;
pub const kpidAttrib: PROPID = PROPID::Attrib;
pub const kpidCTime: PROPID = PROPID::CTime;
pub const kpidATime: PROPID = PROPID::ATime;
pub const kpidMTime: PROPID = PROPID::MTime;
pub const kpidSolid: PROPID = PROPID::Solid;
pub const kpidCommented: PROPID = PROPID::Commented;
pub const kpidEncrypted: PROPID = PROPID::Encrypted;
pub const kpidSplitBefore: PROPID = PROPID::SplitBefore;
pub const kpidSplitAfter: PROPID = PROPID::SplitAfter;
pub const kpidDictionarySize: PROPID = PROPID::DictionarySize;
pub const kpidCRC: PROPID = PROPID::CRC;
pub const kpidType: PROPID = PROPID::Type;
pub const kpidIsAnti: PROPID = PROPID::IsAnti;
pub const kpidMethod: PROPID = PROPID::Method;
pub const kpidHostOS: PROPID = PROPID::HostOS;
pub const kpidFileSystem: PROPID = PROPID::FileSystem;
pub const kpidUser: PROPID = PROPID::User;
pub const kpidGroup: PROPID = PROPID::Group;
pub const kpidBlock: PROPID = PROPID::Block;
pub const kpidComment: PROPID = PROPID::Comment;
pub const kpidPosition: PROPID = PROPID::Position;
pub const kpidPrefix: PROPID = PROPID::Prefix;
pub const kpidNumSubDirs: PROPID = PROPID::NumSubDirs;
pub const kpidNumSubFiles: PROPID = PROPID::NumSubFiles;
pub const kpidUnpackVer: PROPID = PROPID::UnpackVer;
pub const kpidVolume: PROPID = PROPID::Volume;
pub const kpidIsVolume: PROPID = PROPID::IsVolume;
pub const kpidOffset: PROPID = PROPID::Offset;
pub const kpidLinks: PROPID = PROPID::Links;
pub const kpidBlocks: PROPID = PROPID::Blocks;
pub const kpidNumVolumes: PROPID = PROPID::NumVolumes;
pub const kpidTimeType: PROPID = PROPID::TimeType;
pub const kpidBit64: PROPID = PROPID::Bit64;
pub const kpidBigEndian: PROPID = PROPID::BigEndian;
pub const kpidCpu: PROPID = PROPID::Cpu;
pub const kpidOs: PROPID = PROPID::Os;
pub const kpidTextMode: PROPID = PROPID::TextMode;
pub const kpidCodePage: PROPID = PROPID::CodePage;
pub const kpidIsTree: PROPID = PROPID::IsTree;
pub const kpidCRCError: PROPID = PROPID::CRCError;
pub const kpidNumErrors: PROPID = PROPID::NumErrors;
pub const kpidErrorFlags: PROPID = PROPID::ErrorFlags;
pub const kpidErrorDataIndex: PROPID = PROPID::ErrorDataIndex;
pub const kpidNumAltStreams: PROPID = PROPID::NumAltStreams;
pub const kpidAltStreamsSize: PROPID = PROPID::AltStreamsSize;
pub const kpidIsAltStream: PROPID = PROPID::IsAltStream;
pub const kpidCopyLink: PROPID = PROPID::CopyLink;
pub const kpidHardLink: PROPID = PROPID::HardLink;
pub const kpidInode: PROPID = PROPID::Inode;
pub const kpidDevice: PROPID = PROPID::Device;
pub const kpidUserId: PROPID = PROPID::UserId;
pub const kpidGroupId: PROPID = PROPID::GroupId;
pub const kpidAttributes: PROPID = PROPID::Attributes;
pub const kpidTotalSize: PROPID = PROPID::TotalSize;
pub const kpidFreeSpace: PROPID = PROPID::FreeSpace;
pub const kpidClusterSize: PROPID = PROPID::ClusterSize;
pub const kpidVolumeName: PROPID = PROPID::VolumeName;
pub const kpidLocalName: PROPID = PROPID::LocalName;
pub const kpidProvider: PROPID = PROPID::Provider;
pub const kpidNTSecurity: PROPID = PROPID::NTSecurity;
pub const kpidIsSorted: PROPID = PROPID::IsSorted;
pub const kpidExtension_A: PROPID = PROPID::Extension_A;
pub const kpidCreatorApp: PROPID = PROPID::CreatorApp;
pub const kpidSectorSize: PROPID = PROPID::SectorSize;
pub const kpidPosixAttrib: PROPID = PROPID::PosixAttrib;
pub const kpidLink: PROPID = PROPID::Link;
pub const kpidErrorType: PROPID = PROPID::ErrorType;
pub const kpidSetattr: PROPID = PROPID::Setattr;
pub const kpidTotalBlocks: PROPID = PROPID::TotalBlocks;
pub const kpidVolumeIndex: PROPID = PROPID::VolumeIndex;
pub const kpidSubType: PROPID = PROPID::SubType;
pub const kpidShortName: PROPID = PROPID::ShortName;
pub const kpidCreatorApp64: PROPID = PROPID::CreatorApp64;

/// PROPVARIANT - A variant type used by COM interfaces
#[repr(C)]
pub struct PROPVARIANT {
    pub vt: u16,
    pub wReserved1: u16,
    pub wReserved2: u16,
    pub wReserved3: u16,
    pub data: [u8; 16], // Anonymous union
}

impl Default for PROPVARIANT {
    fn default() -> Self {
        PROPVARIANT {
            vt: VARENUM::VT_EMPTY as u16,
            wReserved1: 0,
            wReserved2: 0,
            wReserved3: 0,
            data: [0; 16],
        }
    }
}

impl PROPVARIANT {
    /// Clear the variant
    pub unsafe fn clear(&mut self) {
        // For simplicity, we just reset to empty
        self.vt = VARENUM::VT_EMPTY as u16;
        self.data = [0; 16];
    }
    
    /// Check if the variant is empty
    pub fn is_empty(&self) -> bool {
        self.vt == VARENUM::VT_EMPTY as u16
    }
}

/// Convert PROPVARIANT to String
pub unsafe fn propvariant_to_string(prop: &PROPVARIANT) -> Result<String> {
    let vt = prop.vt as u32;
    
    if vt == VARENUM::VT_LPSTR as u32 {
        // LPSTR (ANSI string)
        let ptr = *(prop.data.as_ptr() as *const *const i8);
        if ptr.is_null() {
            return Ok(String::new());
        }
        let cstr = std::ffi::CStr::from_ptr(ptr);
        Ok(cstr.to_string_lossy().to_string())
    } else if vt == VARENUM::VT_BSTR as u32 {
        // BSTR (BSTR string)
        let ptr = *(prop.data.as_ptr() as *const *const u16);
        if ptr.is_null() {
            return Ok(String::new());
        }
        // Find length (BSTR has length prefix at -4 bytes)
        let len = *(ptr.offset(-1) as *const u32) as usize;
        let slice = std::slice::from_raw_parts(ptr, len);
        String::from_utf16(slice).map_err(|e| {
            Bit7zError::ExtractFailed(format!("Invalid UTF-16 string: {}", e))
        })
    } else if vt == VARENUM::VT_LPWSTR as u32 {
        // LPWSTR (Unicode string)
        let ptr = *(prop.data.as_ptr() as *const *const u16);
        if ptr.is_null() {
            return Ok(String::new());
        }
        // Find null terminator
        let mut len = 0;
        while *ptr.add(len) != 0 {
            len += 1;
        }
        let slice = std::slice::from_raw_parts(ptr, len);
        String::from_utf16(slice).map_err(|e| {
            Bit7zError::ExtractFailed(format!("Invalid UTF-16 string: {}", e))
        })
    } else {
        Ok(String::new())
    }
}

/// Convert PROPVARIANT to u32
pub unsafe fn propvariant_to_u32(prop: &PROPVARIANT) -> u32 {
    let vt = prop.vt as u32;
    
    if vt == VARENUM::VT_UI1 as u32 {
        prop.data[0] as u32
    } else if vt == VARENUM::VT_UI2 as u32 {
        u16::from_le_bytes([prop.data[0], prop.data[1]]) as u32
    } else if vt == VARENUM::VT_UI4 as u32 {
        u32::from_le_bytes([
            prop.data[0], prop.data[1], prop.data[2], prop.data[3]
        ])
    } else if vt == VARENUM::VT_I4 as u32 {
        i32::from_le_bytes([
            prop.data[0], prop.data[1], prop.data[2], prop.data[3]
        ]) as u32
    } else {
        0
    }
}

/// Convert PROPVARIANT to u64
pub unsafe fn propvariant_to_u64(prop: &PROPVARIANT) -> u64 {
    let vt = prop.vt as u32;
    
    if vt == VARENUM::VT_UI8 as u32 {
        u64::from_le_bytes([
            prop.data[0], prop.data[1], prop.data[2], prop.data[3],
            prop.data[4], prop.data[5], prop.data[6], prop.data[7]
        ])
    } else if vt == VARENUM::VT_I8 as u32 {
        i64::from_le_bytes([
            prop.data[0], prop.data[1], prop.data[2], prop.data[3],
            prop.data[4], prop.data[5], prop.data[6], prop.data[7]
        ]) as u64
    } else if vt == VARENUM::VT_UI4 as u32 {
        propvariant_to_u32(prop) as u64
    } else if vt == VARENUM::VT_I4 as u32 {
        propvariant_to_u32(prop) as i64 as u64
    } else {
        0
    }
}

/// Convert PROPVARIANT to bool
pub unsafe fn propvariant_to_bool(prop: &PROPVARIANT) -> bool {
    let vt = prop.vt as u32;
    
    if vt == VARENUM::VT_BOOL as u32 {
        (prop.data[0] as i16) != 0
    } else if vt == VARENUM::VT_UI1 as u32 {
        prop.data[0] != 0
    } else if vt == VARENUM::VT_I4 as u32 {
        propvariant_to_u32(prop) != 0
    } else {
        false
    }
}