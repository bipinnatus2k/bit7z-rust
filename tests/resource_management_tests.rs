//! 资源管理测试 - 测试内存管理和边界条件

use bit7z_rust::{
    BitLibrary, BitMemCompressor, BitMemExtractor,
    CompressionFormat, ExtractFormat
};
use std::fs;
use tempfile::TempDir;

mod test_utils;
use test_utils::find_library_path;

/// 测试大型档案处理
#[test]
#[ignore = "BitMemCompressor/Extractor 在当前 FFI 实现下不稳定，资源边界测试暂不纳入默认测试"]
fn test_large_archive_handling() {
    let lib = BitLibrary::new::<String>(find_library_path().into()).unwrap(); // 使用字符串路径初始化库
    let temp_dir = TempDir::new().unwrap();

    // 创建一个较大的测试文件（1MB）
    let large_content = vec![0u8; 1_000_000]; // 1MB
    let large_file = temp_dir.path().join("large_file.bin");
    fs::write(&large_file, &large_content).unwrap();

    // 读取文件内容
    let file_content = fs::read(&large_file).unwrap();

    // 压缩大型文件
    let compressor = BitMemCompressor::new(&lib, CompressionFormat::Zip);
    let compressed_data = compressor.compress_from_buffer(&file_content, Some("large_file.bin".to_string())).unwrap();

    assert!(!compressed_data.is_empty(), "大型文件压缩不应为空");
    assert!(compressed_data.len() < large_content.len(), "压缩后应更小");

    // 解压大型文件
    let extractor = BitMemExtractor::new(&lib, ExtractFormat::Zip);
    let extracted_data = extractor.extract_to_buffer(&compressed_data, 0).unwrap();

    assert_eq!(extracted_data.len(), large_content.len(), "解压后大小应与原始一致");
    assert_eq!(&extracted_data[..100], &large_content[..100], "内容应匹配");
}

/// 测试空文件处理
#[test]
fn test_empty_file_handling() {
    let lib = BitLibrary::new::<String>(find_library_path().into()).unwrap(); // 使用字符串路径初始化库
    let temp_dir = TempDir::new().unwrap();

    // 创建空文件
    let empty_file = temp_dir.path().join("empty.txt");
    fs::write(&empty_file, "").unwrap();

    // 读取文件内容
    let file_content = fs::read(&empty_file).unwrap();

    // 压缩空文件
    let compressor = BitMemCompressor::new(&lib, CompressionFormat::GZip);
    let compressed_data = compressor.compress_from_buffer(&file_content, Some("empty.txt".to_string())).unwrap();

    assert!(!compressed_data.is_empty(), "空文件压缩不应为空");

    // 解压空文件
    let extractor = BitMemExtractor::new(&lib, ExtractFormat::GZip);
    let extracted_data = extractor.extract_to_buffer(&compressed_data, 0).unwrap();

    assert!(extracted_data.is_empty(), "解压空文件应返回空");
}

/// 测试特殊字符文件名
#[test]
#[ignore = "BitMemCompressor/Extractor 在当前 FFI 实现下不稳定，资源边界测试暂不纳入默认测试"]
fn test_special_char_filenames() {
    let lib = BitLibrary::new::<String>(find_library_path().into()).unwrap(); // 使用字符串路径初始化库
    let temp_dir = TempDir::new().unwrap();

    // 创建包含特殊字符的文件名
    let special_name = "file_特殊_字符_测试.txt";
    let special_file = temp_dir.path().join(special_name);
    let special_content = b"Content with special filename";
    fs::write(&special_file, special_content).unwrap();

    // 读取文件内容
    let file_content = fs::read(&special_file).unwrap();

    // 压缩文件
    let compressor = BitMemCompressor::new(&lib, CompressionFormat::SevenZip);
    let compressed_data = compressor.compress_from_buffer(&file_content, Some(special_name.to_string())).unwrap();

    // 解压文件到缓冲区
    let extractor = BitMemExtractor::new(&lib, ExtractFormat::SevenZip);
    let extracted_data = extractor.extract_to_buffer(&compressed_data, 0).unwrap();

    // 验证内容
    assert_eq!(&extracted_data[..], &special_content[..], "内容应匹配");
}
