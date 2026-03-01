use bit7z_rust::{BitLibrary, BitExtractor, ExtractFormat};
use std::path::Path;

/// 测试单个格式
fn test_format(
    lib: &BitLibrary,
    format: ExtractFormat,
    format_name: &str,
    archive_path: &str,
    output_dir: &str,
) -> bool {
    println!("\n---------- 测试 {} 格式 ----------", format_name);
    println!("测试文件：{}", archive_path);
    println!("输出目录：{}\n", output_dir);

    // 检查测试文件是否存在
    if !Path::new(archive_path).exists() {
        println!("⚠ 测试文件不存在：{}", archive_path);
        return false;
    }

    let extractor = BitExtractor::new(lib, format);

    match extractor.extract(archive_path, output_dir) {
        Ok(_) => {
            println!("✓ {} 解压成功！", format_name);
            true
        }
        Err(e) => {
            eprintln!("✗ {} 解压失败：{}", format_name, e);
            false
        }
    }
}

fn main() {
    println!("╔════════════════════════════════════════════╗");
    println!("║     bit7z-rust 格式兼容性测试工具          ║");
    println!("╚════════════════════════════════════════════╝\n");

    // 加载 7-Zip 库
    let lib = match BitLibrary::new(Some("/usr/lib/7zip/7z.so")) {
        Ok(l) => {
            println!("✓ 7-Zip 库加载成功\n");
            l
        }
        Err(e) => {
            eprintln!("✗ 库加载失败：{}", e);
            eprintln!("提示：请确保已安装 7-Zip 并找到正确的库文件路径");
            return;
        }
    };

    let mut total_tests = 0;
    let mut passed_tests = 0;

    // ========== 压缩/解压格式测试 ==========
    println!("\n【压缩/解压格式测试】");
    println!("═══════════════════════════════════════\n");

    let compression_formats = vec![
        ("7z", ExtractFormat::SevenZip, "test_debug/test.7z", "test_debug/extracted_7z"),
        ("ZIP", ExtractFormat::Zip, "test_debug/test.zip", "test_debug/extracted_zip"),
        ("GZIP", ExtractFormat::GZip, "test_debug/test.gz", "test_debug/extracted_gz"),
        ("BZIP2", ExtractFormat::BZip2, "test_debug/test.bz2", "test_debug/extracted_bz2"),
        ("TAR", ExtractFormat::Tar, "test_debug/test.tar", "test_debug/extracted_tar"),
        ("XZ", ExtractFormat::Xz, "test_debug/test.xz", "test_debug/extracted_xz"),
    ];

    for (name, format, archive, output) in compression_formats {
        total_tests += 1;
        if test_format(&lib, format, name, archive, output) {
            passed_tests += 1;
        }
    }

    // ========== 只读格式测试 ==========
    println!("\n\n【只读格式测试】");
    println!("═══════════════════════════════════════\n");

    let readonly_formats = vec![
        ("RAR", ExtractFormat::Rar, "test_debug/test.rar", "test_debug/extracted_rar"),
        ("RAR5", ExtractFormat::Rar5, "test_debug/test.rar5", "test_debug/extracted_rar5"),
        ("CAB", ExtractFormat::Cab, "test_debug/test.cab", "test_debug/extracted_cab"),
        ("ARJ", ExtractFormat::Arj, "test_debug/test.arj", "test_debug/extracted_arj"),
        ("LZH", ExtractFormat::Lzh, "test_debug/test.lzh", "test_debug/extracted_lzh"),
        ("ISO", ExtractFormat::Iso, "test_debug/test.iso", "test_debug/extracted_iso"),
        ("CPIO", ExtractFormat::Cpio, "test_debug/test.cpio", "test_debug/extracted_cpio"),
        ("RPM", ExtractFormat::Rpm, "test_debug/test.rpm", "test_debug/extracted_rpm"),
        ("DEB", ExtractFormat::Deb, "test_debug/test.deb", "test_debug/extracted_deb"),
    ];

    for (name, format, archive, output) in readonly_formats {
        total_tests += 1;
        if test_format(&lib, format, name, archive, output) {
            passed_tests += 1;
        }
    }

    // ========== 测试结果汇总 ==========
    println!("\n\n╔════════════════════════════════════════════╗");
    println!("║           测试结果汇总                     ║");
    println!("╚════════════════════════════════════════════╝");
    println!("总测试数：{}", total_tests);
    println!("通过数：{}", passed_tests);
    println!("失败数：{}", total_tests - passed_tests);
    
    if total_tests > 0 {
        let success_rate = (passed_tests as f64 / total_tests as f64) * 100.0;
        println!("成功率：{:.1}%", success_rate);
    }

    println!("\n提示：如果测试失败，请检查：");
    println!("  1. 测试文件是否存在于 test_debug/ 目录");
    println!("  2. 7-Zip 库路径是否正确");
    println!("  3. 是否有足够的权限访问文件");
}
