use bit7z_rust::{BitLibrary, BitExtractor, ExtractFormat};
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== TAR 格式解压测试 (调试版) ===\n");

    let lib = match BitLibrary::new(Some("/usr/lib/7zip/7z.so")) {
        Ok(l) => {
            println!("✓ 库加载成功");
            l
        }
        Err(e) => {
            eprintln!("✗ 库加载失败：{}", e);
            return Err(e.into());
        }
    };

    let archive_path = "test_debug/test.tar";
    let extract_dir = "test_debug/extracted_tar_debug";

    println!("档案路径：{}", archive_path);
    println!("输出目录：{}\n", extract_dir);

    // 检查文件是否存在
    if !Path::new(archive_path).exists() {
        eprintln!("✗ 档案文件不存在：{}", archive_path);
        return Err("Archive file not found".into());
    }

    // 检查文件大小
    let metadata = fs::metadata(archive_path)?;
    println!("档案大小：{} 字节\n", metadata.len());

    let extractor = BitExtractor::new(&lib, ExtractFormat::Tar);

    println!("开始解压...");
    match extractor.extract(archive_path, extract_dir) {
        Ok(_) => {
            println!("✓ TAR 解压成功");
            let extracted_file = format!("{}/test.txt", extract_dir);
            if Path::new(&extracted_file).exists() {
                println!("✓ 文件验证成功");
                if let Ok(content) = fs::read_to_string(&extracted_file) {
                    println!("✓ 文件内容：{}", content.trim());
                }
            }
        }
        Err(e) => {
            eprintln!("✗ 解压失败：{}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
