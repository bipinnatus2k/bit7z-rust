//! Stream implementations for file I/O
//! 
//! This module provides implementations of 7-Zip stream interfaces
//! for reading from and writing to files.

use crate::ffi::{
    IInStream, IInStreamVTable, IOutStream, IOutStreamVTable,
    ISequentialInStream, ISequentialOutStream, IUnknown, IUnknownVTable,
    SEEK_CUR, SEEK_END, SEEK_SET,
};
use crate::error::Result;
use std::cell::UnsafeCell;
use std::ffi::c_void;
use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::Path;
use std::pin::Pin;

/// Wrapper for File that implements IInStream
pub struct FileStream {
    file: File,
    ref_count: UnsafeCell<u32>,
    vtable: Pin<Box<IInStreamVTable>>,
}

impl FileStream {
    /// Create a new file stream
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path.as_ref())?;
        
        let vtable = Box::pin(IInStreamVTable {
            base: crate::ffi::ISequentialInStreamVTable {
                base: IUnknownVTable {
                    query_interface: Self::query_interface,
                    add_ref: Self::add_ref,
                    release: Self::release,
                },
                read: Self::read,
            },
            seek: Self::seek,
        });
        
        Ok(FileStream {
            file,
            ref_count: UnsafeCell::new(1),
            vtable,
        })
    }
    
    /// Get a pointer to the IInStream interface
    pub fn as_i_in_stream(&self) -> *mut IInStream {
        self as *const FileStream as *mut FileStream as *mut IInStream
    }
    
    // IUnknown methods
    unsafe extern "system" fn query_interface(
        _this: *mut IUnknown,
        _iid: *const crate::ffi::GUID,
        _out: *mut *mut c_void,
    ) -> crate::ffi::HRESULT {
        // Not implemented
        -1 // E_NOINTERFACE
    }
    
    unsafe extern "system" fn add_ref(this: *mut IUnknown) -> u32 {
        let stream = this as *mut FileStream;
        let ref_count = &(*stream).ref_count;
        let count = *ref_count.get();
        *ref_count.get() = count + 1;
        count + 1
    }
    
    unsafe extern "system" fn release(this: *mut IUnknown) -> u32 {
        let stream = this as *mut FileStream;
        let ref_count = &(*stream).ref_count;
        let count = *ref_count.get();
        if count > 0 {
            *ref_count.get() = count - 1;
            count - 1
        } else {
            0
        }
    }
    
    // ISequentialInStream method
    unsafe extern "system" fn read(
        this: *mut ISequentialInStream,
        data: *mut c_void,
        size: usize,
        processed_size: *mut usize,
    ) -> crate::ffi::HRESULT {
        let stream = this as *mut FileStream;
        let mut file = &(*stream).file;
        
        let buf = std::slice::from_raw_parts_mut(data as *mut u8, size);
        match file.read(buf) {
            Ok(n) => {
                *processed_size = n;
                if n == 0 && size > 0 {
                    1 // S_OK, but no more data
                } else {
                    0 // S_OK
                }
            }
            Err(_) => {
                *processed_size = 0;
                -2147467259 // E_FAIL
            }
        }
    }
    
    // IInStream method
    unsafe extern "system" fn seek(
        this: *mut IInStream,
        offset: i64,
        seek_origin: u32,
        new_position: *mut u64,
    ) -> crate::ffi::HRESULT {
        let stream = this as *mut FileStream;
        let mut file = &(*stream).file;
        
        let from = match seek_origin {
            SEEK_SET => SeekFrom::Start(offset as u64),
            SEEK_CUR => SeekFrom::Current(offset),
            SEEK_END => SeekFrom::End(offset),
            _ => return -2147467259, // E_FAIL
        };
        
        match file.seek(from) {
            Ok(pos) => {
                *new_position = pos;
                0 // S_OK
            }
            Err(_) => -2147467259, // E_FAIL
        }
    }
}

/// Wrapper for File that implements IOutStream
pub struct FileStreamWrite {
    file: File,
    ref_count: UnsafeCell<u32>,
    vtable: Pin<Box<IOutStreamVTable>>,
}

impl FileStreamWrite {
    /// Create a new file write stream
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::create(path.as_ref())?;
        
        let vtable = Box::pin(IOutStreamVTable {
            base: crate::ffi::ISequentialOutStreamVTable {
                base: IUnknownVTable {
                    query_interface: Self::query_interface,
                    add_ref: Self::add_ref,
                    release: Self::release,
                },
                write: Self::write,
            },
            seek: Self::seek,
            set_size: Self::set_size,
        });
        
        Ok(FileStreamWrite {
            file,
            ref_count: UnsafeCell::new(1),
            vtable,
        })
    }
    
    /// Get a pointer to the IOutStream interface
    pub fn as_i_out_stream(&self) -> *mut IOutStream {
        self as *const FileStreamWrite as *mut FileStreamWrite as *mut IOutStream
    }
    
    // IUnknown methods
    unsafe extern "system" fn query_interface(
        _this: *mut IUnknown,
        _iid: *const crate::ffi::GUID,
        _out: *mut *mut c_void,
    ) -> crate::ffi::HRESULT {
        -1 // E_NOINTERFACE
    }
    
    unsafe extern "system" fn add_ref(this: *mut IUnknown) -> u32 {
        let stream = this as *mut FileStreamWrite;
        let ref_count = &(*stream).ref_count;
        let count = *ref_count.get();
        *ref_count.get() = count + 1;
        count + 1
    }
    
    unsafe extern "system" fn release(this: *mut IUnknown) -> u32 {
        let stream = this as *mut FileStreamWrite;
        let ref_count = &(*stream).ref_count;
        let count = *ref_count.get();
        if count > 0 {
            *ref_count.get() = count - 1;
            count - 1
        } else {
            0
        }
    }
    
    // ISequentialOutStream method
    unsafe extern "system" fn write(
        this: *mut ISequentialOutStream,
        data: *const c_void,
        size: usize,
        processed_size: *mut usize,
    ) -> crate::ffi::HRESULT {
        let stream = this as *mut FileStreamWrite;
        let mut file = &mut (*stream).file;
        
        let buf = std::slice::from_raw_parts(data as *const u8, size);
        match file.write_all(buf) {
            Ok(_) => {
                *processed_size = size;
                0 // S_OK
            }
            Err(_) => {
                *processed_size = 0;
                -2147467259 // E_FAIL
            }
        }
    }
    
    // IOutStream methods
    unsafe extern "system" fn seek(
        this: *mut IOutStream,
        offset: i64,
        seek_origin: u32,
        new_position: *mut u64,
    ) -> crate::ffi::HRESULT {
        let stream = this as *mut FileStreamWrite;
        let mut file = &mut (*stream).file;
        
        let from = match seek_origin {
            SEEK_SET => SeekFrom::Start(offset as u64),
            SEEK_CUR => SeekFrom::Current(offset),
            SEEK_END => SeekFrom::End(offset),
            _ => return -2147467259,
        };
        
        match file.seek(from) {
            Ok(pos) => {
                *new_position = pos;
                0
            }
            Err(_) => -2147467259,
        }
    }
    
    unsafe extern "system" fn set_size(
        this: *mut IOutStream,
        new_size: u64,
    ) -> crate::ffi::HRESULT {
        let stream = this as *mut FileStreamWrite;
        let file = &(*stream).file;
        
        match file.set_len(new_size) {
            Ok(_) => 0,
            Err(_) => -2147467259,
        }
    }
}