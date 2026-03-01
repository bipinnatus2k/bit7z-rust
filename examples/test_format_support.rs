use bit7z_rust::{BitLibrary, ExtractFormat};

fn main() {
    println!("=== 7-Zip 格式支持诊断 ===\n");

    let lib = match BitLibrary::new(Some("/usr/lib/7zip/7z.so")) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("✗ 库加载失败：{}", e);
            return;
        }
    };
    println!("✓ 库加载成功\n");

    let formats = vec![
        // 基础格式
        ("7z", ExtractFormat::SevenZip),
        ("ZIP", ExtractFormat::Zip),
        ("GZip", ExtractFormat::GZip),
        ("BZip2", ExtractFormat::BZip2),
        ("TAR", ExtractFormat::Tar),
        ("XZ", ExtractFormat::Xz),
        ("WIM", ExtractFormat::Wim),
        ("LZMA", ExtractFormat::Lzma),
        // 只读压缩格式
        ("RAR", ExtractFormat::Rar),
        ("RAR5", ExtractFormat::Rar5),
        ("ARJ", ExtractFormat::Arj),
        ("LZH", ExtractFormat::Lzh),
        ("CAB", ExtractFormat::Cab),
        ("NSIS", ExtractFormat::Nsis),
        ("ISO", ExtractFormat::Iso),
        ("UDF", ExtractFormat::Udf),
        ("CHM", ExtractFormat::Chm),
        ("SPLIT", ExtractFormat::Split),
        ("RPM", ExtractFormat::Rpm),
        ("DEB", ExtractFormat::Deb),
        ("CPIO", ExtractFormat::Cpio),
        ("Z", ExtractFormat::Z),
        // 磁盘镜像格式
        ("DMG", ExtractFormat::Dmg),
        ("Ext", ExtractFormat::Ext),
        ("Fat", ExtractFormat::Fat),
        ("HFS", ExtractFormat::Hfs),
        ("NTFS", ExtractFormat::Ntfs),
        ("QCOW", ExtractFormat::Qcow),
        ("VDI", ExtractFormat::Vdi),
        ("VHD", ExtractFormat::Vhd),
        ("VHDX", ExtractFormat::Vhdx),
        ("VMDK", ExtractFormat::Vmdk),
        ("CramFS", ExtractFormat::Cramfs),
        ("SquashFS", ExtractFormat::Squashfs),
        ("APFS", ExtractFormat::Apfs),
        // 可执行文件格式
        ("ELF", ExtractFormat::Elf),
        ("MachO", ExtractFormat::Macho),
        ("PE", ExtractFormat::Pe),
        // 固件格式
        ("UEFIc", ExtractFormat::Uefic),
        ("UEFIf", ExtractFormat::Uefif),
        ("TE", ExtractFormat::Te),
        // 分区格式
        ("GPT", ExtractFormat::Gpt),
        ("MBR", ExtractFormat::Mbr),
        ("APM", ExtractFormat::Apm),
        // 其他格式
        ("Xar", ExtractFormat::Xar),
        ("Ar", ExtractFormat::Ar),
        ("Compound", ExtractFormat::Compound),
        ("Base64", ExtractFormat::Base64),
        ("COFF", ExtractFormat::Coff),
        ("IHex", ExtractFormat::IHex),
        ("Mub", ExtractFormat::Mub),
        ("LP", ExtractFormat::LP),
        ("Hxs", ExtractFormat::Hxs),
    ];

    println!("{:<12} {:<20} {}", "格式", "GUID", "状态");
    println!("{}", "-".repeat(66));

    for (name, format) in formats {
        let guid = format.guid();
        unsafe {
            let result = lib.create_in_archive(&guid);
            match result {
                Ok(_) => println!("{:<12} {:<20} ✓ 可用", name, format_guid(&guid)),
                Err(e) => println!("{:<12} {:<20} ✗ 失败：{}", name, format_guid(&guid), e),
            }
        }
    }
}

fn format_guid(guid: &bit7z_rust::GUID) -> String {
    format!(
        "{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
        guid.data1,
        guid.data2,
        guid.data3,
        guid.data4[0],
        guid.data4[1],
        guid.data4[2],
        guid.data4[3],
        guid.data4[4],
        guid.data4[5],
        guid.data4[6],
        guid.data4[7],
    )
}
