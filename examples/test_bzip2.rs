use bit7z_rust::{BitLibrary, BitExtractor, ExtractFormat};
use std::fs;

fn main() {
    println!("=== BZip2 格式解压测试 ===\n");

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

    let archive_path = "test_debug/test.bz2";
    let extract_dir = "test_debug/extracted_bz2";
    
    println!("档案路径：{}", archive_path);
    println!("输出目录：{}\n", extract_dir);

    let extractor = BitExtractor::new(&lib, ExtractFormat::BZip2);
    
    match extractor.extract(archive_path, extract_dir) {
        Ok(_) => {
            println!("✓ BZip2 解压成功");
            if let Ok(entries) = fs::read_dir(extract_dir) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        println!("  解压文件：{:?}", path.file_name());
                        if let Ok(content) = fs::read_to_string(&path) {
                            println!("  文件内容：{}", content.trim());
                        }
                    }
                }
            }
        }
        Err(e) => eprintln!("✗ 解压失败：{}", e),
    }
}
