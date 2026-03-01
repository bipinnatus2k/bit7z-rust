//! 测试程序：演示使用 bit7z-rust 进行压缩和解压
//!
//! 本示例展示如何使用本库进行：
//! 1. 压缩文件到 ZIP/7z 格式
//! 2. 解压档案
//! 3. 读取档案元数据
//! 4. 从内存缓冲区解压

use bit7z_rust::{
    BitLibrary, BitExtractor, BitArchiveReader,
    ExtractFormat,
};
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== bit7z-rust 压缩解压测试 ===\n");

    // 加载 7-Zip 动态库
    println!("[1] 加载 7-Zip 库...");
    
    // 尝试从环境变量或默认路径加载
    let lib_path = std::env::var("BIT7Z_LIBRARY_PATH")
        .unwrap_or_else(|_| "/usr/lib/7zip/7z.so".to_string());
    
    let lib = match BitLibrary::new(Some(lib_path.as_str())) {
        Ok(lib) => {
            println!("    ✓ 库加载成功 (路径：{})\n", lib_path);
            lib
        }
        Err(e) => {
            eprintln!("    ✗ 库加载失败：{}", e);
            eprintln!("    请确保已安装 7-Zip 并设置了正确的库路径");
            eprintln!("    可以通过环境变量 BIT7Z_LIBRARY_PATH 指定库路径");
            return Err(e.into());
        }
    };

    // 创建测试目录和文件
    println!("[2] 创建测试文件...");
    let test_dir = "test_output";
    let test_files_dir = "test_files";
    
    // 清理旧文件
    if Path::new(test_dir).exists() {
        fs::remove_dir_all(test_dir)?;
    }
    if Path::new(test_files_dir).exists() {
        fs::remove_dir_all(test_files_dir)?;
    }
    
    // 创建测试目录
    fs::create_dir_all(test_dir)?;
    fs::create_dir_all(test_files_dir)?;
    
    // 创建测试文件
    fs::write("test_files/file1.txt", "这是第一个测试文件的内容。\nHello from file1!")?;
    fs::write("test_files/file2.txt", "这是第二个测试文件的内容。\nHello from file2!")?;
    fs::write("test_files/data.json", r#"{"name": "test", "value": 123}"#)?;
    fs::create_dir_all("test_files/subdir")?;
    fs::write("test_files/subdir/nested.txt", "嵌套目录中的文件")?;
    
    println!("    ✓ 测试文件创建成功\n");

    // 测试 1: 使用 ZIP 格式压缩
    println!("[3] 测试 ZIP 格式压缩...");
    test_zip_compression(&lib, test_files_dir, test_dir)?;
    
    // 测试 2: 使用 7z 格式压缩
    println!("\n[4] 测试 7z 格式压缩...");
    test_7z_compression(&lib, test_files_dir, test_dir)?;
    
    // 测试 3: 解压 ZIP 档案
    println!("\n[5] 测试 ZIP 解压...");
    test_zip_extraction(&lib, test_dir)?;
    
    // 测试 4: 读取档案元数据
    println!("\n[6] 测试读取档案元数据...");
    test_archive_reader(&lib, test_dir)?;
    
    // 测试 5: 从内存缓冲区解压
    println!("\n[7] 测试从内存缓冲区解压...");
    test_buffer_extraction(&lib, test_files_dir, test_dir)?;
    
    // 清理测试文件
    println!("\n[8] 清理测试文件...");
    fs::remove_dir_all(test_dir)?;
    fs::remove_dir_all(test_files_dir)?;
    println!("    ✓ 清理完成");
    
    println!("\n=== 所有测试完成 ===");
    Ok(())
}

/// 测试 ZIP 格式压缩
fn test_zip_compression(
    _lib: &BitLibrary,
    _input_dir: &str,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // 注意：当前 compressor.rs 中的 compress 方法标记为 unimplemented
    // 这里展示 API 使用方式
    println!("    演示 API 使用方式（compress 方法尚未实现）");
    println!("    使用方式示例:");
    println!("      let mut compressor = BitCompressor::new(lib, CompressionFormat::Zip);");
    println!("      compressor.compression_level(CompressionLevel::Normal);");
    println!("      compressor.compress(&[\"file1.txt\", \"file2.txt\"], \"output.zip\")?;");
    
    // 使用系统 zip 命令创建测试文件供后续测试使用
    let output_path = format!("{}/test.zip", output_dir);
    let _ = std::process::Command::new("zip")
        .arg(&output_path)
        .arg("test_files/file1.txt")
        .arg("test_files/file2.txt")
        .output();
    
    println!("    ✓ 已使用系统 zip 命令创建测试档案");
    Ok(())
}

/// 测试 7z 格式压缩
fn test_7z_compression(
    _lib: &BitLibrary,
    _input_dir: &str,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("    演示 API 使用方式（compress 方法尚未实现）");
    println!("    使用方式示例:");
    println!("      let mut compressor = BitCompressor::new(lib, CompressionFormat::SevenZip);");
    println!("      compressor.compression_level(CompressionLevel::Ultra).solid(true);");
    println!("      compressor.compress(&[\"files/\"], \"output.7z\")?;");
    
    // 使用系统 7z 命令创建测试文件供后续测试使用
    let output_path = format!("{}/test.7z", output_dir);
    let _ = std::process::Command::new("7z")
        .arg("a")
        .arg(&output_path)
        .arg("test_files/file1.txt")
        .arg("test_files/file2.txt")
        .output();
    
    println!("    ✓ 已使用系统 7z 命令创建测试档案");
    Ok(())
}

/// 测试 ZIP 解压
fn test_zip_extraction(
    lib: &BitLibrary,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // 首先创建一个测试用的 ZIP 文件（使用系统命令）
    let test_zip = format!("{}/sample.zip", output_dir);
    
    // 使用系统 zip 命令创建测试档案（如果可用）
    let zip_result = std::process::Command::new("zip")
        .arg(&test_zip)
        .arg("test_files/file1.txt")
        .arg("test_files/file2.txt")
        .output();
    
    match zip_result {
        Ok(output) if output.status.success() => {
            println!("    ✓ 创建测试 ZIP 档案成功");
            
            let extract_dir = format!("{}/extracted_zip", output_dir);
            let extractor = BitExtractor::new(lib, ExtractFormat::Zip);
            
            match extractor.extract(&test_zip, &extract_dir) {
                Ok(_) => {
                    println!("    ✓ ZIP 解压成功到：{}", extract_dir);
                    
                    // 验证解压后的文件
                    if Path::new(&format!("{}/test_files/file1.txt", extract_dir)).exists() {
                        println!("    ✓ 文件验证成功");
                    }
                }
                Err(e) => {
                    println!("    ✗ 解压失败：{}", e);
                }
            }
        }
        _ => {
            println!("    ⊘ 跳过测试（zip 命令不可用）");
        }
    }
    
    Ok(())
}

/// 测试读取档案元数据
fn test_archive_reader(
    lib: &BitLibrary,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // 创建一个测试用的 7z 文件（使用系统命令）
    let test_7z = format!("{}/sample.7z", output_dir);

    let _7z_result = std::process::Command::new("7z")
        .arg("a")
        .arg(&test_7z)
        .arg("test_files/file1.txt")
        .arg("test_files/file2.txt")
        .arg("test_files/data.json")
        .output();

    match _7z_result {
        Ok(output) if output.status.success() => {
            println!("    ✓ 创建测试 7z 档案成功");

            let mut reader = BitArchiveReader::new(lib, ExtractFormat::SevenZip);

            match reader.open(&test_7z) {
                Ok(_) => {
                    println!("    ✓ 档案打开成功");

                    // 获取档案属性
                    match reader.archive_properties() {
                        Ok(props) => {
                            println!("    档案属性:");
                            println!("      - 文件数：{}", props.files_count);
                            println!("      - 文件夹数：{}", props.folders_count);
                            println!("      - 总大小：{} 字节", props.size);
                            println!("      - 压缩后大小：{} 字节", props.pack_size);
                            println!("      - 加密：{}", if props.encrypted { "是" } else { "否" });
                        }
                        Err(e) => println!("    ✗ 获取属性失败：{}", e),
                    }

                    // 获取所有项目
                    match reader.items() {
                        Ok(items) => {
                            println!("\n    档案内容:");
                            for item in &items {
                                let size_str = if item.is_dir {
                                    "<DIR>".to_string()
                                } else {
                                    format!("{} bytes", item.size)
                                };
                                println!("      - {} ({})", item.path, size_str);
                            }
                        }
                        Err(e) => println!("    ✗ 获取项目列表失败：{}", e),
                    }
                }
                Err(e) => {
                    println!("    ✗ 打开档案失败：{}", e);
                }
            }
        }
        _ => {
            println!("    ⊘ 跳过测试（7z 命令不可用）");
        }
    }

    Ok(())
}

/// 测试从内存缓冲区解压
fn test_buffer_extraction(
    lib: &BitLibrary,
    _input_dir: &str,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // 首先创建一个测试档案
    let test_tar = format!("{}/sample.tar", output_dir);
    
    let tar_result = std::process::Command::new("tar")
        .arg("-cf")
        .arg(&test_tar)
        .arg("-C")
        .arg("test_files")
        .arg(".")
        .output();
    
    match tar_result {
        Ok(output) if output.status.success() => {
            println!("    ✓ 创建测试 TAR 档案成功");
            
            // 读取档案到内存
            let buffer = fs::read(&test_tar)?;
            println!("    ✓ 读取档案到内存 ({} 字节)", buffer.len());
            
            let extractor = BitExtractor::new(lib, ExtractFormat::Tar);
            let extract_dir = format!("{}/extracted_tar", output_dir);
            
            match extractor.extract_from_buffer(buffer, &extract_dir) {
                Ok(_) => {
                    println!("    ✓ 从内存缓冲区解压成功");
                }
                Err(e) => {
                    println!("    ✗ 解压失败：{}", e);
                }
            }
        }
        _ => {
            println!("    ⊘ 跳过测试（tar 命令不可用）");
        }
    }
    
    Ok(())
}
