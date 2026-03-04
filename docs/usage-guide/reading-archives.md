# 存档信息查询

本章介绍如何使用 `BitArchiveReader` 读取现有存档的内容列表、元数据和属性。

## 1. 打开存档

在读取信息前，首先需要打开存档文件。

```rust
use bit7z_rust::{BitLibrary, BitArchiveReader, ExtractFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lib = BitLibrary::new::<String>(None)?;
    let mut reader = BitArchiveReader::new(&lib, ExtractFormat::SevenZip);
    
    // 打开存档
    reader.open("backup.7z")?;
    
    // 获取项目总数
    let count = reader.items_count()?;
    println!("存档中共有 {} 个项目", count);
    
    Ok(())
}
```

## 2. 获取项目元数据

`BitArchiveReader::items()` 返回存档中所有项目的元数据列表。

```rust
for item in reader.items()? {
    println!("项目: {}", item.path);
    println!("  - 原始大小: {} 字节", item.size);
    println!("  - 压缩大小: {} 字节", item.pack_size);
    println!("  - 是否目录: {}", item.is_dir);
    println!("  - 是否加密: {}", item.encrypted);
    
    if let Some(mtime) = item.modification_time {
        println!("  - 修改时间: {:?}", mtime);
    }
}
```

## 3. 获取特定项目

如果您已经知道索引，可以直接获取特定项目的元数据。

```rust
let item = reader.item(0)?; // 获取索引为 0 的项目
```

## 4. 快速检查加密状态

可以使用静态方法快速检查存档是否受密码保护。

```rust
use bit7z_rust::{BitLibrary, BitArchiveReader, ExtractFormat};

let encrypted = BitArchiveReader::is_encrypted_static(
    &lib, 
    "secure.zip", 
    ExtractFormat::Zip
)?;

if encrypted {
    println!("存档已加密");
}
```

## 5. 存档项目属性 (ArchiveItem)

`ArchiveItem` 结构体包含以下常用字段：

| 字段 | 类型 | 说明 |
| :--- | :--- | :--- |
| `index` | `u32` | 存档中的位置索引 |
| `path` | `String` | 项目路径（使用正斜杠 `/`） |
| `is_dir` | `bool` | 是否为目录 |
| `size` | `u64` | 未压缩的原始大小 |
| `pack_size` | `u64` | 压缩后的占用大小 |
| `encrypted` | `bool` | 该项目是否已加密 |
| `crc` | `Option<u32>` | CRC32 校验码（如果可用） |
| `modification_time` | `Option<SystemTime>` | 最后修改时间 |

## 注意事项

- **只读模式**: `BitArchiveReader` 以只读方式打开存档，不会锁定或修改原始文件。
- **性能**: 对于包含数十万个项目的大型存档，调用 `items()` 可能会消耗一定内存和时间，因为它会遍历整个目录结构。
