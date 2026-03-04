# BitArchiveReader

`BitArchiveReader` 用于读取存档文件的元数据（如文件列表、大小、修改时间等），而无需解压实际内容。

## 结构体定义

```rust
pub struct BitArchiveReader<'a> {
    // 内部字段
}

pub struct ArchiveItem {
    pub index: u32,                          // 存档中的索引
    pub path: String,                         // 相对路径
    pub is_dir: bool,                        // 是否为目录
    pub size: u64,                           // 未压缩大小
    pub pack_size: u64,                      // 压缩后大小
    pub attributes: u32,                     // 文件属性
    pub creation_time: Option<SystemTime>,   // 创建时间
    pub access_time: Option<SystemTime>,     // 访问时间
    pub modification_time: Option<SystemTime>, // 修改时间
    pub encrypted: bool,                     // 是否加密
    pub crc: Option<u32>,                    // CRC32 校验和
}
```

## 方法

### `new`

```rust
pub fn new(library: &'a BitLibrary, format: ExtractFormat) -> Self
```

### `open`

打开指定的存档文件以供读取。

```rust
pub fn open<P: AsRef<Path>>(&mut self, archive_path: P) -> Result<()>
```

### `items_count`

获取存档中的项目总数。

```rust
pub fn items_count(&self) -> Result<u32>
```

### `items`

获取存档中所有项目的元数据列表。

```rust
pub fn items(&self) -> Result<Vec<ArchiveItem>>
```

### `item`

获取存档中指定索引的单个项目元数据。

```rust
pub fn item(&self, index: u32) -> Result<ArchiveItem>
```

## 静态方法

### `is_encrypted_static`

快速检查存档是否加密（无需手动管理 `BitArchiveReader` 生命周期）。

```rust
pub fn is_encrypted_static<P: AsRef<Path>>(
    library: &'a BitLibrary,
    archive_path: P,
    format: ExtractFormat,
) -> Result<bool>
```

## 示例

```rust
use bit7z_rust::{BitLibrary, BitArchiveReader, ExtractFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lib = BitLibrary::new::<String>(None)?;
    let mut reader = BitArchiveReader::new(&lib, ExtractFormat::SevenZip);
    
    reader.open("archive.7z")?;
    
    println!("Total items: {}", reader.items_count()?);
    
    for item in reader.items()? {
        println!("{}: {} bytes (compressed: {} bytes)", 
                 item.path, item.size, item.pack_size);
    }
    
    Ok(())
}
```
