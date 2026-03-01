use bit7z_rust::{BitLibrary, BitExtractor, ExtractFormat};

fn main() {
    println!("=== 其他只读格式解压测试 ===\n");

    // 加载 7-Zip 库
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

    // 测试各种只读格式
    let test_cases = vec![
        ("test_debug/test.cab", ExtractFormat::Cab, "test_debug/extracted_cab", "CAB"),
        ("test_debug/test.arj", ExtractFormat::Arj, "test_debug/extracted_arj", "ARJ"),
        ("test_debug/test.iso", ExtractFormat::Iso, "test_debug/extracted_iso", "ISO"),
        ("test_debug/test.lzh", ExtractFormat::Lzh, "test_debug/extracted_lzh", "LZH"),
        ("test_debug/test.cpio", ExtractFormat::Cpio, "test_debug/extracted_cpio", "CPIO"),
        ("test_debug/test.rpm", ExtractFormat::Rpm, "test_debug/extracted_rpm", "RPM"),
        ("test_debug/test.deb", ExtractFormat::Deb, "test_debug/extracted_deb", "DEB"),
    ];

    for (archive_path, format, output_dir, format_name) in test_cases {
        println!("\n========== 测试 {} 格式 ==========", format_name);
        println!("测试文件：{}", archive_path);
        println!("输出目录：{}\n", output_dir);

        let extractor = BitExtractor::new(&lib, format);

        match extractor.extract(archive_path, output_dir) {
            Ok(_) => {
                println!("✓ {} 解压成功！", format_name);
            }
            Err(e) => {
                eprintln!("✗ {} 解压失败：{}", format_name, e);
            }
        }
        println!();
    }
}
