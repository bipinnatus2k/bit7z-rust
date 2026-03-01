//! 测试 CreateObject 直接创建档案

use bit7z_rust::BitLibrary;
use std::ffi::c_void;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== CreateObject 测试 ===\n");

    let lib = BitLibrary::new(Some("/usr/lib/7zip/7z.so"))?;
    println!("✓ 7-Zip 库加载成功");

    // 测试创建 7z 档案对象
    unsafe {
        use bit7z_rust::ffi::{CLSID_CFormat7z, IUnknown, ISetProperties, IID_ISetProperties};

        let archive_result = lib.create_out_archive(&CLSID_CFormat7z);
        
        match archive_result {
            Ok(archive_nonnull) => {
                println!("✓ 档案对象创建成功");
                let archive_ptr = archive_nonnull.as_ptr() as *mut c_void;
                println!("档案指针：{:?}", archive_ptr);

                // 尝试获取 ISetProperties 接口
                let mut set_props_ptr: *mut c_void = std::ptr::null_mut();

                let archive_unknown = archive_ptr as *mut IUnknown;
                let archive = &*archive_nonnull.as_ptr();
                let vtable = &*archive.vtable;

                let qi_result = (vtable.base.query_interface)(
                    archive_unknown,
                    &IID_ISetProperties,
                    &mut set_props_ptr,
                );

                println!("QueryInterface(ISetProperties) 结果：0x{:X}", qi_result);
                println!("ISetProperties 指针：{:?}", set_props_ptr);

                if qi_result == 0 && !set_props_ptr.is_null() {
                    println!("✓ ISetProperties 接口可用");

                    // 释放接口
                    let set_props = set_props_ptr as *mut ISetProperties;
                    let set_props_vtable = &*(*set_props).vtable;
                    (set_props_vtable.base.release)(set_props as *mut IUnknown);
                }

                // 释放档案对象
                (vtable.base.release)(archive_ptr as *mut IUnknown);
            },
            Err(e) => {
                println!("✗ 档案对象创建失败：{:?}", e);
            }
        }
    }

    Ok(())
}
