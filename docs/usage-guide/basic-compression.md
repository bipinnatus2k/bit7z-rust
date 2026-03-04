# 基本压缩

本章介绍如何使用 `BitCompressor` 进行常见的文件压缩操作。

## 1. 压缩单个或多个文件

压缩操作通常需要 `BitLibrary` 引用和 `CompressionFormat` 格式。

```rust
use bit7z_rust::{BitLibrary, BitCompressor, CompressionFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lib = BitLibrary::new::<String>(None)?;
    let mut compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);
    
    // 压缩列表中的文件
    compressor.compress(&["file1.txt", "file2.txt"], "archive.7z")?;
    
    Ok(())
}
```

## 2. 压缩目录

压缩整个目录及其子目录。

```rust
// 压缩目录内容
compressor.compress_directory_contents(
    "source_dir/", 
    "archive.7z", 
    true, // 是否递归压缩子目录
    None  // 过滤器，如 Some("*.txt")
)?;
```

## 3. 设置压缩参数

可以使用链式调用设置常用的压缩参数。

```rust
use bit7z_rust::{BitLibrary, BitCompressor, CompressionFormat, CompressionLevel, CompressionMethod};

let mut compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);

compressor
    .compression_level(CompressionLevel::Ultra)  // 极致压缩
    .compression_method(CompressionMethod::Lzma2) // 使用 LZMA2 算法
    .password("secure_password")                  // 设置密码 (AES-256)
    .solid(true)                                  // 开启固实压缩
    .compress(&["data/"], "backup.7z")?;
```

## 4. 常见压缩格式说明

| 格式 | 枚举值 | 特点 |
| :--- | :--- | :--- |
| **7z** | `SevenZip` | 压缩率最高，支持固实压缩和文件头加密。 |
| **ZIP** | `Zip` | 通用性最好，支持基本加密。 |
| **GZIP** | `GZip` | 通常用于压缩单个文件，常与 TAR 结合（.tar.gz）。 |
| **TAR** | `Tar` | 仅归档不压缩，常用于 Linux。 |
| **XZ** | `Xz` | 现代高压缩率格式。 |

## 注意事项

- **多文件限制**: 某些格式（如 GZIP、BZIP2、XZ）仅支持压缩单个文件。如果尝试使用这些格式压缩多个文件，将返回 `FeatureNotSupported` 错误。
- **文件独占**: 压缩时请确保源文件未被其他程序独占。
