//! Example: Using advanced extraction features
//!
//! This example demonstrates:
//! - Extract to buffer
//! - Extract to stream
//! - Extract items by index list
//! - Extract matching wildcard pattern
//! - Extract matching regex pattern

use bit7z_rust::{BitLibrary, BitExtractor, ExtractFormat, BitCompressor, CompressionFormat};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load 7-Zip library
    let lib = BitLibrary::new(None::<&str>)?;
    
    // Create a test archive first
    let test_archive = "test_advanced_extract.7z";
    
    // Clean up if exists
    if Path::new(test_archive).exists() {
        std::fs::remove_file(test_archive)?;
    }
    
    println!("=== 创建测试档案 ===");
    
    // Create test archive with some files
    {
        let compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);
        
        // Create some test files
        std::fs::write("test1.txt", "Content of test1")?;
        std::fs::write("test2.txt", "Content of test2")?;
        std::fs::write("data.bin", &[0x00, 0x01, 0x02, 0x03, 0x04])?;
        std::fs::create_dir_all("docs")?;
        std::fs::write("docs/readme.md", "# README")?;
        std::fs::write("docs/guide.md", "# Guide")?;
        
        compressor.compress(&["test1.txt", "test2.txt", "data.bin", "docs"], test_archive)?;
        println!("测试档案已创建：{}", test_archive);
    }
    
    // Create extractor
    let extractor = BitExtractor::new(&lib, ExtractFormat::SevenZip);
    
    println!("\n=== 1. 提取到缓冲区 ===");
    {
        let buffer = extractor.extract_to_buffer(test_archive, 0)?;
        println!("提取项目 0 (test1.txt) 到缓冲区，大小：{} bytes", buffer.len());
        println!("内容：{}", String::from_utf8_lossy(&buffer));
    }
    
    println!("\n=== 2. 按索引列表提取 ===");
    {
        let output_dir = "output_by_indices";
        if Path::new(output_dir).exists() {
            std::fs::remove_dir_all(output_dir)?;
        }
        
        // Extract only items 0 and 1 (test1.txt and test2.txt)
        extractor.extract_items(test_archive, &[0, 1], output_dir)?;
        println!("提取项目 0 和 1 到：{}", output_dir);
        
        // List extracted files
        for entry in std::fs::read_dir(output_dir)? {
            let entry = entry?;
            println!("  - {}", entry.file_name().to_string_lossy());
        }
    }
    
    println!("\n=== 3. 通配符匹配提取 ===");
    {
        let output_dir = "output_wildcard";
        if Path::new(output_dir).exists() {
            std::fs::remove_dir_all(output_dir)?;
        }
        
        // Extract all .txt files
        extractor.extract_matching(test_archive, "*.txt", output_dir)?;
        println!("提取 *.txt 文件到：{}", output_dir);
        
        // List extracted files
        if let Ok(entries) = std::fs::read_dir(output_dir) {
            for entry in entries.flatten() {
                println!("  - {}", entry.file_name().to_string_lossy());
            }
        }
    }
    
    println!("\n=== 4. 正则表达式匹配提取 ===");
    {
        let output_dir = "output_regex";
        if Path::new(output_dir).exists() {
            std::fs::remove_dir_all(output_dir)?;
        }
        
        // Extract files from docs/ directory
        extractor.extract_matching_regex(test_archive, r"^docs/.*\.md$", output_dir)?;
        println!("提取 docs/*.md 文件 (正则：^docs/.*\\.md$) 到：{}", output_dir);
        
        // List extracted files
        if let Ok(entries) = std::fs::read_dir(output_dir) {
            for entry in entries.flatten() {
                println!("  - {}", entry.file_name().to_string_lossy());
            }
        }
    }
    
    println!("\n=== 5. 提取到流 ===");
    {
        let mut output_data = Vec::new();
        extractor.extract_to_stream(test_archive, &mut output_data, 2)?;
        println!("提取项目 2 (data.bin) 到流，大小：{} bytes", output_data.len());
        println!("内容：{:02X?}", output_data);
    }
    
    // Clean up test files
    let _ = std::fs::remove_file(test_archive);
    let _ = std::fs::remove_file("test1.txt");
    let _ = std::fs::remove_file("test2.txt");
    let _ = std::fs::remove_file("data.bin");
    let _ = std::fs::remove_dir_all("docs");
    let _ = std::fs::remove_dir_all("output_by_indices");
    let _ = std::fs::remove_dir_all("output_wildcard");
    let _ = std::fs::remove_dir_all("output_regex");
    
    println!("\n=== 示例完成 ===");
    
    Ok(())
}
