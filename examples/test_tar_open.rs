use bit7z_rust::{BitLibrary, BitArchiveReader, ExtractFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== TAR 格式打开测试 ===\n");

    let lib = BitLibrary::new(Some("/usr/lib/7zip/7z.so"))?;
    println!("✓ 库加载成功");

    let mut reader = BitArchiveReader::new(&lib, ExtractFormat::Tar);
    println!("✓ Reader 创建成功");

    println!("正在打开档案：test_debug/test.tar");
    match reader.open("test_debug/test.tar") {
        Ok(_) => {
            println!("✓ TAR 档案打开成功");
            match reader.items() {
                Ok(items) => {
                    println!("✓ 档案包含 {} 个文件:", items.len());
                    for item in items {
                        println!("  - {} ({} bytes)", item.path, item.size);
                    }
                }
                Err(e) => eprintln!("✗ 读取文件列表失败：{}", e),
            }
        }
        Err(e) => {
            eprintln!("✗ 打开档案失败：{}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
