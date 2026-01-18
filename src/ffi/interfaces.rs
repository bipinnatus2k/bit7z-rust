//! COM interface definitions for 7-Zip
//! 
//! This module provides low-level COM interface vtable definitions.

use crate::ffi::{GUID, HRESULT, ULONG, ULONGLONG, PROPVARIANT, PROPID};
use std::ffi::c_void;

// IUnknown - base COM interface
#[repr(C)]
pub struct IUnknownVTable {
    pub query_interface: unsafe extern "system" fn(
        this: *mut IUnknown,
        iid: *const GUID,
        out: *mut *mut c_void,
    ) -> HRESULT,
    pub add_ref: unsafe extern "system" fn(this: *mut IUnknown) -> ULONG,
    pub release: unsafe extern "system" fn(this: *mut IUnknown) -> ULONG,
}

#[repr(C)]
pub struct IUnknown {
    pub vtable: *const IUnknownVTable,
}

// ISequentialInStream
#[repr(C)]
pub struct ISequentialInStreamVTable {
    pub base: IUnknownVTable,
    pub read: unsafe extern "system" fn(
        this: *mut ISequentialInStream,
        data: *mut c_void,
        size: usize,
        processed_size: *mut usize,
    ) -> HRESULT,
}

#[repr(C)]
pub struct ISequentialInStream {
    pub vtable: *const ISequentialInStreamVTable,
}

// ISequentialOutStream
#[repr(C)]
pub struct ISequentialOutStreamVTable {
    pub base: IUnknownVTable,
    pub write: unsafe extern "system" fn(
        this: *mut ISequentialOutStream,
        data: *const c_void,
        size: usize,
        processed_size: *mut usize,
    ) -> HRESULT,
}

#[repr(C)]
pub struct ISequentialOutStream {
    pub vtable: *const ISequentialOutStreamVTable,
}

// IInStream - extends ISequentialInStream
#[repr(C)]
pub struct IInStreamVTable {
    pub base: ISequentialInStreamVTable,
    pub seek: unsafe extern "system" fn(
        this: *mut IInStream,
        offset: i64,
        seek_origin: u32,
        new_position: *mut u64,
    ) -> HRESULT,
}

#[repr(C)]
pub struct IInStream {
    pub vtable: *const IInStreamVTable,
}

// IOutStream - extends ISequentialOutStream
#[repr(C)]
pub struct IOutStreamVTable {
    pub base: ISequentialOutStreamVTable,
    pub seek: unsafe extern "system" fn(
        this: *mut IOutStream,
        offset: i64,
        seek_origin: u32,
        new_position: *mut u64,
    ) -> HRESULT,
    pub set_size: unsafe extern "system" fn(
        this: *mut IOutStream,
        new_size: u64,
    ) -> HRESULT,
}

#[repr(C)]
pub struct IOutStream {
    pub vtable: *const IOutStreamVTable,
}

// IInArchive
#[repr(C)]
pub struct IInArchiveVTable {
    pub base: IUnknownVTable,
    pub open: unsafe extern "system" fn(
        this: *mut IInArchive,
        stream: *mut IInStream,
        max_check_start_position: u64,
        open_callback: *mut IArchiveOpenCallback,
    ) -> HRESULT,
    pub close: unsafe extern "system" fn(this: *mut IInArchive) -> HRESULT,
    pub get_number_of_items: unsafe extern "system" fn(
        this: *mut IInArchive,
        num_items: *mut u32,
    ) -> HRESULT,
    pub get_property: unsafe extern "system" fn(
        this: *mut IInArchive,
        index: u32,
        prop_id: PROPID,
        value: *mut PROPVARIANT,
    ) -> HRESULT,
    pub extract: unsafe extern "system" fn(
        this: *mut IInArchive,
        indices: *const u32,
        num_items: u32,
        test_mode: i32,
        extract_callback: *mut IArchiveExtractCallback,
    ) -> HRESULT,
    pub get_archive_property: unsafe extern "system" fn(
        this: *mut IInArchive,
        prop_id: PROPID,
        value: *mut PROPVARIANT,
    ) -> HRESULT,
    pub get_number_of_properties: unsafe extern "system" fn(
        this: *mut IInArchive,
        num_properties: *mut u32,
    ) -> HRESULT,
    pub get_property_info: unsafe extern "system" fn(
        this: *mut IInArchive,
        index: u32,
        name: *mut *const u16,
        prop_id: *mut PROPID,
        var_type: *mut u32,
    ) -> HRESULT,
}

#[repr(C)]
pub struct IInArchive {
    pub vtable: *const IInArchiveVTable,
}

// IOutArchive
#[repr(C)]
pub struct IOutArchiveVTable {
    pub base: IUnknownVTable,
    pub update_items: unsafe extern "system" fn(
        this: *mut IOutArchive,
        out_stream: *mut IOutStream,
        num_items: u32,
        item_data: *const *const c_void,
        update_callback: *mut IArchiveUpdateCallback,
    ) -> HRESULT,
    pub get_file_time_type: unsafe extern "system" fn(
        this: *mut IOutArchive,
        type_: *mut u32,
    ) -> HRESULT,
}

#[repr(C)]
pub struct IOutArchive {
    pub vtable: *const IOutArchiveVTable,
}

// IArchiveOpenCallback
#[repr(C)]
pub struct IArchiveOpenCallbackVTable {
    pub base: IUnknownVTable,
    pub set_completed: unsafe extern "system" fn(
        this: *mut IArchiveOpenCallback,
        files: *const u64,
        bytes: *const u64,
    ) -> HRESULT,
    pub set_total: unsafe extern "system" fn(
        this: *mut IArchiveOpenCallback,
        files: *const u64,
        bytes: *const u64,
    ) -> HRESULT,
}

#[repr(C)]
pub struct IArchiveOpenCallback {
    pub vtable: *const IArchiveOpenCallbackVTable,
}

// IArchiveExtractCallback
#[repr(C)]
pub struct IArchiveExtractCallbackVTable {
    pub base: IUnknownVTable,
    pub set_completed: unsafe extern "system" fn(
        this: *mut IArchiveExtractCallback,
        complete_value: *const u64,
    ) -> HRESULT,
    pub set_total: unsafe extern "system" fn(
        this: *mut IArchiveExtractCallback,
        total: u64,
    ) -> HRESULT,
    pub get_stream: unsafe extern "system" fn(
        this: *mut IArchiveExtractCallback,
        index: u32,
        out_stream: *mut *mut ISequentialOutStream,
        ask_extract_mode: *mut i32,
    ) -> HRESULT,
    pub prepare_operation: unsafe extern "system" fn(
        this: *mut IArchiveExtractCallback,
        ask_extract_mode: i32,
    ) -> HRESULT,
    pub set_operation_result: unsafe extern "system" fn(
        this: *mut IArchiveExtractCallback,
        result_e_operation_result: i32,
    ) -> HRESULT,
}

#[repr(C)]
pub struct IArchiveExtractCallback {
    pub vtable: *const IArchiveExtractCallbackVTable,
}

// IArchiveUpdateCallback
#[repr(C)]
pub struct IArchiveUpdateCallbackVTable {
    pub base: IUnknownVTable,
    pub set_total: unsafe extern "system" fn(
        this: *mut IArchiveUpdateCallback,
        size: u64,
    ) -> HRESULT,
    pub set_completed: unsafe extern "system" fn(
        this: *mut IArchiveUpdateCallback,
        complete_value: *const u64,
    ) -> HRESULT,
    pub get_update_item_info: unsafe extern "system" fn(
        this: *mut IArchiveUpdateCallback,
        index: u32,
        new_data: *mut i32,
        new_properties: *mut i32,
        index_in_archive: *mut u32,
    ) -> HRESULT,
    pub get_property: unsafe extern "system" fn(
        this: *mut IArchiveUpdateCallback,
        index: u32,
        prop_id: PROPID,
        value: *mut PROPVARIANT,
    ) -> HRESULT,
    pub get_stream: unsafe extern "system" fn(
        this: *mut IArchiveUpdateCallback,
        index: u32,
        in_stream: *mut *mut ISequentialInStream,
    ) -> HRESULT,
    pub set_operation_result: unsafe extern "system" fn(
        this: *mut IArchiveUpdateCallback,
        result_e_operation_result: i32,
    ) -> HRESULT,
}

#[repr(C)]
pub struct IArchiveUpdateCallback {
    pub vtable: *const IArchiveUpdateCallbackVTable,
}

// IProgress
#[repr(C)]
pub struct IProgressVTable {
    pub base: IUnknownVTable,
    pub set_completed: unsafe extern "system" fn(
        this: *mut IProgress,
        complete_value: *const u64,
    ) -> HRESULT,
    pub set_total: unsafe extern "system" fn(
        this: *mut IProgress,
        total: u64,
    ) -> HRESULT,
}

#[repr(C)]
pub struct IProgress {
    pub vtable: *const IProgressVTable,
}

// ICryptoGetTextPassword
#[repr(C)]
pub struct ICryptoGetTextPasswordVTable {
    pub base: IUnknownVTable,
    pub crypto_get_text_password: unsafe extern "system" fn(
        this: *mut ICryptoGetTextPassword,
        password: *mut *mut u16,
    ) -> HRESULT,
}

#[repr(C)]
pub struct ICryptoGetTextPassword {
    pub vtable: *const ICryptoGetTextPasswordVTable,
}

// Seek origin constants
pub const SEEK_SET: u32 = 0;
pub const SEEK_CUR: u32 = 1;
pub const SEEK_END: u32 = 2;