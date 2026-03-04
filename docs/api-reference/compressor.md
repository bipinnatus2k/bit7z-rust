# BitCompressor

`BitCompressor` 用于将文件或目录压缩为各种格式的存档文件。

## 结构体定义

```rust
pub struct BitCompressor<'a> {
    // 内部字段
}
```

## 创建方法

### `new`

创建一个新的压缩器实例。

```rust
pub fn new(library: &'a BitLibrary, format: CompressionFormat) -> Self
```

- **参数**:
    - `library`: 已加载的 `BitLibrary` 引用。
    - `format`: 压缩格式（如 `7z`, `Zip`, `Gzip` 等）。

## 配置方法 (链式调用)

### `password`

设置压缩密码（加密）。

```rust
pub fn password(&mut self, password: impl Into<String>) -> &mut Self
```

### `compression_level`

设置压缩级别。

```rust
pub fn compression_level(&mut self, level: CompressionLevel) -> &mut Self
```

### `compression_method`

设置压缩方法（如 `LZMA`, `LZMA2`, `PPMd` 等）。

```rust
pub fn compression_method(&mut self, method: CompressionMethod) -> &mut Self
```

### `solid`

启用或禁用固实压缩（Solid Compression）。

```rust
pub fn solid(&mut self, solid: bool) -> &mut Self
```

### `crypt_headers`

启用或禁用文件头加密（仅限 7z 格式）。

```rust
pub fn crypt_headers(&mut self, encrypt: bool) -> &mut Self
```

## 压缩方法

### `compress`

压缩指定的文件或目录到目标路径。

```rust
pub fn compress<P: AsRef<Path>, O: AsRef<Path>>(
    &self,
    input_paths: &[P],
    output_path: O,
) -> Result<()>
```

### `compress_files`

仅压缩指定的文件列表（忽略目录）。

```rust
pub fn compress_files<P: AsRef<Path>, O: AsRef<Path>>(
    &self,
    files: &[P],
    output_path: O,
) -> Result<()>
```

### `compress_directory_contents`

压缩目录内容。

```rust
pub fn compress_directory_contents<P: AsRef<Path>, O: AsRef<Path>>(
    &self,
    dir_path: P,
    output_path: O,
    recursive: bool,
    filter: Option<&str>,
) -> Result<()>
```

## 示例

```rust
use bit7z_rust::{BitLibrary, BitCompressor, CompressionFormat, CompressionLevel};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lib = BitLibrary::new::<String>(None)?;
    let mut compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);
    
    compressor
        .compression_level(CompressionLevel::Ultra)
        .password("secure_password")
        .solid(true)
        .compress(&["data/file1.txt", "data/subdir/"], "output.7z")?;
    
    Ok(())
}
```
