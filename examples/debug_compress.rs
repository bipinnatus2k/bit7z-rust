use bit7z_rust::{BitLibrary, BitCompressor, CompressionFormat, CompressionLevel};
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 压缩调试测试 ===\n");

    let lib = BitLibrary::new(None::<&str>)?;
    println!("✓ 库加载成功\n");

    // 创建测试文件
    let test_dir = Path::new("test_debug");
    if !test_dir.exists() {
        fs::create_dir_all(test_dir)?;
    }
    
    let test_file = test_dir.join("test.txt");
    fs::write(&test_file, "Hello, World!")?;
    println!("✓ 测试文件创建成功\n");

    // 尝试压缩
    let mut compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);
    compressor.compression_level(CompressionLevel::Normal);
    
    let output_file = test_dir.join("output.7z");
    println!("尝试压缩到 {:?}", output_file);
    
    match compressor.compress(&[&test_file], &output_file) {
        Ok(_) => {
            println!("✓ 压缩成功!");
            if output_file.exists() {
                println!("✓ 输出文件大小：{} 字节", fs::metadata(&output_file)?.len());
            }
        },
        Err(e) => {
            println!("✗ 压缩失败：{}", e);
        }
    }
    
    Ok(())
}
