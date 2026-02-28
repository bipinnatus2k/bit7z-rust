use std::ffi::c_void;
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

const VT_BSTR: u16 = 8;
const VT_UI4: u16 = 19;

extern "C" {
    fn GetNumberOfFormats(num_formats: *mut u32) -> i32;
    fn GetHandlerProperty2(format_index: u32, prop_id: u32, value: *mut PROPVARIANT) -> i32;
}

fn main() {
    println!("=== 7-Zip 格式列表 ===\n");

    unsafe {
        let mut num_formats: u32 = 0;
        let result = GetNumberOfFormats(&mut num_formats);
        println!("GetNumberOfFormats 返回：0x{:08X}, 格式数量：{}", result, num_formats);

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
                let name_ptr = name_prop.data.bstrVal;
                if !name_ptr.is_null() {
                    // 读取 BSTR 长度（前 4 字节）
                    let len_ptr = (name_ptr as *const u8).offset(-4) as *const u32;
                    let byte_len = *len_ptr as usize;
                    let char_len = byte_len / 2;
                    
                    let name_slice = std::slice::from_raw_parts(name_ptr, char_len);
                    let name = String::from_utf16_lossy(name_slice);
                    
                    // kpidExtension = 4
                    let mut ext_prop = PROPVARIANT {
                        vt: 0,
                        wReserved1: 0,
                        wReserved2: 0,
                        wReserved3: 0,
                        data: PropVariantData { data: [0; 16] },
                    };
                    GetHandlerProperty2(i, 4, &mut ext_prop);
                    
                    let ext = if ext_prop.vt == VT_BSTR && !ext_prop.data.bstrVal.is_null() {
                        let ext_ptr = ext_prop.data.bstrVal;
                        let len_ptr = (ext_ptr as *const u8).offset(-4) as *const u32;
                        let byte_len = *len_ptr as usize;
                        let char_len = byte_len / 2;
                        let ext_slice = std::slice::from_raw_parts(ext_ptr, char_len);
                        String::from_utf16_lossy(ext_slice)
                    } else {
                        String::new()
                    };

                    println!("  格式 {}: {} ({})", i, name, ext);
                }
            }
        }
    }
}
