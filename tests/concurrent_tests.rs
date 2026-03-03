//! 并发测试 - 测试多线程环境下的操作

use bit7z_rust::{
    BitLibrary, BitMemCompressor, BitMemExtractor,
    CompressionFormat, ExtractFormat
};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

mod test_utils;
use test_utils::find_library_path;

/// 测试多线程压缩
#[test]
fn test_concurrent_compression() {
    let lib = Arc::new(BitLibrary::new::<String>(find_library_path().into()).unwrap()); // 使用字符串路径初始化库
    let test_data = vec![0u8; 1_000_000]; // 1MB 测试数据
    
    // 创建多个线程进行压缩
    let mut handles = Vec::new();
    
    for i in 0..5 {
        let lib_clone = Arc::clone(&lib);
        let data_clone = test_data.clone();
        
        let handle = thread::spawn(move || {
            let start = Instant::now();
            let compressor = BitMemCompressor::new(&lib_clone, CompressionFormat::SevenZip);
            let compressed_data = compressor.compress_from_buffer(&data_clone, Some("test_data.bin".to_string())).unwrap();
            let duration = start.elapsed();
            
            println!("Thread {}: Compression took {} ms", i, duration.as_millis());
            compressed_data
        });
        
        handles.push(handle);
    }
    
    // 等待所有线程完成
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.join().unwrap());
    }
    
    // 验证所有结果都有效
    for result in results {
        assert!(!result.is_empty());
    }
}

/// 测试多线程解压
#[test]
fn test_concurrent_extraction() {
    let lib = Arc::new(BitLibrary::new::<String>(find_library_path().into()).unwrap()); // 使用字符串路径初始化库
    let test_data = vec![0u8; 1_000_000]; // 1MB 测试数据
    
    // 先压缩数据
    let compressor = BitMemCompressor::new(&lib, CompressionFormat::SevenZip);
    let compressed_data = compressor.compress_from_buffer(&test_data, Some("test_data.bin".to_string())).unwrap();
    
    // 创建多个线程进行解压
    let mut handles = Vec::new();
    
    for i in 0..5 {
        let lib_clone = Arc::clone(&lib);
        let compressed_clone = compressed_data.clone();
        
        let handle = thread::spawn(move || {
            let start = Instant::now();
            let extractor = BitMemExtractor::new(&lib_clone, ExtractFormat::SevenZip);
            let extracted_data = extractor.extract_to_buffer(&compressed_clone, 0).unwrap();
            let duration = start.elapsed();
            
            println!("Thread {}: Extraction took {} ms", i, duration.as_millis());
            extracted_data
        });
        
        handles.push(handle);
    }
    
    // 等待所有线程完成
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.join().unwrap());
    }
    
    // 验证所有结果都正确
    for result in results {
        assert_eq!(result, test_data);
    }
}

/// 测试混合并发操作（压缩和解压同时进行）
#[test]
fn test_mixed_concurrent_operations() {
    let lib = Arc::new(BitLibrary::new::<String>(find_library_path().into()).unwrap()); // 使用字符串路径初始化库
    let test_data = vec![0u8; 500_000]; // 500KB 测试数据
    
    // 先压缩一些数据用于解压测试
    let compressor = BitMemCompressor::new(&lib, CompressionFormat::Zip);
    let compressed_data = compressor.compress_from_buffer(&test_data, Some("test_data.bin".to_string())).unwrap();
    
    // 创建混合操作线程
    let mut handles = Vec::new();
    
    // 2个压缩线程
    for _i in 0..2 {
        let lib_clone = Arc::clone(&lib);
        let data_clone = test_data.clone();
        
        let handle = thread::spawn(move || {
            let compressor = BitMemCompressor::new(&lib_clone, CompressionFormat::Zip);
            compressor.compress_from_buffer(&data_clone, Some("test_data.bin".to_string())).unwrap()
        });
        
        handles.push(handle);
    }
    
    /*
    3个解压线程
    */
    for i in 0..3 {
        let lib_clone = Arc::clone(&lib);
        let compressed_clone = compressed_data.clone();
        
        let handle = thread::spawn(move || {
            let extractor = BitMemExtractor::new(&lib_clone, ExtractFormat::Zip);
            extractor.extract_to_buffer(&compressed_clone, 0).unwrap()
        });
        
        handles.push(handle);
    }
    
    // 等待所有线程完成
    for handle in handles {
        let _ = handle.join().unwrap();
    }
    
    println!("混合并发操作测试完成");
}
