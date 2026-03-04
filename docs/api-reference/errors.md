# 错误处理

`bit7z-rust` 使用统一的错误枚举 `Bit7zError` 来报告所有库操作中的错误，包括加载失败、压缩/解压错误以及 I/O 问题。

## 错误类型 (Bit7zError)

```rust
pub enum Bit7zError {
    LibraryLoadFailed(String),        // 加载 7-Zip 库文件失败
    SymbolNotFound(String),           // 库中缺少必要函数符号
    CreateFailed(i32),                // 创建 COM 对象失败 (HRESULT)
    OpenFailed(String),               // 打开存档文件失败
    CompressFailed(String),           // 压缩操作中出错
    ExtractFailed(String),            // 解压操作中出错
    EncryptedArchive,                 // 存档已加密，但未提供密码
    InvalidPassword,                  // 密码错误
    Cancelled,                        // 用户通过回调取消了操作
    Io(std::io::Error),               // 标准 I/O 错误
    NulError(std::ffi::NulError),     // 路径或字符串包含非法 NUL 字节
    PathTraversal(String),            // 检测到路径遍历攻击风险
    InvalidFormat(String),            // 格式不受支持或无效
    FeatureNotSupported(String),      // 当前格式不支持该功能 (如多文件 ZIP)
    CorruptedArchive,                 // 存档损坏或无效
    UnknownError(String),             // 未分类错误
    InvalidItemIndex(u32),            // 索引超出范围
    ArchiveIntegrityCheckFailed(String), // 完整性检查 (Test) 失败
    TempFileCreationFailed(String),   // 创建临时文件失败
}
```

## 结果类型 (Result)

为了方便，库定义了一个 Result 类型别名：

```rust
pub type Result<T> = std::result::Result<T, Bit7zError>;
```

## 示例：处理错误

```rust
use bit7z_rust::{BitLibrary, Bit7zError, Result};

fn main() -> Result<()> {
    let lib = match BitLibrary::new::<String>(None) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("无法加载 7z 库: {}", e);
            // 转换为 Bit7zError
            return Err(e.into());
        }
    };
    
    // 或者使用 ? 运算符自动转换并返回
    // let lib = BitLibrary::new::<String>(None)?;
    
    Ok(())
}
```
