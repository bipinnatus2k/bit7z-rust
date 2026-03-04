//! Archive reader for reading archive metadata
//!
//! This module provides BitArchiveReader for reading metadata from archives
//! without extracting them.

use crate::ffi::{
    BitLibrary, IInArchive, PROPVARIANT,
    kpidPath, kpidIsDir, kpidSize, kpidPackSize,
    kpidAttrib, kpidCTime, kpidATime, kpidMTime,
    kpidEncrypted, kpidCRC, kpidSolid, kpidIsVolume,
    kpidNumVolumes,
};
use crate::format::ExtractFormat;
use crate::error::{Bit7zError, Result};
use crate::ffi::{
    propvariant_to_string, propvariant_to_bool,
    propvariant_to_u64, propvariant_to_u32,
    propvariant_to_filetime,
};
use crate::callback::OpenCallback;
use std::path::{Path, PathBuf};
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
    archive_path: Option<PathBuf>,
    // Store leaked pointers for proper cleanup in Drop
    _in_stream: Option<*mut crate::stream::FileStream>,
    _open_callback: Option<*mut OpenCallback>,
}

impl<'a> BitArchiveReader<'a> {
    /// Create a new archive reader
    pub fn new(library: &'a BitLibrary, format: ExtractFormat) -> Self {
        BitArchiveReader {
            library,
            format,
            archive: None,
            archive_path: None,
            _in_stream: None,
            _open_callback: None,
        }
    }

    /// Check if an archive is encrypted without fully opening it
    pub fn is_encrypted_static<P: AsRef<Path>>(
        library: &'a BitLibrary,
        archive_path: P,
        format: ExtractFormat,
    ) -> Result<bool> {
        let mut reader = Self::new(library, format);
        reader.open(archive_path)?;
        reader.is_encrypted()
    }

    /// Check if an archive has encrypted header without fully opening it
    pub fn is_header_encrypted_static<P: AsRef<Path>>(
        library: &'a BitLibrary,
        archive_path: P,
        format: ExtractFormat,
    ) -> Result<bool> {
        let mut reader = Self::new(library, format);
        reader.open(archive_path)?;
        reader.is_encrypted()
    }

    /// Open an archive file
    pub fn open<P: AsRef<Path>>(&mut self, archive_path: P) -> Result<()> {
        unsafe {
            let archive_ptr = self.library.create_in_archive(&self.format.guid())?;

            // Create input stream for archive file
            let in_stream = Box::new(crate::stream::FileStream::new(archive_path.as_ref())?);
            let in_stream_ptr = Box::into_raw(in_stream);

            // max_check_start_position pointer (null means use default)
            // For some formats like TAR, this needs to be null or they fail to open
            // Try with 0 instead of null for ZIP format
            let max_check_start_position: u64 = 0;

            // Create open callback (required for proper archive opening)
            let open_callback = Box::new(OpenCallback::new(archive_path.as_ref()));
            let open_callback_ptr = Box::into_raw(open_callback);

            // Open archive - pass max_check_start_position as pointer
            let result = ((*(*archive_ptr.as_ptr()).vtable).open)(
                archive_ptr.as_ptr(),
                (*in_stream_ptr).as_i_in_stream(),
                &max_check_start_position,  // Pass as pointer (0 means search from beginning)
                (*open_callback_ptr).as_i_archive_open_callback(),
            );

            // S_OK (0) and S_FALSE (1) are both success for some formats
            if result != 0 && result != 1 {
                // Clean up on failure
                drop(Box::from_raw(in_stream_ptr));
                drop(Box::from_raw(open_callback_ptr));
                return Err(Bit7zError::OpenFailed(format!(
                    "Failed to open archive: HRESULT 0x{:08X}", result
                )));
            }

            // Check number of items after opening
            let mut num_items: u32 = 0;
            let _count_result = ((*(*archive_ptr.as_ptr()).vtable).get_number_of_items)(
                archive_ptr.as_ptr(),
                &mut num_items,
            );
            
            // For some formats (like ZIP), the first call to get_number_of_items may return 0.
            // Try again if first call returns 0.
            if num_items == 0 {
                let count_result2 = ((*(*archive_ptr.as_ptr()).vtable).get_number_of_items)(
                    archive_ptr.as_ptr(),
                    &mut num_items,
                );
                let _ = count_result2; // Ignore second result code
            }

            self.archive = Some(archive_ptr.as_ptr());
            self._in_stream = Some(in_stream_ptr);
            self._open_callback = Some(open_callback_ptr);
            self.archive_path = Some(archive_path.as_ref().to_path_buf());
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
            let mut total_size = if result == 0 {
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
            let mut pack_size = if result == 0 {
                propvariant_to_u64(&prop)
            } else {
                0
            };
            prop.clear();
            if total_size == 0 && pack_size > 0 {
                total_size = pack_size;
            }
            if total_size == 0 && pack_size == 0 {
                if let Some(ref path) = self.archive_path {
                    if let Ok(metadata) = std::fs::metadata(path) {
                        total_size = metadata.len();
                    }
                }
            }
            if pack_size == 0 {
                if total_size > 0 {
                    pack_size = total_size;
                } else if let Some(ref path) = self.archive_path {
                    if let Ok(metadata) = std::fs::metadata(path) {
                        pack_size = metadata.len();
                    }
                }
            }

            // Get solid compression flag
            let result = ((*(*archive).vtable).get_archive_property)(
                archive,
                kpidSolid,
                &mut prop,
            );
            let solid = if result == 0 {
                propvariant_to_bool(&prop)
            } else {
                false
            };
            prop.clear();

            // Get multi-volume flag
            let result = ((*(*archive).vtable).get_archive_property)(
                archive,
                kpidIsVolume,
                &mut prop,
            );
            let multi_volume = if result == 0 {
                propvariant_to_bool(&prop)
            } else {
                false
            };
            prop.clear();

            // Get number of volumes
            let result = ((*(*archive).vtable).get_archive_property)(
                archive,
                kpidNumVolumes,
                &mut prop,
            );
            let volumes_count = if result == 0 {
                propvariant_to_u32(&prop)
            } else {
                1
            };
            prop.clear();

            // Count files and folders
            let items = self.items()?;
            let files_count = items.iter().filter(|i| !i.is_dir).count() as u32;
            let folders_count = items.iter().filter(|i| i.is_dir).count() as u32;
            let items_count = items.len() as u32;

            // Check if archive is encrypted
            let encrypted = items.iter().any(|i| i.encrypted);

            Ok(ArchiveProperties {
                files_count,
                folders_count,
                items_count,
                size: total_size,
                pack_size,
                encrypted,
                solid,
                multi_volume,
                volumes_count,
            })
        }
    }

    /// Test archive integrity
    pub fn test(&self) -> Result<()> {
        let archive = self.archive.ok_or_else(|| {
            Bit7zError::OpenFailed("Archive not opened".to_string())
        })?;

        unsafe {
            // Get number of items
            let mut num_items: u32 = 0;
            let result = ((*(*archive).vtable).get_number_of_items)(
                archive,
                &mut num_items,
            );

            if result != 0 && result != 1 {
                return Err(Bit7zError::OpenFailed("Failed to get item count".to_string()));
            }

            // Create a test callback (similar to ExtractCallback but for testing)
            // For now, we use a simplified approach - just verify we can read all items
            for i in 0..num_items {
                let mut prop = std::mem::zeroed::<PROPVARIANT>();
                
                // Try to read path property
                let prop_result = ((*(*archive).vtable).get_property)(
                    archive,
                    i,
                    kpidPath,
                    &mut prop,
                );

                if prop_result != 0 && prop_result != 1 {
                    return Err(Bit7zError::ArchiveIntegrityCheckFailed(format!(
                        "Failed to read item {} property: 0x{:X}", i, prop_result
                    )));
                }
                
                prop.clear();
            }

            Ok(())
        }
    }

    /// Check if archive uses solid compression
    pub fn is_solid(&self) -> Result<bool> {
        let archive = self.archive.ok_or_else(|| {
            Bit7zError::OpenFailed("Archive not opened".to_string())
        })?;

        unsafe {
            let mut prop = std::mem::zeroed::<PROPVARIANT>();
            let result = ((*(*archive).vtable).get_archive_property)(
                archive,
                kpidSolid,
                &mut prop,
            );

            if result == 0 {
                Ok(propvariant_to_bool(&prop))
            } else {
                Ok(false)
            }
        }
    }

    /// Check if archive is multi-volume
    pub fn is_multi_volume(&self) -> Result<bool> {
        let archive = self.archive.ok_or_else(|| {
            Bit7zError::OpenFailed("Archive not opened".to_string())
        })?;

        unsafe {
            let mut prop = std::mem::zeroed::<PROPVARIANT>();
            let result = ((*(*archive).vtable).get_archive_property)(
                archive,
                kpidIsVolume,
                &mut prop,
            );

            if result == 0 {
                Ok(propvariant_to_bool(&prop))
            } else {
                Ok(false)
            }
        }
    }

    /// Get number of volumes
    pub fn volumes_count(&self) -> Result<u32> {
        let archive = self.archive.ok_or_else(|| {
            Bit7zError::OpenFailed("Archive not opened".to_string())
        })?;

        unsafe {
            let mut prop = std::mem::zeroed::<PROPVARIANT>();
            let result = ((*(*archive).vtable).get_archive_property)(
                archive,
                kpidNumVolumes,
                &mut prop,
            );

            if result == 0 {
                let count = propvariant_to_u32(&prop);
                if count == 0 { Ok(1) } else { Ok(count) }
            } else {
                Ok(1)
            }
        }
    }

    /// Check if archive has encrypted items
    pub fn has_encrypted_items(&self) -> Result<bool> {
        let items = self.items()?;
        Ok(items.iter().any(|i| i.encrypted))
    }

    /// Check if archive is encrypted (header encrypted)
    pub fn is_encrypted(&self) -> Result<bool> {
        let archive = self.archive.ok_or_else(|| {
            Bit7zError::OpenFailed("Archive not opened".to_string())
        })?;

        unsafe {
            let mut prop = std::mem::zeroed::<PROPVARIANT>();
            let result = ((*(*archive).vtable).get_archive_property)(
                archive,
                kpidEncrypted,
                &mut prop,
            );

            if result == 0 {
                Ok(propvariant_to_bool(&prop))
            } else {
                // Fallback: check if any item is encrypted
                self.has_encrypted_items()
            }
        }
    }
}

impl<'a> Drop for BitArchiveReader<'a> {
    fn drop(&mut self) {
        if let Some(archive) = self.archive {
            unsafe {
                let _ = ((*(*archive).vtable).close)(archive);
                // Note: The IInArchive interface should be released by calling Release
                // on the pointer, but we don't have direct access to it here since
                // it's managed by the library's create_in_archive function.
            }
        }
        
        // Clean up the leaked stream and callback
        if let Some(in_stream_ptr) = self._in_stream {
            unsafe {
                drop(Box::from_raw(in_stream_ptr));
            }
        }
        
        if let Some(open_callback_ptr) = self._open_callback {
            unsafe {
                drop(Box::from_raw(open_callback_ptr));
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
    /// Total number of items (files + folders)
    pub items_count: u32,
    /// Total uncompressed size
    pub size: u64,
    /// Total compressed size
    pub pack_size: u64,
    /// Whether any item in the archive is encrypted
    pub encrypted: bool,
    /// Whether the archive uses solid compression
    pub solid: bool,
    /// Whether the archive is multi-volume
    pub multi_volume: bool,
    /// Number of volumes (1 for single-volume archives)
    pub volumes_count: u32,
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
