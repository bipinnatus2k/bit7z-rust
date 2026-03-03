//! Stream implementations using vtable crate
//!
//! This module provides vtable-crate-based implementations of 7-Zip stream interfaces:
//! - VTableFileStream - File input stream
//! - VTableFileStreamWrite - File output stream
//! - VTableBufferInStream - Memory buffer input stream
//! - VTableBufferOutStream - Memory buffer output stream

use vtable::*;
use crate::vtable_base::*;
use crate::ffi::{GUID, HRESULT};
use crate::error::Result;
use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};

// ============================================================================
// FileStream - Input File Stream
// ============================================================================

#[vtable]
struct FileStreamVTable {
    // IUnknown
    query_interface: fn(VRef<FileStreamVTable>, &GUID) -> *mut std::ffi::c_void,
    add_ref: fn(VRef<FileStreamVTable>) -> u32,
    release: fn(VRefMut<FileStreamVTable>) -> u32,
    
    // ISequentialInStream
    read: fn(VRefMut<FileStreamVTable>, *mut std::ffi::c_void, u32, *mut u32) -> HRESULT,
    
    // IInStream
    seek: fn(VRefMut<FileStreamVTable>, i64, u32, *mut u64) -> HRESULT,
    
    drop: fn(VRefMut<FileStreamVTable>),
}

/// File input stream using vtable crate
pub struct VTableFileStream {
    file: File,
    ref_count: AtomicU32,
}

impl VTableFileStream {
    /// Create a new FileStream from a file path
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path.as_ref())?;
        Ok(VTableFileStream {
            file,
            ref_count: AtomicU32::new(1),
        })
    }

    /// Get as IInStream pointer
    pub fn as_i_in_stream(&self) -> *mut crate::ffi::IInStream {
        self as *const VTableFileStream as *mut VTableFileStream as *mut crate::ffi::IInStream
    }
}

impl FileStream for VTableFileStream {
    fn query_interface(&self, iid: &GUID) -> *mut std::ffi::c_void {
        let iid_iunknown = crate::ffi::IID_IUnknown;
        let iid_sequential_in = crate::ffi::IID_ISequentialInStream;
        let iid_in_stream = crate::ffi::IID_IInStream;
        
        if *iid == iid_iunknown || *iid == iid_sequential_in || *iid == iid_in_stream {
            return self as *const _ as *mut _;
        }
        std::ptr::null_mut()
    }

    fn add_ref(&self) -> u32 {
        self.ref_count.fetch_add(1, Ordering::SeqCst) + 1
    }

    fn release(&mut self) -> u32 {
        self.ref_count.fetch_sub(1, Ordering::SeqCst) - 1
    }

    fn read(&mut self, data: *mut std::ffi::c_void, size: u32, processed_size: *mut u32) -> HRESULT {
        use std::ptr::write_unaligned;
        
        let buf = unsafe { std::slice::from_raw_parts_mut(data as *mut u8, size as usize) };
        match self.file.read(buf) {
            Ok(n) => {
                let n_u32 = n as u32;
                if !processed_size.is_null() {
                    unsafe { write_unaligned(processed_size, n_u32); }
                }
                if n == 0 && size > 0 {
                    S_FALSE // End of stream
                } else {
                    S_OK
                }
            }
            Err(_) => {
                if !processed_size.is_null() {
                    unsafe { write_unaligned(processed_size, 0); }
                }
                E_FAIL
            }
        }
    }

    fn seek(&mut self, offset: i64, seek_origin: u32, new_position: *mut u64) -> HRESULT {
        use std::ptr::write_unaligned;
        
        let from = match seek_origin {
            SEEK_SET => SeekFrom::Start(offset as u64),
            SEEK_CUR => SeekFrom::Current(offset),
            SEEK_END => SeekFrom::End(offset),
            _ => return E_FAIL,
        };

        match self.file.seek(from) {
            Ok(pos) => {
                if !new_position.is_null() {
                    unsafe { write_unaligned(new_position, pos); }
                }
                S_OK
            }
            Err(_) => E_FAIL,
        }
    }
}

FileStreamVTable_static!(static FILE_STREAM_VT for VTableFileStream);

// ============================================================================
// FileStreamWrite - Output File Stream
// ============================================================================

#[vtable]
struct FileStreamWriteVTable {
    // IUnknown
    query_interface: fn(VRef<FileStreamWriteVTable>, &GUID) -> *mut std::ffi::c_void,
    add_ref: fn(VRef<FileStreamWriteVTable>) -> u32,
    release: fn(VRefMut<FileStreamWriteVTable>) -> u32,
    
    // ISequentialOutStream
    write: fn(VRefMut<FileStreamWriteVTable>, *const std::ffi::c_void, u32, *mut u32) -> HRESULT,
    
    // IOutStream
    seek: fn(VRefMut<FileStreamWriteVTable>, i64, u32, *mut u64) -> HRESULT,
    set_size: fn(VRefMut<FileStreamWriteVTable>, u64) -> HRESULT,
    
    drop: fn(VRefMut<FileStreamWriteVTable>),
}

/// File output stream using vtable crate
pub struct VTableFileStreamWrite {
    file: File,
    ref_count: AtomicU32,
}

impl VTableFileStreamWrite {
    /// Create a new FileStreamWrite from a file path
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::create(path.as_ref())?;
        Ok(VTableFileStreamWrite {
            file,
            ref_count: AtomicU32::new(1),
        })
    }

    /// Get as IOutStream pointer
    pub fn as_i_out_stream(&self) -> *mut crate::ffi::IOutStream {
        self as *const VTableFileStreamWrite as *mut VTableFileStreamWrite as *mut crate::ffi::IOutStream
    }
}

impl FileStreamWrite for VTableFileStreamWrite {
    fn query_interface(&self, iid: &GUID) -> *mut std::ffi::c_void {
        let iid_iunknown = crate::ffi::IID_IUnknown;
        let iid_sequential_out = crate::ffi::IID_ISequentialOutStream;
        let iid_out_stream = crate::ffi::IID_IOutStream;
        
        if *iid == iid_iunknown || *iid == iid_sequential_out || *iid == iid_out_stream {
            return self as *const _ as *mut _;
        }
        std::ptr::null_mut()
    }

    fn add_ref(&self) -> u32 {
        self.ref_count.fetch_add(1, Ordering::SeqCst) + 1
    }

    fn release(&mut self) -> u32 {
        self.ref_count.fetch_sub(1, Ordering::SeqCst) - 1
    }

    fn write(&mut self, data: *const std::ffi::c_void, size: u32, processed_size: *mut u32) -> HRESULT {
        use std::ptr::write_unaligned;
        
        let buf = unsafe { std::slice::from_raw_parts(data as *const u8, size as usize) };
        match self.file.write(buf) {
            Ok(n) => {
                let n_u32 = n as u32;
                if !processed_size.is_null() {
                    unsafe { write_unaligned(processed_size, n_u32); }
                }
                S_OK
            }
            Err(_) => {
                if !processed_size.is_null() {
                    unsafe { write_unaligned(processed_size, 0); }
                }
                E_FAIL
            }
        }
    }

    fn seek(&mut self, offset: i64, seek_origin: u32, new_position: *mut u64) -> HRESULT {
        use std::ptr::write_unaligned;
        
        let from = match seek_origin {
            SEEK_SET => SeekFrom::Start(offset as u64),
            SEEK_CUR => SeekFrom::Current(offset),
            SEEK_END => SeekFrom::End(offset),
            _ => return E_FAIL,
        };

        match self.file.seek(from) {
            Ok(pos) => {
                if !new_position.is_null() {
                    unsafe { write_unaligned(new_position, pos); }
                }
                S_OK
            }
            Err(_) => E_FAIL,
        }
    }

    fn set_size(&mut self, new_size: u64) -> HRESULT {
        match self.file.set_len(new_size) {
            Ok(_) => S_OK,
            Err(_) => E_FAIL,
        }
    }
}

FileStreamWriteVTable_static!(static FILE_STREAM_WRITE_VT for VTableFileStreamWrite);

// ============================================================================
// BufferInStream - Memory Buffer Input Stream
// ============================================================================

#[vtable]
struct BufferInStreamVTable {
    // IUnknown
    query_interface: fn(VRef<BufferInStreamVTable>, &GUID) -> *mut std::ffi::c_void,
    add_ref: fn(VRef<BufferInStreamVTable>) -> u32,
    release: fn(VRefMut<BufferInStreamVTable>) -> u32,
    
    // ISequentialInStream
    read: fn(VRefMut<BufferInStreamVTable>, *mut std::ffi::c_void, u32, *mut u32) -> HRESULT,
    
    // IInStream
    seek: fn(VRefMut<BufferInStreamVTable>, i64, u32, *mut u64) -> HRESULT,
    
    drop: fn(VRefMut<BufferInStreamVTable>),
}

/// Memory buffer input stream using vtable crate
pub struct VTableBufferInStream {
    buffer: Vec<u8>,
    position: usize,
    ref_count: AtomicU32,
}

impl VTableBufferInStream {
    /// Create a new BufferInStream from a buffer
    pub fn new(buffer: Vec<u8>) -> Self {
        VTableBufferInStream {
            buffer,
            position: 0,
            ref_count: AtomicU32::new(1),
        }
    }

    /// Get as IInStream pointer
    pub fn as_i_in_stream(&self) -> *mut crate::ffi::IInStream {
        self as *const VTableBufferInStream as *mut VTableBufferInStream as *mut crate::ffi::IInStream
    }
}

impl BufferInStream for VTableBufferInStream {
    fn query_interface(&self, iid: &GUID) -> *mut std::ffi::c_void {
        let iid_iunknown = crate::ffi::IID_IUnknown;
        let iid_sequential_in = crate::ffi::IID_ISequentialInStream;
        let iid_in_stream = crate::ffi::IID_IInStream;
        
        if *iid == iid_iunknown || *iid == iid_sequential_in || *iid == iid_in_stream {
            return self as *const _ as *mut _;
        }
        std::ptr::null_mut()
    }

    fn add_ref(&self) -> u32 {
        self.ref_count.fetch_add(1, Ordering::SeqCst) + 1
    }

    fn release(&mut self) -> u32 {
        self.ref_count.fetch_sub(1, Ordering::SeqCst) - 1
    }

    fn read(&mut self, data: *mut std::ffi::c_void, size: u32, processed_size: *mut u32) -> HRESULT {
        use std::ptr::write_unaligned;
        
        let pos = self.position;
        if pos >= self.buffer.len() {
            if !processed_size.is_null() {
                unsafe { write_unaligned(processed_size, 0); }
            }
            return S_FALSE; // End of stream
        }

        let available = self.buffer.len() - pos;
        let to_read = std::cmp::min(size as usize, available);
        let buf = unsafe { std::slice::from_raw_parts_mut(data as *mut u8, to_read) };
        buf.copy_from_slice(&self.buffer[pos..pos + to_read]);
        
        self.position += to_read;
        let to_read_u32 = to_read as u32;
        
        if !processed_size.is_null() {
            unsafe { write_unaligned(processed_size, to_read_u32); }
        }
        S_OK
    }

    fn seek(&mut self, offset: i64, seek_origin: u32, new_position: *mut u64) -> HRESULT {
        use std::ptr::write_unaligned;
        
        let buffer_len = self.buffer.len() as i64;
        let new_pos = match seek_origin {
            SEEK_SET => offset,
            SEEK_CUR => self.position as i64 + offset,
            SEEK_END => buffer_len + offset,
            _ => return E_FAIL,
        };

        if new_pos < 0 {
            return E_FAIL;
        }

        self.position = std::cmp::min(new_pos as usize, self.buffer.len());
        
        if !new_position.is_null() {
            unsafe { write_unaligned(new_position, self.position as u64); }
        }
        S_OK
    }
}

BufferInStreamVTable_static!(static BUFFER_IN_STREAM_VT for VTableBufferInStream);

// ============================================================================
// BufferOutStream - Memory Buffer Output Stream
// ============================================================================

#[vtable]
struct BufferOutStreamVTable {
    // IUnknown
    query_interface: fn(VRef<BufferOutStreamVTable>, &GUID) -> *mut std::ffi::c_void,
    add_ref: fn(VRef<BufferOutStreamVTable>) -> u32,
    release: fn(VRefMut<BufferOutStreamVTable>) -> u32,
    
    // ISequentialOutStream
    write: fn(VRefMut<BufferOutStreamVTable>, *const std::ffi::c_void, u32, *mut u32) -> HRESULT,
    
    // IOutStream
    seek: fn(VRefMut<BufferOutStreamVTable>, i64, u32, *mut u64) -> HRESULT,
    set_size: fn(VRefMut<BufferOutStreamVTable>, u64) -> HRESULT,
    
    drop: fn(VRefMut<BufferOutStreamVTable>),
}

/// Memory buffer output stream using vtable crate
pub struct VTableBufferOutStream {
    buffer: Vec<u8>,
    position: usize,
    ref_count: AtomicU32,
}

impl VTableBufferOutStream {
    /// Create a new BufferOutStream
    pub fn new() -> Self {
        VTableBufferOutStream {
            buffer: Vec::new(),
            position: 0,
            ref_count: AtomicU32::new(1),
        }
    }

    /// Get as IOutStream pointer
    pub fn as_i_out_stream(&self) -> *mut crate::ffi::IOutStream {
        self as *const VTableBufferOutStream as *mut VTableBufferOutStream as *mut crate::ffi::IOutStream
    }

    /// Get the internal buffer (consumes the stream)
    pub fn into_buffer(self) -> Vec<u8> {
        self.buffer
    }

    /// Get a reference to the internal buffer
    pub fn get_buffer(&self) -> &Vec<u8> {
        &self.buffer
    }
}

impl Default for VTableBufferOutStream {
    fn default() -> Self {
        Self::new()
    }
}

impl BufferOutStream for VTableBufferOutStream {
    fn query_interface(&self, iid: &GUID) -> *mut std::ffi::c_void {
        let iid_iunknown = crate::ffi::IID_IUnknown;
        let iid_sequential_out = crate::ffi::IID_ISequentialOutStream;
        let iid_out_stream = crate::ffi::IID_IOutStream;
        
        if *iid == iid_iunknown || *iid == iid_sequential_out || *iid == iid_out_stream {
            return self as *const _ as *mut _;
        }
        std::ptr::null_mut()
    }

    fn add_ref(&self) -> u32 {
        self.ref_count.fetch_add(1, Ordering::SeqCst) + 1
    }

    fn release(&mut self) -> u32 {
        self.ref_count.fetch_sub(1, Ordering::SeqCst) - 1
    }

    fn write(&mut self, data: *const std::ffi::c_void, size: u32, processed_size: *mut u32) -> HRESULT {
        use std::ptr::write_unaligned;
        
        let buf = unsafe { std::slice::from_raw_parts(data as *const u8, size as usize) };
        
        // Ensure buffer is large enough
        let end_pos = self.position + buf.len();
        if end_pos > self.buffer.len() {
            self.buffer.resize(end_pos, 0);
        }
        
        self.buffer[self.position..end_pos].copy_from_slice(buf);
        self.position += buf.len();
        
        let written = buf.len() as u32;
        if !processed_size.is_null() {
            unsafe { write_unaligned(processed_size, written); }
        }
        S_OK
    }

    fn seek(&mut self, offset: i64, seek_origin: u32, new_position: *mut u64) -> HRESULT {
        use std::ptr::write_unaligned;
        
        let buffer_len = self.buffer.len() as i64;
        let new_pos = match seek_origin {
            SEEK_SET => offset,
            SEEK_CUR => self.position as i64 + offset,
            SEEK_END => buffer_len + offset,
            _ => return E_FAIL,
        };

        if new_pos < 0 {
            return E_FAIL;
        }

        self.position = new_pos as usize;
        
        if !new_position.is_null() {
            unsafe { write_unaligned(new_position, self.position as u64); }
        }
        S_OK
    }

    fn set_size(&mut self, new_size: u64) -> HRESULT {
        self.buffer.resize(new_size as usize, 0);
        S_OK
    }
}

BufferOutStreamVTable_static!(static BUFFER_OUT_STREAM_VT for VTableBufferOutStream);

// ============================================================================
// Constants (re-export from crate::ffi)
// ============================================================================

const SEEK_SET: u32 = 0;
const SEEK_CUR: u32 = 1;
const SEEK_END: u32 = 2;
