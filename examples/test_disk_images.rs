use bit7z_rust::{BitLibrary, BitExtractor, ExtractFormat};

fn main() {
    println!("╔════════════════════════════════════════════╗");
    println!("║    磁盘镜像和可执行文件格式测试            ║");
    println!("╚════════════════════════════════════════════╝\n");

    // 加载 7-Zip 库
    let lib = match BitLibrary::new(Some("/usr/lib/7zip/7z.so")) {
        Ok(l) => {
            println!("✓ 7-Zip 库加载成功\n");
            l
        }
        Err(e) => {
            eprintln!("✗ 库加载失败：{}", e);
            return;
        }
    };

    // 磁盘镜像格式
    println!("【磁盘镜像格式测试】");
    println!("═══════════════════════════════════════\n");

    let disk_images = vec![
        ("DMG (macOS)", "test_debug/test.dmg", ExtractFormat::Dmg, "test_debug/extracted_dmg"),
        ("Ext (Linux)", "test_debug/test.ext", ExtractFormat::Ext, "test_debug/extracted_ext"),
        ("FAT (Windows)", "test_debug/test.fat", ExtractFormat::Fat, "test_debug/extracted_fat"),
        ("HFS (macOS)", "test_debug/test.hfs", ExtractFormat::Hfs, "test_debug/extracted_hfs"),
        ("NTFS (Windows)", "test_debug/test.ntfs", ExtractFormat::Ntfs, "test_debug/extracted_ntfs"),
        ("QCOW2 (QEMU)", "test_debug/test.qcow2", ExtractFormat::Qcow, "test_debug/extracted_qcow"),
        ("VDI (VirtualBox)", "test_debug/test.vdi", ExtractFormat::Vdi, "test_debug/extracted_vdi"),
        ("VHD (Virtual PC)", "test_debug/test.vhd", ExtractFormat::Vhd, "test_debug/extracted_vhd"),
        ("VHDX (Hyper-V)", "test_debug/test.vhdx", ExtractFormat::Vhdx, "test_debug/extracted_vhdx"),
        ("VMDK (VMware)", "test_debug/test.vmdk", ExtractFormat::Vmdk, "test_debug/extracted_vmdk"),
        ("CramFS", "test_debug/test.cramfs", ExtractFormat::Cramfs, "test_debug/extracted_cramfs"),
        ("SquashFS", "test_debug/test.squashfs", ExtractFormat::Squashfs, "test_debug/extracted_squashfs"),
        ("APFS (Apple)", "test_debug/test.apfs", ExtractFormat::Apfs, "test_debug/extracted_apfs"),
        ("UDF", "test_debug/test.udf", ExtractFormat::Udf, "test_debug/extracted_udf2"),
    ];

    for (name, archive, format, output) in disk_images {
        test_format(&lib, format, name, archive, output);
    }

    // 可执行文件格式
    println!("\n【可执行文件格式测试】");
    println!("═══════════════════════════════════════\n");

    let exec_formats = vec![
        ("ELF (Linux)", "test_debug/test.elf", ExtractFormat::Elf, "test_debug/extracted_elf"),
        ("Mach-O (macOS)", "test_debug/test.macho", ExtractFormat::Macho, "test_debug/extracted_macho"),
        ("PE (Windows)", "test_debug/test.exe", ExtractFormat::Pe, "test_debug/extracted_pe"),
    ];

    for (name, archive, format, output) in exec_formats {
        test_format(&lib, format, name, archive, output);
    }

    // 固件格式
    println!("\n【固件格式测试】");
    println!("═══════════════════════════════════════\n");

    let firmware_formats = vec![
        ("UEFIc", "test_debug/test.scap", ExtractFormat::Uefic, "test_debug/extracted_uefic"),
        ("UEFIf", "test_debug/test.uefif", ExtractFormat::Uefif, "test_debug/extracted_uefif"),
        ("TE", "test_debug/test.te", ExtractFormat::Te, "test_debug/extracted_te"),
    ];

    for (name, archive, format, output) in firmware_formats {
        test_format(&lib, format, name, archive, output);
    }

    // 其他格式
    println!("\n【其他格式测试】");
    println!("═══════════════════════════════════════\n");

    let other_formats = vec![
        ("GPT", "test_debug/test.gpt", ExtractFormat::Gpt, "test_debug/extracted_gpt"),
        ("MBR", "test_debug/test.mbr", ExtractFormat::Mbr, "test_debug/extracted_mbr"),
        ("APM", "test_debug/test.apm", ExtractFormat::Apm, "test_debug/extracted_apm"),
        ("Xar", "test_debug/test.xar", ExtractFormat::Xar, "test_debug/extracted_xar"),
        ("Ar", "test_debug/test.a", ExtractFormat::Ar, "test_debug/extracted_ar"),
        ("Compound (MSI)", "test_debug/test.msi", ExtractFormat::Compound, "test_debug/extracted_msi"),
        ("Base64", "test_debug/test.b64", ExtractFormat::Base64, "test_debug/extracted_b64"),
        ("COFF", "test_debug/test.obj", ExtractFormat::Coff, "test_debug/extracted_coff"),
        ("IHex", "test_debug/test.ihex", ExtractFormat::IHex, "test_debug/extracted_ihex"),
        ("Mub", "test_debug/test.mub", ExtractFormat::Mub, "test_debug/extracted_mub"),
        ("LP", "test_debug/test.lpimg", ExtractFormat::LP, "test_debug/extracted_lp"),
        ("Hxs", "test_debug/test.hxs", ExtractFormat::Hxs, "test_debug/extracted_hxs"),
    ];

    for (name, archive, format, output) in other_formats {
        test_format(&lib, format, name, archive, output);
    }
}

fn test_format(
    lib: &BitLibrary,
    format: ExtractFormat,
    format_name: &str,
    archive_path: &str,
    output_dir: &str,
) {
    println!("---------- 测试 {} ----------", format_name);
    println!("测试文件：{}", archive_path);

    // 检查文件是否存在
    if !std::path::Path::new(archive_path).exists() {
        println!("⚠ 测试文件不存在，跳过\n");
        return;
    }

    println!("输出目录：{}\n", output_dir);

    let extractor = BitExtractor::new(lib, format);

    match extractor.extract(archive_path, output_dir) {
        Ok(_) => {
            println!("✓ {} 解压成功！\n", format_name);
        }
        Err(e) => {
            eprintln!("✗ {} 解压失败：{}\n", format_name, e);
        }
    }
}
