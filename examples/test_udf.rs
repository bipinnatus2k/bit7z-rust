use bit7z_rust::{BitLibrary, BitExtractor, ExtractFormat};

fn main() {
    println!("=== UDF 格式解压测试 ===\n");

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

    // 测试 UDF 解压
    let test_udf = "test_debug/test.udf";
    let output_dir = "test_debug/extracted_udf";

    println!("\n测试文件：{}", test_udf);
    println!("输出目录：{}\n", output_dir);

    let extractor = BitExtractor::new(&lib, ExtractFormat::Udf);

    match extractor.extract(test_udf, output_dir) {
        Ok(_) => {
            println!("✓ UDF 解压成功！");
        }
        Err(e) => {
            eprintln!("✗ 解压失败：{}", e);
        }
    }
}
