# 安装与初始化

在开始使用 `bit7z-rust` 之前，需要确保您的系统已安装 7-Zip 或包含其共享库（`7z.dll`, `7z.so`, 或 `7z.dylib`）。

## 1. 添加依赖

在 `Cargo.toml` 中添加 `bit7z-rust`：

```toml
[dependencies]
bit7z-rust = "0.1.0"
```

## 2. 环境准备

`bit7z-rust` 依赖于 7-Zip 的核心动态库。

### Windows
通常库名为 `7z.dll`。库文件通常位于 `C:\Program Files\7-Zip\7z.dll`。

### Linux
通常库名为 `7z.so`。库文件通常位于 `/usr/lib/7zip/7z.so`。

### macOS
通常库名为 `7z.dylib`。可以通过 Homebrew 安装：`brew install 7zip`。

## 3. 初始化 BitLibrary

在使用任何压缩、解压或读取功能之前，必须先加载 7-Zip 库。

### 自动查找加载 (推荐)

```rust
use bit7z_rust::BitLibrary;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 自动在环境变量 BIT7Z_LIBRARY_PATH 和系统默认路径中查找
    let lib = BitLibrary::new::<String>(None)?;
    println!("成功加载 7-Zip 库");
    Ok(())
}
```

### 指定路径加载

```rust
let lib = BitLibrary::new(Some("C:\\path\\to\\7z.dll"))?;
```

## 4. 常见问题：库加载失败

如果遇到 `LibraryLoadFailed` 错误，请检查：
1. 库文件路径是否正确。
2. 库的架构（x64/x86）是否与 Rust 程序一致。
3. 缺少必要的系统运行时库（如 Windows 上的 VC++ Runtime）。
