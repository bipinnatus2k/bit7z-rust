//! GUID (Globally Unique Identifier) definitions
//! 
//! This module defines GUID structures and constants for 7-Zip interfaces and formats.

use uuid::{uuid, Uuid};

/// Globally Unique Identifier
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GUID {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

impl GUID {
    /// Create a GUID from a UUID
    pub const fn from_uuid(uuid: Uuid) -> Self {
        let bytes = uuid.as_bytes();
        GUID {
            data1: u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            data2: u16::from_be_bytes([bytes[4], bytes[5]]),
            data3: u16::from_be_bytes([bytes[6], bytes[7]]),
            data4: [
                bytes[8], bytes[9], bytes[10], bytes[11],
                bytes[12], bytes[13], bytes[14], bytes[15],
            ],
        }
    }
    
    /// Create a GUID from raw values
    pub const fn from_raw(data1: u32, data2: u16, data3: u16, data4: [u8; 8]) -> Self {
        GUID {
            data1,
            data2,
            data3,
            data4,
        }
    }
}

// Interface GUIDs
pub const IID_IUnknown: GUID = GUID::from_uuid(uuid!("00000000-0000-0000-C000-000000000046"));
pub const IID_ISequentialInStream: GUID = GUID::from_uuid(uuid!("23170F69-33C1-278A-1000-000110070000"));
pub const IID_ISequentialOutStream: GUID = GUID::from_uuid(uuid!("23170F69-33C1-278A-1000-000110070001"));
pub const IID_IInStream: GUID = GUID::from_uuid(uuid!("23170F69-33C1-278A-1000-000110070003"));
pub const IID_IOutStream: GUID = GUID::from_uuid(uuid!("23170F69-33C1-278A-1000-000110070004"));
pub const IID_IInArchive: GUID = GUID::from_uuid(uuid!("23170F69-33C1-278A-1000-000110070000"));
pub const IID_IOutArchive: GUID = GUID::from_uuid(uuid!("23170F69-33C1-278A-1000-000110050000"));
pub const IID_IArchiveExtractCallback: GUID = GUID::from_uuid(uuid!("23170F69-33C1-278A-1000-000110070012"));
pub const IID_IArchiveUpdateCallback: GUID = GUID::from_uuid(uuid!("23170F69-33C1-278A-1000-000110070014"));
pub const IID_ICryptoGetTextPassword: GUID = GUID::from_uuid(uuid!("23170F69-33C1-278A-1000-00011007001C"));

// Format CLSIDs (main formats)
pub const CLSID_CFormat7z: GUID = GUID::from_uuid(uuid!("23170F69-40C1-278A-1000-000110070000"));
pub const CLSID_CFormatZip: GUID = GUID::from_uuid(uuid!("23170F69-40C1-278A-1000-000110060000"));
pub const CLSID_CFormatGZip: GUID = GUID::from_uuid(uuid!("23170F69-40C1-278A-1000-000110060100"));
pub const CLSID_CFormatBZip2: GUID = GUID::from_uuid(uuid!("23170F69-40C1-278A-1000-000110060200"));
pub const CLSID_CFormatRar: GUID = GUID::from_uuid(uuid!("23170F69-40C1-278A-1000-000110050400"));
pub const CLSID_CFormatRar5: GUID = GUID::from_uuid(uuid!("23170F69-40C1-278A-1000-000110050500"));
pub const CLSID_CFormatTar: GUID = GUID::from_uuid(uuid!("23170F69-40C1-278A-1000-000110060300"));
pub const CLSID_CFormatXz: GUID = GUID::from_uuid(uuid!("23170F69-40C1-278A-1000-000110060800"));
pub const CLSID_CFormatWim: GUID = GUID::from_uuid(uuid!("23170F69-40C1-278A-1000-000110060A00"));

// Additional formats (read-only extraction)
pub const CLSID_CFormatArj: GUID = GUID::from_uuid(uuid!("23170F69-40C1-278A-1000-000110070400"));
pub const CLSID_CFormatLzh: GUID = GUID::from_uuid(uuid!("23170F69-40C1-278A-1000-000110070200"));
pub const CLSID_CFormatCab: GUID = GUID::from_uuid(uuid!("23170F69-40C1-278A-1000-000110060600"));
pub const CLSID_CFormatNsis: GUID = GUID::from_uuid(uuid!("23170F69-40C1-278A-1000-000110060900"));
pub const CLSID_CFormatLzma: GUID = GUID::from_uuid(uuid!("23170F69-40C1-278A-1000-000110060500"));
pub const CLSID_CFormatIso: GUID = GUID::from_uuid(uuid!("23170F69-40C1-278A-1000-000110060700"));
pub const CLSID_CFormatUdf: GUID = GUID::from_uuid(uuid!("23170F69-40C1-278A-1000-000110060B00"));
pub const CLSID_CFormatChm: GUID = GUID::from_uuid(uuid!("23170F69-40C1-278A-1000-000110060C00"));
pub const CLSID_CFormatSplit: GUID = GUID::from_uuid(uuid!("23170F69-40C1-278A-1000-000110050600"));
pub const CLSID_CFormatRpm: GUID = GUID::from_uuid(uuid!("23170F69-40C1-278A-1000-000110060D00"));
pub const CLSID_CFormatDeb: GUID = GUID::from_uuid(uuid!("23170F69-40C1-278A-1000-000110060E00"));
pub const CLSID_CFormatCpio: GUID = GUID::from_uuid(uuid!("23170F69-40C1-278A-1000-000110060F00"));