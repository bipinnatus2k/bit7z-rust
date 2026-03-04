# 回调接口 (Callbacks)

`bit7z-rust` 允许通过回调函数追踪操作进度、提供密码或处理特定事件。回调在 `BitCompressor` 和 `BitExtractor` 中均可使用。

## 常用回调类型

### `TotalCallback`

在操作开始时调用，告知待处理的总字节数。

```rust
pub type TotalCallback = Arc<Mutex<dyn FnMut(u64) + Send + Sync>>;
```

### `ProgressCallback`

在处理过程中周期性调用，提供已处理的字节数。返回 `false` 可取消当前操作。

```rust
pub type ProgressCallback = Arc<Mutex<dyn FnMut(u64) -> bool + Send + Sync>>;
```

### `RatioCallback`

提供当前输入字节数与输出字节数的比例。

```rust
pub type RatioCallback = Arc<Mutex<dyn FnMut(u64, u64) + Send + Sync>>;
```

### `FileCallback`

在处理每个具体文件之前调用，提供文件名或路径。

```rust
pub type FileCallback = Arc<Mutex<dyn FnMut(String) + Send + Sync>>;
```

### `PasswordCallback`

当存档加密且未提前提供密码时调用，用于动态获取密码。

```rust
pub type PasswordCallback = Arc<Mutex<dyn FnMut() -> String + Send + Sync>>;
```

## 在压缩器中使用回调

```rust
use bit7z_rust::{BitLibrary, BitCompressor, CompressionFormat};
use std::sync::{Arc, Mutex};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lib = BitLibrary::new::<String>(None)?;
    let mut compressor = BitCompressor::new(&lib, CompressionFormat::Zip);
    
    // 设置进度回调
    compressor.set_progress_callback(|processed, total| {
        let percent = (processed as f64 / total as f64) * 100.0;
        println!("进度: {:.2}% ({} / {})", percent, processed, total);
        true // 继续操作
    });
    
    // 设置文件处理回调
    compressor.set_file_callback(|file_path| {
        println!("正在压缩: {}", file_path);
    });
    
    compressor.compress(&["data/"], "output.zip")?;
    Ok(())
}
```

## 注意事项

- **线程安全**: 回调通常包装在 `Arc<Mutex<...>>` 中，以便在多线程环境（或 7-Zip 的内部工作线程）中安全使用。
- **性能**: 回调函数会被频繁调用，请避免在回调中执行重型操作（如大量的磁盘 I/O 或复杂的计算）。
