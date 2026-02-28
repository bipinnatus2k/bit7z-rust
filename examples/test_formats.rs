use bit7z_rust::{BitLibrary, BitExtractor, ExtractFormat};
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 多格式解压测试 ===\n");

    // 加载 7-Zip 库
    let lib = match BitLibrary::new(Some("/usr/lib/7zip/7z.so")) {
        Ok(l) => {
            println!("✓ 库加载成功\n");
            l
        }
        Err(e) => {
            eprintln!("✗ 库加载失败：{}", e);
            return Err(Box::new(e));
        }
    };

    // 创建测试目录
    let test_dir = "test_debug/formats_test";
    let _ = fs::remove_dir_all(test_dir);
    fs::create_dir_all(test_dir)?;

    // 测试 7z 格式
    println!("[1] 测试 7z 格式...");
    test_7z_format(&lib, test_dir)?;

    // 测试 TAR 格式
    println!("\n[2] 测试 TAR 格式...");
    test_tar_format(&lib, test_dir)?;

    // 测试 GZip 格式
    println!("\n[3] 测试 GZip 格式...");
    test_gzip_format(&lib, test_dir)?;

    // 测试 BZip2 格式
    println!("\n[4] 测试 BZip2 格式...");
    test_bzip2_format(&lib, test_dir)?;

    // 测试 XZ 格式
    println!("\n[5] 测试 XZ 格式...");
    test_xz_format(&lib, test_dir)?;

    println!("\n=== 所有测试完成 ===");
    
    // 清理
    println!("\n清理测试文件...");
    let _ = fs::remove_dir_all(test_dir);
    println!("✓ 清理完成");

    Ok(())
}

fn test_7z_format(lib: &BitLibrary, test_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 创建测试文件
    let content_dir = format!("{}/7z_content", test_dir);
    fs::create_dir_all(&content_dir)?;
    fs::write(format!("{}/test.txt", content_dir), "7z test content")?;
    
    // 使用系统 7z 创建档案（使用相对路径）
    let archive_path = format!("{}/test.7z", test_dir);
    let status = std::process::Command::new("7z")
        .arg("a")
        .arg(&archive_path)
        .arg("test.txt")
        .current_dir(&content_dir)
        .status()?;
    
    if !status.success() {
        println!("    ⊘ 跳过测试（无法创建 7z 档案）");
        return Ok(());
    }
    
    // 验证档案内容
    let output = std::process::Command::new("7z")
        .arg("l")
        .arg(&archive_path)
        .output()?;
    let archive_content = String::from_utf8_lossy(&output.stdout);
    println!("    档案内容：{}", archive_content.lines().last().unwrap_or(""));
    
    // 测试解压
    let extract_dir = format!("{}/extracted_7z", test_dir);
    let extractor = BitExtractor::new(lib, ExtractFormat::SevenZip);
    
    match extractor.extract(&archive_path, &extract_dir) {
        Ok(_) => {
            if Path::new(&format!("{}/test.txt", extract_dir)).exists() {
                println!("    ✓ 7z 解压成功");
            } else {
                println!("    ✗ 7z 解压成功但文件验证失败");
            }
        }
        Err(e) => println!("    ✗ 7z 解压失败：{}", e),
    }
    
    Ok(())
}

fn test_tar_format(lib: &BitLibrary, test_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 创建测试文件
    let content_dir = format!("{}/tar_content", test_dir);
    fs::create_dir_all(&content_dir)?;
    fs::write(format!("{}/test.txt", content_dir), "tar test content")?;
    
    // 使用系统 tar 创建档案（使用相对路径）
    let archive_path = format!("{}/test.tar", test_dir);
    let status = std::process::Command::new("tar")
        .arg("-cf")
        .arg(&archive_path)
        .arg("test.txt")
        .current_dir(&content_dir)
        .status()?;
    
    if !status.success() {
        println!("    ⊘ 跳过测试（无法创建 tar 档案）");
        return Ok(());
    }
    
    // 测试解压
    let extract_dir = format!("{}/extracted_tar", test_dir);
    let extractor = BitExtractor::new(lib, ExtractFormat::Tar);
    
    match extractor.extract(&archive_path, &extract_dir) {
        Ok(_) => {
            if Path::new(&format!("{}/test.txt", extract_dir)).exists() {
                println!("    ✓ TAR 解压成功");
            } else {
                println!("    ✗ TAR 解压成功但文件验证失败");
            }
        }
        Err(e) => println!("    ✗ TAR 解压失败：{}", e),
    }
    
    Ok(())
}

fn test_gzip_format(lib: &BitLibrary, test_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 创建测试文件
    let content_dir = format!("{}/gzip_content", test_dir);
    fs::create_dir_all(&content_dir)?;
    let test_file = format!("{}/test.txt", content_dir);
    fs::write(&test_file, "gzip test content")?;
    
    // 使用系统 gzip 创建档案
    let gz_file = format!("{}.gz", test_file);
    let status = std::process::Command::new("gzip")
        .arg("-k")  // 保留原文件
        .arg(&test_file)
        .status()?;
    
    if !status.success() {
        println!("    ⊘ 跳过测试（无法创建 gzip 档案）");
        return Ok(());
    }
    
    let archive_path = format!("{}/test.txt.gz", test_dir);
    fs::copy(&gz_file, &archive_path)?;
    
    // 测试解压
    let extract_dir = format!("{}/extracted_gzip", test_dir);
    fs::create_dir_all(&extract_dir)?;
    let extractor = BitExtractor::new(lib, ExtractFormat::GZip);
    
    match extractor.extract(&archive_path, &extract_dir) {
        Ok(_) => {
            println!("    ✓ GZip 解压成功");
        }
        Err(e) => println!("    ✗ GZip 解压失败：{}", e),
    }
    
    Ok(())
}

fn test_bzip2_format(lib: &BitLibrary, test_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 创建测试文件
    let content_dir = format!("{}/bzip2_content", test_dir);
    fs::create_dir_all(&content_dir)?;
    let test_file = format!("{}/test.txt", content_dir);
    fs::write(&test_file, "bzip2 test content")?;
    
    // 使用系统 bzip2 创建档案
    let status = std::process::Command::new("bzip2")
        .arg("-k")  // 保留原文件
        .arg(&test_file)
        .status()?;
    
    if !status.success() {
        println!("    ⊘ 跳过测试（无法创建 bzip2 档案）");
        return Ok(());
    }
    
    let archive_path = format!("{}/test.txt.bz2", test_dir);
    fs::copy(format!("{}.bz2", test_file), &archive_path)?;
    
    // 测试解压
    let extract_dir = format!("{}/extracted_bzip2", test_dir);
    fs::create_dir_all(&extract_dir)?;
    let extractor = BitExtractor::new(lib, ExtractFormat::BZip2);
    
    match extractor.extract(&archive_path, &extract_dir) {
        Ok(_) => {
            println!("    ✓ BZip2 解压成功");
        }
        Err(e) => println!("    ✗ BZip2 解压失败：{}", e),
    }
    
    Ok(())
}

fn test_xz_format(lib: &BitLibrary, test_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 创建测试文件
    let content_dir = format!("{}/xz_content", test_dir);
    fs::create_dir_all(&content_dir)?;
    let test_file = format!("{}/test.txt", content_dir);
    fs::write(&test_file, "xz test content")?;
    
    // 使用系统 xz 创建档案
    let status = std::process::Command::new("xz")
        .arg("-k")  // 保留原文件
        .arg(&test_file)
        .status()?;
    
    if !status.success() {
        println!("    ⊘ 跳过测试（无法创建 xz 档案）");
        return Ok(());
    }
    
    let archive_path = format!("{}/test.txt.xz", test_dir);
    fs::copy(format!("{}.xz", test_file), &archive_path)?;
    
    // 测试解压
    let extract_dir = format!("{}/extracted_xz", test_dir);
    fs::create_dir_all(&extract_dir)?;
    let extractor = BitExtractor::new(lib, ExtractFormat::Xz);
    
    match extractor.extract(&archive_path, &extract_dir) {
        Ok(_) => {
            println!("    ✓ XZ 解压成功");
        }
        Err(e) => println!("    ✗ XZ 解压失败：{}", e),
    }
    
    Ok(())
}
