//! Stream implementations for file I/O and memory operations
//!
//! This module provides implementations of 7-Zip stream interfaces
//! for reading from and writing to files, as well as memory buffers.
//!
//! Memory layout is critical for COM interop: the vtable pointer must be
//! at the beginning of the struct to match C++ COM object layout.

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
use std::ptr::{self, write_unaligned};

/// Wrapper for File that implements IInStream
///
/// Memory layout: vtable must be first to match C++ COM object layout
/// Note: We don't implement IStreamGetSize/IStreamGetProps as separate interfaces
/// to avoid COM pointer issues. 7-Zip can work without these interfaces for most formats.
#[repr(C)]
pub struct FileStream {
    vtable: Pin<Box<IInStreamVTable>>,
    ref_count: UnsafeCell<u32>,
    file: File,
}

impl FileStream {
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
            vtable,
            ref_count: UnsafeCell::new(1),
            file,
        })
    }

    pub fn as_i_in_stream(&self) -> *mut IInStream {
        self as *const FileStream as *mut FileStream as *mut IInStream
    }
    
    unsafe extern "system" fn query_interface(
        this: *mut IUnknown,
        iid: *const crate::ffi::GUID,
        out: *mut *mut c_void,
    ) -> crate::ffi::HRESULT {
        if out.is_null() || iid.is_null() {
            return -2147467261; // E_POINTER
        }

        // IID_IUnknown
        let iid_iunknown = crate::ffi::IID_IUnknown;
        if *iid == iid_iunknown {
            *out = this as *mut c_void;
            FileStream::add_ref(this);
            return 0; // S_OK
        }

        // IID_ISequentialInStream
        let iid_sequential_in = crate::ffi::IID_ISequentialInStream;
        if *iid == iid_sequential_in {
            *out = this as *mut c_void;
            FileStream::add_ref(this);
            return 0; // S_OK
        }

        // IID_IInStream
        let iid_in_stream = crate::ffi::IID_IInStream;
        if *iid == iid_in_stream {
            *out = this as *mut c_void;
            FileStream::add_ref(this);
            return 0; // S_OK
        }

        // IStreamGetSize and IStreamGetProps are not supported
        // This matches bit7z's CStdInStream implementation
        *out = ptr::null_mut();
        -2147467262 // E_NOINTERFACE
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
        if count > 1 {
            *ref_count.get() = count - 1;
            count - 1
        } else {
            // Reference count reached 0, free the object
            *ref_count.get() = 0;
            // Drop the stream - this will also drop the Pin<Box<>> vtable and close the file
            let _ = Box::from_raw(stream);
            0
        }
    }
    
    unsafe extern "system" fn read(
        this: *mut ISequentialInStream,
        data: *mut c_void,
        size: u32,
        processed_size: *mut u32,
    ) -> crate::ffi::HRESULT {
        if this.is_null() {
            return -2147467261; // E_POINTER
        }

        if !processed_size.is_null() {
            write_unaligned(processed_size, 0);
        }

        if size == 0 {
            // 7-Zip expects S_OK for zero-byte reads.
            return 0; // S_OK
        }

        if data.is_null() {
            return -2147467261; // E_POINTER
        }

        let stream = this as *mut FileStream;
        let file = &mut (*stream).file;

        let buf = std::slice::from_raw_parts_mut(data as *mut u8, size as usize);
        match file.read(buf) {
            Ok(n) => {
                let n_u32 = n as u32;
                if !processed_size.is_null() {
                    write_unaligned(processed_size, n_u32);
                }
                // Match bit7z/7-Zip stream contract:
                // EOF is reported as processed_size = 0 with S_OK.
                0 // S_OK
            }
            Err(_e) => {
                if !processed_size.is_null() {
                    write_unaligned(processed_size, 0);
                }
                -2147467259 // E_FAIL
            }
        }
    }

    unsafe extern "system" fn seek(
        this: *mut IInStream,
        offset: i64,
        seek_origin: u32,
        new_position: *mut u64,
    ) -> crate::ffi::HRESULT {
        if this.is_null() {
            return -2147467261; // E_POINTER
        }

        let stream = this as *mut FileStream;
        let file = &mut (*stream).file;

        let from = match seek_origin {
            SEEK_SET => SeekFrom::Start(offset as u64),
            SEEK_CUR => SeekFrom::Current(offset),
            SEEK_END => SeekFrom::End(offset),
            _ => return -2147467259,
        };

        match file.seek(from) {
            Ok(pos) => {
                if !new_position.is_null() {
                    write_unaligned(new_position, pos);
                }
                0 // S_OK
            }
            Err(_) => -2147467259, // E_FAIL
        }
    }
}

impl Drop for FileStreamWrite {
    fn drop(&mut self) {
        // Ensure file is flushed and synced before closing
        let _ = self.file.sync_all();
    }
}

/// Wrapper for File that implements IOutStream
///
/// Memory layout: vtable must be first to match C++ COM object layout
#[repr(C)]
pub struct FileStreamWrite {
    vtable: Pin<Box<IOutStreamVTable>>,
    ref_count: UnsafeCell<u32>,
    file: File,
}

impl FileStreamWrite {
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
            vtable,
            ref_count: UnsafeCell::new(1),
            file,
        })
    }
    
    pub fn as_i_out_stream(&self) -> *mut IOutStream {
        self as *const FileStreamWrite as *mut FileStreamWrite as *mut IOutStream
    }
    
    unsafe extern "system" fn query_interface(
        this: *mut IUnknown,
        iid: *const crate::ffi::GUID,
        out: *mut *mut c_void,
    ) -> crate::ffi::HRESULT {
        if out.is_null() || iid.is_null() {
            return -2147467261; // E_POINTER
        }

        let iid_iunknown = crate::ffi::IID_IUnknown;
        let iid_sequential_out = crate::ffi::IID_ISequentialOutStream;
        let iid_out_stream = crate::ffi::IID_IOutStream;

        if *iid == iid_iunknown || *iid == iid_sequential_out || *iid == iid_out_stream {
            *out = this as *mut c_void;
            FileStreamWrite::add_ref(this);
            return 0; // S_OK
        }

        *out = ptr::null_mut();
        -2147467262 // E_NOINTERFACE
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
        if count > 1 {
            *ref_count.get() = count - 1;
            count - 1
        } else {
            // Reference count reached 0, free the object
            *ref_count.get() = 0;
            // Drop the stream - this will also drop the Pin<Box<>> vtable and close the file
            let _ = Box::from_raw(stream);
            0
        }
    }
    
    unsafe extern "system" fn write(
        this: *mut ISequentialOutStream,
        data: *const c_void,
        size: u32,
        processed_size: *mut u32,
    ) -> crate::ffi::HRESULT {
        if this.is_null() {
            return -2147467261; // E_POINTER
        }

        if !processed_size.is_null() {
            write_unaligned(processed_size, 0);
        }

        if size == 0 {
            return 0; // S_OK
        }

        if data.is_null() {
            return -2147467261; // E_POINTER
        }

        eprintln!("[FileStreamWrite::write] size={}, this={:?}", size, this);

        let stream = this as *mut FileStreamWrite;
        let file = &mut (*stream).file;

        let buf = std::slice::from_raw_parts(data as *const u8, size as usize);
        match file.write_all(buf) {
            Ok(_) => {
                eprintln!("[FileStreamWrite::write] write_all succeeded");
                if !processed_size.is_null() {
                    write_unaligned(processed_size, size);
                }
                0 // S_OK
            }
            Err(e) => {
                eprintln!("[FileStreamWrite::write] write_all failed: {}", e);
                if !processed_size.is_null() {
                    write_unaligned(processed_size, 0);
                }
                -2147467259 // E_FAIL
            }
        }
    }

    unsafe extern "system" fn seek(
        this: *mut IOutStream,
        offset: i64,
        seek_origin: u32,
        new_position: *mut u64,
    ) -> crate::ffi::HRESULT {
        if this.is_null() {
            return -2147467261; // E_POINTER
        }

        let stream = this as *mut FileStreamWrite;
        let file = &mut (*stream).file;

        let from = match seek_origin {
            SEEK_SET => SeekFrom::Start(offset as u64),
            SEEK_CUR => SeekFrom::Current(offset),
            SEEK_END => SeekFrom::End(offset),
            _ => return -2147467259,
        };

        match file.seek(from) {
            Ok(pos) => {
                if !new_position.is_null() {
                    write_unaligned(new_position, pos);
                }
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

/// Memory input stream for reading from Vec<u8>
#[repr(C)]
pub struct BufferInStream {
    vtable: Pin<Box<IInStreamVTable>>,
    buffer: Vec<u8>,
    position: UnsafeCell<usize>,
    ref_count: UnsafeCell<u32>,
}

impl BufferInStream {
    pub fn new(buffer: Vec<u8>) -> Self {
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
        
        BufferInStream {
            vtable,
            buffer,
            position: UnsafeCell::new(0),
            ref_count: UnsafeCell::new(1),
        }
    }
    
    pub fn as_i_in_stream(&self) -> *mut IInStream {
        self as *const BufferInStream as *mut BufferInStream as *mut IInStream
    }
    
    unsafe extern "system" fn query_interface(
        this: *mut IUnknown,
        iid: *const crate::ffi::GUID,
        out: *mut *mut c_void,
    ) -> crate::ffi::HRESULT {
        if out.is_null() || iid.is_null() {
            return -2147467261; // E_POINTER
        }

        let iid_iunknown = crate::ffi::IID_IUnknown;
        let iid_sequential_in = crate::ffi::IID_ISequentialInStream;
        let iid_in_stream = crate::ffi::IID_IInStream;

        if *iid == iid_iunknown || *iid == iid_sequential_in || *iid == iid_in_stream {
            *out = this as *mut c_void;
            BufferInStream::add_ref(this);
            return 0; // S_OK
        }

        *out = ptr::null_mut();
        -2147467262 // E_NOINTERFACE
    }

    unsafe extern "system" fn add_ref(this: *mut IUnknown) -> u32 {
        let stream = this as *mut BufferInStream;
        let ref_count = &(*stream).ref_count;
        let count = *ref_count.get();
        *ref_count.get() = count + 1;
        count + 1
    }

    unsafe extern "system" fn release(this: *mut IUnknown) -> u32 {
        let stream = this as *mut BufferInStream;
        let ref_count = &(*stream).ref_count;
        let count = *ref_count.get();
        if count > 1 {
            *ref_count.get() = count - 1;
            count - 1
        } else {
            // Reference count reached 0, free the object
            *ref_count.get() = 0;
            // Drop the stream - this will also drop the Pin<Box<>> vtable and the buffer
            let _ = Box::from_raw(stream);
            0
        }
    }
    
    unsafe extern "system" fn read(
        this: *mut ISequentialInStream,
        data: *mut c_void,
        size: u32,
        processed_size: *mut u32,
    ) -> crate::ffi::HRESULT {
        if this.is_null() {
            return -2147467261; // E_POINTER
        }

        if !processed_size.is_null() {
            write_unaligned(processed_size, 0);
        }

        if size == 0 {
            return 0; // S_OK
        }

        if data.is_null() {
            return -2147467261; // E_POINTER
        }

        let stream = this as *mut BufferInStream;
        let buffer = &(*stream).buffer;
        let position = &(*stream).position;
        
        let pos = *position.get();
        if pos >= buffer.len() {
            return 0; // S_OK, EOF
        }
        
        let available = buffer.len() - pos;
        let to_read = (size as usize).min(available);
        
        let dst = std::slice::from_raw_parts_mut(data as *mut u8, to_read);
        dst.copy_from_slice(&buffer[pos..pos + to_read]);
        
        if !processed_size.is_null() {
            write_unaligned(processed_size, to_read as u32);
        }
        *position.get() = pos + to_read;
        
        0 // S_OK
    }
    
    unsafe extern "system" fn seek(
        this: *mut IInStream,
        offset: i64,
        seek_origin: u32,
        new_position: *mut u64,
    ) -> crate::ffi::HRESULT {
        if this.is_null() {
            return -2147467261; // E_POINTER
        }

        let stream = this as *mut BufferInStream;
        let buffer = &(*stream).buffer;
        let position = &(*stream).position;
        
        let cur = *position.get() as i128;
        let end = buffer.len() as i128;
        let off = offset as i128;
        let new_pos_i128 = match seek_origin {
            SEEK_SET => off,
            SEEK_CUR => cur + off,
            SEEK_END => end + off,
            _ => return -2147467259,
        };

        if new_pos_i128 < 0 || new_pos_i128 > usize::MAX as i128 {
            return -2147467259; // E_FAIL
        }
        let new_pos = new_pos_i128 as u64;
        
        if !new_position.is_null() {
            write_unaligned(new_position, new_pos);
        }
        *position.get() = new_pos as usize;
        0
    }
}

/// Memory output stream for writing to Vec<u8>
#[repr(C)]
pub struct BufferOutStream {
    vtable: Pin<Box<IOutStreamVTable>>,
    buffer: *mut Vec<u8>,
    position: UnsafeCell<usize>,
    ref_count: UnsafeCell<u32>,
}

impl BufferOutStream {
    pub fn new() -> Self {
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
        
        let buffer = Box::leak(Box::new(Vec::new()));
        
        BufferOutStream {
            vtable,
            buffer,
            position: UnsafeCell::new(0),
            ref_count: UnsafeCell::new(1),
        }
    }
    
    pub fn as_i_out_stream(&self) -> *mut IOutStream {
        self as *const BufferOutStream as *mut BufferOutStream as *mut IOutStream
    }
    
    pub unsafe fn get_buffer(&self) -> Vec<u8> {
        (*self.buffer).clone()
    }
    
    unsafe extern "system" fn query_interface(
        this: *mut IUnknown,
        iid: *const crate::ffi::GUID,
        out: *mut *mut c_void,
    ) -> crate::ffi::HRESULT {
        if out.is_null() || iid.is_null() {
            return -2147467261; // E_POINTER
        }

        let iid_iunknown = crate::ffi::IID_IUnknown;
        let iid_sequential_out = crate::ffi::IID_ISequentialOutStream;
        let iid_out_stream = crate::ffi::IID_IOutStream;

        if *iid == iid_iunknown || *iid == iid_sequential_out || *iid == iid_out_stream {
            *out = this as *mut c_void;
            BufferOutStream::add_ref(this);
            return 0; // S_OK
        }

        *out = ptr::null_mut();
        -2147467262 // E_NOINTERFACE
    }

    unsafe extern "system" fn add_ref(this: *mut IUnknown) -> u32 {
        let stream = this as *mut BufferOutStream;
        let ref_count = &(*stream).ref_count;
        let count = *ref_count.get();
        *ref_count.get() = count + 1;
        count + 1
    }

    unsafe extern "system" fn release(this: *mut IUnknown) -> u32 {
        let stream = this as *mut BufferOutStream;
        let ref_count = &(*stream).ref_count;
        let count = *ref_count.get();
        if count > 1 {
            *ref_count.get() = count - 1;
            count - 1
        } else {
            // Reference count reached 0, free the object
            *ref_count.get() = 0;
            // Free the leaked buffer first
            let buffer_ptr = (*stream).buffer;
            let _ = Box::from_raw(buffer_ptr);
            // Drop the stream - this will also drop the Pin<Box<>> vtable
            let _ = Box::from_raw(stream);
            0
        }
    }
    
    unsafe extern "system" fn write(
        this: *mut ISequentialOutStream,
        data: *const c_void,
        size: u32,
        processed_size: *mut u32,
    ) -> crate::ffi::HRESULT {
        if this.is_null() {
            return -2147467261; // E_POINTER
        }

        if !processed_size.is_null() {
            write_unaligned(processed_size, 0);
        }

        if size == 0 {
            return 0; // S_OK
        }

        if data.is_null() {
            return -2147467261; // E_POINTER
        }

        let stream = this as *mut BufferOutStream;
        let buffer = (*stream).buffer;
        let position = &(*stream).position;

        let pos = *position.get();
        let src = std::slice::from_raw_parts(data as *const u8, size as usize);

        let required_len = pos + size as usize;
        if required_len > (*buffer).len() {
            (*buffer).resize(required_len, 0);
        }

        let dst = unsafe { std::slice::from_raw_parts_mut((*buffer).as_mut_ptr().add(pos), size as usize) };
        dst.copy_from_slice(src);
        *position.get() = pos + size as usize;
        if !processed_size.is_null() {
            write_unaligned(processed_size, size);
        }

        0
    }
    
    unsafe extern "system" fn seek(
        this: *mut IOutStream,
        offset: i64,
        seek_origin: u32,
        new_position: *mut u64,
    ) -> crate::ffi::HRESULT {
        if this.is_null() {
            return -2147467261; // E_POINTER
        }

        let stream = this as *mut BufferOutStream;
        let buffer = (*stream).buffer;
        let position = &(*stream).position;

        let cur = *position.get() as i128;
        let end = (*buffer).len() as i128;
        let off = offset as i128;
        let new_pos_i128 = match seek_origin {
            SEEK_SET => off,
            SEEK_CUR => cur + off,
            SEEK_END => end + off,
            _ => return -2147467259,
        };

        if new_pos_i128 < 0 || new_pos_i128 > usize::MAX as i128 {
            return -2147467259; // E_FAIL
        }
        let new_pos = new_pos_i128 as u64;

        if !new_position.is_null() {
            write_unaligned(new_position, new_pos);
        }
        *position.get() = new_pos as usize;
        0
    }
    
    unsafe extern "system" fn set_size(
        this: *mut IOutStream,
        new_size: u64,
    ) -> crate::ffi::HRESULT {
        let stream = this as *mut BufferOutStream;
        let buffer = (*stream).buffer;

        (*buffer).resize(new_size as usize, 0);
        0
    }
}
