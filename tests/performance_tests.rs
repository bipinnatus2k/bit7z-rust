//! 性能测试 - 测试压缩和解压性能

use std::time::Instant;

use bit7z_rust::{
    BitLibrary, BitMemCompressor, BitMemExtractor,
    CompressionFormat, ExtractFormat
};
use std::fs;
use tempfile::TempDir;

/// 测试不同压缩级别的性能
#[test]
#[ignore = "BitMemCompressor/Extractor 在当前 FFI 实现下不稳定，性能压测暂不纳入默认测试"]
fn test_compression_level_performance() {
    let lib = BitLibrary::new::<String>(None).unwrap();
    let temp_dir = TempDir::new().unwrap();

    // 创建测试数据（10MB）
    let test_data = vec![0u8; 10_000_000];
    let test_file = temp_dir.path().join("large_file.bin");
    fs::write(&test_file, &test_data).unwrap();

    // 读取文件到内存
    let file_content = fs::read(&test_file).unwrap();

    // 测试不同压缩级别
    let levels = [
        (bit7z_rust::CompressionLevel::Fastest, "Fastest"),
        (bit7z_rust::CompressionLevel::Normal, "Normal"),
        (bit7z_rust::CompressionLevel::Max, "Max"),
        (bit7z_rust::CompressionLevel::Ultra, "Ultra"),
    ];

    for (level, name) in levels.iter() {
        let start = Instant::now();
        let mut compressor = BitMemCompressor::new(&lib, CompressionFormat::SevenZip);
        compressor.compression_level(*level);
        let compressed_data = compressor.compress_from_buffer(&file_content, Some("large_file.bin".to_string())).unwrap();
        let duration = start.elapsed();

        println!("Compression level {}: {} ms, ratio: {:.2}%",
                 name, duration.as_millis(),
                 (compressed_data.len() as f64 / test_data.len() as f64) * 100.0);

        // 验证解压
        let start = Instant::now();
        let extractor = BitMemExtractor::new(&lib, ExtractFormat::SevenZip);
        let extracted_data = extractor.extract_to_buffer(&compressed_data, 0).unwrap();
        let duration = start.elapsed();

        println!("Extraction for {}: {} ms", name, duration.as_millis());
        assert_eq!(extracted_data.len(), test_data.len());
    }
}

/// 测试不同格式的压缩性能
#[test]
#[ignore = "BitMemCompressor/Extractor 在当前 FFI 实现下不稳定，性能压测暂不纳入默认测试"]
fn test_format_performance() {
    let lib = BitLibrary::new::<String>(None).unwrap();
    let temp_dir = TempDir::new().unwrap();

    // 创建测试数据（5MB）
    let test_data = vec![0u8; 5_000_000];
    let test_file = temp_dir.path().join("test_data.bin");
    fs::write(&test_file, &test_data).unwrap();

    // 读取文件到内存
    let file_content = fs::read(&test_file).unwrap();

    // 测试不同格式
    let formats = [
        (CompressionFormat::SevenZip, "7z"),
        (CompressionFormat::Zip, "ZIP"),
        (CompressionFormat::GZip, "GZip"),
        (CompressionFormat::BZip2, "BZip2"),
        (CompressionFormat::Xz, "XZ"),
    ];

    for (format, name) in formats.iter() {
        if format.info().features.multiple_files || format == &CompressionFormat::Zip {
            let start = Instant::now();
            let compressor = BitMemCompressor::new(&lib, *format);
            let compressed_data = compressor.compress_from_buffer(&file_content, Some("test_data.bin".to_string())).unwrap();
            let duration = start.elapsed();

            println!("Format {}: {} ms, ratio: {:.2}%",
                     name, duration.as_millis(),
                     (compressed_data.len() as f64 / test_data.len() as f64) * 100.0);
        }
    }
}

/// 测试内存操作性能
#[test]
#[ignore = "BitMemCompressor/Extractor 在当前 FFI 实现下不稳定，性能压测暂不纳入默认测试"]
fn test_memory_operation_performance() {
    let lib = BitLibrary::new::<String>(None).unwrap();

    // 创建内存数据（2MB）
    let test_data = vec![0u8; 2_000_000];

    // 测试内存压缩
    let start = Instant::now();
    let compressor = BitMemCompressor::new(&lib, CompressionFormat::SevenZip);
    let compressed_data = compressor.compress_from_buffer(&test_data, Some("memory_data.bin".to_string())).unwrap();
    let compress_time = start.elapsed();

    // 测试内存解压
    let start = Instant::now();
    let extractor = BitMemExtractor::new(&lib, ExtractFormat::SevenZip);
    let extracted_data = extractor.extract_to_buffer(&compressed_data, 0).unwrap();
    let extract_time = start.elapsed();

    println!("Memory compression: {} ms", compress_time.as_millis());
    println!("Memory extraction: {} ms", extract_time.as_millis());
    println!("Compression ratio: {:.2}%",
             (compressed_data.len() as f64 / test_data.len() as f64) * 100.0);

    assert_eq!(extracted_data, test_data);
}
