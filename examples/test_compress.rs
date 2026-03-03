//! Test example for compression functionality

use bit7z_rust::{BitLibrary, BitCompressor, CompressionFormat, CompressionLevel};
use std::fs;
use std::io::Write;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("=== BitCompressor 测试 ===\n");
    let _ = std::io::stderr().flush();

    // 加载 7-Zip 库
    let lib = BitLibrary::new(None::<&str>)?;
    eprintln!("✓ 7-Zip 库加载成功\n");
    let _ = std::io::stderr().flush();

    // 创建测试文件
    let test_dir = Path::new("test_compression");
    if test_dir.exists() {
        fs::remove_dir_all(test_dir)?;
    }
    fs::create_dir_all(test_dir)?;

    let test_file1 = test_dir.join("test1.txt");
    let test_file2 = test_dir.join("test2.txt");
    
    fs::write(&test_file1, "Hello, World! This is test file 1.")?;
    fs::write(&test_file2, "Hello again! This is test file 2.")?;
    
    println!("✓ 创建测试文件完成\n");

    // 测试 1: 压缩单个文件到 7z 格式
    println!("测试 1: 压缩单个文件到 7z 格式");
    {
        let mut compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);
        compressor.compression_level(CompressionLevel::Normal);
        
        let output_file = test_dir.join("output.7z");
        compressor.compress(&[&test_file1], &output_file)?;
        
        if output_file.exists() {
            let size = fs::metadata(&output_file)?.len();
            println!("  ✓ 压缩成功：{:?} ({} 字节)\n", output_file, size);
        } else {
            println!("  ✗ 压缩失败：输出文件不存在\n");
        }
    }

    // 测试 2: 压缩多个文件到 ZIP 格式
    println!("测试 2: 压缩多个文件到 ZIP 格式");
    {
        let mut compressor = BitCompressor::new(&lib, CompressionFormat::Zip);
        compressor.compression_level(CompressionLevel::Max);
        
        let output_file = test_dir.join("output.zip");
        compressor.compress(&[&test_file1, &test_file2], &output_file)?;
        
        if output_file.exists() {
            let size = fs::metadata(&output_file)?.len();
            println!("  ✓ 压缩成功：{:?} ({} 字节)\n", output_file, size);
        } else {
            println!("  ✗ 压缩失败：输出文件不存在\n");
        }
    }

    // 测试 3: 压缩到内存缓冲区
    println!("测试 3: 压缩到内存缓冲区");
    {
        let mut compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);
        
        let buffer = compressor.compress_to_buffer(&[&test_file1])?;
        println!("  ✓ 压缩成功：缓冲区大小 {} 字节\n", buffer.len());
    }

    // 测试 4: 使用密码加密压缩
    println!("测试 4: 使用密码加密压缩 (7z 格式)");
    {
        let mut compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);
        compressor.password("test123");
        compressor.compression_level(CompressionLevel::Ultra);
        
        let output_file = test_dir.join("output_encrypted.7z");
        compressor.compress(&[&test_file1], &output_file)?;
        
        if output_file.exists() {
            let size = fs::metadata(&output_file)?.len();
            println!("  ✓ 加密压缩成功：{:?} ({} 字节)\n", output_file, size);
        } else {
            println!("  ✗ 压缩失败：输出文件不存在\n");
        }
    }

    // 测试 5: 不同压缩级别比较
    println!("测试 5: 不同压缩级别比较");
    {
        let levels = [
            (CompressionLevel::Fastest, "Fastest"),
            (CompressionLevel::Normal, "Normal"),
            (CompressionLevel::Max, "Max"),
            (CompressionLevel::Ultra, "Ultra"),
        ];

        for (level, name) in levels.iter() {
            let mut compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);
            compressor.compression_level(*level);
            
            let output_file = test_dir.join(format!("output_{}.7z", name.to_lowercase()));
            compressor.compress(&[&test_file1, &test_file2], &output_file)?;
            
            if output_file.exists() {
                let size = fs::metadata(&output_file)?.len();
                println!("  ✓ {}: {} 字节", name, size);
            }
        }
        println!();
    }

    // 清理测试文件
    println!("清理测试文件...");
    fs::remove_dir_all(test_dir)?;
    println!("✓ 清理完成\n");

    println!("=== 所有测试完成 ===");
    Ok(())
}
