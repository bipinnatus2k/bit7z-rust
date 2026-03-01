use bit7z_rust::{BitLibrary, BitCompressor, CompressionFormat};
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 空压缩测试 ===\n");

    let lib = BitLibrary::new(None::<&str>)?;
    println!("✓ 库加载成功\n");

    let test_dir = Path::new("test_debug");
    if !test_dir.exists() {
        fs::create_dir_all(test_dir)?;
    }

    let mut compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);
    let output_file = test_dir.join("empty.7z");
    
    // 尝试压缩 0 个项目
    println!("尝试压缩 0 个项目...");
    match compressor.compress::<&Path>(&[], &output_file) {
        Ok(_) => {
            println!("✓ 空压缩成功!");
            if output_file.exists() {
                println!("✓ 输出文件大小：{} 字节", fs::metadata(&output_file)?.len());
            }
        },
        Err(e) => println!("✗ 空压缩失败：{}", e),
    }

    Ok(())
}
