# 错误处理与资源释放

本章介绍在使用 `bit7z-rust` 时的错误处理机制和资源管理。

## 1. 库加载错误 (LibraryError)

在使用 `BitLibrary` 加载 7-Zip 库时，可能会遇到以下错误：

```rust
use bit7z_rust::{BitLibrary, LibraryError};

fn main() {
    let result = BitLibrary::new::<String>(None);
    match result {
        Ok(lib) => {
            println!("库加载成功");
            // 使用 lib
        }
        Err(e) => {
            match e {
                LibraryError::LoadFailed(le) => {
                    eprintln!("无法加载库文件: {}", le);
                }
                LibraryError::SymbolNotFound(s) => {
                    eprintln!("库中缺少必要符号: {}", s);
                }
                LibraryError::CreateFailed(hr) => {
                    eprintln!("创建对象失败，HRESULT: 0x{:08X}", hr);
                }
                LibraryError::NullPointer => {
                    eprintln!("库返回了空指针");
                }
            }
        }
    }
}
```

## 2. 核心操作错误 (Bit7zError)

所有压缩、解压、读取和编辑操作都返回 `Bit7zError`：

```rust
use bit7z_rust::{Bit7zError, Result};

fn compress_data() -> Result<()> {
    // ... 操作 ...
    // 如果失败，使用 ? 运算符返回错误
    // compressor.compress(&["file"], "out.7z")?;
    Ok(())
}

fn main() {
    if let Err(e) = compress_data() {
        match e {
            Bit7zError::OpenFailed(s) => println!("打开存档失败: {}", s),
            Bit7zError::CompressFailed(s) => println!("压缩过程中出错: {}", s),
            Bit7zError::ExtractFailed(s) => println!("解压过程中出错: {}", s),
            Bit7zError::EncryptedArchive => println!("存档已加密，请提供密码"),
            Bit7zError::InvalidPassword => println!("密码错误"),
            Bit7zError::Cancelled => println!("操作被用户取消"),
            Bit7zError::Io(io_err) => println!("I/O 错误: {}", io_err),
            _ => println!("其他错误: {}", e),
        }
    }
}
```

## 3. 资源释放 (RAII)

`bit7z-rust` 遵循 Rust 的 RAII (Resource Acquisition Is Initialization) 模式来管理资源：

### 库句柄释放 (BitLibrary)

当 `BitLibrary` 实例超出作用域并被丢弃时，它会自动卸载动态库。由于 `BitCompressor` 和 `BitExtractor` 持有对 `BitLibrary` 的引用，因此它们在被销毁前，库句柄不会被提前释放。

```rust
{
    let lib = BitLibrary::new::<String>(None)?;
    let compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);
    // ... 
} // 这里 lib 被释放，7z 库被卸载
```

### 存档读取器 (BitArchiveReader)

`BitArchiveReader` 在被丢弃时会自动关闭所有打开的存档句柄，并清理相关的内部 COM 对象。

```rust
{
    let mut reader = BitArchiveReader::new(&lib, ExtractFormat::SevenZip);
    reader.open("archive.7z")?;
    // ...
} // 存档句柄在此处自动关闭
```

## 4. 最佳实践

- **库重用**: 不要在循环中重复调用 `BitLibrary::new`。由于加载动态库开销很大，建议创建一个长生命周期的 `BitLibrary` 实例。
- **内存安全**: 库内部使用了大量的不安全 (unsafe) 代码来与 7-Zip COM 接口交互。公开 API 已经过安全包装，但请确保传递的路径和数据在操作期间是有效的。
- **错误上下文**: `Bit7zError` 通常包含具体的 HRESULT 错误码或详细的描述信息，建议在日志中记录完整的错误描述。
