# 基本解压

本章介绍如何使用 `BitExtractor` 从不同格式的存档中解压文件或目录。

## 1. 解压整个存档

将整个存档解压到指定的本地目录。

```rust
use bit7z_rust::{BitLibrary, BitExtractor, ExtractFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lib = BitLibrary::new::<String>(None)?;
    let mut extractor = BitExtractor::new(&lib, ExtractFormat::SevenZip);
    
    // 将 archive.7z 解压到 output_dir/ 目录
    extractor.extract("archive.7z", "output_dir/")?;
    
    Ok(())
}
```

## 2. 处理加密存档

解压受密码保护的存档时，需要提供密码。

```rust
let mut extractor = BitExtractor::new(&lib, ExtractFormat::Zip);

extractor
    .password("archive_password")
    .extract("secure.zip", "extracted/")?;
```

## 3. 部分解压

您可以根据索引列表仅解压某些特定的项目。

```rust
// 解压索引为 0, 2, 5 的三个项目
extractor.extract_items("archive.7z", &[0, 2, 5], "output_dir/")?;
```

## 4. 按通配符或正则匹配解压

`bit7z-rust` 支持按照名称或通配符匹配来解压文件。

```rust
// 解压所有 .txt 文件
extractor.extract_matching("archive.7z", "*.txt", "output_dir/")?;

// 使用正则表达式匹配
extractor.extract_matching_regex("archive.7z", r"docs/.*\.pdf$", "output_dir/")?;
```

## 5. 解压到内存

如果只需要读取存档中的单个小文件，而不想将其保存到本地，可以使用 `extract_to_buffer`。

```rust
// 将索引为 0 的文件解压到内存中
let data: Vec<u8> = extractor.extract_to_buffer("archive.7z", 0)?;
println!("解压数据大小: {} 字节", data.len());
```

## 6. 测试存档完整性

在解压之前，您可以测试存档文件是否损坏或密码是否正确。

```rust
match extractor.test("archive.7z") {
    Ok(()) => println!("存档完整"),
    Err(e) => eprintln!("完整性检查失败: {}", e),
}
```

## 注意事项

- **只读格式**: `bit7z-rust` 支持解压许多只读格式（如 RAR、RAR5、CAB、ISO 等）。只需在 `ExtractFormat` 中指定相应的枚举值。
- **目标目录**: 如果目标解压目录不存在，`BitExtractor` 会尝试自动创建它。
