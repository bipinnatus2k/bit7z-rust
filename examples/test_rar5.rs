use bit7z_rust::{BitLibrary, BitExtractor, ExtractFormat};

fn main() {
    println!("=== RAR5 格式解压测试 ===\n");

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

    // 测试 RAR5 解压
    let test_rar5 = "test_debug/test.rar5";
    let output_dir = "test_debug/extracted_rar5";

    println!("\n测试文件：{}", test_rar5);
    println!("输出目录：{}\n", output_dir);

    let extractor = BitExtractor::new(&lib, ExtractFormat::Rar5);

    match extractor.extract(test_rar5, output_dir) {
        Ok(_) => {
            println!("✓ RAR5 解压成功！");

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
