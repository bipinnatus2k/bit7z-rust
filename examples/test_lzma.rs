use bit7z_rust::{BitLibrary, BitExtractor, ExtractFormat};

fn main() {
    println!("=== LZMA 格式解压测试 ===\n");

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

    // 测试 LZMA 解压
    let test_lzma = "test_debug/test.lzma";
    let output_dir = "test_debug/extracted_lzma";

    println!("\n测试文件：{}", test_lzma);
    println!("输出目录：{}\n", output_dir);

    let extractor = BitExtractor::new(&lib, ExtractFormat::Lzma);

    match extractor.extract(test_lzma, output_dir) {
        Ok(_) => {
            println!("✓ LZMA 解压成功！");
        }
        Err(e) => {
            eprintln!("✗ 解压失败：{}", e);
        }
    }
}
