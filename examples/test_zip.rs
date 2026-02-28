use bit7z_rust::{BitLibrary, BitExtractor, ExtractFormat};

fn main() {
    println!("=== ZIP 解压测试 ===\n");

    // 加载 7-Zip 库
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

    // 测试 ZIP 解压
    let test_zip = "test_debug/test.zip";
    let output_dir = "test_debug/extracted";

    println!("\n测试文件：{}", test_zip);
    println!("输出目录：{}\n", output_dir);

    let extractor = BitExtractor::new(&lib, ExtractFormat::Zip);

    match extractor.extract(test_zip, output_dir) {
        Ok(_) => {
            println!("✓ ZIP 解压成功！");
            
            // 验证文件
            let extracted_file = format!("{}/test.txt", output_dir);
            if std::path::Path::new(&extracted_file).exists() {
                println!("✓ 文件验证成功：{}", extracted_file);
                
                // 读取并显示内容
                if let Ok(content) = std::fs::read_to_string(&extracted_file) {
                    println!("✓ 文件内容：{}", content.trim());
                }
            } else {
                println!("✗ 文件验证失败：{} 不存在", extracted_file);
            }
        }
        Err(e) => {
            eprintln!("✗ 解压失败：{}", e);
        }
    }
}
