//! Archive writer for updating existing archives
//! 
//! This module provides BitArchiveWriter for modifying existing archives
//! by adding, renaming, or deleting items.

use crate::ffi::{
    BitLibrary, IInArchive, IOutArchive, IArchiveUpdateCallback,
    ISequentialInStream, ICryptoGetTextPassword,
    PROPVARIANT, PROPID, HRESULT,
};
use crate::format::CompressionFormat;
use crate::error::{Bit7zError, Result};
use crate::stream::FileStream;
use std::cell::UnsafeCell;
use std::ffi::c_void;
use std::path::Path;
use std::pin::Pin;
use std::ptr;
use std::mem;

/// Archive update operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateOperation {
    /// Keep item as is
    Keep,
    /// Add a new file
    Add,
    /// Delete item
    Delete,
    /// Rename item
    Rename,
}

/// Archive update item
#[derive(Debug, Clone)]
pub struct UpdateItem {
    /// Index in original archive (for Keep/Delete/Rename)
    pub index: Option<u32>,
    /// Path to add (for Add)
    pub path: Option<String>,
    /// New name (for Rename)
    pub new_name: Option<String>,
    /// Operation type
    pub operation: UpdateOperation,
}

impl UpdateItem {
    /// Create a "keep" operation
    pub fn keep(index: u32) -> Self {
        UpdateItem {
            index: Some(index),
            path: None,
            new_name: None,
            operation: UpdateOperation::Keep,
        }
    }
    
    /// Create an "add" operation
    pub fn add<P: AsRef<Path>>(path: P) -> Self {
        UpdateItem {
            index: None,
            path: Some(path.as_ref().to_string_lossy().to_string()),
            new_name: None,
            operation: UpdateOperation::Add,
        }
    }
    
    /// Create a "delete" operation
    pub fn delete(index: u32) -> Self {
        UpdateItem {
            index: Some(index),
            path: None,
            new_name: None,
            operation: UpdateOperation::Delete,
        }
    }
    
    /// Create a "rename" operation
    pub fn rename(index: u32, new_name: String) -> Self {
        UpdateItem {
            index: Some(index),
            path: None,
            new_name: Some(new_name),
            operation: UpdateOperation::Rename,
        }
    }
}

/// Archive writer for updating existing archives
pub struct BitArchiveWriter<'a> {
    library: &'a BitLibrary,
    format: CompressionFormat,
    password: Option<String>,
    compression_level: Option<crate::format::CompressionLevel>,
}

impl<'a> BitArchiveWriter<'a> {
    /// Create a new archive writer
    pub fn new(library: &'a BitLibrary, format: CompressionFormat) -> Self {
        BitArchiveWriter {
            library,
            format,
            password: None,
            compression_level: None,
        }
    }
    
    /// Set password for encrypted archives
    pub fn password(&mut self, password: impl Into<String>) -> &mut Self {
        self.password = Some(password.into());
        self
    }
    
    /// Set compression level
    pub fn compression_level(&mut self, level: crate::format::CompressionLevel) -> &mut Self {
        self.compression_level = Some(level);
        self
    }
    
    /// Update an archive with specified operations
    pub fn update<P: AsRef<Path>>(
        &self,
        archive_path: P,
        output_path: P,
        items: &[UpdateItem],
    ) -> Result<()> {
        unimplemented!("Archive update implementation coming soon")
    }
}

/// Update callback for archive operations
struct UpdateCallback {
    items: Vec<UpdateItem>,
    password: Option<String>,
    current_index: UnsafeCell<u32>,
    ref_count: UnsafeCell<u32>,
    vtable: Pin<Box<crate::ffi::IArchiveUpdateCallbackVTable>>,
}

impl UpdateCallback {
    fn new(items: Vec<UpdateItem>, password: Option<String>) -> Self {
        let vtable = Box::pin(crate::ffi::IArchiveUpdateCallbackVTable {
            base: crate::ffi::IProgressVTable {
                base: crate::ffi::IUnknownVTable {
                    query_interface: Self::query_interface,
                    add_ref: Self::add_ref,
                    release: Self::release,
                },
                set_completed: Self::set_completed,
                set_total: Self::set_total,
            },
            get_update_item_info: Self::get_update_item_info,
            get_property: Self::get_property,
            get_stream: Self::get_stream,
            set_operation_result: Self::set_operation_result,
        });

        UpdateCallback {
            items,
            password,
            current_index: UnsafeCell::new(0),
            ref_count: UnsafeCell::new(1),
            vtable,
        }
    }
    
    fn as_i_archive_update_callback(&self) -> *mut crate::ffi::IArchiveUpdateCallback {
        self as *const UpdateCallback as *mut UpdateCallback as *mut crate::ffi::IArchiveUpdateCallback
    }
    
    unsafe extern "system" fn query_interface(
        _this: *mut crate::ffi::IUnknown,
        _iid: *const crate::ffi::GUID,
        _out: *mut *mut c_void,
    ) -> HRESULT {
        -1 // E_NOINTERFACE
    }
    
    unsafe extern "system" fn add_ref(this: *mut crate::ffi::IUnknown) -> u32 {
        let callback = this as *mut UpdateCallback;
        let ref_count = &(*callback).ref_count;
        let count = *ref_count.get();
        *ref_count.get() = count + 1;
        count + 1
    }
    
    unsafe extern "system" fn release(this: *mut crate::ffi::IUnknown) -> u32 {
        let callback = this as *mut UpdateCallback;
        let ref_count = &(*callback).ref_count;
        let count = *ref_count.get();
        if count > 0 {
            *ref_count.get() = count - 1;
            count - 1
        } else {
            0
        }
    }
    
    unsafe extern "system" fn set_total(
        _this: *mut crate::ffi::IProgress,
        _size: u64,
    ) -> HRESULT {
        0 // S_OK
    }

    unsafe extern "system" fn set_completed(
        _this: *mut crate::ffi::IProgress,
        _complete_value: *const u64,
    ) -> HRESULT {
        0 // S_OK
    }
    
    unsafe extern "system" fn get_update_item_info(
        this: *mut crate::ffi::IArchiveUpdateCallback,
        index: u32,
        new_data: *mut i32,
        new_properties: *mut i32,
        index_in_archive: *mut u32,
    ) -> HRESULT {
        let callback = &*(this as *const UpdateCallback);

        if index >= callback.items.len() as u32 {
            return -2147467259; // E_INVALIDARG
        }

        let item = &callback.items[index as usize];
        
        match item.operation {
            UpdateOperation::Add => {
                *new_data = 1; // TRUE
                *new_properties = 1; // TRUE
                *index_in_archive = 0xFFFFFFFF; // UINT32_MAX - new item
            }
            UpdateOperation::Keep | UpdateOperation::Rename | UpdateOperation::Delete => {
                *new_data = 0; // FALSE
                *new_properties = if item.operation == UpdateOperation::Rename {
                    1 // TRUE - properties changed
                } else {
                    0 // FALSE - no change
                };
                *index_in_archive = item.index.unwrap_or(0);
            }
        }
        
        0 // S_OK
    }
    
    unsafe extern "system" fn get_property(
        this: *mut crate::ffi::IArchiveUpdateCallback,
        index: u32,
        prop_id: PROPID,
        value: *mut PROPVARIANT,
    ) -> HRESULT {
        let callback = &*(this as *const UpdateCallback);

        if index >= callback.items.len() as u32 {
            return -2147467259; // E_INVALIDARG
        }

        let item = &callback.items[index as usize];

        // For rename operations, return new name as BSTR
        if prop_id == crate::ffi::kpidPath {
            if let Some(ref name) = item.new_name {
                // Convert string to UTF-16
                let name_utf16: Vec<u16> = name.encode_utf16().collect();
                let len = name_utf16.len();

                // Allocate memory for BSTR (length prefix + data + null terminator)
                // Note: This is simplified - in production should use proper allocator
                let bstr_ptr = libc::malloc((len * 2 + 4) as usize);
                if bstr_ptr.is_null() {
                    return -2147024882; // E_OUTOFMEMORY
                }

                // Set length prefix (at -4 offset)
                *(bstr_ptr as *mut u32).offset(-1) = len as u32;

                // Copy data
                let data_ptr = bstr_ptr as *mut u16;
                ptr::copy_nonoverlapping(name_utf16.as_ptr(), data_ptr, len);
                *data_ptr.add(len) = 0; // null terminator

                (*value).vt = crate::ffi::VARENUM::VT_BSTR as u16;
                (*value).data.as_mut_ptr().cast::<*mut u16>().write(data_ptr);

                return 0; // S_OK
            }
        }

        // For other properties, return empty
        *value = mem::zeroed();
        0 // S_OK
    }
    
    unsafe extern "system" fn get_stream(
        this: *mut crate::ffi::IArchiveUpdateCallback,
        index: u32,
        in_stream: *mut *mut ISequentialInStream,
    ) -> HRESULT {
        let callback = &*(this as *const UpdateCallback);

        if index >= callback.items.len() as u32 {
            return -2147467259; // E_INVALIDARG
        }

        let item = &callback.items[index as usize];
        
        match item.operation {
            UpdateOperation::Add => {
                if let Some(ref path) = item.path {
                    match FileStream::new(path) {
                        Ok(stream) => {
                            let stream_ptr = Box::leak(Box::new(stream));
                            *in_stream = stream_ptr as *mut FileStream as *mut ISequentialInStream;
                            0 // S_OK
                        }
                        Err(_) => -2147467259, // E_INVALIDARG
                    }
                } else {
                    -2147467259 // E_INVALIDARG
                }
            }
            _ => {
                *in_stream = ptr::null_mut();
                0 // S_OK
            }
        }
    }
    
    unsafe extern "system" fn set_operation_result(
        _this: *mut crate::ffi::IArchiveUpdateCallback,
        _result_e_operation_result: i32,
    ) -> HRESULT {
        0 // S_OK
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_item_creation() {
        let keep = UpdateItem::keep(0);
        assert_eq!(keep.operation, UpdateOperation::Keep);
        assert_eq!(keep.index, Some(0));
        
        let add = UpdateItem::add("test.txt");
        assert_eq!(add.operation, UpdateOperation::Add);
        assert!(add.path.is_some());
        
        let delete = UpdateItem::delete(1);
        assert_eq!(delete.operation, UpdateOperation::Delete);
        assert_eq!(delete.index, Some(1));
        
        let rename = UpdateItem::rename(2, "newname.txt".to_string());
        assert_eq!(rename.operation, UpdateOperation::Rename);
        assert_eq!(rename.index, Some(2));
        assert_eq!(rename.new_name, Some("newname.txt".to_string()));
    }
}

