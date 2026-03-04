# BitExtractor

`BitExtractor` 用于从各种格式的存档文件中解压文件或目录。

## 结构体定义

```rust
pub struct BitExtractor<'a> {
    // 内部字段
}
```

## 创建方法

### `new`

创建一个新的解压器实例。

```rust
pub fn new(library: &'a BitLibrary, format: ExtractFormat) -> Self
```

- **参数**:
    - `library`: 已加载的 `BitLibrary` 引用。
    - `format`: 解压格式（如 `SevenZip`, `Zip`, `Rar`, `Tar` 等）。

## 配置方法 (链式调用)

### `password`

设置解压密码（用于加密存档）。

```rust
pub fn password(&mut self, password: impl Into<String>) -> &mut Self
```

## 解压方法

### `extract`

将整个存档解压到指定目录。

```rust
pub fn extract<P: AsRef<Path>>(
    &self,
    archive_path: P,
    output_dir: P,
) -> Result<()>
```

### `extract_items`

解压存档中指定索引的文件列表。

```rust
pub fn extract_items<P: AsRef<Path>>(
    &self,
    archive_path: P,
    indices: &[u32],
    output_dir: P,
) -> Result<()>
```

### `extract_matching`

解压符合通配符模式（如 `*.txt`）的文件。

```rust
pub fn extract_matching<P: AsRef<Path>>(
    &self,
    archive_path: P,
    pattern: &str,
    output_dir: P,
) -> Result<()>
```

### `extract_to_buffer`

将存档中的单个文件解压到内存缓冲区。

```rust
pub fn extract_to_buffer<P: AsRef<Path>>(
    &self,
    archive_path: P,
    index: u32,
) -> Result<Vec<u8>>
```

### `test`

测试存档的完整性而不进行实际解压。

```rust
pub fn test<P: AsRef<Path>>(&self, archive_path: P) -> Result<()>
```

## 示例

```rust
use bit7z_rust::{BitLibrary, BitExtractor, ExtractFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lib = BitLibrary::new::<String>(None)?;
    let mut extractor = BitExtractor::new(&lib, ExtractFormat::SevenZip);
    
    extractor
        .password("secure_password")
        .extract("archive.7z", "output_dir/")?;
    
    Ok(())
}
```
