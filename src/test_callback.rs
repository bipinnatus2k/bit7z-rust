//! 简化的 UpdateCallback 实现用于测试

use crate::ffi::{
    IArchiveUpdateCallback, IArchiveUpdateCallbackVTable,
    ISequentialInStream, IUnknown, IUnknownVTable,
    IProgress, IProgressVTable,
    PROPVARIANT, PROPID, HRESULT, ULONG,
    IID_IUnknown, IID_IArchiveUpdateCallback, IID_IProgress,
};
use crate::ffi::variant::alloc_bstr_from_utf32;
use crate::stream::FileStream;
use std::cell::UnsafeCell;
use std::ffi::c_void;
use std::path::PathBuf;
use std::ptr;

/// 简化的输入项
#[derive(Clone)]
pub struct SimpleInputItem {
    pub path: PathBuf,
    pub name_in_archive: Option<String>,
}

impl SimpleInputItem {
    pub fn new<P: AsRef<PathBuf>>(path: P) -> Self {
        SimpleInputItem {
            path: path.as_ref().clone(),
            name_in_archive: None,
        }
    }
}

/// 简化的 UpdateCallback
#[repr(C)]
pub struct SimpleUpdateCallback {
    vtable: *const IArchiveUpdateCallbackVTable,
    ref_count: UnsafeCell<u32>,
    items: Vec<SimpleInputItem>,
}

static mut VTABLE: Option<IArchiveUpdateCallbackVTable> = None;

fn init_vtable() {
    unsafe {
        if VTABLE.is_none() {
            VTABLE = Some(IArchiveUpdateCallbackVTable {
                base: IProgressVTable {
                    base: IUnknownVTable {
                        query_interface: SimpleUpdateCallback::query_interface,
                        add_ref: SimpleUpdateCallback::add_ref,
                        release: SimpleUpdateCallback::release,
                    },
                    set_completed: SimpleUpdateCallback::set_completed,
                    set_total: SimpleUpdateCallback::set_total,
                },
                get_update_item_info: SimpleUpdateCallback::get_update_item_info,
                get_property: SimpleUpdateCallback::get_property,
                get_stream: SimpleUpdateCallback::get_stream,
                set_operation_result: SimpleUpdateCallback::set_operation_result,
            });
        }
    }
}

impl SimpleUpdateCallback {
    pub fn new(items: Vec<SimpleInputItem>) -> Self {
        init_vtable();
        
        SimpleUpdateCallback {
            vtable: unsafe { VTABLE.as_ref().unwrap() as *const _ },
            ref_count: UnsafeCell::new(1),
            items,
        }
    }
    
    pub fn as_i_archive_update_callback(&self) -> *mut IArchiveUpdateCallback {
        self as *const SimpleUpdateCallback as *mut SimpleUpdateCallback as *mut IArchiveUpdateCallback
    }
    
    unsafe fn from_callback(this: *mut IArchiveUpdateCallback) -> *mut SimpleUpdateCallback {
        this as *mut SimpleUpdateCallback
    }
    
    unsafe extern "system" fn query_interface(
        this: *mut IUnknown,
        iid: *const crate::ffi::GUID,
        out: *mut *mut c_void,
    ) -> HRESULT {
        if out.is_null() || iid.is_null() {
            return -2147467261; // E_POINTER
        }
        
        let callback = this as *mut SimpleUpdateCallback;
        *out = ptr::null_mut();
        
        let iid_iunknown = IID_IUnknown;
        let iid_progress = IID_IProgress;
        let iid_update = IID_IArchiveUpdateCallback;
        
        if *iid == iid_iunknown {
            *out = callback as *mut c_void;
        } else if *iid == iid_progress {
            *out = callback as *mut c_void;
        } else if *iid == iid_update {
            *out = callback as *mut c_void;
        } else {
            return -2147467262; // E_NOINTERFACE
        }
        
        Self::add_ref(this);
        0 // S_OK
    }
    
    unsafe extern "system" fn add_ref(this: *mut IUnknown) -> ULONG {
        let callback = this as *mut SimpleUpdateCallback;
        let count = (*callback).ref_count.get().read();
        (*callback).ref_count.get().write(count + 1);
        count + 1
    }
    
    unsafe extern "system" fn release(this: *mut IUnknown) -> ULONG {
        let callback = this as *mut SimpleUpdateCallback;
        let count = (*callback).ref_count.get().read();
        if count > 1 {
            (*callback).ref_count.get().write(count - 1);
            count - 1
        } else {
            (*callback).ref_count.get().write(0);
            let _ = Box::from_raw(callback);
            0
        }
    }
    
    unsafe extern "system" fn set_total(
        _this: *mut IProgress,
        _size: u64,
    ) -> HRESULT {
        0 // S_OK
    }
    
    unsafe extern "system" fn set_completed(
        _this: *mut IProgress,
        _complete_value: *const u64,
    ) -> HRESULT {
        0 // S_OK
    }
    
    unsafe extern "system" fn get_update_item_info(
        this: *mut IArchiveUpdateCallback,
        index: u32,
        new_data: *mut i32,
        new_properties: *mut i32,
        index_in_archive: *mut u32,
    ) -> HRESULT {
        let callback = Self::from_callback(this);
        let items = &(*callback).items;
        
        if index as usize >= items.len() {
            return -2147467259; // E_FAIL
        }
        
        if !new_data.is_null() {
            *new_data = 1;
        }
        if !new_properties.is_null() {
            *new_properties = 1;
        }
        if !index_in_archive.is_null() {
            *index_in_archive = 0xFFFFFFFF; // -1
        }
        
        0 // S_OK
    }
    
    unsafe extern "system" fn get_property(
        this: *mut IArchiveUpdateCallback,
        index: u32,
        prop_id: PROPID,
        value: *mut PROPVARIANT,
    ) -> HRESULT {
        if value.is_null() {
            return -2147467261; // E_POINTER
        }
        
        let callback = Self::from_callback(this);
        let items = &(*callback).items;
        
        if index as usize >= items.len() {
            return -2147467259; // E_FAIL
        }
        
        let item = &items[index as usize];
        
        // Initialize PROPVARIANT
        (*value).vt = 0;
        (*value).wReserved1 = 0;
        (*value).wReserved2 = 0;
        (*value).wReserved3 = 0;
        (*value).data = [0; 16];
        
        match prop_id {
            PROPID::IsAnti => {
                (*value).vt = 11; // VT_BOOL
                (*value).data[0] = 0;
            }
            PROPID::Path => {
                let path_str = if let Some(ref name) = item.name_in_archive {
                    name.clone()
                } else {
                    item.path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string()
                };
                
                let bstr = alloc_bstr_from_utf32(&path_str);
                if bstr.is_null() {
                    return -2147467259; // E_FAIL
                }
                
                (*value).vt = 8; // VT_BSTR
                let data_ptr = (*value).data.as_mut_ptr() as *mut *mut u16;
                std::ptr::write_unaligned(data_ptr, bstr);
            }
            PROPID::IsDir => {
                let is_dir = item.path.is_dir();
                (*value).vt = 11;
                (*value).data[0] = if is_dir { 0xFF } else { 0 };
                (*value).data[1] = if is_dir { 0xFF } else { 0 };
            }
            PROPID::Size => {
                if !item.path.is_dir() {
                    if let Ok(metadata) = std::fs::metadata(&item.path) {
                        let size = metadata.len();
                        (*value).vt = 21; // VT_UI8
                        let data_ptr = (*value).data.as_mut_ptr() as *mut u64;
                        std::ptr::write_unaligned(data_ptr, size);
                    }
                }
            }
            PROPID::Attrib => {
                (*value).vt = 19; // VT_UI4
                (*value).data[0] = 0x80; // FILE_ATTRIBUTE_NORMAL
            }
            _ => {}
        }
        
        0 // S_OK
    }
    
    unsafe extern "system" fn get_stream(
        this: *mut IArchiveUpdateCallback,
        index: u32,
        in_stream: *mut *mut ISequentialInStream,
    ) -> HRESULT {
        if in_stream.is_null() {
            return -2147467261; // E_POINTER
        }
        
        let callback = Self::from_callback(this);
        let items = &(*callback).items;
        
        if index as usize >= items.len() {
            *in_stream = ptr::null_mut();
            return -2147467259; // E_FAIL
        }
        
        let item = &items[index as usize];
        
        if item.path.is_dir() {
            *in_stream = ptr::null_mut();
            return 0; // S_OK
        }
        
        match FileStream::new(&item.path) {
            Ok(file_stream) => {
                let pinned = Box::new(file_stream);
                let stream_ptr = Box::into_raw(pinned);
                *in_stream = (*stream_ptr).as_i_in_stream() as *mut ISequentialInStream;
                0 // S_OK
            }
            Err(_) => {
                *in_stream = ptr::null_mut();
                -2147467259 // E_FAIL
            }
        }
    }
    
    unsafe extern "system" fn set_operation_result(
        _this: *mut IArchiveUpdateCallback,
        _result: i32,
    ) -> HRESULT {
        0 // S_OK
    }
}
