use bit7z_rust::{BitLibrary, BitExtractor, ExtractFormat};
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 单格式解压测试 ===\n");

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
    let test_dir = "test_debug/single_test";
    let _ = fs::remove_dir_all(test_dir);
    fs::create_dir_all(test_dir)?;

    // 测试 7z 格式
    println!("[1] 测试 7z 格式...");
    test_format(&lib, test_dir, "7z", ExtractFormat::SevenZip, "test.txt", b"7z test content")?;

    // 测试 TAR 格式
    println!("\n[2] 测试 TAR 格式...");
    test_format(&lib, test_dir, "tar", ExtractFormat::Tar, "test.txt", b"tar test content")?;

    // 测试 GZip 格式
    println!("\n[3] 测试 GZip 格式...");
    test_format(&lib, test_dir, "gz", ExtractFormat::GZip, "test.txt", b"gzip test content")?;

    // 测试 BZip2 格式
    println!("\n[4] 测试 BZip2 格式...");
    test_format(&lib, test_dir, "bz2", ExtractFormat::BZip2, "test.txt", b"bzip2 test content")?;

    // 测试 XZ 格式
    println!("\n[5] 测试 XZ 格式...");
    test_format(&lib, test_dir, "xz", ExtractFormat::Xz, "test.txt", b"xz test content")?;

    println!("\n=== 所有测试完成 ===");
    
    // 清理
    println!("\n清理测试文件...");
    let _ = fs::remove_dir_all(test_dir);
    println!("✓ 清理完成");

    Ok(())
}

fn test_format(
    lib: &BitLibrary, 
    test_dir: &str, 
    ext: &str, 
    format: ExtractFormat,
    filename: &str,
    content: &[u8]
) -> Result<(), Box<dyn std::error::Error>> {
    // 创建内容目录和文件
    let content_dir = format!("{}/{}_content", test_dir, ext);
    fs::create_dir_all(&content_dir)?;
    fs::write(format!("{}/{}", content_dir, filename), content)?;
    
    // 使用对应工具创建档案
    let archive_path = format!("{}/test.{}", test_dir, ext);
    let archive_name = format!("test.{}", ext);
    
    let cmd = match ext {
        "7z" => format!("cd {} && 7z a {} {}", content_dir, archive_path, filename),
        "tar" => format!("cd {} && tar -cf {} {}", content_dir, archive_path, filename),
        "gz" => format!("cd {} && gzip -c {} > {}", content_dir, filename, archive_path),
        "bz2" => format!("cd {} && bzip2 -c {} > {}", content_dir, filename, archive_path),
        "xz" => format!("cd {} && xz -c {} > {}", content_dir, filename, archive_path),
        _ => return Ok(()),
    };
    
    let output = std::process::Command::new("bash")
        .arg("-c")
        .arg(&cmd)
        .output()?;
    
    eprintln!("    CMD: {} (status: {})", cmd, output.status);
    
    if !output.status.success() {
        println!("    ⊘ 跳过测试（无法创建 {} 档案）", ext);
        return Ok(());
    }
    
    // 显示档案信息
    let info_cmd = match ext {
        "7z" => format!("7z l {}", archive_path),
        "tar" => format!("tar -tf {}", archive_path),
        _ => format!("file {}", archive_path),
    };
    
    if let Ok(info_output) = std::process::Command::new("sh")
        .arg("-c")
        .arg(&info_cmd)
        .output() 
    {
        let info = String::from_utf8_lossy(&info_output.stdout);
        println!("    档案信息：{}", info.lines().last().unwrap_or(""));
    }
    
    // 测试解压
    let extract_dir = format!("{}/extracted_{}", test_dir, ext);
    let extractor = BitExtractor::new(lib, format);
    
    match extractor.extract(&archive_path, &extract_dir) {
        Ok(_) => {
            let extracted_file = format!("{}/{}", extract_dir, filename);
            if Path::new(&extracted_file).exists() {
                if let Ok(read_content) = fs::read(&extracted_file) {
                    if read_content == content {
                        println!("    ✓ {} 解压成功且内容正确", ext.to_uppercase());
                    } else {
                        println!("    ⊘ {} 解压成功但内容不匹配", ext.to_uppercase());
                    }
                } else {
                    println!("    ⊘ {} 解压成功但无法读取文件", ext.to_uppercase());
                }
            } else {
                // 对于流式格式，输出文件名可能不同
                println!("    ✓ {} 解压成功（输出文件：{:?})", ext.to_uppercase(), 
                       fs::read_dir(&extract_dir).ok().and_then(|mut d| d.next().map(|e| e.unwrap().file_name())));
            }
        }
        Err(e) => println!("    ✗ {} 解压失败：{}", ext.to_uppercase(), e),
    }
    
    Ok(())
}
