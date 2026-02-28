use std::ffi::c_void;
use std::mem;
use std::ptr;

#[repr(C)]
struct GUID {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

#[repr(C)]
union PropVariantData {
    data: [u8; 16],
    ulVal: u32,
    bstrVal: *mut u16,
}

#[repr(C)]
struct PROPVARIANT {
    vt: u16,
    wReserved1: u16,
    wReserved2: u16,
    wReserved3: u16,
    data: PropVariantData,
}

impl PROPVARIANT {
    fn clear(&mut self) {
        if self.vt == 8 {  // VT_BSTR
            unsafe {
                if !self.data.bstrVal.is_null() {
                    let len_ptr = (self.data.bstrVal as *const u8).offset(-4) as *const u32;
                    let byte_len = *len_ptr as usize;
                    libc::free(self.data.bstrVal as *mut c_void);
                }
            }
        }
        self.vt = 0;
        self.data.data = [0; 16];
    }
}

const VT_BSTR: u16 = 8;
const VT_UI4: u16 = 19;

const IID_IInArchive: GUID = GUID::from_raw(
    0x23170F69, 0x40C1, 0x278A,
    [0x00, 0x00, 0x00, 0x06, 0x00, 0x60, 0x00, 0x00]
);

impl GUID {
    const fn from_raw(d1: u32, d2: u16, d3: u16, d4: [u8; 8]) -> Self {
        GUID { data1: d1, data2: d2, data3: d3, data4: d4 }
    }
}

#[repr(C)]
struct IInArchiveVTable {
    query_interface: unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> i32,
    add_ref: unsafe extern "system" fn(*mut c_void) -> u32,
    release: unsafe extern "system" fn(*mut c_void) -> u32,
    open: unsafe extern "system" fn(*mut c_void, *mut c_void, *const u64, *mut c_void) -> i32,
    close: unsafe extern "system" fn(*mut c_void) -> i32,
    get_number_of_items: unsafe extern "system" fn(*mut c_void, *mut u32) -> i32,
    get_number_of_properties: unsafe extern "system" fn(*mut c_void, *mut u32) -> i32,
    get_property_info: unsafe extern "system" fn(*mut c_void, u32, *mut *const u16, *mut u32, *mut u16) -> i32,
    get_property: unsafe extern "system" fn(*mut c_void, u32, u32, *mut PROPVARIANT) -> i32,
    extract: unsafe extern "system" fn(*mut c_void, *const u32, u32, i32, *mut c_void) -> i32,
    get_archive_property: unsafe extern "system" fn(*mut c_void, u32, *mut PROPVARIANT) -> i32,
    get_number_of_format_properties: unsafe extern "system" fn(*mut c_void, *mut u32) -> i32,
    get_format_property_info: unsafe extern "system" fn(*mut c_void, u32, *mut *const u16, *mut u32) -> i32,
}

#[repr(C)]
struct IInArchive {
    vtable: *const IInArchiveVTable,
}

extern "C" {
    fn CreateObject(clsid: *const GUID, iid: *const GUID, out_object: *mut *mut c_void) -> i32;
    fn GetNumberOfFormats(num_formats: *mut u32) -> i32;
    fn GetHandlerProperty2(format_index: u32, prop_id: u32, value: *mut PROPVARIANT) -> i32;
}

fn main() {
    println!("=== 7-Zip 格式枚举 ===\n");

    unsafe {
        // 方法 1: 使用 GetNumberOfFormats
        let mut num_formats: u32 = 0;
        let result = GetNumberOfFormats(&mut num_formats);
        println!("GetNumberOfFormats: 0x{:08X}, 数量：{}\n", result, num_formats);

        for i in 0..num_formats {
            let mut name_prop = PROPVARIANT {
                vt: 0,
                wReserved1: 0,
                wReserved2: 0,
                wReserved3: 0,
                data: PropVariantData { data: [0; 16] },
            };

            // kpidName = 0
            let result = GetHandlerProperty2(i, 0, &mut name_prop);
            if result == 0 && name_prop.vt == VT_BSTR {
                let name = bstr_to_string(name_prop.data.bstrVal);
                name_prop.clear();

                // kpidClassID = 1
                let mut clsid_prop = PROPVARIANT {
                    vt: 0,
                    wReserved1: 0,
                    wReserved2: 0,
                    wReserved3: 0,
                    data: PropVariantData { data: [0; 16] },
                };
                let result = GetHandlerProperty2(i, 1, &mut clsid_prop);
                let clsid_str = if result == 0 && clsid_prop.vt == VT_BSTR {
                    bstr_to_string(clsid_prop.data.bstrVal)
                } else {
                    String::new()
                };
                clsid_prop.clear();

                // kpidExtension = 4
                let mut ext_prop = PROPVARIANT {
                    vt: 0,
                    wReserved1: 0,
                    wReserved2: 0,
                    wReserved3: 0,
                    data: PropVariantData { data: [0; 16] },
                };
                let result = GetHandlerProperty2(i, 4, &mut ext_prop);
                let ext = if result == 0 && ext_prop.vt == VT_BSTR {
                    bstr_to_string(ext_prop.data.bstrVal)
                } else {
                    String::new()
                };
                ext_prop.clear();

                println!("  格式 {:2}: name={:10}, ext={:20}, classid={}", i, name, ext, clsid_str);
            }
        }
    }
}

fn bstr_to_string(bstr: *mut u16) -> String {
    if bstr.is_null() {
        return String::new();
    }
    unsafe {
        // 读取 BSTR 长度（前 4 字节）
        let len_ptr = (bstr as *const u8).offset(-4) as *const u32;
        let byte_len = *len_ptr as usize;
        let char_len = byte_len / 2;
        
        let slice = std::slice::from_raw_parts(bstr, char_len);
        String::from_utf16_lossy(slice)
    }
}
