//! 枚举 7-Zip 支持的所有格式
//!
//! 使用 BitLibrary API 来枚举所有可用的压缩格式

use bit7z_rust::BitLibrary;
use bit7z_rust::ffi::{
    GUID, IInArchive, IUnknown,
    CLSID_CFormat7z, CLSID_CFormatZip, CLSID_CFormatGZip,
    CLSID_CFormatBZip2, CLSID_CFormatRar, CLSID_CFormatRar5,
    CLSID_CFormatTar, CLSID_CFormatWim, CLSID_CFormatXz,
};
use std::ffi::c_void;

// 所有支持的格式 CLSID 列表
const FORMAT_CLSIDS: &[(&str, &GUID)] = &[
    ("7z", &CLSID_CFormat7z),
    ("ZIP", &CLSID_CFormatZip),
    ("GZip", &CLSID_CFormatGZip),
    ("BZip2", &CLSID_CFormatBZip2),
    ("RAR", &CLSID_CFormatRar),
    ("RAR5", &CLSID_CFormatRar5),
    ("TAR", &CLSID_CFormatTar),
    ("WIM", &CLSID_CFormatWim),
    ("XZ", &CLSID_CFormatXz),
];

fn main() {
    println!("=== 7-Zip 格式枚举 ===\n");

    let lib = match BitLibrary::new(Some("/usr/lib/7zip/7z.so")) {
        Ok(lib) => {
            println!("✓ 7-Zip 库加载成功\n");
            lib
        },
        Err(e) => {
            println!("✗ 7-Zip 库加载失败：{:?}", e);
            return;
        }
    };

    println!("测试 CreateObject 创建所有格式:\n");
    
    for (name, clsid) in FORMAT_CLSIDS {
        unsafe {
            let result = lib.create_in_archive(clsid);
            match result {
                Ok(archive_nonnull) => {
                    println!("  ✓ {:6} - CreateObject 成功", name);
                    
                    // 释放对象
                    let archive_ptr = archive_nonnull.as_ptr() as *mut c_void;
                    let archive = &*archive_nonnull.as_ptr();
                    let vtable = &*archive.vtable;
                    (vtable.base.release)(archive_ptr as *mut IUnknown);
                },
                Err(e) => {
                    println!("  ✗ {:6} - CreateObject 失败：{:?}", name, e);
                }
            }
        }
    }

    println!("\n=== 枚举完成 ===");
}
