use bit7z_rust::{BitLibrary, BitExtractor, ExtractFormat};
use std::fs;

fn main() {
    println!("=== 测试 TAR 格式（使用 7z 命令行创建） ===\n");

    let lib = match BitLibrary::new(Some("/usr/lib/7zip/7z.so")) {
        Ok(l) => {
            println!("✓ 库加载成功");
            l
        }
        Err(e) => {
            eprintln!("✗ 库加载失败：{}", e);
            return;
        }
    };

    // 使用 7z 命令行创建 TAR 档案
    let test_dir = "test_debug/tar_test2";
    let _ = fs::remove_dir_all(test_dir);
    fs::create_dir_all(test_dir).unwrap();
    fs::write(format!("{}/test.txt", test_dir), b"tar test content\n").unwrap();
    
    let archive_path = "test_debug/test2.tar";
    let output = std::process::Command::new("7z")
        .args(["a", "-ttar", archive_path, &format!("{}/test.txt", test_dir)])
        .output()
        .expect("Failed to run 7z");
    
    if !output.status.success() {
        eprintln!("✗ 创建 TAR 档案失败：{}", String::from_utf8_lossy(&output.stderr));
        return;
    }
    println!("✓ 使用 7z 创建 TAR 档案成功");
    
    // 显示档案信息
    let info = std::process::Command::new("7z")
        .args(["l", archive_path])
        .output()
        .expect("Failed to run 7z");
    println!("档案信息：");
    for line in String::from_utf8_lossy(&info.stdout).lines().skip(6).take(5) {
        println!("  {}", line);
    }
    
    // 测试解压
    let extract_dir = "test_debug/extracted_tar2";
    let _ = fs::remove_dir_all(extract_dir);
    
    let extractor = BitExtractor::new(&lib, ExtractFormat::Tar);
    
    match extractor.extract(archive_path, extract_dir) {
        Ok(_) => {
            println!("✓ TAR 解压成功");
            let extracted_file = format!("{}/test.txt", extract_dir);
            if let Ok(content) = fs::read_to_string(&extracted_file) {
                println!("✓ 文件内容：{}", content.trim());
            }
        }
        Err(e) => eprintln!("✗ 解压失败：{}", e),
    }
}
