//! 扩展测试 - 测试所有新增功能

use bit7z_rust::{
    BitLibrary, BitMemCompressor, BitMemExtractor,
    CompressionFormat, ExtractFormat,
};
use std::fs;
use tempfile::TempDir;

/// 测试 BitMemCompressor 和 BitMemExtractor 基本功能
#[test]
fn test_mem_compressor_extractor() {
    let lib = BitLibrary::new::<String>(None).unwrap();
    let temp_dir = TempDir::new().unwrap();

    // 创建测试文件
    let test_content = b"Test content for memory compression";
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, test_content).unwrap();

    // 读取文件内容
    let file_content = fs::read(&test_file).unwrap();

    // 压缩到内存
    let compressor = BitMemCompressor::new(&lib, CompressionFormat::SevenZip);
    let compressed_data = compressor.compress_from_buffer(&file_content, Some("test.txt".to_string()));
    
    match compressed_data {
        Ok(data) => {
            assert!(!data.is_empty());
            
            // 从内存解压
            let extractor = BitMemExtractor::new(&lib, ExtractFormat::SevenZip);
            let extracted_data = extractor.extract_to_buffer(&data, 0);
            
            match extracted_data {
                Ok(extracted) => assert_eq!(extracted, test_content),
                Err(e) => panic!("解压失败：{:?}", e),
            }
        }
        Err(e) => panic!("压缩失败：{:?}", e),
    }
}

/// 测试不同格式的内存压缩
#[test]
fn test_mem_formats() {
    let lib = BitLibrary::new::<String>(None).unwrap();
    let test_content = b"Format test content";

    let formats = [
        (CompressionFormat::SevenZip, ExtractFormat::SevenZip),
        (CompressionFormat::Zip, ExtractFormat::Zip),
    ];

    for (comp_format, ext_format) in formats.iter() {
        let compressor = BitMemCompressor::new(&lib, *comp_format);
        let compressed_data = compressor.compress_from_buffer(test_content, Some("test.bin".to_string()));
        
        match compressed_data {
            Ok(data) => {
                let extractor = BitMemExtractor::new(&lib, *ext_format);
                let extracted_data = extractor.extract_to_buffer(&data, 0);
                
                match extracted_data {
                    Ok(extracted) => assert_eq!(extracted, test_content, "格式 {:?} 压缩/解压失败", comp_format),
                    Err(e) => panic!("格式 {:?} 解压失败：{:?}", comp_format, e),
                }
            }
            Err(e) => panic!("格式 {:?} 压缩失败：{:?}", comp_format, e),
        }
    }
}

/// 测试压缩级别
#[test]
fn test_compression_levels() {
    let lib = BitLibrary::new::<String>(None).unwrap();
    let test_content = vec![0u8; 100_000]; // 100KB 可压缩数据

    let levels = [
        bit7z_rust::CompressionLevel::Fastest,
        bit7z_rust::CompressionLevel::Normal,
        bit7z_rust::CompressionLevel::Max,
        bit7z_rust::CompressionLevel::Ultra,
    ];

    let mut prev_size = 0;
    for level in levels.iter() {
        let mut compressor = BitMemCompressor::new(&lib, CompressionFormat::SevenZip);
        compressor.compression_level(*level);
        let compressed_data = compressor.compress_from_buffer(&test_content, Some("test.bin".to_string()));
        
        match compressed_data {
            Ok(data) => {
                // 更高级别应该有更好的压缩率（或至少相同）
                if prev_size > 0 {
                    assert!(data.len() <= prev_size, "压缩级别 {:?} 应该提供更好的压缩率", level);
                }
                prev_size = data.len();
            }
            Err(e) => panic!("压缩级别 {:?} 压缩失败：{:?}", level, e),
        }
    }
}
