use bit7z_rust::{BitLibrary, format::ExtractFormat};

fn main() {
    println!("=== 测试格式创建 ===\n");

    let lib = match BitLibrary::new(Some("/usr/lib/7zip/7z.so")) {
        Ok(l) => {
            println!("✓ 库加载成功");
            l
        }
        Err(e) => {
            eprintln!("✗ 库加载失败：{}", e);
            return;
        }
    };

    // 测试各种格式的创建
    let formats = [
        ("7z", ExtractFormat::SevenZip),
        ("TAR", ExtractFormat::Tar),
        ("GZip", ExtractFormat::GZip),
        ("BZip2", ExtractFormat::BZip2),
        ("XZ", ExtractFormat::Xz),
        ("ZIP", ExtractFormat::Zip),
    ];

    for (name, format) in &formats {
        let guid = format.guid();
        println!("{} GUID: {{{:08X}-{:04X}-{:04X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}}}",
                 name,
                 guid.data1, guid.data2, guid.data3,
                 guid.data4[0], guid.data4[1], guid.data4[2], guid.data4[3],
                 guid.data4[4], guid.data4[5], guid.data4[6], guid.data4[7]);
        
        unsafe {
            let result = lib.create_in_archive(&guid);
            match result {
                Ok(_) => println!("  ✓ {} 格式创建成功\n", name),
                Err(e) => println!("  ✗ {} 格式创建失败：{}\n", name, e),
            }
        }
    }
}
