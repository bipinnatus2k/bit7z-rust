//! 完整压缩测试 - 测试实际文件的压缩

use bit7z_rust::{BitLibrary, BitCompressor, BitExtractor, CompressionFormat, CompressionLevel, ExtractFormat};
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 完整压缩测试 ===\n");
    
    // 加载 7-Zip 库
    let lib = BitLibrary::new(Some("/usr/lib/7zip/7z.so"))?;
    println!("✓ 7-Zip 库加载成功");
    
    // 创建测试文件
    let test_dir = Path::new("test_full_compress");
    if !test_dir.exists() {
        fs::create_dir_all(test_dir)?;
    }
    
    let test_file1 = test_dir.join("file1.txt");
    let test_file2 = test_dir.join("file2.txt");
    
    fs::write(&test_file1, "Hello, World! This is file 1.")?;
    fs::write(&test_file2, "This is file 2 with different content.")?;
    
    println!("✓ 测试文件创建完成");
    println!("  - {:?}", test_file1);
    println!("  - {:?}", test_file2);
    
    // 创建压缩器
    let mut compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);
    compressor.compression_level(CompressionLevel::Normal);
    
    let output_archive = test_dir.join("output.7z");
    
    println!("\n=== 开始压缩 ===");
    println!("输出档案：{:?}", output_archive);
    
    // 压缩文件
    match compressor.compress(&[&test_file1, &test_file2], &output_archive) {
        Ok(_) => {
            println!("✓ 压缩成功!");
            
            // 验证档案大小
            if let Ok(metadata) = fs::metadata(&output_archive) {
                println!("  档案大小：{} 字节", metadata.len());
            }
        }
        Err(e) => {
            println!("✗ 压缩失败：{}", e);
        }
    }
    
    // 尝试解压验证
    if output_archive.exists() {
        println!("\n=== 开始解压验证 ===");
        
        let extract_dir = test_dir.join("extracted");
        fs::create_dir_all(&extract_dir)?;
        
        let extractor = BitExtractor::new(&lib, ExtractFormat::SevenZip);
        match extractor.extract(&output_archive, &extract_dir) {
            Ok(_) => {
                println!("✓ 解压成功!");
                
                // 验证解压的文件
                let extracted_file1 = extract_dir.join("file1.txt");
                let extracted_file2 = extract_dir.join("file2.txt");
                
                if extracted_file1.exists() {
                    let content = fs::read_to_string(&extracted_file1)?;
                    println!("  file1.txt 内容：{}", content);
                }
                
                if extracted_file2.exists() {
                    let content = fs::read_to_string(&extracted_file2)?;
                    println!("  file2.txt 内容：{}", content);
                }
            }
            Err(e) => {
                println!("✗ 解压失败：{}", e);
            }
        }
    }
    
    // 清理
    println!("\n=== 清理测试文件 ===");
    let _ = fs::remove_dir_all(test_dir);
    println!("✓ 测试完成");
    
    Ok(())
}
