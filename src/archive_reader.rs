//! Archive reader for reading archive metadata
//!
//! This module provides BitArchiveReader for reading metadata from archives
//! without extracting them.

use crate::ffi::{
    BitLibrary, IInArchive, PROPVARIANT, HRESULT,
    kpidPath, kpidIsDir, kpidSize, kpidPackSize,
    kpidAttrib, kpidCTime, kpidATime, kpidMTime,
    kpidEncrypted, kpidCRC,
};
use crate::format::ExtractFormat;
use crate::error::{Bit7zError, Result};
use crate::ffi::{
    propvariant_to_string, propvariant_to_bool,
    propvariant_to_u64, propvariant_to_u32,
    propvariant_to_filetime,
};
use crate::callback::OpenCallback;
use std::path::Path;
#[derive(Debug, Clone)]
pub struct ArchiveItem {
    /// Item index in the archive
    pub index: u32,
    /// Path of the item within the archive
    pub path: String,
    /// Whether the item is a directory
    pub is_dir: bool,
    /// Uncompressed size
    pub size: u64,
    /// Compressed size
    pub pack_size: u64,
    /// File attributes
    pub attributes: u32,
    /// Creation time (if available)
    pub creation_time: Option<std::time::SystemTime>,
    /// Access time (if available)
    pub access_time: Option<std::time::SystemTime>,
    /// Modification time (if available)
    pub modification_time: Option<std::time::SystemTime>,
    /// Whether the item is encrypted
    pub encrypted: bool,
    /// CRC32 checksum (if available)
    pub crc: Option<u32>,
}

/// Archive reader for reading archive metadata
pub struct BitArchiveReader<'a> {
    library: &'a BitLibrary,
    format: ExtractFormat,
    archive: Option<*mut IInArchive>,
}

impl<'a> BitArchiveReader<'a> {
    /// Create a new archive reader
    pub fn new(library: &'a BitLibrary, format: ExtractFormat) -> Self {
        BitArchiveReader {
            library,
            format,
            archive: None,
        }
    }

    /// Open an archive file
    pub fn open<P: AsRef<Path>>(&mut self, archive_path: P) -> Result<()> {
        unsafe {
            let archive_ptr = self.library.create_in_archive(&self.format.guid())?;

            // Create input stream for archive file
            let in_stream = Box::leak(Box::new(
                crate::stream::FileStream::new(archive_path.as_ref())?
            ));

            // max_check_start_position pointer (null means use default)
            // For some formats like TAR, this needs to be null or they fail to open
            let max_check_start_position: *const u64 = std::ptr::null();

            // Create open callback (required for proper archive opening)
            let open_callback = Box::leak(Box::new(OpenCallback::new(archive_path.as_ref())));

            // Open archive
            let result = ((*(*archive_ptr.as_ptr()).vtable).open)(
                archive_ptr.as_ptr(),
                in_stream.as_i_in_stream(),
                max_check_start_position,
                open_callback.as_i_archive_open_callback(),
            );

            // S_OK (0) and S_FALSE (1) are both success for some formats
            if result != 0 && result != 1 {
                return Err(Bit7zError::OpenFailed(format!(
                    "Failed to open archive: HRESULT 0x{:08X}", result
                )));
            }

            // Check number of items after opening
            let mut num_items: u32 = 0;
            let count_result = ((*(*archive_ptr.as_ptr()).vtable).get_number_of_items)(
                archive_ptr.as_ptr(),
                &mut num_items,
            );

            self.archive = Some(archive_ptr.as_ptr());
            Ok(())
        }
    }

    /// Get the number of items in the archive
    pub fn items_count(&self) -> Result<u32> {
        let archive = self.archive.ok_or_else(|| {
            Bit7zError::OpenFailed("Archive not opened".to_string())
        })?;

        unsafe {
            let mut num_items: u32 = 0;
            let result = ((*(*archive).vtable).get_number_of_items)(
                archive,
                &mut num_items,
            );

            if result != 0 {
                return Err(Bit7zError::OpenFailed(format!(
                    "Failed to get item count: HRESULT 0x{:08X}", result
                )));
            }

            Ok(num_items)
        }
    }

    /// Get metadata for all items in the archive
    pub fn items(&self) -> Result<Vec<ArchiveItem>> {
        let archive = self.archive.ok_or_else(|| {
            Bit7zError::OpenFailed("Archive not opened".to_string())
        })?;

        let count = self.items_count()?;
        let mut items = Vec::with_capacity(count as usize);

        for i in 0..count {
            let item = unsafe { self.read_item(archive, i) }?;
            items.push(item);
        }

        Ok(items)
    }

    /// Get metadata for a specific item
    pub fn item(&self, index: u32) -> Result<ArchiveItem> {
        let archive = self.archive.ok_or_else(|| {
            Bit7zError::OpenFailed("Archive not opened".to_string())
        })?;

        let count = self.items_count()?;
        if index >= count {
            return Err(Bit7zError::OpenFailed(format!(
                "Item index {} out of range (max: {})", index, count - 1
            )));
        }

        unsafe { self.read_item(archive, index) }
    }

    /// Read item metadata from archive
    unsafe fn read_item(
        &self,
        archive: *mut IInArchive,
        index: u32,
    ) -> Result<ArchiveItem> {
        let mut prop = std::mem::zeroed::<PROPVARIANT>();

        // Get path
        let result = ((*(*archive).vtable).get_property)(
            archive,
            index,
            kpidPath,
            &mut prop,
        );
        let path = if result == 0 {
            propvariant_to_string(&prop)?
        } else {
            String::new()
        };
        // Clear prop after use
        prop.clear();

        // Get is_dir
        let result = ((*(*archive).vtable).get_property)(
            archive,
            index,
            kpidIsDir,
            &mut prop,
        );
        let is_dir = if result == 0 {
            propvariant_to_bool(&prop)
        } else {
            false
        };
        prop.clear();

        // Get size
        let result = ((*(*archive).vtable).get_property)(
            archive,
            index,
            kpidSize,
            &mut prop,
        );
        let size = if result == 0 {
            propvariant_to_u64(&prop)
        } else {
            0
        };
        prop.clear();

        // Get pack_size
        let result = ((*(*archive).vtable).get_property)(
            archive,
            index,
            kpidPackSize,
            &mut prop,
        );
        let pack_size = if result == 0 {
            propvariant_to_u64(&prop)
        } else {
            0
        };
        prop.clear();

        // Get attributes
        let result = ((*(*archive).vtable).get_property)(
            archive,
            index,
            kpidAttrib,
            &mut prop,
        );
        let attributes = if result == 0 {
            propvariant_to_u32(&prop)
        } else {
            0
        };
        prop.clear();

        // Get creation time
        let result = ((*(*archive).vtable).get_property)(
            archive,
            index,
            kpidCTime,
            &mut prop,
        );
        let creation_time = if result == 0 {
            propvariant_to_filetime(&prop)?
        } else {
            None
        };
        prop.clear();

        // Get access time
        let result = ((*(*archive).vtable).get_property)(
            archive,
            index,
            kpidATime,
            &mut prop,
        );
        let access_time = if result == 0 {
            propvariant_to_filetime(&prop)?
        } else {
            None
        };
        prop.clear();

        // Get modification time
        let result = ((*(*archive).vtable).get_property)(
            archive,
            index,
            kpidMTime,
            &mut prop,
        );
        let modification_time = if result == 0 {
            propvariant_to_filetime(&prop)?
        } else {
            None
        };
        prop.clear();

        // Get encrypted
        let result = ((*(*archive).vtable).get_property)(
            archive,
            index,
            kpidEncrypted,
            &mut prop,
        );
        let encrypted = if result == 0 {
            propvariant_to_bool(&prop)
        } else {
            false
        };
        prop.clear();

        // Get CRC
        let result = ((*(*archive).vtable).get_property)(
            archive,
            index,
            kpidCRC,
            &mut prop,
        );
        let crc = if result == 0 {
            let crc_value = propvariant_to_u32(&prop);
            if crc_value != 0 {
                Some(crc_value)
            } else {
                None
            }
        } else {
            None
        };
        prop.clear();

        Ok(ArchiveItem {
            index,
            path,
            is_dir,
            size,
            pack_size,
            attributes,
            creation_time,
            access_time,
            modification_time,
            encrypted,
            crc,
        })
    }

    /// Get archive-level properties
    pub fn archive_properties(&self) -> Result<ArchiveProperties> {
        let archive = self.archive.ok_or_else(|| {
            Bit7zError::OpenFailed("Archive not opened".to_string())
        })?;

        unsafe {
            let mut prop = std::mem::zeroed::<PROPVARIANT>();

            // Get total size
            let result = ((*(*archive).vtable).get_archive_property)(
                archive,
                kpidSize,
                &mut prop,
            );
            let total_size = if result == 0 {
                propvariant_to_u64(&prop)
            } else {
                0
            };
            prop.clear();

            // Get pack size
            let result = ((*(*archive).vtable).get_archive_property)(
                archive,
                kpidPackSize,
                &mut prop,
            );
            let pack_size = if result == 0 {
                propvariant_to_u64(&prop)
            } else {
                0
            };
            prop.clear();

            // Count files and folders
            let items = self.items()?;
            let files_count = items.iter().filter(|i| !i.is_dir).count() as u32;
            let folders_count = items.iter().filter(|i| i.is_dir).count() as u32;

            // Check if archive is encrypted
            let encrypted = items.iter().any(|i| i.encrypted);

            Ok(ArchiveProperties {
                files_count,
                folders_count,
                size: total_size,
                pack_size,
                encrypted,
            })
        }
    }
}

impl<'a> Drop for BitArchiveReader<'a> {
    fn drop(&mut self) {
        if let Some(archive) = self.archive {
            unsafe {
                let _ = ((*(*archive).vtable).close)(archive);
            }
        }
    }
}

/// Archive-level properties
#[derive(Debug, Clone)]
pub struct ArchiveProperties {
    /// Number of files in the archive
    pub files_count: u32,
    /// Number of folders in the archive
    pub folders_count: u32,
    /// Total uncompressed size
    pub size: u64,
    /// Total compressed size
    pub pack_size: u64,
    /// Whether any item in the archive is encrypted
    pub encrypted: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_creation() {
        let item = ArchiveItem {
            index: 0,
            path: "test.txt".to_string(),
            is_dir: false,
            size: 1024,
            pack_size: 512,
            attributes: 0x20,
            creation_time: None,
            access_time: None,
            modification_time: None,
            encrypted: false,
            crc: Some(0x12345678),
        };

        assert_eq!(item.index, 0);
        assert_eq!(item.path, "test.txt");
        assert!(!item.is_dir);
        assert_eq!(item.size, 1024);
        assert_eq!(item.pack_size, 512);
        assert!(item.crc.is_some());
    }
}
